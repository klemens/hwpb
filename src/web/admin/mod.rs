mod audit;
mod event;
mod experiment;
pub mod export;
mod student;
mod tutor;

use chrono::Local;
use db;
use diesel::PgConnection;
use errors::*;
use rocket::State;
use rocket::response::Redirect;
use rocket_contrib::Template;
use web::session::{IpWhitelisting, SiteAdmin, User};
use web::models;

#[derive(Serialize)]
pub struct BaseContext {
    pub site: &'static str,
    pub year: i16,
    pub read_only_year: bool,
    pub site_admin: bool,
    pub years: Vec<models::Year>,
}

impl BaseContext {
    fn new(site: &'static str, year: i16, user: &User, conn: &PgConnection) -> Result<BaseContext> {
        let filtered_years = models::find_years(&*conn)?
            .into_iter()
            .filter(|year| user.is_admin_for(year.name))
            .collect();

        Ok(BaseContext {
            site: site,
            year: year,
            read_only_year: !models::is_writable_year(year, &conn)?,
            site_admin: user.is_site_admin(),
            years: filtered_years,
        })
    }
}

#[get("/<year>")]
fn index(year: i16, user: User) -> Result<Redirect> {
    user.ensure_admin_for(year)?;

    Ok(Redirect::to(&format!("/admin/{}/experiments", year)))
}

#[get("/<year>/experiments")]
fn experiments(year: i16, conn: db::Conn, user: User) -> Result<Template> {
    user.ensure_admin_for(year)?;

    let context = experiment::Context {
        base: BaseContext::new("experiments", year, &user, &conn)?,
        experiments: experiment::load_experiments(year, &conn)?,
    };

    Ok(Template::render("admin-experiments", context))
}

#[get("/<year>/events")]
fn events(year: i16, conn: db::Conn, user: User) -> Result<Template> {
    user.ensure_admin_for(year)?;

    let context = event::Context {
        base: BaseContext::new("events", year, &user, &conn)?,
        days: event::load_days(year, &conn)?,
    };

    Ok(Template::render("admin-events", context))
}

#[get("/<year>/students")]
fn students(year: i16, conn: db::Conn, user: User) -> Result<Template> {
    user.ensure_admin_for(year)?;

    let context = student::Context {
        base: BaseContext::new("students", year, &user, &conn)?,
        students: student::load_students(year, &conn)?,
    };

    Ok(Template::render("admin-students", context))
}

#[get("/<year>/tutors")]
fn tutors(year: i16, ip_whitelisting: State<IpWhitelisting>, conn: db::Conn, user: SiteAdmin) -> Result<Template> {
    let ip_whitelist = match ip_whitelisting.0 {
        true => Some(tutor::load_whitelist(year, &conn)?),
        false => None,
    };

    let context = tutor::Context {
        base: BaseContext::new("tutors", year, &user, &conn)?,
        tutors: tutor::load_tutors(year, &conn)?,
        ip_whitelist: ip_whitelist,
    };

    Ok(Template::render("admin-tutors", context))
}

#[get("/<year>/audit")]
fn audit_index(year: i16, _user: SiteAdmin) -> Redirect {
    Redirect::to(&format!("/admin/{}/audit?limit=100", year))
}

#[get("/<year>/audit?<filters>")]
fn audit(year: i16, filters: audit::Filters, conn: db::Conn, user: SiteAdmin) -> Result<Template> {
    let context = audit::Context {
        base: BaseContext::new("audit", year, &user, &conn)?,
        logs: audit::load_logs(year, &filters, &conn)?,
        filters: filters,
        authors: audit::load_authors(&conn)?,
    };

    Ok(Template::render("admin-audit", context))
}

#[get("/<year>/export")]
fn export(year: i16, conn: db::Conn, _user: SiteAdmin) -> Result<export::CsvResponse> {
    let name = format!("hwpb-export-{}.csv", Local::today().format("%Y-%m-%d"));
    let csv = export::create_csv(year, &conn)?;

    Ok(export::CsvResponse {
        filename: name,
        content: csv
    })
}
