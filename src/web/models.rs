use chrono::NaiveDate;
use db;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use errors::*;
use std::collections::{HashMap, HashSet};

#[derive(Serialize)]
pub struct Index {
    pub events: Vec<Event>,
    pub version: &'static str,
    pub commit_id: &'static str,
}

#[derive(Serialize)]
pub struct Group {
    pub id: i32,
    pub desk: i32,
    pub students: Vec<Student>,
    pub tasks: Vec<(i32, String, bool)>,
    pub elaboration: Option<(bool, bool)>,
    pub comment: String,
}

#[derive(Serialize)]
pub struct Event {
    pub date: String,
    pub day: String,
    pub experiment: String,
    pub groups: Vec<Group>,
}

#[derive(Serialize)]
pub struct Student {
    pub id: String,
    pub name: String,
}

pub fn find_events(conn: &PgConnection) -> Result<Vec<Event>> {
    Ok(db::events::table.order(db::events::date.asc()).load::<db::Event>(conn)?
        .into_iter().map(|e| Event {
            date: format!("{}", e.date),
            day: e.day_id,
            experiment: e.experiment_id,
            groups: vec![],
        }).collect())
}

pub fn load_event(date: &NaiveDate, conn: &PgConnection) -> Result<Event> {
    use db::{completions, elaborations, events, tasks};

    let event: db::Event = events::table.filter(events::date.eq(date)).first(conn)?;

    let tasks = tasks::table.filter(tasks::experiment_id.eq(&event.experiment_id))
        .order(tasks::name.asc()).load::<db::Task>(conn)?;
    let groups = load_groups_with_students(&event.day_id, conn)?;

    // belonging_to uses eq_any internally, but supports only one parent table
    let task_ids: Vec<_> = tasks.iter().map(Identifiable::id).collect();
    let group_ids: Vec<_> = groups.iter().map(|&(ref g, _)| g.id).collect();
    let completions = completions::table
        .filter(completions::task_id.eq_any(task_ids))
        .filter(completions::group_id.eq_any(&group_ids)).load::<db::Completion>(conn)?;
    let elaborations = elaborations::table
        .filter(elaborations::experiment_id.eq(&event.experiment_id))
        .filter(elaborations::group_id.eq_any(&group_ids))
        .load::<db::Elaboration>(conn)?;

    // build set with all groups that completed a task and a map for the status
    // of the elaboration of a specific group
    let completions: HashSet<_> = completions.into_iter()
        .map(|c| (c.group_id, c.task_id)).collect();
    let elaborations: HashMap<_,_> = elaborations.into_iter()
        .map(|e| (e.group_id, (e.rework_required, e.accepted))).collect();

    let mut web_groups = vec![];

    for (group, students) in groups {
        let mut web_group = Group {
            id: group.id,
            desk: group.desk,
            students: students.into_iter().map(|s| Student {
                id: s.id,
                name: s.name,
            }).collect(),
            tasks: Vec::with_capacity(tasks.len()),
            elaboration: elaborations.get(&group.id).cloned(),
            comment: group.comment,
        };

        for task in &tasks {
            let completed = completions.contains(&(group.id, task.id));
            web_group.tasks.push((task.id, task.name.clone(), completed));
        }

        web_groups.push(web_group)
    }

    Ok(Event {
        date: format!("{}", date),
        day: event.day_id,
        experiment: event.experiment_id,
        groups: web_groups,
    })
}

pub fn find_students<T: AsRef<str>>(terms: &[T], conn: &PgConnection) -> Result<Vec<Student>> {
    use db::students;

    let mut query = students::table.into_boxed();
    for term in terms {
        query = query.filter(students::name.ilike(format!("%{}%", term.as_ref())))
    }

    Ok(query.order(students::name.asc()).load::<db::Student>(conn).map(|students| {
        students.into_iter().map(|student| {
            Student {
                id: student.id,
                name: student.name,
            }
        }).collect()
    })?)
}

pub fn load_groups_with_students(day: &str, conn: &PgConnection) -> Result<Vec<(db::Group, Vec<db::Student>)>> {
    use db::{students, groups};

    let groups = groups::table
        .filter(groups::day_id.eq(day))
        .order(groups::desk.asc())
        .load::<db::Group>(conn)?;
    let mappings = db::GroupMapping::belonging_to(&groups)
        .load::<db::GroupMapping>(conn)?;

    // TODO: replace with proper multi-join once diesel 0.14 lands
    let student_map: HashMap<_,_> = {
        let student_ids: Vec<_> = mappings.iter()
            .map(|m| m.student_id.as_str())
            .collect();
        let students = students::table
            .filter(students::id.eq_any(&student_ids))
            .load::<db::Student>(conn)?;
        students.into_iter().map(|s| (s.id, s.name)).collect()
    };

    let mappings = mappings.grouped_by(&groups);
    Ok(groups.into_iter().zip(mappings).map(|(group, mappings)| {
        let students = mappings.into_iter().map(|mapping| {
            let name = student_map[&mapping.student_id].clone();
            db::Student {
                id: mapping.student_id,
                name: name,
            }
        }).collect();

        (group, students)
    }).collect())
}
