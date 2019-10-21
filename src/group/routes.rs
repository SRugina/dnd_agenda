use crate::database::DnDAgendaDB;
use crate::group;

use rocket_contrib::json::Json;
use rocket_contrib::json::JsonError;
use rocket_contrib::json::JsonValue;

use crate::api::ApiResponse;
use crate::api::Auth;
use rocket::http::Status;

use crate::api::validate_user_exists;
use crate::api::FieldValidator;
use validator::Validate;

#[get("/")]
pub fn get_all(
    auth: Result<Auth, JsonValue>,
    connection: DnDAgendaDB,
) -> ApiResponse {
    match auth {
        Ok(_auth) => match group::Group::read(&connection) {
            Ok(groups) => ApiResponse {
                json: json!({ "groups": groups, "groupsCount": groups.len()  }),
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

#[get("/<group_id>")]
pub fn get_group(
    auth: Result<Auth, JsonValue>,
    group_id: i32,
    connection: DnDAgendaDB,
) -> ApiResponse {
    match auth {
        Ok(_auth) => match group::Group::find_as_json(group_id, &connection) {
            Ok(group_json) => ApiResponse {
                json: json!({ "group": group_json }),
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