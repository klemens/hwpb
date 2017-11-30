use chrono::{DateTime, NaiveDate, Utc};

use super::schema::*;

#[derive(Debug, Queryable, Identifiable, Associations)]
pub struct Day {
    pub id: i32,
    pub name: String,
    pub year: i16,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
pub struct Experiment {
    pub id: i32,
    pub name: String,
    pub year: i16,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[belongs_to(Day)]
#[belongs_to(Experiment)]
pub struct Event {
    pub id: i32,
    pub day_id: i32,
    pub experiment_id: i32,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
pub struct Group {
    pub id: i32,
    pub desk: i32,
    pub day_id: i32,
    pub comment: String,
}

#[derive(Debug, Deserialize, Insertable)]
#[table_name="groups"]
pub struct NewGroup {
    pub desk: i32,
    pub day_id: i32,
    pub comment: String,
}

#[derive(Debug, Clone, Queryable, Identifiable, Associations)]
pub struct Student {
    pub id: i32,
    pub matrikel: String,
    pub name: String,
    pub year: i16,
}

#[derive(Debug, Queryable, Insertable, Identifiable, Associations)]
#[table_name="group_mappings"]
#[primary_key(student_id, group_id)]
#[belongs_to(Student)]
#[belongs_to(Group)]
pub struct GroupMapping {
    pub student_id: i32,
    pub group_id: i32,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[belongs_to(Experiment)]
pub struct Task {
    pub id: i32,
    pub experiment_id: i32,
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
}

#[derive(Debug, Queryable, Insertable, AsChangeset, Identifiable, Associations)]
#[table_name="elaborations"]
#[primary_key(group_id, experiment_id)]
#[belongs_to(Group)]
#[belongs_to(Experiment)]
pub struct Elaboration {
    pub group_id: i32,
    pub experiment_id: i32,
    pub rework_required: bool,
    pub accepted: bool,
}

#[derive(Debug, Queryable, Identifiable)]
pub struct AuditLog {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub year: i16,
    pub author: String,
    pub affected_group: Option<i32>,
    pub change: String,
}

#[derive(Debug, Insertable)]
#[table_name="audit_logs"]
pub struct NewAuditLog<'a, 'b> {
    pub year: i16,
    pub author: &'a str,
    pub affected_group: Option<i32>,
    pub change: &'b str,
}

#[derive(Debug, Queryable, Identifiable)]
pub struct Year {
    pub id: i16,
    pub writable: bool,
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
