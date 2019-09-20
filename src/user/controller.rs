use crate::database::DnDAgendaDB;
use crate::user;

use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::SerdeError;


use crate::api::ApiResponse;


#[get("/")]
pub fn read(connection: DnDAgendaDB) -> Json<JsonValue> {
    user::User::read(&connection);
}

#[post("/", data = "<user>")] // data attribute tells rocket to expect Body Data - then map the body to a parameter
pub fn create(user: Result<Json<user::InsertableUser>, SerdeError>, connection: DnDAgendaDB) -> ApiResponse {
    user::InsertableUser::create(user.into_inner(), &connection)
}

#[derive(Deserialize)]
pub struct AuthUser {
    username: String,
    password: String,
}

#[post("/login", data = "<user>")]
pub fn login(user: Json<user::InsertableUser>, connection: DnDAgendaDB) -> ApiResponse {

    user::InsertableUser::create(user.into_inner(), &connection)
}