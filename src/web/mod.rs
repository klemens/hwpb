pub mod admin;
pub mod analysis;
pub mod api;
mod models;
pub mod push;
pub mod session;

use crate::db;
use crate::errors::*;
use crate::web::session::User;
use rocket::State;
use rocket::http::RawStr;
use rocket::request::FromParam;
use rocket::response::{NamedFile, Redirect};
use rocket_contrib::templates::Template;
use std::ops::Deref;
use std::path::{Path, PathBuf};

pub struct Date(chrono::NaiveDate);

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
pub fn index(conn: db::Conn, user: User) -> Result<Template> {
    let filtered_years = models::find_years(&conn)?
        .into_iter()
        .filter(|year| user.is_tutor_for(year.name))
        .collect();

    let context = models::Index {
        years: filtered_years,
        version: env!("CARGO_PKG_VERSION"),
        commit_id: include_str!(concat!(env!("OUT_DIR"), "/commit-id")),
    };

    Ok(Template::render("index", &context))
}

#[get("/<year>", rank = 2)]
pub fn overview(year: i16, conn: db::Conn, user: User) -> Result<Template> {
    user.ensure_tutor_for(year)?;

    let context = models::Overview {
        year: year,
        read_only: !models::is_writable_year(year, &conn)?,
        is_admin: user.is_admin_for(year),
        experiments: models::find_events(year, &conn)?,
    };

    Ok(Template::render("overview", &context))
}

#[get("/<date>")]
pub fn event_finder(date: Date, conn: db::Conn, _user: User) -> Result<Redirect> {
    let day = models::find_event_day_by_date(&date, &conn)?;

    Ok(Redirect::to(format!("/{}/{}", *date, day)))
}

#[get("/<date>/<day>", rank = 2)]
pub fn event(date: Date, day: String, push_url: State<push::Url>, conn: db::Conn, user: User) -> Result<Template> {
    let context = models::load_event(&date, &day, &push_url.0, &conn)?;

    user.ensure_tutor_for(context.year)?;

    Ok(Template::render("event", &context))
}

#[get("/group/<group>")]
pub fn group(group: i32, push_url: State<push::Url>, conn: db::Conn, user: User) -> Result<Template> {
    let context = models::load_group(group, &push_url.0, &conn)?;

    user.ensure_tutor_for(context.year)?;

    Ok(Template::render("group", &context))
}

#[get("/static/<path..>")]
pub fn static_file(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("templates/static/").join(path)).ok()
}

#[get("/manifest.json")]
pub fn manifest() -> Option<NamedFile> {
    NamedFile::open(Path::new("templates/manifest.json")).ok()
}

#[get("/service-worker.js")]
pub fn service_worker() -> Option<NamedFile> {
    NamedFile::open(Path::new("templates/static/service-worker.js")).ok()
}
