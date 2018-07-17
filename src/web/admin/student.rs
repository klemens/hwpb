use db;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use errors::*;

#[derive(Serialize)]
pub struct Context {
    pub base: super::BaseContext,
    pub students: Vec<Student>,
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

pub fn load_students(year: i16, conn: &PgConnection) -> Result<Vec<Student>> {
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
        use std::cmp::Ordering;

        // sort students without a group to the top
        match (a.groups.len() == 0, b.groups.len() == 0) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (_, _) => {
                a.family_name.cmp(&b.family_name)
                    .then_with(|| a.given_name.cmp(&b.given_name))
            }
        }
    });

    Ok(students)
}