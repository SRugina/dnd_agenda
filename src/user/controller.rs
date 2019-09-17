use crate::database::DnDAgendaDB;
use crate::user;
use rocket_contrib::json::{Json, JsonValue};

use crate::api::ApiResponse;

#[get("/")]
pub fn read(connection: DnDAgendaDB) -> Json<JsonValue> {
    Json(json!(user::User::read(&connection)))
}

#[post("/", data = "<user>")] // data attribute tells rocket to expect Body Data - then map the body to a parameter
pub fn create(user: Json<user::InsertableUser>, connection: DnDAgendaDB) -> ApiResponse {
    user::InsertableUser::create(user.into_inner(), &connection)
}