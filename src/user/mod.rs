use crate::schema::users;
use diesel::prelude::*;

use bcrypt::{hash, verify, DEFAULT_COST};

use crate::api::ApiResponse;
use rocket::http::Status;

use crate::api::Auth;
use chrono::{Duration, Utc};

use validator::Validate;

use crate::api::FieldValidator;

pub mod routes;

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

    pub fn read(connection: &PgConnection) -> ApiResponse {
        // users::table
        //     .order(users::id)
        //     .load::<User>(connection)
        //     .unwrap()
        return ApiResponse {
            json: json!({"yup": "indeed" }),
            status: Status::Ok,
        };
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
#[derive(Serialize, Deserialize, Insertable, Validate)]
pub struct InsertableUser {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
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
    pub fn create(mut user: InsertableUser, connection: &PgConnection) -> ApiResponse {

        let new_user = user.user;

        let mut extractor = FieldValidator::validate(&new_user);
        let username = extractor.extract("username", new_user.username);
        let email = extractor.extract("email", new_user.email);
        let password = extractor.extract("password", new_user.password);

        extractor.check()?;

        match hash(user.password, DEFAULT_COST) {
            Ok(hashed) => user.password = hashed,
            Err(_) => {
                return ApiResponse {
                    json: json!({"error": "error hashing"}),
                    status: Status::InternalServerError,
                }
            }
        };

        let result: Result<User, UserCreationError> = diesel::insert_into(users::table)
            .values(&user.user)
            .get_result::<User>(connection)
            .map_err(Into::into);

        match result {
            Ok(user) => {
                return ApiResponse {
                    json: json!({ "user": user }),
                    status: Status::Created,
                }
            }
            Err(error) => {
                let field = match error {
                    UserCreationError::DuplicatedEmail => "email",
                    UserCreationError::DuplicatedUsername => "username",
                };
                println!("Cannot create user: {:#?}", error);
                return ApiResponse {
                    json: json!({"error": format!("{} already taken", field) }),
                    status: Status::UnprocessableEntity,
                };
            }
        };
    }
}
