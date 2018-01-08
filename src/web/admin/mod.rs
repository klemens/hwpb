mod audit;
mod event;
mod experiment;
mod student;
mod tutor;

use db;
use diesel::PgConnection;
use errors::*;
use rocket::response::Redirect;
use rocket_contrib::Template;
use web::session::User;
use web::models;

#[derive(Serialize)]
pub struct BaseContext {
    pub site: &'static str,
    pub year: i16,
    pub read_only_year: bool,
    pub years: Vec<models::Year>,
}

impl BaseContext {
    fn new(site: &'static str, year: i16, conn: &PgConnection) -> Result<BaseContext> {
        Ok(BaseContext {
            site: site,
            year: year,
            read_only_year: !models::is_writable_year(year, &conn)?,
            years: models::find_years(&*conn)?,
        })
    }
}

#[get("/<year>")]
fn index(year: i16, _user: User) -> Redirect {
    Redirect::to(&format!("/admin/{}/experiments", year))
}

#[get("/<year>/experiments")]
fn experiments(year: i16, conn: db::Conn, _user: User) -> Result<Template> {
    let context = experiment::Context {
        base: BaseContext::new("experiments", year, &conn)?,
        experiments: experiment::load_experiments(year, &conn)?,
    };

    Ok(Template::render("admin-experiments", context))
}

#[get("/<year>/events")]
fn events(year: i16, conn: db::Conn, _user: User) -> Result<Template> {
    let context = event::Context {
        base: BaseContext::new("events", year, &conn)?,
        days: event::load_days(year, &conn)?,
    };

    Ok(Template::render("admin-events", context))
}

#[get("/<year>/students")]
fn students(year: i16, conn: db::Conn, _user: User) -> Result<Template> {
    let context = student::Context {
        base: BaseContext::new("students", year, &conn)?,
        students: student::load_students(year, &conn)?,
    };

    Ok(Template::render("admin-students", context))
}

#[get("/<year>/tutors")]
fn tutors(year: i16, conn: db::Conn, _user: User) -> Result<Template> {
    let context = tutor::Context {
        base: BaseContext::new("tutors", year, &conn)?,
        tutors: tutor::load_tutors(year, &conn)?,
    };

    Ok(Template::render("admin-tutors", context))
}

#[get("/<year>/audit")]
fn audit_index(year: i16, _user: User) -> Redirect {
    Redirect::to(&format!("/admin/{}/audit?limit=100", year))
}

#[get("/<year>/audit?<filters>")]
fn audit(year: i16, filters: audit::Filters, conn: db::Conn, _user: User) -> Result<Template> {
    let context = audit::Context {
        base: BaseContext::new("audit", year, &conn)?,
        logs: audit::load_logs(year, &filters, &conn)?,
        filters: filters,
        authors: audit::load_authors(&conn)?,
    };

    Ok(Template::render("admin-audit", context))
}
