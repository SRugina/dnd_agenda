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

mod group;
mod session;
mod user;

pub fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount(
            "/api/v1/users",
            routes![
                user::routes::create,
                user::routes::login,
                user::routes::get_self,
                user::routes::get_all,
                user::routes::get_sessions,
                user::routes::patch_self,
                user::routes::patch_pwd_of_self,
                user::routes::delete_self,
                user::routes::get_profile,
            ],
        )
        .mount(
            "/api/v1/sessions",
            routes![
                session::routes::create,
                session::routes::get_session,
                session::routes::get_all,
                session::routes::patch_session,
                session::routes::patch_dm_of_session,
                session::routes::delete_session,
                session::routes::get_users,
                session::routes::join_session,
                session::routes::accept_to_session,
                session::routes::invite_to_session,
                session::routes::accept_invite_to_session,
                session::routes::leave_session,
                session::routes::remove_user_from_session,
                session::routes::get_guest_link,
                session::routes::get_session_as_guest,
                session::routes::get_guests,
                session::routes::remove_guest_from_session,
            ],
        )
        .mount(
            "/api/v1/groups",
            routes![
                group::routes::create,
                group::routes::get_group,
                group::routes::get_all,
                group::routes::patch_group,
                group::routes::patch_admin_of_group,
                group::routes::delete_group,
                group::routes::join_group,
                group::routes::accept_to_group,
                group::routes::invite_to_group,
                group::routes::accept_invite_to_group,
                group::routes::leave_group,
                group::routes::remove_user_from_group,
            ],
        )
        .attach(database::DnDAgendaDB::fairing())
}
