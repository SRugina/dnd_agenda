use crate::schema::groups;
use crate::schema::groups_sessions;
use crate::schema::groups_users;

use crate::session::Session;
use crate::user::User;

pub mod routes;

use crate::api::ApiResponse;
use rocket::http::Status;

#[table_name = "groups"]
#[derive(Debug, Identifiable, AsChangeset, Serialize, Deserialize, Queryable)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub admin: i32,
}

#[derive(Identifiable, Queryable, Debug, Associations, Serialize, Deserialize)]
#[belongs_to(Group)]
#[belongs_to(Session)]
#[primary_key(group_id, session_id)]
#[table_name = "groups_sessions"]
pub struct GroupSession {
    pub group_id: i32,
    pub session_id: i32,
}

#[derive(Identifiable, Queryable, Debug, Associations, Serialize, Deserialize)]
#[belongs_to(Group)]
#[belongs_to(User)]
#[primary_key(group_id, user_id)]
#[table_name = "groups_users"]
pub struct GroupUser {
    pub group_id: i32,
    pub user_id: i32,
    pub admin_accepted: bool,
    pub user_accepted: bool,
}
