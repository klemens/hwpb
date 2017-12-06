mod models;
mod schema;

pub use self::models::*;
pub use self::schema::*;

use diesel;
use diesel::prelude::*;
use errors::*;
use r2d2;
use r2d2_diesel::ConnectionManager;
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
