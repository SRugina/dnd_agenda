use crate::schema::sessions;
use crate::schema::sessions_users;
use diesel::prelude::*;

use crate::user::User;

#[table_name = "sessions"]
#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Queryable)]
pub struct Session {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub dm: i32,
    pub session_date: String,
}

#[derive(Identifiable, Queryable, Debug, Associations, Serialize, Deserialize)]
#[belongs_to(Session)]
#[belongs_to(User)]
#[primary_key(session_id, user_id)]
#[table_name = "sessions_users"]
pub struct SessionUser {
    pub session_id: i32,
    pub user_id: i32,
}
