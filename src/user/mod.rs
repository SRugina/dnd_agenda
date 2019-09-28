use crate::schema::sessions;
use crate::schema::users;
use diesel::prelude::*;

use bcrypt::{hash, verify, DEFAULT_COST};

use crate::api::ApiResponse;
use rocket::http::Status;

use crate::api::Auth;
use chrono::{Duration, Utc};

use crate::session;

type Url = String;

pub mod routes;

#[table_name = "users"]
#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
    pub image: Option<Url>,
    #[serde(skip_serializing)]
    pub password: String,
}

#[derive(Serialize)]
pub struct UserAuth<'a> {
    username: &'a str,
    email: &'a str,
    bio: Option<&'a str>,
    image: Option<&'a str>,
    token: String,
}

// #[derive(Serialize)]
// pub struct Profile {
//     username: String,
//     bio: Option<String>,
//     image: Option<String>,
//     following: bool,
// }

impl User {
    pub fn to_user_auth(&self) -> UserAuth {
        let exp = Utc::now() + Duration::hours(1);
        let token = Auth {
            id: self.id,
            username: self.username.clone(),
            exp: exp.timestamp(),
        }
        .token();

        UserAuth {
            username: &self.username,
            email: &self.email,
            bio: self.bio.as_ref().map(String::as_str),
            image: self.image.as_ref().map(String::as_str),
            token,
        }
    }

    // pub fn to_profile(self, following: bool) -> Profile {
    //     Profile {
    //         username: self.username,
    //         bio: self.bio,
    //         image: self.image,
    //         following,
    //     }
    // }

    pub fn login(
        email: &str,
        password: &str,
        connection: &PgConnection,
    ) -> Result<User, ApiResponse> {
        match users::table
            .filter(users::email.eq(email))
            .get_result::<User>(connection)
        {
            Ok(user) => match verify(password, &user.password) {
                Ok(password_matches) => {
                    if password_matches {
                        Ok(user)
                    } else {
                        Err(ApiResponse {
                            json: json!({ "error": "incorrect email/password" }),
                            status: Status::Unauthorized,
                        })
                    }
                }
                Err(error) => {
                    println!("Error: {}", error);
                    Err(ApiResponse {
                        json: json!({"error": "verifying failed" }),
                        status: Status::InternalServerError,
                    })
                }
            },
            Err(error) => {
                // the email was wrong.(not found)
                println!("Error: {}", error);
                Err(ApiResponse {
                    json: json!({"error": "incorrect email/password" }),
                    status: Status::Unauthorized,
                })
            }
        }
    }

    pub fn read(connection: &PgConnection) -> Result<Vec<User>, ApiResponse> {
        match users::table.order(users::id).load::<User>(connection) {
            Ok(users) => Ok(users),
            Err(error) => {
                println!("Error: {:#?}", error);
                Err(ApiResponse {
                    json: json!({"error": "Users not found" }),
                    status: Status::NotFound,
                })
            }
        }
    }

    pub fn find(user_id: i32, connection: &PgConnection) -> Result<User, ApiResponse> {
        match users::table.find(user_id).first::<User>(connection) {
            Ok(user) => Ok(user),
            Err(error) => {
                println!("Error: {:#?}", error);
                Err(ApiResponse {
                    json: json!({"error": "User not found" }),
                    status: Status::NotFound,
                })
            }
        }
    }

    pub fn read_sessions(
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<Vec<session::Session>, ApiResponse> {
        match users::table.find(user_id).first::<User>(connection) {
            Ok(user) => {
                match session::SessionUser::belonging_to(&user)
                    .inner_join(sessions::table)
                    .select(sessions::all_columns)
                    .load::<session::Session>(connection)
                {
                    Ok(sessions) => Ok(sessions),
                    Err(error) => {
                        println!("Error: {:#?}", error);
                        Err(ApiResponse {
                            json: json!({"error": "Sessions not found" }),
                            status: Status::NotFound,
                        })
                    }
                }
            }
            Err(error) => {
                println!("Error: {:#?}", error);
                Err(ApiResponse {
                    json: json!({"error": "User not found" }),
                    status: Status::NotFound,
                })
            }
        }
    }

    // pub fn update(id: i32, user: User, connection: &PgConnection) -> bool {
    //     diesel::update(users::table.find(id))
    //         .set(&user)
    //         .execute(connection)
    //         .is_ok()
    // }

    // pub fn delete(id: i32, connection: &PgConnection) -> bool {
    //     diesel::delete(users::table.find(id))
    //         .execute(connection)
    //         .is_ok()
    // }
}

#[table_name = "users"]
#[derive(Serialize, Deserialize, Insertable)]
pub struct InsertableUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub enum UserCreationError {
    DuplicatedEmail,
    DuplicatedUsername,
}

impl From<diesel::result::Error> for UserCreationError {
    fn from(err: diesel::result::Error) -> UserCreationError {
        if let diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            info,
        ) = &err
        {
            match info.constraint_name() {
                Some("users_username_key") => return UserCreationError::DuplicatedUsername,
                Some("users_email_key") => return UserCreationError::DuplicatedEmail,
                _ => {}
            }
        }
        panic!("Error creating user: {:?}", err)
    }
}

impl InsertableUser {
    pub fn create(
        mut user: InsertableUser,
        connection: &PgConnection,
    ) -> Result<User, ApiResponse> {
        match hash(user.password, DEFAULT_COST) {
            Ok(hashed) => user.password = hashed,
            Err(error) => {
                println!("Cannot hash password: {:#?}", error);
                return Err(ApiResponse {
                    json: json!({"error": "error hashing"}),
                    status: Status::InternalServerError,
                });
            }
        };

        let result: Result<User, UserCreationError> = diesel::insert_into(users::table)
            .values(user)
            .get_result::<User>(connection)
            .map_err(Into::into);

        match result {
            Ok(user) => Ok(user),
            Err(error) => {
                let field = match error {
                    UserCreationError::DuplicatedEmail => "email",
                    UserCreationError::DuplicatedUsername => "username",
                };
                println!("Cannot create user: {:#?}", error);
                Err(ApiResponse {
                    json: json!({ "error": format!("{} has already been taken", field) }),
                    status: Status::UnprocessableEntity,
                })
            }
        }
    }
}
