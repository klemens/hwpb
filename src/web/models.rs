use chrono::NaiveDate;
use db;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use errors::*;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Serialize)]
pub struct Index {
    pub experiments: Vec<Experiment>,
    pub version: &'static str,
    pub commit_id: &'static str,
}


#[derive(Serialize)]
pub struct Experiment {
    pub id: String,
    pub events: Vec<Event>,
}

#[derive(Serialize)]
pub struct Event {
    pub date: String,
    pub day: String,
    pub experiment: String,
    pub groups: Vec<EventGroup>,
    pub prev_event: Option<String>,
    pub next_event: Option<String>,
}

#[derive(Serialize)]
pub struct EventGroup {
    pub id: i32,
    pub desk: i32,
    pub students: Vec<Student>,
    pub tasks: Vec<(i32, String, bool)>,
    pub elaboration: Option<(bool, bool)>,
    pub disqualified: bool,
    pub comment: String,
}

#[derive(Serialize)]
pub struct GroupOverview {
    pub id: i32,
    pub desk: i32,
    pub day: String,
    pub comment: String,
    pub students: Vec<Student>,
    pub events: Vec<GroupOverviewEvent>,
}

#[derive(Serialize)]
pub struct GroupOverviewEvent {
    pub experiment: String,
    pub group: GroupOverviewGroup,
}

#[derive(Serialize)]
pub struct GroupOverviewGroup {
    pub id: i32,
    pub disqualified: bool,
    pub tasks: Vec<(i32, String, bool)>,
    pub elaboration: Option<(bool, bool)>,
}

#[derive(Serialize)]
pub struct Student {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct SearchGroup {
    pub id: i32,
    pub desk: i32,
    pub day: String,
    pub students: Vec<Student>,
}

pub fn find_events(conn: &PgConnection) -> Result<Vec<Experiment>> {
    let events = db::events::table
        .order((db::events::experiment_id.asc(), db::events::date.asc()))
        .load::<db::Event>(conn)?
        .into_iter().map(|e| Event {
            date: format!("{}", e.date),
            day: e.day_id,
            experiment: e.experiment_id,
            groups: vec![],
            prev_event: None,
            next_event: None,
        });

    // group the events by experiment
    let mut result: Vec<Experiment> = vec![];
    for new_event in events {
        if let Some(event) = result.last_mut() {
            if new_event.experiment == event.id {
                event.events.push(new_event);
                continue;
            }
        }

        result.push(Experiment {
            id: new_event.experiment.clone(),
            events: vec![new_event],
        });
    }

    Ok(result)
}

pub fn load_event(date: &NaiveDate, conn: &PgConnection) -> Result<Event> {
    use db::{completions, elaborations, events, groups, tasks};

    let event: db::Event = events::table.filter(events::date.eq(date)).first(conn)?;

    let tasks = tasks::table.filter(tasks::experiment_id.eq(&event.experiment_id))
        .order(tasks::name.asc()).load::<db::Task>(conn)?;
    let groups = groups::table
        .filter(groups::day_id.eq(&event.day_id))
        .order((groups::comment.like("%(ENDE)%".to_string()).asc(), groups::desk.asc()))
        .load::<db::Group>(conn)?;
    let groups = load_students_for_groups(groups, conn)?;

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
        let mut web_group = EventGroup {
            id: group.id,
            desk: group.desk,
            students: students.into_iter().map(|s| Student {
                id: s.id,
                name: s.name,
            }).collect(),
            tasks: Vec::with_capacity(tasks.len()),
            elaboration: elaborations.get(&group.id).cloned(),
            disqualified: group.comment.contains("(ENDE)"),
            comment: group.comment,
        };

        for task in &tasks {
            let completed = completions.contains(&(group.id, task.id));
            web_group.tasks.push((task.id, task.name.clone(), completed));
        }

        web_groups.push(web_group)
    }

    // find previous and next event if any
    let prev_event: Option<db::Event> = events::table
        .filter(events::day_id.eq(&event.day_id))
        .filter(events::date.lt(&event.date))
        .order(events::date.desc())
        .first(conn).optional()?;
    let next_event: Option<db::Event> = events::table
        .filter(events::day_id.eq(&event.day_id))
        .filter(events::date.gt(&event.date))
        .order(events::date.asc())
        .first(conn).optional()?;

    Ok(Event {
        date: format!("{}", date),
        day: event.day_id,
        experiment: event.experiment_id,
        groups: web_groups,
        prev_event: prev_event.map(|e| format!("{}", e.date)),
        next_event: next_event.map(|e| format!("{}", e.date)),
    })
}

pub fn load_group(group: i32, conn: &PgConnection) -> Result<GroupOverview> {
    use db::{completions, elaborations, groups, tasks};

    let group: db::Group = groups::table.find(group).first(conn)?;
    let disqualified = group.comment.contains("(ENDE)");

    // Load all available tasks and group by experiment
    let tasks: BTreeMap<_,Vec<_>> = tasks::table
        .order((tasks::experiment_id.asc(), tasks::name.asc()))
        .load::<db::Task>(conn)?.into_iter()
        .group_by(|task| task.experiment_id.clone()).into_iter()
        .map(|(k, v)| (k, v.collect()))
        .collect();

    // Load all completions and elaborations for the group
    let completions: HashSet<_> = completions::table
        .filter(completions::group_id.eq(group.id))
        .load::<db::Completion>(conn)?.into_iter()
        .map(|c| c.task_id)
        .collect();
    let elaborations: HashMap<_,_> = elaborations::table
        .filter(elaborations::group_id.eq(group.id))
        .load::<db::Elaboration>(conn)?.into_iter()
        .map(|e| (e.experiment_id, (e.rework_required, e.accepted)))
        .collect();

    let events = tasks.into_iter().map(|(experiment, tasks)| {
        // Check which tasks the the group has completed
        let tasks = tasks.into_iter().map(|task| {
            let completed = completions.contains(&task.id);
            (task.id, task.name, completed)
        }).collect();

        GroupOverviewEvent {
            group: GroupOverviewGroup {
                id: group.id,
                disqualified: disqualified,
                tasks: tasks,
                elaboration: elaborations.get(&experiment).cloned(),
            },
            experiment: experiment,
        }
    }).collect();

    let students = load_students_for_groups(vec![group.clone()], conn)?
        .pop().ok_or("error while loading students of group")?
        .1.into_iter()
        .map(|student| Student {
            id: student.id,
            name: student.name,
        })
        .collect();

    Ok(GroupOverview {
        id: group.id,
        desk: group.desk,
        day: group.day_id,
        comment: group.comment,
        students: students,
        events: events,
    })
}

pub fn find_students<T: AsRef<str>>(terms: &[T], conn: &PgConnection) -> Result<Vec<Student>> {
    use db::students;

    let mut query = students::table.into_boxed();
    for term in terms {
        query = query.filter(
            students::name.ilike(format!("%{}%", term.as_ref())).or(
                students::id.ilike(format!("%{}%", term.as_ref()))
            )
        );
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

pub fn find_groups<T: AsRef<str>>(terms: &[T], conn: &PgConnection) -> Result<Vec<SearchGroup>> {
    use db::{groups, group_mappings, students};

    let group_ids = {
        let mut query = students::table
            .inner_join(group_mappings::table)
            .select(group_mappings::group_id)
            .into_boxed();
        for term in terms {
            query = query.filter(
                students::name.ilike(format!("%{}%", term.as_ref())).or(
                    students::id.ilike(format!("%{}%", term.as_ref()))
                )
            );
        }
        query.load::<i32>(conn)?
    };
    let groups = groups::table
        .filter(groups::id.eq_any(group_ids))
        .order((groups::day_id, groups::desk))
        .load(conn)?;

    let search_groups = load_students_for_groups(groups, conn)?
        .into_iter().map(|(group, students)| {
            let students = students.into_iter().map(|student| {
                Student {
                    id: student.id,
                    name: student.name,
                }
            }).collect();

            SearchGroup {
                id: group.id,
                desk: group.desk,
                day: group.day_id,
                students: students,
            }
        })
        .collect();

    Ok(search_groups)
}

fn load_students_for_groups(groups: Vec<db::Group>, conn: &PgConnection) -> Result<Vec<(db::Group, Vec<db::Student>)>> {
    use db::students;

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
