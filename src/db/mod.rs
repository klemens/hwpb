mod inet;
mod models;
mod schema;

pub use self::inet::{inet as to_inet, PgInetExpressionMethods};
pub use self::models::*;
pub use self::schema::*;

use chrono::{Datelike, Utc};
use diesel;
use diesel::{delete, dsl::any, prelude::*};
use errors::*;
use diesel::r2d2::{self, ConnectionManager};
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use std::ops::Deref;

embed_migrations!();

pub fn run_migrations(database_url: &str) -> Result<()> {
    let connection = PgConnection::establish(database_url)
        .chain_err(|| "Could not connect to DB to run migrations")?;
    embedded_migrations::run(&connection)
        .chain_err(|| "Could not run pending migrations.")
}

pub fn init_year(database_url: &str) -> Result<()> {
    let conn = PgConnection::establish(database_url)
        .chain_err(|| "Could not connect to DB to init year")?;

    conn.transaction(|| {
        let count: i64 = years::table
            .count()
            .get_result(&conn)?;

        if count == 0 {
            let year = Year {
                id: Utc::today().year() as i16,
                writable: true,
            };

            diesel::insert_into(years::table)
                .values(&year)
                .execute(&conn)?;
        }

        Ok(())
    })
}

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn init_pool(database_url: &str) -> Result<Pool> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::new(manager).chain_err(|| "Could not init DB pool")
}

pub struct Conn(r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl Deref for Conn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Conn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Conn, ()> {
        let pool = match <State<Pool> as FromRequest>::from_request(request) {
            Outcome::Success(pool) => pool,
            Outcome::Failure(e) => return Outcome::Failure(e),
            Outcome::Forward(_) => return Outcome::Forward(()),
        };

        match pool.get() {
            Ok(conn) => Outcome::Success(Conn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

pub fn expect1(count: usize) -> QueryResult<usize> {
    match count {
        1 => Ok(count),
        _ => Err(diesel::NotFound),
    }
}

/// Delete the group with the given id
///
/// Also removes all completions and elaborations of the group.
///
/// Should be run inside a transaction.
pub(crate) fn delete_group(group: i32, conn: &PgConnection) -> Result<()> {
    // Delete all completions of the group
    diesel::delete(completions::table
        .filter(completions::group_id.eq(group)))
        .execute(conn)?;

    // Delete all elaborations of the group
    diesel::delete(elaborations::table
        .filter(elaborations::group_id.eq(group)))
        .execute(conn)?;

    // Delete group mappings (but not the students themselves)
    diesel::delete(group_mappings::table
        .filter(group_mappings::group_id.eq(group)))
        .execute(conn)?;

    diesel::delete(groups::table
        .find(group))
        .execute(conn)?;

    Ok(())
}

/// Delete the entire year with the given id
///
/// Also deletes everything associated with the year, including groups,
/// students, completions, elaborations, events, experiments, tasks,
/// tutors and audit log entries.
///
/// Should be run inside a transaction.
pub(crate) fn delete_year(year: i16, conn: &PgConnection) -> Result<()> {
    // Load all days for the deletion of groups and events
    let days = days::table
        .filter(days::year.eq(year))
        .select(days::id)
        .load::<i32>(conn)?;

    // Load all groups belonging to any of the days…
    let groups = groups::table
        .filter(groups::day_id.eq(any(&days)))
        .select(groups::id)
        .load(conn)?;
    // …and delete them one by one
    for group in groups {
        delete_group(group, conn)?;
    }

    // Delete all events belonging to any of the days and the days
    delete(events::table.filter(events::day_id.eq(any(&days)))).execute(conn)?;
    delete(days::table.filter(days::year.eq(year))).execute(conn)?;

    // Load all experiments of the given year…
    let experiments = experiments::table
        .filter(experiments::year.eq(year))
        .select(experiments::id)
        .load::<i32>(conn)?;
    // …and delete all tasks referencing any of them
    diesel::delete(tasks::table
        .filter(tasks::experiment_id.eq(any(experiments))))
        .execute(conn)?;

    // Delete all experiments, students, tutors, and whitelist and audit log entries
    delete(experiments::table.filter(experiments::year.eq(year))).execute(conn)?;
    delete(students::table.filter(students::year.eq(year))).execute(conn)?;
    delete(tutors::table.filter(tutors::year.eq(year))).execute(conn)?;
    delete(ip_whitelist::table.filter(ip_whitelist::year.eq(year))).execute(conn)?;
    delete(audit_logs::table.filter(audit_logs::year.eq(year))).execute(conn)?;

    // Delete the given year
    delete(years::table.find(year)).execute(conn)?;

    Ok(())
}
