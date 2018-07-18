use db;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use errors::*;
use std::borrow::Borrow;
use std::cmp::Ordering;

#[derive(Serialize)]
pub struct Context {
    pub base: super::BaseContext,
    pub students: Vec<Student>,
    pub order: Order,
}

#[derive(Serialize)]
pub struct Student {
    pub id: i32,
    pub matrikel: String,
    pub username: Option<String>,
    pub given_name: String,
    pub family_name: String,
    pub groups: Vec<i32>,
    pub instructed: bool,
}

#[derive(Default, FromForm, Serialize)]
pub struct Order {
    order: Option<String>,
    reverse: Option<bool>,
}

pub fn load_students(year: i16, mut order: Order, conn: &PgConnection) -> Result<(Vec<Student>, Order)> {
    let students = db::students::table
        .filter(db::students::year.eq(year))
        .load::<db::Student>(conn)?;

    let groups = db::GroupMapping::belonging_to(&students)
        .load::<db::GroupMapping>(conn)?
        .grouped_by(&students);

    let mut students: Vec<_> = students.into_iter()
        .zip(groups)
        .map(|(student, groups)| {
            let groups = groups.into_iter()
                .map(|group| group.group_id)
                .collect();

            Student {
                id: student.id,
                matrikel: student.matrikel,
                username: student.username,
                given_name: student.given_name,
                family_name: student.family_name,
                groups: groups,
                instructed: student.instructed,
            }
        })
        .collect();

    students.sort_unstable_by(|a, b| {
        let ordering = match order.order.as_ref().map(Borrow::borrow) {
            Some("matrikel") => a.matrikel.cmp(&b.matrikel),
            Some("given-name") => a.given_name.cmp(&b.given_name),
            Some("family-name") => a.family_name.cmp(&b.family_name),
            Some("instructed") => a.instructed.cmp(&b.instructed),
            Some("username") => a.username.cmp(&b.username),
            _ => order_by_groups(a, b),
        };

        match order.reverse {
            Some(true) => ordering.reverse(),
            _ => ordering,
        }
    });

    // Always return the chosen ordering for rendering
    order.order.get_or_insert("groups".into());
    order.reverse.get_or_insert(false);

    Ok((students, order))
}

/// Order students without a group to the top, otherwise order by
/// family and then first name.
fn order_by_groups(a: &Student, b: &Student) -> Ordering {
    match (a.groups.len() == 0, b.groups.len() == 0) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (_, _) => {
            a.family_name.cmp(&b.family_name)
                .then_with(|| a.given_name.cmp(&b.given_name))
        }
    }
}
