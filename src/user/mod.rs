use crate::schema::users;
use diesel::prelude::*;

use bcrypt::{hash, verify, DEFAULT_COST};

use crate::api::ApiResponse;
use rocket::http::Status;

use crate::api::Auth;
use chrono::{Duration, Utc};

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

#[derive(Serialize)]
pub struct UserAuth<'a> {
    username: &'a str,
    email: &'a str,
    token: String,
}

impl User {
    pub fn to_user_auth(&self) -> UserAuth {
        let exp = Utc::now() + Duration::days(60); // TODO: move to config
        let token = Auth {
            id: self.id,
            username: self.username.clone(),
            exp: exp.timestamp(),
        }
        .token();

        UserAuth {
            username: &self.username,
            email: &self.email,
            token,
        }
    }
    pub fn login(email: &str, password: &str, connection: &PgConnection) -> ApiResponse {
        match users::table
            .filter(users::email.eq(email))
            .get_result::<User>(connection)
        {
            Ok(user) => match verify(password, &user.password) {
                Ok(password_matches) => {
                    if password_matches {
                        return ApiResponse {
                            json: json!({ "user": user.to_user_auth() }),
                            status: Status::Accepted,
                        };
                    } else {
                        return ApiResponse {
                            json: json!({ "error": "incorrect email/password" }),
                            status: Status::Unauthorized,
                        };
                    }
                }
                Err(error) => {
                    println!("Error: {}", error);
                    return ApiResponse {
                        json: json!({"error": "verifying failed" }),
                        status: Status::InternalServerError,
                    };
                }
            },
            Err(error) => {
                // the email was wrong.(not found)
                println!("Error: {}", error);
                return ApiResponse {
                    json: json!({"error": "incorrect email/password" }),
                    status: Status::Unauthorized,
                };
            }
        };
    }

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
            Err(_) => {
                return ApiResponse {
                    json: json!({"error": "error hashing"}),
                    status: Status::InternalServerError,
                }
            }
        };

        let result = diesel::insert_into(users::table)
            .values(&user)
            .get_result::<User>(connection);

        match result {
            Ok(user) => {
                return ApiResponse {
                    json: json!({ "data": user }),
                    status: Status::Created,
                }
            }
            Err(error) => {
                println!("Cannot create user: {:?}", error);
                return ApiResponse {
                    json: json!({"error": error.to_string() }),
                    status: Status::UnprocessableEntity,
                };
            }
        };
    }
}
