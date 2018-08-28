use crate::db::{self, PgInetExpressionMethods};
use crate::errors::*;
use diesel::prelude::*;
use diesel::pg::PgConnection;

#[derive(Serialize)]
pub struct Context {
    pub base: super::BaseContext,
    pub tutors: Vec<Tutor>,
    pub ip_whitelist: Option<Vec<WhitelistEntry>>,
}

#[derive(Serialize)]
pub struct Tutor {
    pub id: i32,
    pub username: String,
    pub is_admin: bool,
}

#[derive(Serialize)]
pub struct WhitelistEntry {
    pub id: i32,
    pub ipnet: String,
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

pub fn load_whitelist(year: i16, conn: &PgConnection) -> Result<Vec<WhitelistEntry>> {
    let whitelist = db::ip_whitelist::table
        .filter(db::ip_whitelist::year.eq(year))
        .select((db::ip_whitelist::id, db::ip_whitelist::ipnet.abbrev()))
        .order(db::ip_whitelist::ipnet)
        .load::<(i32, String)>(conn)?;

    Ok(whitelist.into_iter()
        .map(|(id, ipnet)| WhitelistEntry { id, ipnet })
        .collect())
}
