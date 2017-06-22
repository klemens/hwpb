#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate chrono;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate dotenv;
extern crate pam_auth;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

mod db;
mod errors;
mod user;
mod web;

fn main() {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL not set");

    // run any pending database migrations
    db::run_migrations(&database_url);

    rocket::ignite()
        .manage(db::init_pool(&database_url))
        .mount("/", routes![
            web::index,
            web::event,
            web::static_file,
            web::session::nologin_index,
            web::session::nologin_path,
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
            web::api::put_comment,
            web::api::put_group_student,
            web::api::delete_group_student,
            web::api::search_students,
        ])
        .attach(rocket_contrib::Template::fairing())
        .attach(rocket::fairing::AdHoc::on_attach(|rocket| {
            let allowed_users = {
                let default = vec![];
                let users = rocket.config().get_slice("allowed_users").unwrap_or(&default);
                let users = users.iter().filter_map(|u| u.as_str());
                web::session::AllowedUsers::new(users)
            };
            Ok(rocket.manage(allowed_users))
        }))
        .launch();
}
