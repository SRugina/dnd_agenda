use crate::schema::{groups, groups_users, sessions, users};
use diesel::prelude::*;

use crate::session::Session;
use crate::user::{Profile, User};

pub mod routes;

use crate::api::ApiResponse;
use rocket::http::Status;

use crate::config::DEFAULT_LIMIT;

use crate::database::dsl;

use crate::database::Paginate;

#[table_name = "groups"]
#[derive(
    Debug, Identifiable, AsChangeset, Serialize, Deserialize, Queryable, Clone, PartialEq, Eq, Hash,
)]
pub struct Group {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub image: Option<String>,
    pub admin: i32,
}

#[table_name = "groups"]
#[derive(Serialize, Deserialize, Insertable)]
pub struct InsertableGroup {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub admin: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupJson {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub admin: Profile,
    pub members: Vec<Profile>,
    pub sessions: Vec<Session>,
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

#[table_name = "groups_users"]
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
pub struct InsertableGroupUser {
    pub group_id: i32,
    pub user_id: i32,
    pub admin_accepted: bool,
    pub user_accepted: bool,
}

// TODO: remove clone when diesel will allow skipping fields
#[derive(Deserialize, AsChangeset, Default, Clone)]
#[table_name = "groups_users"]
pub struct UpdateGroupUser {
    admin_accepted: bool,
    user_accepted: bool,
}

#[derive(FromForm, Default)]
pub struct FindGroups {
    global_search: Option<bool>,
    name: Option<String>,
    pub limit: Option<i64>,
    pub page: Option<i64>,
    order: Option<String>,
}

impl Group {
    pub fn attach(
        &self,
        admin: Profile,
        members: Vec<Profile>,
        sessions: Vec<Session>,
    ) -> GroupJson {
        GroupJson {
            id: self.id,
            slug: self.slug.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            admin,
            members,
            sessions,
        }
    }
    pub fn read(
        params: &FindGroups,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<(Vec<GroupJson>, i64), ApiResponse> {
        if params.global_search.unwrap_or(false) {
            //get all groups regardless of what groups the current user is in

            let mut pages_count: i64 = 0;

            let mut query = groups::table
                .inner_join(users::table) // admin details
                .select((groups::all_columns, users::all_columns))
                .into_boxed();

            if let Some(ref name) = params.name {
                query = query
                    .filter(dsl::similar_to(groups::name, name))
                    .order(dsl::similarity(groups::name, name).desc())
            } else if let Some(ref order) = params.order {
                match order.to_lowercase().as_ref() {
                    "asc" => query = query.order(groups::name.asc()),
                    "desc" => query = query.order(groups::name.desc()),
                    _ => query = query.order(groups::name.asc()),
                }
            } else {
                // default to asc
                query = query.order(groups::name.asc())
            }

            query
                .paginate(params.page.unwrap_or(1))
                .per_page(params.limit.unwrap_or(DEFAULT_LIMIT))
                .load_and_count_pages::<(Group, User)>(connection)
                .map_err(|error| {
                    println!("Error: {:#?}", error);
                    ApiResponse {
                        json: json!({"error": "Groups not found" }),
                        status: Status::NotFound,
                    }
                })
                .map(|(groups_and_admins, count)| {
                    pages_count += count;
                    groups_and_admins
                })?
                .iter()
                .map(|(group, admin)| populate(group, admin.to_profile(), connection))
                .collect::<Result<Vec<_>, _>>()
                .map(|group_jsons| (group_jsons, pages_count))
        } else {
            // get all groups belonging to the current user

            let mut pages_count: i64 = 0;

            let user = User::find(user_id, connection).map_err(|response| response)?;

            let mut query = GroupUser::belonging_to(&user)
                .filter(groups_users::columns::admin_accepted.eq(true))
                .filter(groups_users::columns::user_accepted.eq(true))
                .inner_join(groups::table.inner_join(users::table))
                .select((groups::all_columns, users::all_columns))
                .into_boxed();

            if let Some(ref name) = params.name {
                query = query
                    .filter(dsl::similar_to(groups::name, name))
                    .order(dsl::similarity(groups::name, name).desc())
            } else if let Some(ref order) = params.order {
                match order.to_lowercase().as_ref() {
                    "asc" => query = query.order(groups::name.asc()),
                    "desc" => query = query.order(groups::name.desc()),
                    _ => query = query.order(groups::name.asc()),
                }
            } else {
                // default to asc
                query = query.order(groups::name.asc())
            }

            query
                .paginate(params.page.unwrap_or(1))
                .per_page(params.limit.unwrap_or(DEFAULT_LIMIT))
                .load_and_count_pages::<(Group, User)>(connection)
                .map_err(|error| {
                    println!("Error: {:#?}", error);
                    ApiResponse {
                        json: json!({"error": "Groups not found" }),
                        status: Status::NotFound,
                    }
                })
                .map(|(groups_and_admins, count)| {
                    pages_count += count;
                    groups_and_admins
                })?
                .iter()
                .map(|(group, admin)| populate(group, admin.to_profile(), connection))
                .collect::<Result<Vec<_>, _>>()
                .map(|group_jsons| (group_jsons, pages_count))
        }
    }

    pub fn find(group_id: i32, connection: &PgConnection) -> Result<Group, ApiResponse> {
        groups::table
            .find(group_id)
            .first::<Group>(connection)
            .map(|group| group)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Group not found" }),
                    status: Status::NotFound,
                }
            })
    }

    pub fn find_as_json(
        group_slug: &str,
        connection: &PgConnection,
    ) -> Result<GroupJson, ApiResponse> {
        let group_and_admin = groups::table
            .filter(groups::slug.eq(group_slug))
            .inner_join(users::table) // admin details
            .select((groups::all_columns, users::all_columns))
            .first::<(Group, User)>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Group not found" }),
                    status: Status::NotFound,
                }
            })?;
        let (group, admin) = group_and_admin;
        populate(&group, admin.to_profile(), connection).map_err(|response| response)
    }

    pub fn request_to_join(
        group_id: i32,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<(), ApiResponse> {
        let new_group_user = &InsertableGroupUser {
            group_id,
            user_id,
            admin_accepted: false,
            user_accepted: true,
        };

        diesel::insert_into(groups_users::table)
            .values(new_group_user)
            .get_result::<GroupUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Could not request to join the group", "details": error.to_string() }),
                    status: Status::InternalServerError,
                }
            })?;
        Ok(())
    }

    pub fn accept_to_join(
        group: &Group,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<(), ApiResponse> {
        let group_user = GroupUser::belonging_to(group)
            .filter(groups_users::columns::admin_accepted.eq(false))
            .filter(groups_users::columns::user_accepted.eq(true))
            .filter(groups_users::columns::user_id.eq(user_id))
            .get_result::<GroupUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User not found", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;
        let updated_group_user = &UpdateGroupUser {
            admin_accepted: true,
            user_accepted: true,
        };

        diesel::update(groups_users::table.find((group_user.group_id, user_id)))
            .set(updated_group_user)
            .get_result::<GroupUser>(connection)
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
        group_id: i32,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<(), ApiResponse> {
        let new_group_user = &InsertableGroupUser {
            group_id,
            user_id,
            admin_accepted: true,
            user_accepted: false,
        };

        diesel::insert_into(groups_users::table)
            .values(new_group_user)
            .get_result::<GroupUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Could not make an invite to the user to join the group" }),
                    status: Status::InternalServerError,
                }
            })?;

        Ok(())
    }

    pub fn accept_invite_to_join(
        group: &Group,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<(), ApiResponse> {
        let group_user = GroupUser::belonging_to(group)
            .filter(groups_users::columns::admin_accepted.eq(true))
            .filter(groups_users::columns::user_accepted.eq(false))
            .filter(groups_users::columns::user_id.eq(user_id))
            .get_result::<GroupUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User not found", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;
        let updated_group_user = &UpdateGroupUser {
            admin_accepted: true,
            user_accepted: true,
        };

        diesel::update(groups_users::table.find((group_user.group_id, user_id)))
            .set(updated_group_user)
            .get_result::<GroupUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Could not accept invite", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;

        Ok(())
    }

    pub fn is_user_waiting_to_join(
        group_id: i32,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<bool, ApiResponse> {
        groups_users::table
            .find((group_id, user_id))
            .select((
                groups_users::columns::admin_accepted,
                groups_users::columns::user_accepted,
            ))
            .get_result::<(bool, bool)>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Group/User not found", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })
            
            .map(|(admin_accepted, user_accepted)| !admin_accepted && user_accepted)
    }
    pub fn is_user_invited_to_join(
        group_id: i32,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<bool, ApiResponse> {
        groups_users::table
            .find((group_id, user_id))
            .select((
                groups_users::columns::admin_accepted,
                groups_users::columns::user_accepted,
            ))
            .get_result::<(bool, bool)>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Group/User not found", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })
            .map(|(admin_accepted, user_accepted)| admin_accepted && !user_accepted)
    }

    pub fn remove_user(
        group: &Group,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<(), ApiResponse> {
        let group_user = GroupUser::belonging_to(group)
            .filter(groups_users::columns::user_id.eq(user_id))
            .get_result::<GroupUser>(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "User not found", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;

        diesel::delete(groups_users::table.find((group_user.group_id, user_id)))
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

    pub fn delete(group: &Group, connection: &PgConnection) -> Result<(), ApiResponse> {
        diesel::delete(groups::table.find(group.id))
            .execute(connection)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({"error": "Group could not be deleted", "details": error.to_string() }),
                    status: Status::NotFound,
                }
            })?;

        Ok(())
    }
}

impl InsertableGroup {
    pub fn create(
        group: InsertableGroup,
        creator_id: i32,
        connection: &PgConnection,
    ) -> Result<GroupJson, ApiResponse> {
        match connection
            .build_transaction()
            .run::<Group, diesel::result::Error, _>(|| {
                let new_group = diesel::insert_into(groups::table)
                    .values(&group)
                    .get_result::<Group>(connection)?;

                let new_group_user = &InsertableGroupUser {
                    group_id: new_group.id,
                    user_id: new_group.admin,
                    admin_accepted: true,
                    user_accepted: true,
                };

                diesel::insert_into(groups_users::table)
                    .values(new_group_user)
                    .get_result::<GroupUser>(connection)?;

                // add creator of group as member if they are not the admin
                if new_group.admin != creator_id {
                    let creator_group_user = &InsertableGroupUser {
                        group_id: new_group.id,
                        user_id: creator_id,
                        admin_accepted: true,
                        user_accepted: true,
                    };

                    diesel::insert_into(groups_users::table)
                        .values(creator_group_user)
                        .get_result::<GroupUser>(connection)?;
                }

                Ok(new_group)
            }) {
            Ok(group) => {
                let admin = User::find(group.admin, connection)
                    .map(|user| user.to_profile())
                    .map_err(|response| response)?;

                populate(&group, admin, connection)
                    .map(|group_json| group_json)
                    .map_err(|response| response)
            }
            Err(error) => {
                println!("Error: {:#?}", error);
                Err(ApiResponse {
                    json: json!({"errors":  { "name": [ "Name must be unique" ] }, "details": error.to_string() }), // assume this as the most common cause due to slug and name not being unique
                    status: Status::InternalServerError,
                })
            }
        }
    }
}

// TODO: remove clone when diesel will allow skipping fields
#[derive(Deserialize, AsChangeset, Default, Clone)]
#[table_name = "groups"]
pub struct UpdateGroup {
    name: Option<String>,
    description: Option<String>,
    admin: Option<i32>,
    #[serde(skip)]
    slug: Option<String>,
}

impl UpdateGroup {
    pub fn update(
        id: i32,
        group: &UpdateGroup,
        connection: &PgConnection,
    ) -> Result<GroupJson, ApiResponse> {
        let updated_group = diesel::update(groups::table.find(id))
            .set(group)
            .get_result::<Group>(connection)
            .map_err(|error| {
                println!("Cannot update group: {:#?}", error);
                ApiResponse {
                    json: json!({ "error": "cannot update group" }),
                    status: Status::UnprocessableEntity,
                }
            })?;

        let admin = User::find(updated_group.admin, connection)
            .map(|user| user.to_profile())
            .map_err(|response| response)?;

        populate(&updated_group, admin, connection)
            .map(|group_json| group_json)
            .map_err(|response| response)
    }
}

impl GroupUser {
    pub fn check_user_in_group(
        group_id: i32,
        user_id: i32,
        connection: &PgConnection,
    ) -> Result<bool, ApiResponse> {
        groups_users::table.find((group_id, user_id))
            .filter(groups_users::columns::admin_accepted.eq(true))
            .filter(groups_users::columns::user_accepted.eq(true))
            .get_result::<GroupUser>(connection)
            .map(|_| true)
            .map_err(|error| {
                println!("Error: {:#?}", error);
                ApiResponse {
                    json: json!({ "errors": { "group": [ "You are not a member of that group" ] } }),
                    status: Status::NotFound,
                }
            })
    }
}

pub fn populate(
    group: &Group,
    admin: Profile,
    connection: &PgConnection,
) -> Result<GroupJson, ApiResponse> {
    let members = GroupUser::belonging_to(group)
        .filter(groups_users::columns::admin_accepted.eq(true))
        .filter(groups_users::columns::user_accepted.eq(true))
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
    let sessions = Session::belonging_to(group)
        .select(sessions::all_columns)
        .load::<Session>(connection)
        .map_err(|error| {
            println!("Error: {:#?}", error);
            ApiResponse {
                json: json!({"error": "Sessions not found" }),
                status: Status::NotFound,
            }
        })?;

    Ok(group.attach(admin, members, sessions))
}
