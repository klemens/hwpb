use bit_vec::BitVec;
use db;
use diesel::dsl::not;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use errors::*;
use itertools::Itertools;
use rocket_contrib::Template;
use std::cmp::Ordering;
use std::collections::{HashMap, BTreeSet};
use web::session::User;
use web::models::is_writable_year;

#[derive(Serialize)]
struct Analysis {
    heading: &'static str,
    students: Vec<Student>,
    year: i16,
    read_only_year: bool,
}

#[derive(FromForm)]
struct Export {
    format: String,
}

#[get("/passed/<year>?<export>")]
fn passed(year: i16, export: Export, conn: db::Conn, _user: User) -> Result<Template> {
    let students = load_elaborations_by_student(year, None, Some(true), &*conn)?
        .into_iter()
        .filter_map(|(student, elaboration)| {
            if elaboration.all() { Some(student) } else { None }
        })
        .collect();

    let context = Analysis {
        heading: "Zugelassene Studenten",
        students: students,
        year: year,
        read_only_year: !is_writable_year(year, &conn)?,
    };

    match export.format.as_str() {
        "html" => Ok(Template::render("analysis", &context)),
        "text" => Ok(Template::render("analysis-text", &context)),
        e => Err(format!("Invalid format specified: {}", e).into()),
    }
}

#[get("/missing-reworks/<year>?<export>")]
fn missing_reworks(year: i16, export: Export, conn: db::Conn, _user: User) -> Result<Template> {
    let students_with_all_tasks = load_tasks_by_student(year, &*conn)?
        .into_iter()
        .filter_map(|(student, tasks)| {
            // Only consider students that have completed all tasks
            if tasks.all() { Some(student) } else { None }
        })
        .collect();

    let students_with_unaccepted_reworks: BTreeSet<_> =
        load_elaborations_by_student(year, Some(true), Some(false), &*conn)?
        .into_iter()
        .filter_map(|(student, elaboration)| {
            // Consider all students with at least one unaccepted rework
            if elaboration.any() { Some(student) } else { None }
        })
        .collect();

    let mut students: Vec<_> = students_with_unaccepted_reworks
        .intersection(&students_with_all_tasks)
        .cloned()
        .collect();

    // try to sort students from the same group together
    students.sort_by(|left, right| {
        // iter is always sorted, so next_back is the max element
        left.groups.iter().next_back().cmp(
            &right.groups.iter().next_back())
    });

    let context = Analysis {
        heading: "Fehlende Nachbesserungen",
        students: students,
        year: year,
        read_only_year: !is_writable_year(year, &conn)?,
    };

    match export.format.as_str() {
        "html" => Ok(Template::render("analysis", &context)),
        "text" => Ok(Template::render("analysis-text", &context)),
        e => Err(format!("Invalid format specified: {}", e).into()),
    }
}


#[derive(Clone, Debug, Eq, Serialize)]
struct Student {
    id: i32,
    matrikel: String,
    name: String,
    groups: BTreeSet<i32>,
}

impl Ord for Student {
    fn cmp(&self, other: &Student) -> Ordering {
        self.id.cmp(&other.id)
    }
}
impl PartialOrd for Student {
    fn partial_cmp(&self, other: &Student) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Student {
    fn eq(&self, other: &Student) -> bool {
        self.id == other.id
    }
}

// Load all students with their completed tasks
fn load_tasks_by_student(year: i16, conn: &PgConnection) -> Result<Vec<(Student, BitVec)>> {
    // Load map (task_id, index) where the indices start at 0 and are
    // contiguous. Tasks that start with [Zz] (Zusatzaufgabe) are ignored
    let tasks: HashMap<_,_> = db::tasks::table
        .filter(db::tasks::experiment_id.eq_any(
            db::experiments::table
                .filter(db::experiments::year.eq(year))
                .select(db::experiments::id)
        ))
        .filter(not(db::tasks::name.ilike("Z%")))
        .order((db::tasks::experiment_id.asc(), db::tasks::name.asc()))
        .load::<db::Task>(conn)?.into_iter()
        .enumerate()
        .map(|(i, task)| (task.id, i))
        .collect();

    Ok(db::completions::table
        .inner_join(db::group_mappings::table
            .on(db::completions::group_id.eq(db::group_mappings::group_id)))
        .inner_join(db::students::table
            .on(db::group_mappings::student_id.eq(db::students::id)))
        .filter(db::students::year.eq(year))
        .order(db::students::id)
        .select((db::completions::all_columns, db::students::all_columns))
        .load::<(db::Completion, db::Student)>(conn)?.into_iter()
        .group_by(|&(_, ref student)| student.id).into_iter()
        .map(|(_, completions)| {
            let mut completed_tasks = BitVec::from_elem(tasks.len(), false);
            let mut groups = BTreeSet::new();

            let mut student = None;

            for (completion, db_student) in completions {
                // Can be None because of the additional tasks
                if let Some(index) = tasks.get(&completion.task_id) {
                    completed_tasks.set(*index, true);
                    groups.insert(completion.group_id);
                }

                student.get_or_insert(db_student);
            }

            let student = student.expect("empty group (itertools)");

            (Student {
                id: student.id,
                matrikel: student.matrikel,
                name: student.name,
                groups: groups,
            }, completed_tasks)
        })
        .collect())
}

// Load all students with their handed in elaborations, optionally filtered by
// the rework_required and accepted states
fn load_elaborations_by_student(year: i16, rework_required: Option<bool>, accepted: Option<bool>,
                                conn: &PgConnection) -> Result<Vec<(Student, BitVec)>> {
    // Load map (experiment_id, index) where the indices start at 0 and are
    // contiguous
    let experiments: HashMap<_,_> = db::experiments::table
        .filter(db::experiments::year.eq(year))
        .order(db::experiments::id.asc())
        .load::<db::Experiment>(conn)?.into_iter()
        .enumerate()
        .map(|(i, experiment)| (experiment.id, i))
        .collect();

    let mut query = db::elaborations::table
        .inner_join(db::group_mappings::table
            .on(db::elaborations::group_id.eq(db::group_mappings::group_id)))
        .inner_join(db::students::table
            .on(db::group_mappings::student_id.eq(db::students::id)))
        .filter(db::students::year.eq(year))
        .into_boxed();
    if let Some(rework) = rework_required {
        query = query.filter(db::elaborations::rework_required.eq(rework));
    }
    if let Some(accepted) = accepted {
        query = query.filter(db::elaborations::accepted.eq(accepted));
    }

    Ok(query
        .order(db::students::id)
        .select((db::elaborations::all_columns, db::students::all_columns))
        .load::<(db::Elaboration, db::Student)>(conn)?.into_iter()
        .group_by(|&(_, ref student)| student.id).into_iter()
        .map(|(_, elaborations)| {
            let mut existing_elaborations = BitVec::from_elem(experiments.len(), false);
            let mut groups = BTreeSet::new();

            let mut student = None;

            for (elaboration, db_student) in elaborations {
                // Cannot be none, because experiments is complete
                let index = experiments.get(&elaboration.experiment_id).unwrap();
                existing_elaborations.set(*index, true);
                groups.insert(elaboration.group_id);

                student.get_or_insert(db_student);
            }

            let student = student.expect("empty group (itertools)");

            (Student {
                id: student.id,
                matrikel: student.matrikel,
                name: student.name,
                groups: groups,
            }, existing_elaborations)
        })
        .collect())
}
