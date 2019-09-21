use crate::database::DnDAgendaDB;
use crate::user;

use rocket_contrib::json::Json;
use rocket_contrib::json::JsonError;

use crate::api::ApiResponse;
use rocket::http::Status;

// #[get("/")]
// pub fn read(connection: DnDAgendaDB) -> Json<JsonValue> {
//     user::User::read(&connection)
// }

#[post("/", format = "application/json", data = "<user>")] // data attribute tells rocket to expect Body Data - then map the body to a parameter
pub fn create(user: Result<Json<user::InsertableUser>, JsonError>, connection: DnDAgendaDB) -> ApiResponse {
    match user {
        Ok(json_user) => {
            return user::InsertableUser::create(json_user.into_inner(), &connection)
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

#[derive(Serialize, Deserialize)]
pub struct LoginUser {
    email: String,
    password: String,
}

#[post("/login", format = "application/json", data = "<user>")]
pub fn login(user: Result<Json<LoginUser>, JsonError>, connection: DnDAgendaDB) -> ApiResponse {
    match user {
        Ok(json_user) => {
            let user_details = json_user.into_inner();
            return user::User::login(&user_details.email, &user_details.password, &connection)
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
