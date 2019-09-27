#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;

use validator;
#[macro_use]
extern crate validator_derive;

extern crate frank_jwt;

mod api;
mod database;
mod schema;

mod config;

mod session;
mod user;

pub fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount(
            "/api/v1/users",
            routes![
                user::routes::create,
                user::routes::login,
                user::routes::get_all,
                user::routes::get_sessions
            ],
        )
        .attach(database::DnDAgendaDB::fairing())
}
