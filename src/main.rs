#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

pub mod db;
pub mod web;

use dotenv::dotenv;

use db::*;

fn main() {
    dotenv().ok();

    rocket::ignite()
        .manage(db::init_pool())
        .mount("/", routes![ web::index, web::event, web::static_file ])
        .mount("/api", routes![
            web::api::put_completion,
            web::api::delete_completion,
            web::api::put_elaboration,
            web::api::delete_elaboration,
            web::api::put_comment,
        ])
        .launch();
}
