use crate::schema::groups;
use crate::schema::groups_users;
use diesel::prelude::*;

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
#[belongs_to(User)]
#[primary_key(group_id, user_id)]
#[table_name = "groups_users"]
pub struct GroupUser {
    pub group_id: i32,
    pub user_id: i32,
    pub admin_accepted: bool,
    pub user_accepted: bool,
}

impl Group {
    pub fn read(connection: &PgConnection) -> Result<Vec<Group>, ApiResponse> {
        groups::table
            .order(groups::id)
            .load::<Group>(connection)
            .map(|groups| groups)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Groups not found" }),
                    status: Status::NotFound,
                }
            })
    }

    pub fn find(group_id: i32, connection: &PgConnection) -> Result<Group, ApiResponse> {
        groups::table
            .find(group_id)
            .first::<Group>(connection)
            .map(|group| group)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User not found" }),
                    status: Status::NotFound,
                }
            })
    }
}
