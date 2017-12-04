mod audit;
mod event;

use db;
use errors::*;
use rocket::response::Redirect;
use rocket_contrib::Template;
use web::session::User;
use web::models;

#[get("/<year>")]
fn index(year: i16, _user: User) -> Redirect {
    Redirect::to(&format!("/admin/{}/events", year))
}

#[get("/<year>/events")]
fn events(year: i16, conn: db::Conn, _user: User) -> Result<Template> {
    let context = event::Context {
        site: "events",
        year: year,
        years: models::find_years(&*conn)?,
    };

    Ok(Template::render("admin-events", context))
}

#[get("/<year>/audit")]
fn audit_index(year: i16, _user: User) -> Redirect {
    Redirect::to(&format!("/admin/{}/audit?", year))
}

#[get("/<year>/audit?<filters>")]
fn audit(year: i16, filters: audit::Filters, conn: db::Conn, _user: User) -> Result<Template> {
    let context = audit::Context {
        site: "audit",
        year: year,
        logs: audit::load_logs(year, &filters, &conn)?,
        filters: filters,
        years: models::find_years(&conn)?,
        authors: audit::load_authors(&conn)?,
    };

    Ok(Template::render("admin-audit", context))
}
