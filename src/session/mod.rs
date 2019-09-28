use crate::schema::sessions;
use crate::schema::sessions_users;
use crate::schema::users;
use diesel::prelude::*;

use crate::user::User;

use chrono::{DateTime, Utc};

pub mod routes;

use crate::api::ApiResponse;
use rocket::http::Status;

#[table_name = "sessions"]
#[derive(Debug, Identifiable, AsChangeset, Serialize, Deserialize, Queryable)]
pub struct Session {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub dm: i32,
    pub session_date: DateTime<Utc>,
    pub colour: String,
}

impl Session {
    pub fn read(connection: &PgConnection) -> Result<Vec<Session>, ApiResponse> {
        match sessions::table.order(sessions::id).load::<Session>(connection) {
            Ok(sessions) => Ok(sessions),
            Err(error) => {
                println!("Error: {:#?}", error);
                Err(ApiResponse {
                    json: json!({"error": "Sessions not found" }),
                    status: Status::NotFound,
                })
            }
        }
    }

    pub fn find(session_id: i32, connection: &PgConnection) -> Result<Session, ApiResponse> {
        match sessions::table.find(session_id).first::<Session>(connection) {
            Ok(session) => Ok(session),
            Err(error) => {
                println!("Error: {:#?}", error);
                Err(ApiResponse {
                    json: json!({"error": "Session not found" }),
                    status: Status::NotFound,
                })
            }
        }
    }

    pub fn read_users(
        session_id: i32,
        connection: &PgConnection,
    ) -> Result<Vec<User>, ApiResponse> {
        match sessions::table.find(session_id).first::<Session>(connection) {
            Ok(session) => {
                match SessionUser::belonging_to(&session)
                    .inner_join(users::table)
                    .select(users::all_columns)
                    .load::<User>(connection)
                {
                    Ok(users) => Ok(users),
                    Err(error) => {
                        println!("Error: {:#?}", error);
                        Err(ApiResponse {
                            json: json!({"error": "Users not found" }),
                            status: Status::NotFound,
                        })
                    }
                }
            }
            Err(error) => {
                println!("Error: {:#?}", error);
                Err(ApiResponse {
                    json: json!({"error": "Session not found" }),
                    status: Status::NotFound,
                })
            }
        }
    }
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

/*
Ok so let's say you have those three tables: users, groups and user_groups. Now you want to get all groups for a specific user.
There are two variants doing that:

    users::table.inner_join(user_groups::table.inner_join(groups::table)).filter(users::id.eq(user_id)).select(groups::all_columns).load::<Group>(conn)?;

    let user = users::table.find(user_id).first::<User>(conn)?;
    UserGroup::belonging_to(&user).inner_join(groups::table).select(groups::all_columns).load::<Group>(conn)?;

    The first variant is only on query, but may not match your required data layout if you want to do that for all users,
    The second variant allows to group the results for each user, using the build in associations api.

For inserting (because this was also ask in the stackoverflow question): Yes this requires three separate inserts. We do not try to hide that because in the end it's a user choice how to handle for example data consistency in case of an failed insert there. (And yes, just using a transaction is a common choice)
*/

#[table_name = "sessions"]
#[derive(Serialize, Deserialize, Insertable)]
pub struct InsertableSession {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub dm: i32,
    pub session_date: DateTime<Utc>,
    pub colour: String,
}

#[table_name = "sessions_users"]
#[derive(Serialize, Deserialize, Insertable)]
pub struct InsertableSessionsUsers {
    pub session_id: i32,
    pub user_id: i32,
}

impl InsertableSession {
    pub fn create(
        session: InsertableSession,
        connection: &PgConnection,
    ) -> Result<Session, ApiResponse> {
        match connection
            .build_transaction()
            .run::<Session, diesel::result::Error, _>(|| {
                let new_session = diesel::insert_into(sessions::table)
                    .values(&session)
                    .get_result::<Session>(connection)?;

                let new_sessions_users = &InsertableSessionsUsers {
                    session_id: new_session.id,
                    user_id: new_session.dm,
                };

                diesel::insert_into(sessions_users::table)
                    .values(new_sessions_users)
                    .get_result::<SessionUser>(connection)?;

                Ok(new_session)
            }) {
            Ok(session) => Ok(session),
            Err(error) => {
                println!("Error: {:#?}", error);
                Err(ApiResponse {
                    json: json!({"error": "Could not create session" }),
                    status: Status::InternalServerError,
                })
            }
        }
    }
}
