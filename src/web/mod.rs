pub mod api;
mod models;
pub mod session;

use chrono;
use db;
use errors::*;
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

    fn from_param(date: &'a RawStr) -> ::std::result::Result<Self, Self::Error> {
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
fn index(conn: db::Conn, _user: User) -> Result<Template> {
    let context = models::Index {
        experiments: models::find_events(&conn)?,
        version: env!("CARGO_PKG_VERSION"),
        commit_id: include_str!(concat!(env!("OUT_DIR"), "/commit-id")),
    };

    Ok(Template::render("index", &context))
}

#[get("/<date>")]
fn event(date: Date, conn: db::Conn, _user: User) -> Result<Template> {
    let context = models::load_event(&date, &conn)?;

    Ok(Template::render("event", &context))
}

#[get("/static/<path..>")]
fn static_file(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("templates/static/").join(path)).ok()
}
