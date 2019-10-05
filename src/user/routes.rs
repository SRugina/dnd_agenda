use crate::database::DnDAgendaDB;
use crate::user;

use rocket_contrib::json::Json;
use rocket_contrib::json::JsonError;
use rocket_contrib::json::JsonValue;

use crate::api::ApiResponse;
use crate::api::Auth;
use rocket::http::Status;

use crate::api::FieldValidator;
use validator::Validate;

#[get("/")]
pub fn get_all(
    auth: Result<Auth, JsonValue>,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(_auth) => user::User::read(&connection)
            .map(|user| ApiResponse {
                json: json!({ "user": user }),
                status: Status::Ok,
            })
            .map_err(|response| response),
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[get("/self")]
pub fn get_self(
    auth: Result<Auth, JsonValue>,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => user::User::find(auth.id, &connection)
            .map(|user| ApiResponse {
                json: json!({ "user": user }),
                status: Status::Ok,
            })
            .map_err(|response| response),
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[get("/<user_id>/sessions")]
pub fn get_sessions(
    auth: Result<Auth, JsonValue>,
    user_id: i32,
    connection: DnDAgendaDB,
) -> ApiResponse {
    match auth {
        Ok(_auth) => match user::User::read_sessions(user_id, &connection) {
            Ok(sessions) => ApiResponse {
                json: json!({ "sessions": sessions }),
                status: Status::Ok,
            },
            Err(response) => response,
        },
        Err(auth_error) => ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        },
    }
}

#[derive(Deserialize, Validate)]
pub struct NewUserData {
    #[validate(length(min = 1, code = "Username must be at least 1 character long"))]
    pub username: Option<String>,
    #[validate(email(code = "Email is not a valid email"))]
    pub email: Option<String>,
    #[validate(length(min = 8, code = "Password must be at least 8 characters long"))]
    pub password: Option<String>,
}

#[post("/", format = "application/json", data = "<user>")] // data attribute tells rocket to expect Body Data - then map the body to a parameter
pub fn create(
    user: Result<Json<NewUserData>, JsonError>,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    let new_user = user.map_err(|json_error| {
        match json_error {
            JsonError::Parse(_req, err) => ApiResponse {
                json: json!({ "error": err.to_string() }),
                status: Status::BadRequest,
            },
            JsonError::Io(_err) => ApiResponse {
                json: json!({ "error": "I/O error occured while reading the incoming request data" }),
                status: Status::InternalServerError,
            },
        }
    })?.into_inner();

    let empty_flag = false; //i.e. emit error if empty
    let mut extractor = FieldValidator::validate(&new_user);
    let username = extractor.extract("username", new_user.username, empty_flag);
    let email = extractor.extract("email", new_user.email, empty_flag);
    let password = extractor.extract("password", new_user.password, empty_flag);

    extractor.check()?;

    let insertable_user = user::InsertableUser {
        username,
        email,
        password,
    };

    user::InsertableUser::create(insertable_user, &connection)
        .map(|user| ApiResponse {
            json: json!({ "user": user.to_user_auth() }),
            status: Status::Created,
        })
        .map_err(|response| response)
}

#[derive(Deserialize)]
pub struct LoginUserData {
    email: Option<String>,
    password: Option<String>,
}

#[post("/login", format = "application/json", data = "<user>")]
pub fn login(
    user: Result<Json<LoginUserData>, JsonError>,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    let login_user = user.map_err(|json_error| {
        match json_error {
            JsonError::Parse(_req, err) => ApiResponse {
                json: json!({ "error": err.to_string() }),
                status: Status::BadRequest,
            },
            JsonError::Io(_err) => ApiResponse {
                json: json!({ "error": "I/O error occured while reading the incoming request data" }),
                status: Status::InternalServerError,
            },
        }
    })?.into_inner();

    let empty_flag = false; // i.e. emit error if empty
    let mut extractor = FieldValidator::default();
    let email = extractor.extract("email", login_user.email, empty_flag);
    let password = extractor.extract("password", login_user.password, empty_flag);
    extractor.check()?;

    user::User::login(&email, &password, &connection)
        .map(|user| ApiResponse {
            json: json!({ "user": user.to_user_auth() }),
            status: Status::Accepted,
        })
        .map_err(|response| response)
}

#[derive(Deserialize, Validate, Clone)]
pub struct UpdateUserData {
    #[validate(length(min = 1, code = "Username must be at least 1 character long"))]
    username: Option<String>,
    #[validate(email(code = "Email must be a valid email"))]
    email: Option<String>,
    bio: Option<String>,
    #[validate(url(code = "Image must be a valid url"))]
    image: Option<String>,
}

#[put("/self", format = "application/json", data = "<user>")]
pub fn put_self(
    auth: Result<Auth, JsonValue>,
    user: Result<Json<UpdateUserData>, JsonError>,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            let user_details = user.map_err(|json_error| {
                match json_error {
                    JsonError::Parse(_req, err) => ApiResponse {
                        json: json!({ "error": err.to_string() }),
                        status: Status::BadRequest,
                    },
                    JsonError::Io(_err) => ApiResponse {
                        json: json!({ "error": "I/O error occured while reading the incoming request data" }),
                        status: Status::InternalServerError,
                    },
                }
            })?.into_inner();

            let user_validator_details = user_details.clone();

            let empty_flag = true; // i.e. do not emit error if empty
            let mut extractor = FieldValidator::validate(&user_details);
            let _username =
                extractor.extract("username", user_validator_details.username, empty_flag);
            let _email = extractor.extract("email", user_validator_details.email, empty_flag);
            let _image = extractor.extract("image", user_validator_details.image, empty_flag);

            extractor.check()?;

            //don't use values above because we want to pass on the Option<>, if extractor fails this won't execute anyway
            let update_user = user::UpdateUser {
                username: user_details.username,
                email: user_details.email,
                bio: user_details.bio,
                image: user_details.image,

                password: None,
            };

            user::UpdateUser::update(auth.id, &update_user, &connection)
                .map(|user| ApiResponse {
                    json: json!({ "user": user }),
                    status: Status::Ok,
                })
                .map_err(|response| response)
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[derive(Deserialize, Validate, Clone)]
pub struct UpdateUserPasswordData {
    old_password: Option<String>,
    #[validate(length(min = 8, code = "Password must be at least 8 characters long"))]
    password: Option<String>,
}

#[put("/self/pwd", format = "application/json", data = "<user>")]
pub fn put_pwd_of_self(
    auth: Result<Auth, JsonValue>,
    user: Result<Json<UpdateUserPasswordData>, JsonError>,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            let user_details = user.map_err(|json_error| {
                match json_error {
                    JsonError::Parse(_req, err) => ApiResponse {
                        json: json!({ "error": err.to_string() }),
                        status: Status::BadRequest,
                    },
                    JsonError::Io(_err) => ApiResponse {
                        json: json!({ "error": "I/O error occured while reading the incoming request data" }),
                        status: Status::InternalServerError,
                    },
                }
            })?.into_inner();

            let user_validator_details = user_details.clone();

            let empty_flag1 = true; // i.e. do not emit error if empty
            let mut extractor1 = FieldValidator::validate(&user_details);
            let _password =
                extractor1.extract("password", user_validator_details.password, empty_flag1);

            extractor1.check()?;

            let empty_flag2 = false; // i.e. emit error if empty
            let mut extractor2 = FieldValidator::default();
            let _old_password = extractor2.extract(
                "old_password",
                user_validator_details.old_password,
                empty_flag2,
            );
            extractor2.check()?;

            //don't use values above because we want to pass on the Option<>, if extractor fails this won't execute anyway
            let update_user = user::UpdateUser {
                username: None,
                email: None,
                bio: None,
                image: None,

                password: user_details.password,
            };

            user::UpdateUser::update(auth.id, &update_user, &connection)
                .map(|user| ApiResponse {
                    json: json!({ "user": user }),
                    status: Status::Ok,
                })
                .map_err(|response| response)
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[get("/<username>/profile", format = "application/json")]
pub fn get_profile(
    auth: Result<Auth, JsonValue>,
    username: String,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(_auth) => user::User::find_profile(&username, &connection)
            .map(|profile| ApiResponse {
                json: json!({ "profile": profile }),
                status: Status::Ok,
            })
            .map_err(|response| response),
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[delete("/self")]
pub fn delete_self(
    auth: Result<Auth, JsonValue>,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => user::User::delete(auth.id, &connection)
            .map(|_| ApiResponse {
                json: json!({ "message": "user deleted successfully" }),
                status: Status::Ok,
            })
            .map_err(|response| response),
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}
