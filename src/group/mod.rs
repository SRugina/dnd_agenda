use crate::schema::{groups, groups_users, users, groups};
use diesel::prelude::*;

use crate::user::{User, Profile};
use crate::group::Group;

pub mod routes;

use crate::api::ApiResponse;
use rocket::http::Status;

#[table_name = "groups"]
#[derive(Debug, Identifiable, AsChangeset, Serialize, Deserialize, Queryable)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub admin: i32,
}

#[table_name = "groups"]
#[derive(Serialize, Deserialize, Insertable)]
pub struct InsertableGroup {
    pub name: String,
    pub admin: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupJson {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub admin: Profile,
    pub members: Vec<Profile>,
    pub groups: Vec<Group>
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

impl Group {
    pub fn attach(
        &self,
        admin: Profile,
        members: Vec<Profile>,
        groups: Vec<Group>,
    ) -> GroupJson {
        GroupJson {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            admin,
            members,
            groups,
        }
    }
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
                    json: json!({"error": "Group not found" }),
                    status: Status::NotFound,
                }
            })
    }

        pub fn find_as_json(
        group_id: i32,
        connection: &PgConnection,
    ) -> Result<GroupJson, ApiResponse> {
        let group_and_admin = groups::table
            .find(group_id)
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
                    json: json!({"error": "Title must be unique", "details": error.to_string() }), // assume this as the most common cause due to slug and title being unique
                    status: Status::InternalServerError,
                })
            }
        }
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
    let groups = Group::belonging_to(group)
        .select(groups::all_columns)
        .load::<Group>(connection)
        .map_err(|error| {
            println!("Error: {:#?}", error);
            ApiResponse {
                json: json!({"error": "Groups not found" }),
                status: Status::NotFound,
            }
        })?;

    Ok(group.attach(admin, members, groups))
}