pub mod models;
pub mod schema;

pub use models::*;
pub use schema::*;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use r2d2;
use r2d2_diesel::ConnectionManager;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use std::collections::HashMap;
use std::ops::Deref;

embed_migrations!();

pub fn run_migrations(database_url: &str) {
    let connection = PgConnection::establish(database_url)
        .expect("Could not connect to DB to run migrations");
    embedded_migrations::run(&connection)
        .expect("Could not run pending migrations.");
}

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn init_pool(database_url: &str) -> Pool {
    let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::new(config, manager).expect("Could not init DB pool")
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

pub fn groups_with_students(day: &str, conn: &PgConnection) -> QueryResult<Vec<(Group, Vec<Student>)>> {
    let groups = groups::table.filter(groups::day_id.eq(day)).order(groups::desk.asc()).load::<Group>(conn)?;
    let mappings = GroupMapping::belonging_to(&groups).load::<GroupMapping>(conn)?;

    // TODO: replace with proper multi-join once diesel 0.14 lands
    let student_map: HashMap<_,_> = {
        let student_ids: Vec<_> = mappings.iter().map(|m| m.student_id.as_str()).collect();
        let students = students::table.filter(students::id.eq_any(&student_ids)).load::<Student>(conn)?;
        students.into_iter().map(|s| (s.id, s.name)).collect()
    };

    let mappings = mappings.grouped_by(&groups);
    Ok(groups.into_iter().zip(mappings).map(|(group, mappings)| {
        let students = mappings.into_iter().map(|mapping| {
            let name = student_map[&mapping.student_id].clone();
            Student {
                id: mapping.student_id,
                name: name,
            }
        }).collect();

        (group, students)
    }).collect())
}
