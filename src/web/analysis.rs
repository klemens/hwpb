use bit_vec::BitVec;
use crate::db;
use crate::errors::*;
use crate::web::admin::export::CsvResponse;
use crate::web::session::{SiteAdmin, User};
use crate::web::models::is_writable_year;
use csv::Writer;
use diesel::dsl::not;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use itertools::Itertools;
use rocket_contrib::templates::Template;
use std::cmp::Ordering;
use std::collections::{HashMap, BTreeSet};

#[derive(Serialize)]
struct Analysis {
    heading: &'static str,
    show_export_links: bool,
    students: Vec<Student>,
    year: i16,
    read_only_year: bool,
}

#[get("/passed/<year>")]
pub fn passed(year: i16, conn: db::Conn, user: User) -> Result<Template> {
    user.ensure_tutor_for(year)?;

    let (elaborations_by_student, _) =
        load_elaborations_by_student(year, None, Some(true), &*conn)?;

    let students = elaborations_by_student.into_iter()
        .filter_map(|(student, elaboration)| {
            if elaboration.all() { Some(student) } else { None }
        })
        .collect();

    let context = Analysis {
        heading: "Zugelassene Studenten",
        show_export_links: user.is_site_admin(),
        students: students,
        year: year,
        read_only_year: !is_writable_year(year, &conn)?,
    };

    Ok(Template::render("analysis", &context))
}

#[get("/passed-complete/<year>")]
pub fn passed_complete(year: i16, conn: db::Conn, _user: SiteAdmin) -> Result<CsvResponse> {
    // Load all students
    let mut students = db::students::table
        .filter(db::students::year.eq(year))
        .order(db::students::matrikel)
        .load::<db::Student>(&*conn)?;

    // Load all students that pass the course
    let (elaborations_by_student, _) =
        load_elaborations_by_student(year, None, Some(true), &*conn)?;
    let passed_students: BTreeSet<_> = elaborations_by_student.into_iter()
        .filter_map(|(student, elaboration)| {
            if elaboration.all() { Some(student.matrikel) } else { None }
        })
        .collect();

    // Sort passed students to the front
    students.sort_by_key(|student| {
        !passed_students.contains(&student.matrikel)
    });

    let mut csv = Writer::from_writer(vec![]);

    for student in students {
        let passed = match passed_students.contains(&student.matrikel) {
            true => "bestanden",
            false => "nicht bestanden",
        };

        csv.write_field(student.matrikel)?;
        csv.write_field(student.given_name)?;
        csv.write_field(student.family_name)?;
        csv.write_field(passed)?;
        csv.write_record(None::<&[u8]>)?; // Finish record
    }

    Ok(CsvResponse {
        filename: format!("Hardwarepraktikum-{}.csv", year),
        content: csv.into_inner().chain_err(|| "Could not finalize csv writer")?
    })
}

#[get("/missing-reworks/<year>")]
pub fn missing_reworks(year: i16, conn: db::Conn, user: User) -> Result<Template> {
    user.ensure_tutor_for(year)?;

    let (tasks_by_student, _) = load_tasks_by_student(year, false, &*conn)?;
    let (elaborations_by_student, _) =
        load_elaborations_by_student(year, Some(true), Some(false), &*conn)?;

    let students_with_all_tasks = tasks_by_student.into_iter()
        .filter_map(|(student, tasks)| {
            // Only consider students that have completed all tasks
            if tasks.all() { Some(student) } else { None }
        })
        .collect();

    let students_with_unaccepted_reworks = elaborations_by_student.into_iter()
        .filter_map(|(student, elaboration)| {
            // Consider all students with at least one unaccepted rework
            if elaboration.any() { Some(student) } else { None }
        })
        .collect::<BTreeSet<_>>();

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
        show_export_links: false,
        students: students,
        year: year,
        read_only_year: !is_writable_year(year, &conn)?,
    };

    Ok(Template::render("analysis", &context))
}


#[derive(Clone, Debug, Eq, Serialize)]
pub struct Student {
    pub id: i32,
    matrikel: String,
    name: String,
    username: Option<String>,
    groups: BTreeSet<i32>,
    instructed: bool,
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
pub fn load_tasks_by_student(year: i16, include_extra_tasks: bool, conn: &PgConnection)
                             -> Result<(Vec<(Student, BitVec)>, Vec<db::Task>)> {
    let mut tasks_query = db::tasks::table
        .filter(db::tasks::experiment_id.eq_any(
            db::experiments::table
                .filter(db::experiments::year.eq(year))
                .select(db::experiments::id)
        ))
        .into_boxed();

    // Ignore tasks that start with [Zz] (Zusatzaufgabe) if requested
    if !include_extra_tasks {
        tasks_query = tasks_query.filter(not(db::tasks::name.ilike("Z%")));
    }

    let tasks = tasks_query
        .order((db::tasks::experiment_id.asc(), db::tasks::name.asc()))
        .load::<db::Task>(conn)?;

    // Load map (task_id, index) where the indices start at 0 and are
    // contiguous, used as a lookup table for filling a bit vector
    let task_map: HashMap<_,_> = tasks.iter()
        .enumerate()
        .map(|(i, task)| (task.id, i))
        .collect();

    let tasks_by_student = db::completions::table
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
            let mut completed_tasks = BitVec::from_elem(task_map.len(), false);
            let mut groups = BTreeSet::new();

            let mut student = None;

            for (completion, db_student) in completions {
                // Can be None because of the additional tasks
                if let Some(index) = task_map.get(&completion.task_id) {
                    completed_tasks.set(*index, true);
                    groups.insert(completion.group_id);
                }

                student.get_or_insert(db_student);
            }

            let student = student.expect("empty group (itertools)");

            (Student {
                id: student.id,
                name: student.name(),
                matrikel: student.matrikel,
                username: student.username,
                groups: groups,
                instructed: student.instructed,
            }, completed_tasks)
        })
        .collect();

    Ok((tasks_by_student, tasks))
}

// Load all students with their handed in elaborations, optionally filtered by
// the rework_required and accepted states
pub fn load_elaborations_by_student(year: i16, rework_required: Option<bool>,
                                    accepted: Option<bool>, conn: &PgConnection)
                                    -> Result<(Vec<(Student, BitVec)>, Vec<db::Experiment>)> {
    let experiments = db::experiments::table
        .filter(db::experiments::year.eq(year))
        .order(db::experiments::id.asc())
        .load::<db::Experiment>(conn)?;

    // Generate map (experiment_id, index) where the indices start at 0 and are
    // contiguous, used as a lookup table for filling a bit vector
    let experiment_map: HashMap<_,_> = experiments.iter()
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

    let elaborations_by_student = query
        .order(db::students::id)
        .select((db::elaborations::all_columns, db::students::all_columns))
        .load::<(db::Elaboration, db::Student)>(conn)?.into_iter()
        .group_by(|&(_, ref student)| student.id).into_iter()
        .map(|(_, elaborations)| {
            let mut existing_elaborations = BitVec::from_elem(experiment_map.len(), false);
            let mut groups = BTreeSet::new();

            let mut student = None;

            for (elaboration, db_student) in elaborations {
                // Cannot be none, because experiments is complete
                let index = experiment_map.get(&elaboration.experiment_id).unwrap();
                existing_elaborations.set(*index, true);
                groups.insert(elaboration.group_id);

                student.get_or_insert(db_student);
            }

            let student = student.expect("empty group (itertools)");

            (Student {
                id: student.id,
                name: student.name(),
                matrikel: student.matrikel,
                username: student.username,
                groups: groups,
                instructed: student.instructed,
            }, existing_elaborations)
        })
        .collect();

    Ok((elaborations_by_student, experiments))
}
