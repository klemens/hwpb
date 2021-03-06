#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;

mod db;
mod errors;
mod user;
mod web;

use crate::errors::*;
use crate::web::session::{IpWhitelisting, LoginMessage};
use crate::web::push;

quick_main!(run);

fn run() -> Result<()> {
    let rocket = rocket::ignite();

    let database_url = rocket.config().get_str("database")
        .chain_err(|| "DATABASE_URL not set")?
        .to_owned();

    // check if we should truncate the database (delete all years)
    let truncate_database = rocket.config()
        .get_bool("truncate_database_on_start")
        .unwrap_or(false);

    // run any pending database migrations
    db::run_migrations(&database_url)?;

    // add current year on first run
    db::init_year(truncate_database, &database_url)?;

    // check if ip whitelisting is enabled (default is disabled)
    let ip_whitelisting = rocket.config().get_bool("ip_whitelisting")
        .unwrap_or(false);
    // get the login message if one is set
    let login_message = rocket.config().get_str("login_message")
        .map(String::from).ok();

    // start push server
    let (push_url, listen_addr) = push::parameters(rocket.config())?;
    push::SERVER.spawn(listen_addr);

    rocket
        .manage(db::init_pool(&database_url)?)
        .manage(IpWhitelisting(ip_whitelisting))
        .manage(LoginMessage(login_message))
        .manage(push_url)
        .mount("/", routes![
            web::index,
            web::overview,
            web::event_finder,
            web::event,
            web::group,
            web::static_file,
            web::manifest,
            web::service_worker,
            web::session::nologin_index,
            web::session::nologin_path,
            web::session::login_redirect,
            web::session::get_login,
            web::session::post_login,
            web::session::logout,
        ])
        .mount("/api", routes![
            web::api::post_group,
            web::api::put_completion,
            web::api::delete_completion,
            web::api::put_elaboration,
            web::api::delete_elaboration,
            web::api::put_group_comment,
            web::api::put_group_desk,
            web::api::put_group_student,
            web::api::delete_group_student,
            web::api::search_groups,
            web::api::search_students,
            web::api::put_year,
            web::api::delete_year,
            web::api::put_year_writable,
            web::api::post_experiment,
            web::api::delete_experiment,
            web::api::post_experiment_task,
            web::api::delete_experiment_task,
            web::api::put_event,
            web::api::delete_event,
            web::api::post_day,
            web::api::delete_day,
            web::api::post_student,
            web::api::post_students_csv,
            web::api::delete_student,
            web::api::put_student_instucted,
            web::api::post_tutor,
            web::api::delete_tutor,
            web::api::put_tutor_admin,
            web::api::post_ip_whitelist,
            web::api::delete_ip_whitelist,
        ])
        .mount("/analysis", routes![
            web::analysis::passed,
            web::analysis::passed_complete,
            web::analysis::missing_reworks,
        ])
        .mount("/admin", routes![
            web::admin::index,
            web::admin::experiments,
            web::admin::events,
            web::admin::students,
            web::admin::students_ordered,
            web::admin::tutors,
            web::admin::audit_index,
            web::admin::audit,
            web::admin::export,
        ])
        .attach(rocket_contrib::templates::Template::fairing())
        .attach(rocket::fairing::AdHoc::on_attach("Load admins", |rocket| {
            match web::session::load_site_admins(rocket.config()) {
                Ok(site_admins) => Ok(rocket.manage(site_admins)),
                Err(error) => {
                    eprintln!("{}", error);
                    Err(rocket)
                }
            }
        }))
        .launch();

    Ok(())
}
