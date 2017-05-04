extern crate chrono;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate dotenv;

mod db;

use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

use db::*;

fn main() {
    dotenv().ok();
    let connection = PgConnection::establish(&env::var("DATABASE_URL").unwrap()).unwrap();

    let dates = [
        NaiveDate::from_ymd(2017, 4, 17),
        NaiveDate::from_ymd(2017, 4, 18),
        NaiveDate::from_ymd(2017, 4, 20),
    ];

    for date in dates.iter() {
        println!("\nPraktikum am {}:", date);
        print_table(date, &connection).unwrap();
    }
}
