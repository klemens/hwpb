use db;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use errors::*;

#[derive(Serialize)]
pub struct Context {
    pub base: super::BaseContext,
    pub tutors: Vec<Tutor>,
}

#[derive(Serialize)]
pub struct Tutor {
    pub id: i32,
    pub username: String,
    pub is_admin: bool,
}

pub fn load_tutors(year: i16, conn: &PgConnection) -> Result<Vec<Tutor>> {
    let tutors = db::tutors::table
        .filter(db::tutors::year.eq(year))
        .order(db::tutors::username)
        .load::<db::Tutor>(conn)?;

    Ok(tutors.into_iter()
        .map(|tutor| {
            Tutor {
                id: tutor.id,
                username: tutor.username,
                is_admin: tutor.is_admin,
            }
        })
        .collect())
}
