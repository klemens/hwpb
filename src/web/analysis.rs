use bit_vec::BitVec;
use db;
use diesel::expression::not;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use errors::*;
use itertools::Itertools;
use rocket_contrib::Template;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use web::session::User;

#[derive(Serialize)]
struct Analysis {
    heading: &'static str,
    students: Vec<Student>,
}

#[get("/passed")]
fn passed(conn: db::Conn, _user: User) -> Result<Template> {
    let students = load_elaborations_by_student(None, Some(true), &*conn)?
        .into_iter()
        .filter_map(|(student, elaboration)| {
            if elaboration.all() { Some(student) } else { None }
        })
        .collect();

    let context = Analysis {
        heading: "Zugelassene Studenten",
        students: students,
    };

    Ok(Template::render("analysis", &context))
}

#[get("/missing-reworks")]
fn missing_reworks(conn: db::Conn, _user: User) -> Result<Template> {
    let students_with_all_tasks = load_tasks_by_student(&*conn)?
        .into_iter()
        .filter_map(|(student, tasks)| {
            // Only consider students that have completed all tasks
            if tasks.all() { Some(student) } else { None }
        })
        .collect();

    let students_with_unaccepted_reworks: HashSet<_> =
        load_elaborations_by_student(Some(true), Some(false), &*conn)?
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

    students.sort_by(|left, right| left.id.cmp(&right.id));

    let context = Analysis {
        heading: "Fehlende Nachbesserungen",
        students: students,
    };

    Ok(Template::render("analysis", &context))
}


#[derive(Clone, Debug, Eq, Serialize)]
struct Student {
    id: String,
    groups: HashSet<i32>,
}

impl PartialEq for Student {
    fn eq(&self, other: &Student) -> bool {
        self.id == other.id
    }
}
impl Hash for Student {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// Load all students with their completed tasks
fn load_tasks_by_student(conn: &PgConnection) -> Result<Vec<(Student, BitVec)>> {
    // Load map (task_id, index) where the indices start at 0 and are
    // contiguous. Tasks that start with [Zz] (Zusatzaufgabe) are ignored
    let tasks: HashMap<_,_> = db::tasks::table
        .filter(not(db::tasks::name.ilike("Z%")))
        .order((db::tasks::experiment_id.asc(), db::tasks::name.asc()))
        .load::<db::Task>(conn)?.into_iter()
        .enumerate()
        .map(|(i, task)| (task.id, i))
        .collect();

    Ok(db::completions::table
        .inner_join(db::group_mappings::table)
        .order(db::group_mappings::student_id.asc())
        .load::<(db::Completion, db::GroupMapping)>(conn)?.into_iter()
        .group_by(|&(_, ref mapping)| mapping.student_id.clone()).into_iter()
        .map(|(student_id, completions)| {
            let mut completed_tasks = BitVec::from_elem(tasks.len(), false);
            let mut groups = HashSet::new();

            for (completion, _) in completions {
                // Can be None because of the additional tasks
                if let Some(index) = tasks.get(&completion.task_id) {
                    completed_tasks.set(*index, true);
                    groups.insert(completion.group_id);
                }
            }

            (Student {
                id: student_id,
                groups: groups,
            }, completed_tasks)
        })
        .collect())
}

// Load all students with their handed in elaborations, optionally filtered by
// the rework_required and accepted states
fn load_elaborations_by_student(rework_required: Option<bool>, accepted: Option<bool>,
                                conn: &PgConnection) -> Result<Vec<(Student, BitVec)>> {
    // Load map (experiment_id, index) where the indices start at 0 and are
    // contiguous
    let experiments: HashMap<_,_> = db::experiments::table
        .order(db::experiments::id.asc())
        .load::<db::Experiment>(conn)?.into_iter()
        .enumerate()
        .map(|(i, experiment)| (experiment.id, i))
        .collect();

    let mut query = db::elaborations::table
        .inner_join(db::group_mappings::table)
        .into_boxed();
    if let Some(rework) = rework_required {
        query = query.filter(db::elaborations::rework_required.eq(rework));
    }
    if let Some(accepted) = accepted {
        query = query.filter(db::elaborations::accepted.eq(accepted));
    }

    Ok(query
        .order(db::group_mappings::student_id.asc())
        .load::<(db::Elaboration, db::GroupMapping)>(conn)?.into_iter()
        .group_by(|&(_, ref mapping)| mapping.student_id.clone()).into_iter()
        .map(|(student_id, elaborations)| {
            let mut existing_elaborations = BitVec::from_elem(experiments.len(), false);
            let mut groups = HashSet::new();

            for (elaboration, _) in elaborations {
                // Cannot be none, because experiments is complete
                let index = experiments.get(&elaboration.experiment_id).unwrap();
                existing_elaborations.set(*index, true);
                groups.insert(elaboration.group_id);
            }

            (Student {
                id: student_id,
                groups: groups,
            }, existing_elaborations)
        })
        .collect())
}
