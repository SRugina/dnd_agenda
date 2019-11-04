use crate::schema::sessions;
use crate::schema::users;
use diesel::prelude::*;

use bcrypt::{hash, verify, DEFAULT_COST};

use crate::group::{Group, GroupUser};
use crate::schema::{groups, groups_users};

use crate::api::ApiResponse;
use rocket::http::Status;

use crate::api::Auth;
use chrono::{Duration, Utc};

use crate::session;

type Url = String;

pub mod routes;

use crate::config::DEFAULT_LIMIT;

use crate::database::dsl;

use crate::database::Paginate;

use itertools::Itertools;

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

#[derive(FromForm, Default)]
pub struct FindUsers {
    global_search: Option<bool>,
    username: Option<String>,
    limit: Option<i64>,
    page: Option<i64>,
}

#[derive(Serialize)]
pub struct UserAuth<'a> {
    username: &'a str,
    email: &'a str,
    bio: Option<&'a str>,
    image: Option<&'a str>,
    token: String,
}

#[derive(Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Profile {
    id: i32, // to get the sessions for the profile
    username: String,
    bio: Option<String>,
    image: Option<String>,
}

impl User {
    pub fn to_user_auth(&self) -> UserAuth {
        let exp = Utc::now() + Duration::hours(1);
        let token = Auth {
            id: self.id,
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

    pub fn to_profile(&self) -> Profile {
        Profile {
            id: self.id,
            username: self.username.clone(),
            bio: self.bio.clone(),
            image: self.image.clone(),
        }
    }

    pub fn check_password(
        old_password: String,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<bool, ApiResponse> {
        let user = users::table
            .find(user_id)
            .first::<User>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User not found" }),
                    status: Status::NotFound,
                }
            })?;

        verify(old_password, &user.password).map_err(|error| {
            println!("Error: {}", error);
            ApiResponse {
                json: json!({"error": "verifying failed", "details": error.to_string() }),
                status: Status::InternalServerError,
            }
        })
    }

    pub fn login(
        email: &str,
        password: &str,
        connection: &PgConnection,
    ) -> Result<User, ApiResponse> {
        let user = users::table
            .filter(users::email.eq(email))
            .get_result::<User>(connection)
            .map_err(|error| {
                println!("Error: {}", error);
                ApiResponse {
                    json: json!({"error": "incorrect email/password" }),
                    status: Status::Unauthorized,
                }
            })?;

        let password_matches = verify(password, &user.password).map_err(|error| {
            println!("Error: {}", error);
            ApiResponse {
                json: json!({"error": "verifying failed", "details": error.to_string() }),
                status: Status::InternalServerError,
            }
        })?;
        if password_matches {
            Ok(user)
        } else {
            Err(ApiResponse {
                json: json!({ "error": "incorrect email/password" }),
                status: Status::Unauthorized,
            })
        }
    }

    pub fn read(
        params: &FindUsers,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<(Vec<Profile>, i64), ApiResponse> {
        if params.global_search.unwrap_or(false) {
            //get all users regardless of what groups the current user is in

            let mut query = users::table.select(users::all_columns).into_boxed();

            if let Some(ref username) = params.username {
                query = query
                    .filter(dsl::similar_to(users::username, username))
                    .order(dsl::similarity(users::username, username).desc())
            }

            query
                .paginate(params.page.unwrap_or(1))
                .per_page(params.limit.unwrap_or(DEFAULT_LIMIT))
                .load_and_count_pages::<User>(connection)
                .map(|(users, pages_count)| {
                    (
                        users.iter().map(|user| user.to_profile()).collect(),
                        pages_count,
                    )
                })
                .map_err(|error| {
                    println!("Error: {:#?}", error);
                    ApiResponse {
                        json: json!({"error": "Users not found" }),
                        status: Status::NotFound,
                    }
                })
        } else {
            let mut pages_count: i64 = 0;

            // get all users belonging to the same groups as the current user

            let user = User::find(user_id, connection).map_err(|response| response)?;

            GroupUser::belonging_to(&user)
                .filter(groups_users::columns::admin_accepted.eq(true))
                .filter(groups_users::columns::user_accepted.eq(true))
                .inner_join(groups::table)
                .select(groups::all_columns)
                .load::<Group>(connection)
                .map_err(|error| {
                    println!("Error: {:#?}", error);
                    ApiResponse {
                        json: json!({"error": "Groups not found" }),
                        status: Status::NotFound,
                    }
                })?
                .iter()
                .map(|group| {
                    let mut query = GroupUser::belonging_to(group)
                        .filter(groups_users::columns::admin_accepted.eq(true))
                        .filter(groups_users::columns::user_accepted.eq(true))
                        .inner_join(users::table) // each user's details
                        .select(users::all_columns)
                        .into_boxed();

                    if let Some(ref username) = params.username {
                        query = query
                            .filter(dsl::similar_to(users::username, username))
                            .order(dsl::similarity(users::username, username).desc())
                    }

                    query
                        .paginate(params.page.unwrap_or(1))
                        .per_page(params.limit.unwrap_or(DEFAULT_LIMIT))
                        .load_and_count_pages::<User>(connection)
                        .map_err(|error| {
                            println!("Error: {:#?}", error);
                            ApiResponse {
                                json: json!({"error": "Users not found" }),
                                status: Status::NotFound,
                            }
                        })
                        .map(|(users, count)| {
                            pages_count += count;
                            users
                        })?
                        .iter()
                        .map(|user| Ok(user.to_profile()))
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|profiles| {
                    (profiles
                        .into_iter()
                        .flatten()
                        .unique_by(|profile| profile.id)
                        .collect(), pages_count)
                })
        }
    }

    pub fn find(user_id: i32, connection: &PgConnection) -> Result<User, ApiResponse> {
        users::table
            .find(user_id)
            .first::<User>(connection)
            .map(|user| user)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User not found" }),
                    status: Status::NotFound,
                }
            })
    }

    pub fn find_profile(username: &str, connection: &PgConnection) -> Result<Profile, ApiResponse> {
        users::table
            .filter(users::username.eq(username))
            .first::<User>(connection)
            .map(|user| user.to_profile())
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User not found" }),
                    status: Status::NotFound,
                }
            })
    }

    pub fn read_sessions(
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<Vec<session::SessionJson>, ApiResponse> {
        let user = User::find(user_id, connection).map_err(|response| response)?;

        session::SessionUser::belonging_to(&user)
            .inner_join(sessions::table)
            .select(sessions::all_columns)
            .order(sessions::session_date.desc())
            .load::<session::Session>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Sessions not found" }),
                    status: Status::NotFound,
                }
            })?
            .iter()
            .map(|session| {
                let dm = User::find(session.dm, connection)
                    .map(|user| user.to_profile())
                    .map_err(|response| response)?;

                session::populate(&session, dm, connection)
                    .map(|session_json| session_json)
                    .map_err(|response| response)
            })
            .collect()
    }

    pub fn delete(user_id: i32, connection: &PgConnection) -> Result<(), ApiResponse> {
        diesel::delete(users::table.find(user_id))
            .execute(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User could not be deleted", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;

        Ok(())
    }
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
        user.password = hash(user.password, DEFAULT_COST).map_err(|error| {
            println!("Cannot hash password: {:#?}", error);
            ApiResponse {
                json: json!({"error": "error hashing" }),
                status: Status::InternalServerError,
            }
        })?;

        diesel::insert_into(users::table)
            .values(&user)
            .get_result::<User>(connection)
            .map(|user| user)
            .map_err(Into::into) // convert to UserCreationError
            .map_err(|error| {
                let field = match error {
                    UserCreationError::DuplicatedEmail => "email",
                    UserCreationError::DuplicatedUsername => "username",
                };
                println!("Cannot create user: {:#?}", error);
                ApiResponse {
                    json: json!({ "errors": { field: [ "has already been taken" ] } }),
                    status: Status::UnprocessableEntity,
                }
            })
    }
}

// TODO: remove clone when diesel will allow skipping fields
#[derive(Deserialize, AsChangeset, Default, Clone)]
#[table_name = "users"]
pub struct UpdateUser {
    username: Option<String>,
    email: Option<String>,
    bio: Option<String>,
    image: Option<String>,

    // hack to skip the field
    password: Option<String>,
}

impl UpdateUser {
    pub fn update(
        id: i32,
        user: &UpdateUser,
        connection: &PgConnection,
    ) -> Result<User, ApiResponse> {
        diesel::update(users::table.find(id))
            .set(user)
            .get_result::<User>(connection)
            .map(|user| user)
            .map_err(|error| {
                println!("Cannot update user: {:#?}", error);
                ApiResponse {
                    json: json!({ "error": "cannot update user" }),
                    status: Status::UnprocessableEntity,
                }
            })
    }
}
