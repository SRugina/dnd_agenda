use crate::database::DnDAgendaDB;
use crate::group;

use rocket_contrib::json::Json;
use rocket_contrib::json::JsonError;
use rocket_contrib::json::JsonValue;

use crate::api::ApiResponse;
use crate::api::Auth;
use rocket::http::Status;

use crate::api::validate_colour;
use crate::api::validate_user_exists;
use crate::api::FieldValidator;
use validator::Validate;

