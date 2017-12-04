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
