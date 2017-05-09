pub mod models;

use chrono;
use db;
use rocket::http::RawStr;
use rocket::request::FromParam;
use rocket::response::NamedFile;
use rocket_contrib::Template;
use std::collections::HashMap;
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
fn index(conn: db::Conn) -> Template {
    let mut context = HashMap::new();
    context.insert("events", models::find_events(&conn).unwrap());

    Template::render("index", &context)
}

#[get("/<date>")]
fn event(date: Date, conn: db::Conn) -> Template {
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
    use diesel::prelude::*;

    #[derive(Debug)]
    struct Error{}

    #[put("/completed/<group>/<task>")]
    fn mark_completed(group: i32, task: i32, conn: db::Conn) -> Result<&'static str, Error> {
        let completion = db::models::Completion {
            group_id: group,
            task_id: task,
            tutor: None,
            completed_at: None,
        };

        diesel::insert(&completion)
            .into(db::completions::table)
            .execute(&*conn)
            .map_err(|_| Error{})?;
        Ok("")
    }

    #[delete("/completed/<group>/<task>")]
    fn unmark_completed(group: i32, task: i32, conn: db::Conn) -> Result<&'static str, Error> {
        diesel::delete(db::completions::table
            .filter(db::completions::group_id.eq(group))
            .filter(db::completions::task_id.eq(task)))
            .execute(&*conn)
            .map_err(|_| Error{})?;
        Ok("")
    }
}
