mod inet;
mod models;
mod schema;

pub use self::inet::{inet as to_inet, PgInetExpressionMethods};
pub use self::models::*;
pub use self::schema::*;

use chrono::{Datelike, Utc};
use diesel;
use diesel::prelude::*;
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
