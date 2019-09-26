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
pub fn get_all(auth: Result<Auth, JsonValue>, connection: DnDAgendaDB) -> ApiResponse {
    match auth {
        Ok(auth) => {
            println!("{:#?}", auth);
            user::User::read(&connection)
        }
        Err(json_error) => ApiResponse {
            json: json_error,
            status: Status::Unauthorized,
        },
    }
}

#[get("/<user_id>/sessions")]
pub fn get_sessions(auth: Result<Auth, JsonValue>, user_id: i32, connection: DnDAgendaDB) -> ApiResponse {
    match auth {
        Ok(_auth) => user::User::read_sessions(user_id, &connection),
        Err(json_error) => ApiResponse {
            json: json_error,
            status: Status::Unauthorized,
        },
    }
}

#[derive(Deserialize)]
pub struct NewUser {
    user: NewUserData,
}

#[derive(Deserialize, Validate)]
pub struct NewUserData {
    #[validate(length(min = 1, message = "Username must be at least 1 character long"))]
    pub username: Option<String>,
    #[validate(email(message = "Email is not a valid email"))]
    pub email: Option<String>,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: Option<String>,
}

#[post("/", format = "application/json", data = "<user>")] // data attribute tells rocket to expect Body Data - then map the body to a parameter
pub fn create(user: Result<Json<NewUser>, JsonError>, connection: DnDAgendaDB) -> ApiResponse {
    match user {
        Ok(json_user) => {
            let new_user = json_user.into_inner().user;

            let mut extractor = FieldValidator::validate(&new_user);
            let username = extractor.extract("username", new_user.username);
            let email = extractor.extract("email", new_user.email);
            let password = extractor.extract("password", new_user.password);

            let check = extractor.check();
            match check {
                Ok(_) => {
                    let insertable_user = user::InsertableUser {
                        username,
                        email,
                        password,
                    };
                    user::InsertableUser::create(insertable_user, &connection)
                }
                Err(response) => response,
            }
        }
        Err(json_error) => match json_error {
            JsonError::Parse(_req, err) => ApiResponse {
                json: json!({ "error": err.to_string() }),
                status: Status::BadRequest,
            },
            JsonError::Io(_err) => ApiResponse {
                json: json!({ "error": "I/O error occured while reading the incoming request data" }),
                status: Status::InternalServerError,
            },
        },
    }
}

#[derive(Deserialize)]
pub struct LoginUser {
    user: LoginUserData,
}

#[derive(Deserialize)]
struct LoginUserData {
    email: Option<String>,
    password: Option<String>,
}

#[post("/login", format = "application/json", data = "<user>")]
pub fn login(user: Result<Json<LoginUser>, JsonError>, connection: DnDAgendaDB) -> ApiResponse {
    match user {
        Ok(json_user) => {
            let user_details = json_user.into_inner().user;
            let mut extractor = FieldValidator::default();
            let email = extractor.extract("email", user_details.email);
            let password = extractor.extract("password", user_details.password);
            let check = extractor.check();

            match check {
                Ok(_) => user::User::login(&email, &password, &connection),
                Err(response) => response,
            }
        }
        Err(json_error) => match json_error {
            JsonError::Parse(_req, err) => ApiResponse {
                json: json!({ "error": err.to_string() }),
                status: Status::BadRequest,
            },
            JsonError::Io(_err) => ApiResponse {
                json: json!({ "error": "I/O error occured while reading the incoming request data" }),
                status: Status::InternalServerError,
            },
        },
    }
}
