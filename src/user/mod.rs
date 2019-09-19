use crate::schema::users;
use diesel::prelude::*;
use serde::Serialize;

use bcrypt::{hash, DEFAULT_COST};

use crate::api::ApiResponse;
use rocket::http::Status;

pub mod controller;

#[table_name = "users"]
#[derive(AsChangeset, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
}

// our mass assignable properties
#[table_name = "users"]
#[derive(Serialize, Deserialize, Insertable)]
pub struct InsertableUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl InsertableUser {
    pub fn create(user: InsertableUser, connection: &PgConnection) -> ApiResponse {
        match hash(user.password, DEFAULT_COST) {
            Ok(hashed) => user.password = hashed,
            Err(_) => ApiResponse {
                json: json!({"success": false, "error": "error hashing", "data": null}),
                status: Status::InternalServerError,
            },
        };
        let result = diesel::insert_into(users::table)
            .values(&user)
            .get_result::<User>(connection);
            .map_err(Into::into)
        match result {
            Ok(User) => ApiResponse {
                json: json!({"success": true, "error": null, "data": User}),
                status: Status::Ok,
            },
            Err(error) => {
                println!("Cannot create the recipe: {:?}", error);
                ApiResponse {
                    json: json!({"success": false, "error": error.to_string() }),
                    status: Status::UnprocessableEntity,
                }
            }
        }
    }
}

impl User {
    pub fn read(connection: &PgConnection) -> Vec<User> {
        users::table
            .order(users::id)
            .load::<User>(connection)
            .unwrap()
    }

    pub fn update(id: i32, user: User, connection: &PgConnection) -> bool {
        diesel::update(users::table.find(id))
            .set(&user)
            .execute(connection)
            .is_ok()
    }

    pub fn delete(id: i32, connection: &PgConnection) -> bool {
        diesel::delete(users::table.find(id))
            .execute(connection)
            .is_ok()
    }
}
