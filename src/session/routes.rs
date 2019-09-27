use crate::database::DnDAgendaDB;
use crate::session;

use rocket_contrib::json::Json;
use rocket_contrib::json::JsonError;
use rocket_contrib::json::JsonValue;

use crate::api::ApiResponse;
use crate::api::Auth;
use rocket::http::Status;

// use crate::api::validate_session_date;
use crate::api::FieldValidator;
use validator::Validate;

#[derive(Deserialize)]
pub struct NewSession {
    user: NewSessionData,
}

#[derive(Deserialize, Validate)]
pub struct NewSessionData {
    #[validate(length(min = 1, message = "Title must be at least 1 character long"))]
    pub title: Option<String>,
    #[validate(length(min = 1, message = "Description must be at least 1 character long"))]
    pub description: Option<String>,
    pub dm: Option<i32>,
    // #[validate(custom = "validate_session_date")]
    pub session_date: Option<String>,
}

#[post("/", format = "application/json", data = "<session>")] // data attribute tells rocket to expect Body Data - then map the body to a parameter
pub fn create(
    session: Result<Json<NewSession>, JsonError>,
    connection: DnDAgendaDB,
) -> ApiResponse {
    ApiResponse {
        json: json!({ "testing": "yes" }),
        status: Status::Ok,
    }
}
