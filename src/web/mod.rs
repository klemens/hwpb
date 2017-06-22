mod models;
pub mod session;

use chrono;
use db;
use rocket::http::RawStr;
use rocket::request::FromParam;
use rocket::response::NamedFile;
use rocket_contrib::Template;
use web::session::User;
use std::ops::Deref;
use std::path::{Path, PathBuf};

struct Date(chrono::NaiveDate);

impl<'a> FromParam<'a> for Date {
    type Error = chrono::format::ParseError;

    fn from_param(date: &'a RawStr) -> Result<Self, Self::Error> {
        Ok(Date(date.parse()?))
    }
}

impl Deref for Date {
    type Target = chrono::NaiveDate;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[get("/")]
fn index(conn: db::Conn, _user: User) -> Template {
    let context = models::Index {
        events: models::find_events(&conn).unwrap(),
        version: env!("CARGO_PKG_VERSION"),
        commit_id: include_str!(concat!(env!("OUT_DIR"), "/commit-id")),
    };

    Template::render("index", &context)
}

#[get("/<date>")]
fn event(date: Date, conn: db::Conn, _user: User) -> Template {
    let context = models::load_event(&date, &conn).unwrap();

    Template::render("event", &context)
}

#[get("/static/<path..>")]
fn static_file(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("templates/static/").join(path)).ok()
}

pub mod api {
    use db;
    use diesel;
    use diesel::pg::upsert::*;
    use diesel::prelude::*;
    use rocket::response::status;
    use rocket_contrib::JSON;
    use web::session::User;

    #[derive(Debug)]
    struct Error{}

    #[post("/group", data = "<group>")]
    fn post_group(group: JSON<db::NewGroup>, conn: db::Conn, _user: User) -> Result<status::NoContent, Error> {
        diesel::insert(&*group)
            .into(db::groups::table)
            .execute(&*conn)
            .map_err(|_| Error{})?;

        Ok(status::NoContent)
    }

    #[put("/group/<group>/completed/<task>")]
    fn put_completion(group: i32, task: i32, conn: db::Conn, user: User) -> Result<status::NoContent, Error> {
        let completion = db::Completion {
            group_id: group,
            task_id: task,
            tutor: Some(user.name),
            completed_at: None,
        };

        diesel::insert(&completion.on_conflict_do_nothing())
            .into(db::completions::table)
            .execute(&*conn)
            .map_err(|_| Error{})?;

        Ok(status::NoContent)
    }

    #[delete("/group/<group>/completed/<task>")]
    fn delete_completion(group: i32, task: i32, conn: db::Conn, _user: User) -> Result<status::NoContent, Error> {
        diesel::delete(db::completions::table
            .filter(db::completions::group_id.eq(group))
            .filter(db::completions::task_id.eq(task)))
            .execute(&*conn)
            .map_err(|_| Error{})?;

        Ok(status::NoContent)
    }

    #[derive(Deserialize)]
    struct Elaboration {
        rework_required: bool,
        accepted: bool,
    }

    #[put("/group/<group>/elaboration/<experiment>", data = "<elaboration>")]
    fn put_elaboration(group: i32, experiment: String, elaboration: JSON<Elaboration>, conn: db::Conn, user: User) -> Result<status::NoContent, Error> {
        let elaboration = db::Elaboration {
            group_id: group,
            experiment_id: experiment,
            rework_required: elaboration.rework_required,
            accepted: elaboration.accepted,
            accepted_by: Some(user.name),
        };

        diesel::insert(
            &elaboration.on_conflict(
                (db::elaborations::group_id, db::elaborations::experiment_id),
                do_update().set(&elaboration)
            )
        ).into(db::elaborations::table).execute(&*conn)
            .map_err(|_| Error{})?;

        Ok(status::NoContent)
    }

    #[delete("/group/<group>/elaboration/<experiment>")]
    fn delete_elaboration(group: i32, experiment: String, conn: db::Conn, _user: User) -> Result<status::NoContent, Error> {
        diesel::delete(db::elaborations::table
            .filter(db::elaborations::group_id.eq(group))
            .filter(db::elaborations::experiment_id.eq(experiment)))
            .execute(&*conn)
            .map_err(|_| Error{})?;

        Ok(status::NoContent)
    }

    #[put("/group/<group>/comment", data = "<comment>")]
    fn put_comment(group: i32, comment: String, conn: db::Conn, _user: User) -> Result<status::NoContent, Error> {
        diesel::update(db::groups::table.filter(db::groups::id.eq(group)))
            .set(db::groups::comment.eq(comment))
            .execute(&*conn)
            .map_err(|_| Error{})?;

        Ok(status::NoContent)
    }

    #[put("/group/<group>/student/<student>")]
    fn put_group_student(group: i32, student: String, conn: db::Conn, _user: User) -> Result<status::NoContent, Error> {
        let mapping = db::GroupMapping {
            student_id: student,
            group_id: group,
        };

        diesel::insert(&mapping)
            .into(db::group_mappings::table)
            .execute(&*conn)
            .map_err(|_| Error{})?;

        Ok(status::NoContent)
    }

    #[delete("/group/<group>/student/<student>")]
    fn delete_group_student(group: i32, student: String, conn: db::Conn, _user: User) -> Result<status::NoContent, Error> {
        diesel::delete(db::group_mappings::table
            .filter(db::group_mappings::student_id.eq(student))
            .filter(db::group_mappings::group_id.eq(group)))
            .execute(&*conn)
            .map_err(|_| Error{})?;

        Ok(status::NoContent)
    }

    #[post("/student/search", data = "<terms>")]
    fn search_students(terms: JSON<Vec<String>>, conn: db::Conn, _user: User) -> Result<JSON<Vec<super::models::Student>>, Error> {
        let students = super::models::find_students(&*terms, &conn)
            .map_err(|_| Error{})?;
        
        Ok(JSON(students))
    }
}
