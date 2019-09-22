use crate::database::DnDAgendaDB;
use crate::user;

use rocket_contrib::json::Json;
use rocket_contrib::json::JsonError;
use rocket_contrib::json::JsonValue;

use crate::api::ApiResponse;
use crate::api::Auth;
use rocket::http::Status;

#[get("/")]
pub fn get_all(auth: Result<Auth, JsonValue>, connection: DnDAgendaDB) -> ApiResponse {
    match auth {
        Ok(auth) => {
            println!("Auth: {:#?}", auth);
            return user::User::read(&connection);
        }
        Err(json_error) => {
            return ApiResponse {
                json: json_error,
                status: Status::Unauthorized,
            }
        }
    }
}

#[post("/", format = "application/json", data = "<user>")] // data attribute tells rocket to expect Body Data - then map the body to a parameter
pub fn create(
    user: Result<Json<user::InsertableUser>, JsonError>,
    connection: DnDAgendaDB,
) -> ApiResponse {
    match user {
        Ok(json_user) => return user::InsertableUser::create(json_user.into_inner(), &connection),
        Err(json_error) => match json_error {
            JsonError::Parse(_req, err) => {
                return ApiResponse {
                    json: json!({ "error": err.to_string() }),
                    status: Status::BadRequest,
                }
            }
            JsonError::Io(_err) => {
                return ApiResponse {
                    json: json!({ "error": "I/O error occured while reading the incoming request data" }),
                    status: Status::InternalServerError,
                }
            }
        },
    };
}

#[derive(Deserialize)]
pub struct LoginUser {
    user: LoginUserData,
}

#[derive(Deserialize)]
struct LoginUserData {
    email: String,
    password: String,
}

#[post("/login", format = "application/json", data = "<user>")]
pub fn login(user: Result<Json<LoginUser>, JsonError>, connection: DnDAgendaDB) -> ApiResponse {
    match user {
        Ok(json_user) => {
            let user_details = json_user.into_inner().user;
            return user::User::login(&user_details.email, &user_details.password, &connection);
        }
        Err(json_error) => match json_error {
            JsonError::Parse(_req, err) => {
                return ApiResponse {
                    json: json!({ "error": err.to_string() }),
                    status: Status::BadRequest,
                }
            }
            JsonError::Io(_err) => {
                return ApiResponse {
                    json: json!({ "error": "I/O error occured while reading the incoming request data" }),
                    status: Status::InternalServerError,
                }
            }
        },
    };
}
