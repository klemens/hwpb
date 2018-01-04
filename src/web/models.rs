use chrono::NaiveDate;
use db;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use errors::*;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

#[derive(Serialize)]
pub struct Year {
    name: i16,
    read_only: bool,
}

#[derive(Serialize)]
pub struct Index {
    pub years: Vec<Year>,
    pub version: &'static str,
    pub commit_id: &'static str,
}

#[derive(Serialize)]
pub struct Overview {
    pub year: i16,
    pub read_only: bool,
    pub experiments: Vec<Experiment>,
}

#[derive(Serialize)]
pub struct Experiment {
    pub id: i32,
    pub name: String,
    pub events: Vec<Event>,
}

#[derive(Serialize)]
pub struct Event {
    pub year: i16,
    pub read_only: bool,
    pub date: String,
    pub day_id: i32,
    pub day: String,
    pub experiment_id: i32,
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
    pub year: i16,
    pub read_only: bool,
    pub day: String,
    pub comment: String,
    pub students: Vec<Student>,
    pub events: Vec<GroupOverviewEvent>,
}

#[derive(Serialize)]
pub struct GroupOverviewEvent {
    pub experiment_id: i32,
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
    pub id: i32,
    pub name: String,
}

#[derive(Serialize)]
pub struct SearchGroup {
    pub id: i32,
    pub desk: i32,
    pub day: String,
    pub students: Vec<Student>,
}

pub fn find_years(conn: &PgConnection) -> Result<Vec<Year>> {
    let years = db::years::table
        .order(db::years::id.desc())
        .load::<db::Year>(conn)?
        .into_iter()
        .map(|year| Year {
            name: year.id,
            read_only: !year.writable
        })
        .collect();

    Ok(years)
}

pub fn find_writable_year(group: i32, conn: &PgConnection) -> ApiResult<i16> {
    match db::groups::table
        .inner_join(db::days::table
        .inner_join(db::years::table))
        .filter(db::groups::id.eq(group))
        .filter(db::years::writable.eq(true))
        .select(db::years::id)
        .get_result(conn)
        .optional()? {
        Some(year) => Ok(year),
        None => Err(ApiError::Locked),
    }
}

pub fn is_writable_year(year: i16, conn: &PgConnection) -> Result<bool> {
    let count: i64 = db::years::table
        .filter(db::years::id.eq(year))
        .filter(db::years::writable.eq(true))
        .count()
        .get_result(conn)?;

    Ok(count > 0)
}

pub fn find_events(year: i16, conn: &PgConnection) -> Result<Vec<Experiment>> {
    let writable_year = is_writable_year(year, conn)?;
    let days_this_year = db::days::table
        .filter(db::days::year.eq(year))
        .select(db::days::id)
        .load::<i32>(conn)?;

    let events = db::events::table
        .filter(db::events::day_id.eq_any(days_this_year))
        .order((db::events::experiment_id.asc(), db::events::date.asc()))
        .inner_join(db::days::table)
        .inner_join(db::experiments::table)
        .load::<(db::Event, db::Day, db::Experiment)>(conn)?
        .into_iter().map(|(event, day, experiment)| Event {
            year: year,
            read_only: !writable_year,
            date: format!("{}", event.date),
            day_id: day.id,
            day: day.name,
            experiment_id: experiment.id,
            experiment: experiment.name,
            groups: vec![],
            prev_event: None,
            next_event: None,
        });

    // group the events by experiment
    let mut result: Vec<Experiment> = vec![];
    for new_event in events {
        if let Some(event) = result.last_mut() {
            if new_event.experiment_id == event.id {
                event.events.push(new_event);
                continue;
            }
        }

        result.push(Experiment {
            id: new_event.experiment_id,
            name: new_event.experiment.clone(),
            events: vec![new_event],
        });
    }

    Ok(result)
}

pub fn load_event(date: &NaiveDate, conn: &PgConnection) -> Result<Event> {
    use db::{completions, elaborations, events, groups, tasks};

    let (event, day, experiment) = events::table
        .inner_join(db::days::table)
        .inner_join(db::experiments::table)
        .filter(events::date.eq(date))
        .first::<(db::Event, db::Day, db::Experiment)>(conn)?;

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
        year: day.year,
        read_only: !is_writable_year(day.year, conn)?,
        date: format!("{}", date),
        day_id: day.id,
        day: day.name,
        experiment_id: experiment.id,
        experiment: experiment.name,
        groups: web_groups,
        prev_event: prev_event.map(|e| format!("{}", e.date)),
        next_event: next_event.map(|e| format!("{}", e.date)),
    })
}

pub fn load_group(group: i32, conn: &PgConnection) -> Result<GroupOverview> {
    use db::{completions, elaborations, groups, tasks};

    let (group, day) = groups::table
        .inner_join(db::days::table)
        .filter(db::groups::id.eq(group))
        .first::<(db::Group, db::Day)>(conn)?;
    let disqualified = group.comment.contains("(ENDE)");

    // Load all available tasks and group by experiment
    let tasks: Vec<(_, Vec<_>)> = tasks::table
        .inner_join(db::experiments::table)
        .filter(db::experiments::year.eq(day.year))
        .order((db::experiments::name.asc(), tasks::name.asc()))
        .load::<(db::Task, db::Experiment)>(conn)?.into_iter()
        .group_by(|&(_, ref experiment)| experiment.name.clone()).into_iter()
        .map(|(_, grouped_values)| {
            let (tasks, mut experiments): (Vec<_>, Vec<_>) = grouped_values.unzip();
            let experiment = experiments.pop().expect("all groups are non-empty");
            (experiment, tasks)
        })
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
                elaboration: elaborations.get(&experiment.id).cloned(),
            },
            experiment_id: experiment.id,
            experiment: experiment.name,
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
        year: day.year,
        read_only: !is_writable_year(day.year, conn)?,
        day: day.name,
        comment: group.comment,
        students: students,
        events: events,
    })
}

pub fn find_students<T: AsRef<str>>(terms: &[T], year: i16, conn: &PgConnection) -> Result<Vec<Student>> {
    use db::students;

    let mut query = students::table
        .filter(students::year.eq(year))
        .into_boxed();
    for term in terms {
        query = query.filter(
            students::name.ilike(format!("%{}%", term.as_ref())).or(
                students::matrikel.ilike(format!("%{}%", term.as_ref()))
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

pub fn find_groups<T: AsRef<str>>(terms: &[T], year: i16, conn: &PgConnection) -> Result<Vec<SearchGroup>> {
    use db::{groups, group_mappings, students};

    let group_ids = {
        let mut query = students::table
            .inner_join(group_mappings::table)
            .select(group_mappings::group_id)
            .filter(students::year.eq(year))
            .into_boxed();
        for term in terms {
            query = query.filter(
                students::name.ilike(format!("%{}%", term.as_ref())).or(
                    students::matrikel.ilike(format!("%{}%", term.as_ref()))
                )
            );
        }
        query.load::<i32>(conn)?
    };
    let (groups, days): (_, Vec<_>) = groups::table
        .inner_join(db::days::table)
        .filter(groups::id.eq_any(group_ids))
        .order((db::days::name, groups::desk))
        .load::<(db::Group, db::Day)>(conn)?
        .into_iter().unzip();

    let search_groups = load_students_for_groups(groups, conn)?
        .into_iter().zip(days).map(|((group, students), day)| {
            let students = students.into_iter().map(|student| {
                Student {
                    id: student.id,
                    name: student.name,
                }
            }).collect();

            SearchGroup {
                id: group.id,
                desk: group.desk,
                day: day.name,
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
            .map(|m| m.student_id)
            .collect();
        let students = students::table
            .filter(students::id.eq_any(&student_ids))
            .load::<db::Student>(conn)?;
        students.into_iter().map(|s| (s.id, s)).collect()
    };

    let mappings = mappings.grouped_by(&groups);
    Ok(groups.into_iter().zip(mappings).map(|(group, mappings)| {
        let students = mappings.into_iter().map(|mapping| {
            student_map[&mapping.student_id].clone()
        }).collect();

        (group, students)
    }).collect())
}
