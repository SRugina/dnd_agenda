use crate::schema::sessions;
use crate::schema::sessions_guests;
use crate::schema::sessions_users;
use crate::schema::users;
use diesel::prelude::*;

use crate::api::GuestAuth;
use crate::user::Profile;
use crate::user::User;

use crate::group::{Group, GroupUser};
use crate::schema::{groups, groups_users};

use crate::config::DATE_FORMAT;
use chrono::{DateTime, Utc};

pub mod routes;

use crate::api::ApiResponse;
use rocket::http::Status;

const DEFAULT_LIMIT: i64 = 20;

#[table_name = "sessions"]
#[belongs_to(Group)]
#[derive(Associations, Debug, Identifiable, AsChangeset, Serialize, Deserialize, Queryable)]
pub struct Session {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub dm: i32,
    pub session_date: DateTime<Utc>,
    pub colour: String,
    pub group_id: i32,
}

// TODO: remove clone when diesel will allow skipping fields
#[derive(Deserialize, AsChangeset, Default, Clone)]
#[table_name = "sessions_users"]
pub struct UpdateSessionUser {
    dm_accepted: bool,
    user_accepted: bool,
}

#[derive(FromForm, Default)]
pub struct FindSessions {
    // dm: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionJson {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub dm: Profile,
    pub session_date: String,
    pub colour: String,
    pub group: Group,
    pub members: Vec<Profile>,
    pub guests: Vec<(i32, String)>,
}

impl Session {
    pub fn attach(
        &self,
        dm: Profile,
        group: Group,
        members: Vec<Profile>,
        guests: Vec<(i32, String)>,
    ) -> SessionJson {
        SessionJson {
            id: self.id,
            slug: self.slug.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            dm,
            session_date: self.session_date.format(DATE_FORMAT).to_string(),
            colour: self.colour.clone(),
            group,
            members,
            guests,
        }
    }
    pub fn read(
        params: &FindSessions,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<Vec<Vec<SessionJson>>, ApiResponse> {
        let user = User::find(user_id, connection).map_err(|response| response)?;
        // get all sessions belonging to the same groups as the user
        // let sessions: Result<Vec<Vec<SessionJson>>, ApiResponse> =
        GroupUser::belonging_to(&user)
            .filter(groups_users::columns::admin_accepted.eq(true))
            .filter(groups_users::columns::user_accepted.eq(true))
            .inner_join(groups::table)
            .select(groups::all_columns)
            .load::<Group>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Group not found" }),
                    status: Status::NotFound,
                }
            })?
            .iter()
            .map(|group| {
                Session::belonging_to(group)
                    .inner_join(users::table) // dm details
                    .order(sessions::session_date.desc())
                    .select((sessions::all_columns, users::all_columns))
                    .limit(params.limit.unwrap_or(DEFAULT_LIMIT))
                    .offset(params.offset.unwrap_or(0))
                    .load::<(Session, User)>(connection)
                    .map_err(|error| {
                        println!("Error: {:#?}", error);
                        ApiResponse {
                            json: json!({"error": "Sessions not found" }),
                            status: Status::NotFound,
                        }
                    })?
                    .iter()
                    .map(|(session, dm)| populate(session, dm.to_profile(), connection))
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()
        // .map(|session_jsons| session_jsons.into_iter().flatten().collect())
    }

    pub fn find(session_id: i32, connection: &PgConnection) -> Result<Session, ApiResponse> {
        sessions::table
            .find(session_id)
            .first::<Session>(connection)
            .map(|session| session)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Session not found" }),
                    status: Status::NotFound,
                }
            })
    }

    pub fn find_as_json(
        session_id: i32,
        connection: &PgConnection,
    ) -> Result<SessionJson, ApiResponse> {
        let session_and_dm = sessions::table
            .find(session_id)
            .inner_join(users::table) // dm details
            .select((sessions::all_columns, users::all_columns))
            .first::<(Session, User)>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Session not found" }),
                    status: Status::NotFound,
                }
            })?;
        let (session, dm) = session_and_dm;
        populate(&session, dm.to_profile(), connection).map_err(|response| response)
    }

    pub fn read_users(
        session_id: i32,
        connection: &PgConnection,
    ) -> Result<Vec<Profile>, ApiResponse> {
        let session = Session::find(session_id, connection).map_err(|response| response)?;

        SessionUser::belonging_to(&session)
            .filter(sessions_users::columns::dm_accepted.eq(true))
            .filter(sessions_users::columns::user_accepted.eq(true))
            .inner_join(users::table)
            .select(users::all_columns)
            .load::<User>(connection)
            .map(|users| users.iter().map(|user| user.to_profile()).collect())
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Users not found" }),
                    status: Status::NotFound,
                }
            })
    }

    pub fn read_guests(
        session_id: i32,
        connection: &PgConnection,
    ) -> Result<Vec<(i32, String)>, ApiResponse> {
        let session = Session::find(session_id, connection).map_err(|response| response)?;

        SessionGuest::belonging_to(&session)
            .select((
                sessions_guests::columns::guest_id,
                sessions_guests::columns::guest_name,
            ))
            .load::<(i32, String)>(connection)
            .map(|guests| guests)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Users not found" }),
                    status: Status::NotFound,
                }
            })
    }

    pub fn request_to_join(
        session_id: i32,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<(), ApiResponse> {
        let new_session_user = &InsertableSessionUser {
            session_id,
            user_id,
            dm_accepted: false,
            user_accepted: true,
        };

        diesel::insert_into(sessions_users::table)
            .values(new_session_user)
            .get_result::<SessionUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Could not request to join the session", "details": error.to_string() }),
                    status: Status::InternalServerError,
                }
            })?;
        Ok(())
    }

    pub fn accept_to_join(
        session: &Session,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<(), ApiResponse> {
        let session_user = SessionUser::belonging_to(session)
            .filter(sessions_users::columns::dm_accepted.eq(false))
            .filter(sessions_users::columns::user_accepted.eq(true))
            .filter(sessions_users::columns::user_id.eq(user_id))
            .get_result::<SessionUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User not found", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;
        let updated_session_user = &UpdateSessionUser {
            dm_accepted: true,
            user_accepted: true,
        };

        diesel::update(sessions_users::table.find((session_user.session_id, user_id)))
            .set(updated_session_user)
            .get_result::<SessionUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User could not be accepted", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;

        Ok(())
    }

    pub fn invite_to_join(
        session_id: i32,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<(), ApiResponse> {
        let new_session_user = &InsertableSessionUser {
            session_id,
            user_id,
            dm_accepted: true,
            user_accepted: false,
        };

        diesel::insert_into(sessions_users::table)
            .values(new_session_user)
            .get_result::<SessionUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Could not make an invite to the user to join the session" }),
                    status: Status::InternalServerError,
                }
            })?;

        Ok(())
    }

    pub fn accept_invite_to_join(
        session: &Session,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<(), ApiResponse> {
        let session_user = SessionUser::belonging_to(session)
            .filter(sessions_users::columns::dm_accepted.eq(true))
            .filter(sessions_users::columns::user_accepted.eq(false))
            .filter(sessions_users::columns::user_id.eq(user_id))
            .get_result::<SessionUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User not found", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;
        let updated_session_user = &UpdateSessionUser {
            dm_accepted: true,
            user_accepted: true,
        };

        diesel::update(sessions_users::table.find((session_user.session_id, user_id)))
            .set(updated_session_user)
            .get_result::<SessionUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Could not accept invite", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;

        Ok(())
    }

    pub fn delete_user(
        session: &Session,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<(), ApiResponse> {
        let session_user = SessionUser::belonging_to(session)
            .filter(sessions_users::columns::user_id.eq(user_id))
            .get_result::<SessionUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User not found", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;

        diesel::delete(sessions_users::table.find((session_user.session_id, user_id)))
            .execute(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User could not be deleted", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;

        Ok(())
    }

    pub fn delete_guest(
        session: &Session,
        guest_id: i32,
        connection: &PgConnection,
    ) -> Result<(), ApiResponse> {
        let session_guest = SessionGuest::belonging_to(session)
            .filter(sessions_guests::columns::guest_id.eq(guest_id))
            .get_result::<SessionGuest>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Guest not found", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;

        diesel::delete(sessions_guests::table.find((session_guest.session_id, guest_id)))
            .execute(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Guest could not be deleted", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;

        Ok(())
    }

    pub fn delete(session: &Session, connection: &PgConnection) -> Result<(), ApiResponse> {
        diesel::delete(sessions::table.find(session.id))
            .execute(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Session could not be deleted", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;

        Ok(())
    }

    pub fn create_guest_token(
        session_id: i32,
        guest_name: &str,
        connection: &PgConnection,
    ) -> Result<String, ApiResponse> {
        let new_session_guest = &InsertableSessionGuest {
            session_id,
            guest_name: guest_name.to_string(),
        };

        let guest_id = diesel::insert_into(sessions_guests::table)
            .values(new_session_guest)
            .get_result::<SessionGuest>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Could not create a guest link with that guest name" }),
                    status: Status::InternalServerError,
                }
            })?
            .guest_id;

        let token = GuestAuth {
            session_id,
            guest_id,
            guest_name: guest_name.to_string(),
        }
        .token();

        Ok(token)
    }
}

#[derive(Identifiable, Queryable, Debug, Associations, Serialize, Deserialize)]
#[belongs_to(Session)]
#[primary_key(session_id, guest_id)]
#[table_name = "sessions_guests"]
pub struct SessionGuest {
    pub session_id: i32,
    pub guest_id: i32,
    pub guest_name: String,
}

#[derive(Identifiable, Queryable, Debug, Associations, Serialize, Deserialize)]
#[belongs_to(Session)]
#[belongs_to(User)]
#[primary_key(session_id, user_id)]
#[table_name = "sessions_users"]
pub struct SessionUser {
    pub session_id: i32,
    pub user_id: i32,
    pub dm_accepted: bool,
    pub user_accepted: bool,
}

impl SessionUser {
    pub fn check_user_in_session(
        session: &Session,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<bool, ApiResponse> {
        SessionUser::belonging_to(session)
            .filter(sessions_users::columns::dm_accepted.eq(true))
            .filter(sessions_users::columns::user_accepted.eq(true))
            .filter(sessions_users::columns::user_id.eq(user_id))
            .get_result::<SessionUser>(connection)
            .map(|_| true)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({ "errors": { "dm": [ "That user is not a member of this session" ] } }),
                    status: Status::NotFound,
                }
            })
    }
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
    pub group_id: i32,
}

#[table_name = "sessions_users"]
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
pub struct InsertableSessionUser {
    pub session_id: i32,
    pub user_id: i32,
    pub dm_accepted: bool,
    pub user_accepted: bool,
}

#[table_name = "sessions_guests"]
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
pub struct InsertableSessionGuest {
    pub session_id: i32,
    pub guest_name: String,
}

#[table_name = "groups_users"]
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
pub struct InsertableGroupUser {
    pub group_id: i32,
    pub user_id: i32,
    pub admin_accepted: bool,
    pub user_accepted: bool,
}

impl InsertableSession {
    pub fn create(
        session: InsertableSession,
        creator_id: i32,
        connection: &PgConnection,
    ) -> Result<SessionJson, ApiResponse> {
        match connection
            .build_transaction()
            .run::<Session, diesel::result::Error, _>(|| {
                let new_session = diesel::insert_into(sessions::table)
                    .values(&session)
                    .get_result::<Session>(connection)?;

                let new_session_user = &InsertableSessionUser {
                    session_id: new_session.id,
                    user_id: new_session.dm,
                    dm_accepted: true,
                    user_accepted: true,
                };

                diesel::insert_into(sessions_users::table)
                    .values(new_session_user)
                    .get_result::<SessionUser>(connection)?;

                // add creator of session as member of the party if they are not the dm
                if new_session.dm != creator_id {
                    let creator_session_user = &InsertableSessionUser {
                        session_id: new_session.id,
                        user_id: creator_id,
                        dm_accepted: true,
                        user_accepted: true,
                    };

                    diesel::insert_into(sessions_users::table)
                        .values(creator_session_user)
                        .get_result::<SessionUser>(connection)?;
                }

                Ok(new_session)
            }) {
            Ok(session) => {
                let dm = User::find(session.dm, connection)
                    .map(|user| user.to_profile())
                    .map_err(|response| response)?;

                populate(&session, dm, connection)
                    .map(|session_json| session_json)
                    .map_err(|response| response)
            }
            Err(error) => {
                println!("Error: {:#?}", error);
                Err(ApiResponse {
                    json: json!({"error": "Title must be unique", "details": error.to_string() }), // assume this as the most common cause due to slug and title being unique
                    status: Status::InternalServerError,
                })
            }
        }
    }
}

// TODO: remove clone when diesel will allow skipping fields
#[derive(Deserialize, AsChangeset, Default, Clone)]
#[table_name = "sessions"]
pub struct UpdateSession {
    title: Option<String>,
    description: Option<String>,
    session_date: Option<DateTime<Utc>>,
    colour: Option<String>,
    #[serde(skip)]
    slug: Option<String>,
    dm: Option<i32>,
}

impl UpdateSession {
    pub fn update(
        id: i32,
        session: &UpdateSession,
        connection: &PgConnection,
    ) -> Result<SessionJson, ApiResponse> {
        let updated_session = diesel::update(sessions::table.find(id))
            .set(session)
            .get_result::<Session>(connection)
            .map_err(|error| {
                println!("Cannot update session: {:#?}", error);
                ApiResponse {
                    json: json!({ "error": "cannot update session" }),
                    status: Status::UnprocessableEntity,
                }
            })?;

        let dm = User::find(updated_session.dm, connection)
            .map(|user| user.to_profile())
            .map_err(|response| response)?;

        populate(&updated_session, dm, connection)
            .map(|session_json| session_json)
            .map_err(|response| response)
    }
}

pub fn populate(
    session: &Session,
    dm: Profile,
    connection: &PgConnection,
) -> Result<SessionJson, ApiResponse> {
    let members = SessionUser::belonging_to(session)
        .filter(sessions_users::columns::dm_accepted.eq(true))
        .filter(sessions_users::columns::user_accepted.eq(true))
        .inner_join(users::table)
        .select(users::all_columns)
        .load::<User>(connection)
        .map_err(|error| {
            println!("Error: {:#?}", error);
            ApiResponse {
                json: json!({"error": "Members not found" }),
                status: Status::NotFound,
            }
        })?
        .iter()
        .map(|user| user.to_profile())
        .collect();
    let guests = SessionGuest::belonging_to(session)
        .select((sessions_guests::guest_id, sessions_guests::guest_name))
        .load::<(i32, String)>(connection)
        .map_err(|error| {
            println!("Error: {:#?}", error);
            ApiResponse {
                json: json!({"error": "Guests not found" }),
                status: Status::NotFound,
            }
        })?;
    let group = Group::find(session.group_id, connection).map_err(|error| {
        println!("Error: {:#?}", error);
        ApiResponse {
            json: json!({"error": "Group not found" }),
            status: Status::NotFound,
        }
    })?;

    Ok(session.attach(dm, group, members, guests))
}
