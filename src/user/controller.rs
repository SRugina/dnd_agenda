use crate::database::DnDAgendaDB;
use crate::user;

use rocket_contrib::json::Json;

use crate::api::ApiResponse;

// #[get("/")]
// pub fn read(connection: DnDAgendaDB) -> Json<JsonValue> {
//     user::User::read(&connection)
// }

#[post("/", format = "application/json", data = "<user>")] // data attribute tells rocket to expect Body Data - then map the body to a parameter
pub fn create(user: Json<user::InsertableUser>, connection: DnDAgendaDB) -> ApiResponse {
    user::InsertableUser::create(user.into_inner(), &connection)
}

#[derive(Serialize, Deserialize)]
pub struct LoginUser {
    email: String,
    password: String,
}

#[post("/login", format = "application/json", data = "<user>")]
pub fn login(user: Json<LoginUser>, connection: DnDAgendaDB) -> ApiResponse {
    let user_details = user.into_inner();
    user::User::login(&user_details.email, &user_details.password, &connection)
}
