use chrono::NaiveDate;
use db;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::collections::{HashMap, HashSet};

#[derive(Serialize)]
pub struct Group {
    pub id: i32,
    pub desk: i32,
    pub students: Vec<String>,
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

pub fn find_events(conn: &PgConnection) -> QueryResult<Vec<Event>> {
    Ok(db::events::table.order(db::events::date.asc()).load::<db::Event>(conn)?
        .into_iter().map(|e| Event {
            date: format!("{}", e.date),
            day: e.day_id,
            experiment: e.experiment_id,
            groups: vec![],
        }).collect())
}

pub fn load_event(date: &NaiveDate, conn: &PgConnection) -> QueryResult<Event> {
    use db::{completions, elaborations, events, tasks};

    let event: db::Event = match events::table.filter(events::date.eq(date)).first(conn) {
        Ok(event) => event,
        _ => return Ok(Event {
            date: "?".into(),
            day: "?".into(),
            experiment: "?".into(),
            groups: vec![],
        }),
    };

    let tasks = tasks::table.filter(tasks::experiment_id.eq(&event.experiment_id))
        .order(tasks::name.asc()).load::<db::Task>(conn)?;
    let groups = db::groups_with_students(&event.day_id, conn)?;

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
            students: students.into_iter().map(|s| { s.name }).collect(),
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
