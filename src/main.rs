#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

mod api;
mod database;
mod schema;

mod user;

fn main() {
    rocket::ignite()
        .attach(database::DnDAgendaDB::fairing())
        .mount(
            "/api/v1/users",
            routes![
            user::controller::read,
            user::controller::create
            ]
        )
        .launch();
}