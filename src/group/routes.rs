use crate::database::DnDAgendaDB;
use crate::group;

use rocket_contrib::json::Json;
use rocket_contrib::json::JsonError;
use rocket_contrib::json::JsonValue;

use rocket::request::Form;

use crate::api::ApiResponse;
use crate::api::Auth;
use rocket::http::Status;

use crate::api::validate_user_exists;
use crate::api::FieldValidator;
use validator::Validate;

use crate::mailgun::{send_mail, MailType};

use crate::user::User;

use std::thread;

#[get("/?<params..>")]
pub fn get_all(
    auth: Result<Auth, JsonValue>,
    params: Form<group::FindGroups>,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => group::Group::read(&params, auth.id, &connection)
            .map(|(groups, pages_count)| ApiResponse {
                json: json!({ "groups": groups, "groupsPagesCount": pages_count  }),
                status: Status::Ok,
            })
            .map_err(|response| response),
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[get("/<group_slug>")]
pub fn get_group(
    auth: Result<Auth, JsonValue>,
    group_slug: String,
    connection: DnDAgendaDB,
) -> ApiResponse {
    match auth {
        Ok(_auth) => match group::Group::find_as_json(&group_slug, &connection) {
            Ok(group_json) => ApiResponse {
                json: json!({ "group": group_json }),
                status: Status::Ok,
            },
            Err(response) => response,
        },
        Err(auth_error) => ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        },
    }
}

#[derive(Deserialize, Validate)]
pub struct NewGroup {
    #[validate(length(min = 1, code = "Name must be at least 1 character long"))]
    pub name: Option<String>,
    #[validate(length(min = 1, code = "Description must be at least 1 character long"))]
    pub description: Option<String>,
    #[validate(custom = "validate_user_exists")]
    pub admin: Option<i32>,
}

#[post("/", format = "application/json", data = "<group>")] // data attribute tells rocket to expect Body Data - then map the body to a parameter
pub fn create(
    auth: Result<Auth, JsonValue>,
    group: Result<Json<NewGroup>, JsonError>,
    connection: DnDAgendaDB,
) -> ApiResponse {
    match auth {
        Ok(auth) => match group {
            Ok(json_group) => {
                let new_group = json_group.into_inner();

                let empty_flag = false; // i.e. should we ignore empty fields?
                let mut extractor = FieldValidator::validate(&new_group);
                let name = extractor.extract("name", new_group.name, empty_flag);
                let description =
                    extractor.extract("description", new_group.description, empty_flag);
                let admin = extractor.extract("admin", new_group.admin, empty_flag);

                let check = extractor.check();

                match check {
                    Ok(_) => {
                        let insertable_group = group::InsertableGroup {
                            slug: slugify(&name),
                            name,
                            description,
                            admin,
                        };
                        match group::InsertableGroup::create(insertable_group, auth.id, &connection)
                        {
                            Ok(group) => ApiResponse {
                                json: json!({ "group": group }),
                                status: Status::Created,
                            },
                            Err(response) => response,
                        }
                    }
                    Err(response) => response,
                }
            }
            Err(json_error) => match json_error {
                JsonError::Parse(_req, err) => ApiResponse {
                    json: json!({ "error": err.to_string() }),
                    status: Status::BadRequest,
                },
                JsonError::Io(_err) => ApiResponse {
                    json: json!({ "error": "I/O error occured while reading the incoming request data" }),
                    status: Status::InternalServerError,
                },
            },
        },
        Err(auth_error) => ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        },
    }
}

#[derive(Deserialize, Validate, Clone)]
pub struct UpdateGroupData {
    #[validate(length(min = 1, code = "Name must be at least 1 character long"))]
    pub name: Option<String>,
    #[validate(length(min = 1, code = "Description must be at least 1 character long"))]
    pub description: Option<String>,

    slug: Option<String>,
}

#[patch("/<group_id>", format = "application/json", data = "<group>")]
pub fn patch_group(
    auth: Result<Auth, JsonValue>,
    group: Result<Json<UpdateGroupData>, JsonError>,
    group_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. group does not exist)
            let group_details =
                group::Group::find(group_id, &connection).map_err(|response| response)?;

            if auth.id == group_details.admin {
                let mut group_update_details = group.map_err(|json_error| {
                match json_error {
                    JsonError::Parse(_req, err) => ApiResponse {
                        json: json!({ "error": err.to_string() }),
                        status: Status::BadRequest,
                    },
                    JsonError::Io(_err) => ApiResponse {
                        json: json!({ "error": "I/O error occured while reading the incoming request data" }),
                        status: Status::InternalServerError,
                    },
                }
            })?.into_inner();

                if let Some(ref name) = group_update_details.name {
                    group_update_details.slug = Some(slugify(&name));
                } else {
                    group_update_details.slug = None;
                }

                let group_validator_details = group_update_details.clone();

                let empty_flag = true; // i.e. do not emit error if empty
                let mut extractor = FieldValidator::validate(&group_update_details);
                let _name = extractor.extract("name", group_validator_details.name, empty_flag);
                let _description = extractor.extract(
                    "description",
                    group_validator_details.description,
                    empty_flag,
                );

                extractor.check()?;

                let update_group = group::UpdateGroup {
                    name: group_update_details.name,
                    description: group_update_details.description,
                    admin: None,

                    slug: group_update_details.slug,
                };

                group::UpdateGroup::update(group_id, &update_group, &connection)
                    .map(|group| ApiResponse {
                        json: json!({ "group": group }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the admin" }),
                    status: Status::Unauthorized,
                })
            }
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[derive(Deserialize, Validate, Clone)]
pub struct UpdateGroupAdminData {
    #[validate(custom = "validate_user_exists")]
    pub admin: Option<i32>,
}

#[patch("/<group_id>/admin", format = "application/json", data = "<group>")]
pub fn patch_admin_of_group(
    auth: Result<Auth, JsonValue>,
    group: Result<Json<UpdateGroupAdminData>, JsonError>,
    group_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. group does not exist)
            let group_details =
                group::Group::find(group_id, &connection).map_err(|response| response)?;

            if auth.id == group_details.admin {
                let group_update_details = group.map_err(|json_error| {
                match json_error {
                    JsonError::Parse(_req, err) => ApiResponse {
                        json: json!({ "error": err.to_string() }),
                        status: Status::BadRequest,
                    },
                    JsonError::Io(_err) => ApiResponse {
                        json: json!({ "error": "I/O error occured while reading the incoming request data" }),
                        status: Status::InternalServerError,
                    },
                }
            })?.into_inner();

                let group_validator_details = group_update_details.clone();

                let empty_flag = true; // i.e. do not emit error if empty
                let mut extractor = FieldValidator::validate(&group_update_details);
                let _new_admin =
                    extractor.extract("admin", group_validator_details.admin, empty_flag);

                extractor.check()?;

                let update_group = group::UpdateGroup {
                    name: None,
                    description: None,
                    slug: None,

                    admin: group_update_details.admin,
                };

                group::UpdateGroup::update(group_id, &update_group, &connection)
                    .map(|group| ApiResponse {
                        json: json!({ "group": group }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the admin" }),
                    status: Status::Unauthorized,
                })
            }
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[get("/<group_id>/join", format = "application/json")]
pub fn join_group(
    auth: Result<Auth, JsonValue>,
    group_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. group does not exist)
            let group_details =
                group::Group::find(group_id, &connection).map_err(|response| response)?;

            group::Group::request_to_join(group_details.id, auth.id, &connection)
                .map(|_| ApiResponse {
                    json: json!({ "message": "requested to join group successfully" }),
                    status: Status::Ok,
                })
                .map_err(|response| response)
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[get("/<group_id>/accept/<user_id>", format = "application/json")]
pub fn accept_to_group(
    auth: Result<Auth, JsonValue>,
    group_id: i32,
    user_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            let group_details =
                group::Group::find(group_id, &connection).map_err(|response| response)?;

            if auth.id == group_details.admin {
                group::Group::accept_to_join(&group_details, user_id, &connection)
                    .map(|_| ApiResponse {
                        json: json!({ "message": "successfully accepted user to group" }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the admin" }),
                    status: Status::Unauthorized,
                })
            }
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[get("/<group_id>/deny/<user_id>", format = "application/json")]
pub fn deny_to_group(
    auth: Result<Auth, JsonValue>,
    group_id: i32,
    user_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            let group_details =
                group::Group::find(group_id, &connection).map_err(|response| response)?;

            if auth.id == group_details.admin {
                group::Group::remove_user(&group_details, user_id, &connection)
                    .map(|_| ApiResponse {
                        json: json!({ "message": "successfully denied user to group" }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the Admin" }),
                    status: Status::Unauthorized,
                })
            }
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[get("/<group_id>/invite/<user_id>", format = "application/json", rank = 2)]
pub fn invite_to_group(
    auth: Result<Auth, JsonValue>,
    group_id: i32,
    user_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. group does not exist)
            let group_details =
                group::Group::find(group_id, &connection).map_err(|response| response)?;
            if auth.id == group_details.admin {
                group::Group::invite_to_join(group_details.id, user_id, &connection)
                    .map(|_| {
                        // since invite_to_join succeeded, the user and admin must exist
                        let user = User::find(user_id, &connection).unwrap();
                        let admin = User::find(group_details.admin, &connection).unwrap();

                        thread::spawn(|| {
                            send_mail(
                                MailType::GroupInviteReceived,
                                user,
                                group_details.name,
                                group_details.slug,
                                admin,
                            );
                        });
                        ApiResponse {
                            json: json!({ "message": "invited user to join group successfully" }),
                            status: Status::Ok,
                        }
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the admin" }),
                    status: Status::Unauthorized,
                })
            }
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[get("/<group_id>/invite/accept", format = "application/json")]
pub fn accept_invite_to_group(
    auth: Result<Auth, JsonValue>,
    group_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. group does not exist)
            let group_details =
                group::Group::find(group_id, &connection).map_err(|response| response)?;

            group::Group::accept_invite_to_join(&group_details, auth.id, &connection)
                .map(|_| {
                    // since accept_invite_to_join succeeded, the user and admin must exist
                    let user = User::find(auth.id, &connection).unwrap();
                    let admin = User::find(group_details.admin, &connection).unwrap();

                    thread::spawn(|| {
                        send_mail(
                            MailType::GroupInviteAccepted,
                            user,
                            group_details.name,
                            group_details.slug,
                            admin,
                        );
                    });
                    ApiResponse {
                        json: json!({ "message": "Joined group successfully" }),
                        status: Status::Ok,
                    }
                })
                .map_err(|response| response)
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[get("/<group_id>/invite/deny", format = "application/json")]
pub fn deny_invite_to_group(
    auth: Result<Auth, JsonValue>,
    group_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. session does not exist)
            let group_details =
                group::Group::find(group_id, &connection).map_err(|response| response)?;

            group::Group::remove_user(&group_details, auth.id, &connection)
                .map(|_| {
                    // since remove_user succeeded, the user and admin must exist
                    let user = User::find(auth.id, &connection).unwrap();
                    let admin = User::find(group_details.admin, &connection).unwrap();

                    thread::spawn(|| {
                        send_mail(
                            MailType::GroupInviteDeclined,
                            user,
                            group_details.name,
                            group_details.slug,
                            admin,
                        );
                    });
                    ApiResponse {
                        json: json!({ "message": "Denied invite to group successfully" }),
                        status: Status::Ok,
                    }
                })
                .map_err(|response| response)
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[get("/<group_id>/waiting/<user_id>", format = "application/json")]
pub fn is_user_waiting_to_join(
    auth: Result<Auth, JsonValue>,
    group_id: i32,
    user_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(_auth) => group::Group::is_user_waiting_to_join(group_id, user_id, &connection)
            .map(|is_waiting| ApiResponse {
                json: json!({ "waiting": is_waiting }),
                status: Status::Ok,
            })
            .map_err(|response| response),
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[get("/<group_id>/invited/<user_id>", format = "application/json")]
pub fn is_user_invited_to_join(
    auth: Result<Auth, JsonValue>,
    group_id: i32,
    user_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(_auth) => group::Group::is_user_invited_to_join(group_id, user_id, &connection)
            .map(|is_invited| ApiResponse {
                json: json!({ "invited": is_invited }),
                status: Status::Ok,
            })
            .map_err(|response| response),
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[delete("/<group_id>/leave")]
pub fn leave_group(
    auth: Result<Auth, JsonValue>,
    group_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. group does not exist)
            let group_details =
                group::Group::find(group_id, &connection).map_err(|response| response)?;

            if auth.id != group_details.admin {
                group::Group::remove_user(&group_details, auth.id, &connection)
                    .map(|_| ApiResponse {
                        json: json!({ "message": "left group successfully" }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are the admin, so you cannot leave" }),
                    status: Status::Unauthorized,
                })
            }
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[delete("/<group_id>")]
pub fn delete_group(
    auth: Result<Auth, JsonValue>,
    group_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. group does not exist)
            let group_details =
                group::Group::find(group_id, &connection).map_err(|response| response)?;

            if auth.id == group_details.admin {
                group::Group::delete(&group_details, &connection)
                    .map(|_| ApiResponse {
                        json: json!({ "message": "group deleted successfully" }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the admin" }),
                    status: Status::Unauthorized,
                })
            }
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

#[delete("/<group_id>/remove/<user_id>")]
pub fn remove_user_from_group(
    auth: Result<Auth, JsonValue>,
    group_id: i32,
    user_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. group does not exist)
            let group_details =
                group::Group::find(group_id, &connection).map_err(|response| response)?;

            if auth.id == group_details.admin {
                if user_id != auth.id {
                    group::Group::remove_user(&group_details, user_id, &connection)
                        .map(|_| ApiResponse {
                            json: json!({ "message": "removed user from group successfully" }),
                            status: Status::Ok,
                        })
                        .map_err(|response| response)
                } else {
                    Err(ApiResponse {
                        json: json!({ "error": "you are the admin, so you cannot be removed" }),
                        status: Status::Unauthorized,
                    })
                }
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the admin" }),
                    status: Status::Unauthorized,
                })
            }
        }
        Err(auth_error) => Err(ApiResponse {
            json: auth_error,
            status: Status::Unauthorized,
        }),
    }
}

fn slugify(name: &str) -> String {
    slug::slugify(name)
}
