use chrono::{DateTime, NaiveDate, UTC};

use super::schema::*;

#[derive(Debug, Queryable, Identifiable, Associations)]
#[has_many(events)]
pub struct Day {
    pub id: String,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[has_many(events)]
#[has_many(tasks)]
#[has_many(elaborations)]
pub struct Experiment {
    pub id: String,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[belongs_to(Day)]
#[belongs_to(Experiment)]
pub struct Event {
    pub id: i32,
    pub day_id: String,
    pub experiment_id: String,
    pub date: NaiveDate,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[has_many(students)]
#[has_many(completions)]
#[has_many(elaborations)]
pub struct Group {
    pub id: i32,
    pub desk: i32,
    pub date: String,
    pub comment: String,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[belongs_to(Group)]
pub struct Student {
    pub id: String,
    pub name: String,
    pub group_id: Option<i32>,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[belongs_to(Experiment)]
#[has_many(completions)]
pub struct Task {
    pub id: i32,
    pub experiment_id: String,
    pub name: String,
}

#[derive(Debug, Queryable, Insertable, Identifiable, Associations)]
#[table_name="completions"]
#[primary_key(group_id, task_id)]
#[belongs_to(Group)]
#[belongs_to(Task)]
pub struct Completion {
    pub group_id: i32,
    pub task_id: i32,
    pub tutor: Option<String>,
    pub completed_at: Option<DateTime<UTC>>,
}

#[derive(Debug, Queryable, Insertable, AsChangeset, Identifiable, Associations)]
#[table_name="elaborations"]
#[primary_key(group_id, experiment_id)]
#[belongs_to(Group)]
#[belongs_to(Experiment)]
pub struct Elaboration {
    pub group_id: i32,
    pub experiment_id: String,
    pub rework_required: bool,
    pub accepted: bool,
    pub accepted_by: Option<String>,
}

use std::borrow::Borrow;

impl Borrow<str> for Student {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl Borrow<str> for Task {
    fn borrow(&self) -> &str {
        &self.name
    }
}
