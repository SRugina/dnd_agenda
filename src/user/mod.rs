use crate::schema::users;
use diesel::prelude::*;

use bcrypt::{hash, DEFAULT_COST};

use crate::api::ApiResponse;
use rocket::http::Status;

pub mod controller;

#[table_name = "users"]
#[derive(AsChangeset, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
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

#[table_name = "users"]
#[derive(Serialize, Deserialize, Insertable)]
pub struct InsertableUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl InsertableUser {
    pub fn create(mut user: InsertableUser, connection: &PgConnection) -> ApiResponse {
        match hash(user.password, DEFAULT_COST) {
            Ok(hashed) => user.password = hashed,
            Err(_) => return ApiResponse {
                json: json!({"error": "error hashing"}),
                status: Status::InternalServerError,
            },
        };

        let result = diesel::insert_into(users::table)
            .values(&user)
            .get_result::<User>(connection);

        match result {
            Ok(user) => return ApiResponse {
                json: json!({"data": user}),
                status: Status::Created,
            },
            Err(error) => {
                println!("Cannot create the recipe: {:?}", error);
                return ApiResponse {
                    json: json!({"error": error.to_string() }),
                    status: Status::UnprocessableEntity,
                }
            }
        }
    }
}

