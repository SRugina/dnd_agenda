use crate::database::DnDAgendaDB;
use crate::session;

use rocket_contrib::json::Json;
use rocket_contrib::json::JsonError;
use rocket_contrib::json::JsonValue;

use crate::api::ApiResponse;
use crate::api::Auth;
use crate::api::GuestAuth;
use rocket::http::RawStr;
use rocket::http::Status;

use crate::api::validate_colour;
use crate::api::validate_user_exists;
use crate::api::FieldValidator;
use validator::Validate;

use chrono::{DateTime, Utc};

use slug;

use regex::Regex;

lazy_static! {
    static ref SESSION_DATE_FORMAT: Regex =
        Regex::new(r"\d{4,}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{3}Z").unwrap();
}

#[get("/")]
pub fn get_all(auth: Result<Auth, JsonValue>, connection: DnDAgendaDB) -> ApiResponse {
    match auth {
        Ok(_auth) => match session::Session::read(&connection) {
            Ok(sessions) => ApiResponse {
                json: json!({ "sessions": sessions }),
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

#[get("/<session_id>")]
pub fn get_session(
    auth: Result<Auth, JsonValue>,
    session_id: i32,
    connection: DnDAgendaDB,
) -> ApiResponse {
    match auth {
        Ok(_auth) => match session::Session::find_as_json(session_id, &connection) {
            Ok(session_json) => ApiResponse {
                json: json!({ "session": session_json }),
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

#[get("/<session_id>/users")]
pub fn get_users(
    auth: Result<Auth, JsonValue>,
    session_id: i32,
    connection: DnDAgendaDB,
) -> ApiResponse {
    match auth {
        Ok(_auth) => match session::Session::read_users(session_id, &connection) {
            Ok(users) => ApiResponse {
                json: json!({ "users": users }),
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
pub struct NewSession {
    #[validate(length(min = 1, code = "Title must be at least 1 character long"))]
    pub title: Option<String>,
    #[validate(length(min = 1, code = "Description must be at least 1 character long"))]
    pub description: Option<String>,
    #[validate(custom = "validate_user_exists")]
    pub dm: Option<i32>,
    #[validate(regex(
        path = "SESSION_DATE_FORMAT",
        code = "must be valid output of JS toISOString()"
    ))]
    pub session_date: Option<String>,
    #[validate(custom = "validate_colour")]
    pub colour: Option<String>,
}

#[post("/", format = "application/json", data = "<session>")] // data attribute tells rocket to expect Body Data - then map the body to a parameter
pub fn create(
    auth: Result<Auth, JsonValue>,
    session: Result<Json<NewSession>, JsonError>,
    connection: DnDAgendaDB,
) -> ApiResponse {
    match auth {
        Ok(auth) => match session {
            Ok(json_session) => {
                let new_session = json_session.into_inner();

                let empty_flag = false; // i.e. emit error if empty
                let mut extractor = FieldValidator::validate(&new_session);
                let title = extractor.extract("title", new_session.title, empty_flag);
                let description =
                    extractor.extract("description", new_session.description, empty_flag);
                let dm = extractor.extract("dm", new_session.dm, empty_flag);
                let session_date_str =
                    extractor.extract("session_date", new_session.session_date, empty_flag);
                let colour = extractor.extract("colour", new_session.colour, empty_flag);

                let check = extractor.check();

                match check {
                    Ok(_) => {
                        // no need to worry about err here because validator will check the regex of the date above
                        let session_date = session_date_str.parse::<DateTime<Utc>>().unwrap();
                        let insertable_session = session::InsertableSession {
                            slug: slugify(&title),
                            title,
                            description,
                            dm,
                            session_date,
                            colour,
                        };
                        match session::InsertableSession::create(
                            insertable_session,
                            auth.id,
                            &connection,
                        ) {
                            Ok(session) => ApiResponse {
                                json: json!({ "session": session }),
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
pub struct UpdateSessionData {
    #[validate(length(min = 1, code = "Title must be at least 1 character long"))]
    pub title: Option<String>,
    #[validate(length(min = 1, code = "Description must be at least 1 character long"))]
    pub description: Option<String>,
    #[validate(regex(
        path = "SESSION_DATE_FORMAT",
        code = "must be valid output of JS toISOString()"
    ))]
    pub session_date: Option<String>,
    #[validate(custom = "validate_colour")]
    pub colour: Option<String>,
    slug: Option<String>,
}

#[patch("/<session_id>", format = "application/json", data = "<session>")]
pub fn patch_session(
    auth: Result<Auth, JsonValue>,
    session: Result<Json<UpdateSessionData>, JsonError>,
    session_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. session does not exist)
            let session_details =
                session::Session::find(session_id, &connection).map_err(|response| response)?;

            if auth.id == session_details.dm {
                let mut session_update_details = session.map_err(|json_error| {
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

                if let Some(ref title) = session_update_details.title {
                    session_update_details.slug = Some(slugify(&title));
                }

                let session_validator_details = session_update_details.clone();

                let empty_flag = true; // i.e. do not emit error if empty
                let mut extractor = FieldValidator::validate(&session_update_details);
                let _title =
                    extractor.extract("title", session_validator_details.title, empty_flag);
                let _description = extractor.extract(
                    "description",
                    session_validator_details.description,
                    empty_flag,
                );
                let _sess_date = extractor.extract(
                    "session_date",
                    session_validator_details.session_date,
                    empty_flag,
                );
                let _colour =
                    extractor.extract("colour", session_validator_details.colour, empty_flag);

                extractor.check()?;

                let session_date: Option<DateTime<Utc>>;

                if let Some(ref _date) = session_update_details.session_date {
                    session_date = Some(
                        session_update_details
                            .session_date
                            .unwrap()
                            .parse::<DateTime<Utc>>()
                            .unwrap(),
                    );
                } else {
                    session_date = None;
                }

                let update_session = session::UpdateSession {
                    title: session_update_details.title,
                    description: session_update_details.description,
                    session_date,
                    colour: session_update_details.colour,

                    slug: session_update_details.slug,
                    dm: None,
                };

                session::UpdateSession::update(session_id, &update_session, &connection)
                    .map(|session| ApiResponse {
                        json: json!({ "session": session }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the DM" }),
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
pub struct UpdateSessionDMData {
    #[validate(custom = "validate_user_exists")]
    pub dm: Option<i32>,
}

#[patch("/<session_id>/dm", format = "application/json", data = "<session>")]
pub fn patch_dm_of_session(
    auth: Result<Auth, JsonValue>,
    session: Result<Json<UpdateSessionDMData>, JsonError>,
    session_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. session does not exist)
            let session_details =
                session::Session::find(session_id, &connection).map_err(|response| response)?;

            if auth.id == session_details.dm {
                let session_update_details = session.map_err(|json_error| {
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

                let session_validator_details = session_update_details.clone();

                let empty_flag = true; // i.e. do not emit error if empty
                let mut extractor = FieldValidator::validate(&session_update_details);
                let new_dm = extractor.extract("dm", session_validator_details.dm, empty_flag);

                extractor.check()?;

                // we also need to check if the new dm is a member of the session
                let _user_in_session = session::SessionUser::check_user_in_session(
                    &session_details,
                    new_dm,
                    &connection,
                )
                .map_err(|response| response)?;

                let update_session = session::UpdateSession {
                    title: None,
                    description: None,
                    session_date: None,
                    colour: None,
                    slug: None,

                    dm: session_update_details.dm,
                };

                session::UpdateSession::update(session_id, &update_session, &connection)
                    .map(|session| ApiResponse {
                        json: json!({ "session": session }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the DM" }),
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

#[get("/<session_id>/join", format = "application/json")]
pub fn join_session(
    auth: Result<Auth, JsonValue>,
    session_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. session does not exist)
            let session_details =
                session::Session::find(session_id, &connection).map_err(|response| response)?;

            session::Session::request_to_join(session_details.id, auth.id, &connection)
                .map(|_| ApiResponse {
                    json: json!({ "message": "requested to join session successfully" }),
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

#[get("/<session_id>/accept/<user_id>", format = "application/json")]
pub fn accept_to_session(
    auth: Result<Auth, JsonValue>,
    session_id: i32,
    user_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            let session_details =
                session::Session::find(session_id, &connection).map_err(|response| response)?;

            if auth.id == session_details.dm {
                session::Session::accept_to_join(&session_details, user_id, &connection)
                    .map(|_| ApiResponse {
                        json: json!({ "message": "successfully accepted user to session" }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the DM" }),
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

#[get("/<session_id>/invite/<user_id>", format = "application/json")]
pub fn invite_to_session(
    auth: Result<Auth, JsonValue>,
    session_id: i32,
    user_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. session does not exist)
            let session_details =
                session::Session::find(session_id, &connection).map_err(|response| response)?;
            if auth.id == session_details.dm {
                session::Session::invite_to_join(session_details.id, user_id, &connection)
                    .map(|_| ApiResponse {
                        json: json!({ "message": "invited user to join session successfully" }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the DM" }),
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

#[get("/<session_id>/accept_invite", format = "application/json")]
pub fn accept_invite_to_session(
    auth: Result<Auth, JsonValue>,
    session_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. session does not exist)
            let session_details =
                session::Session::find(session_id, &connection).map_err(|response| response)?;

            session::Session::accept_invite_to_join(&session_details, auth.id, &connection)
                .map(|_| ApiResponse {
                    json: json!({ "message": "Joined session successfully" }),
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

#[delete("/<session_id>/leave", format = "application/json")]
pub fn leave_session(
    auth: Result<Auth, JsonValue>,
    session_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. session does not exist)
            let session_details =
                session::Session::find(session_id, &connection).map_err(|response| response)?;

            if auth.id != session_details.dm {
                session::Session::delete_user(&session_details, auth.id, &connection)
                    .map(|_| ApiResponse {
                        json: json!({ "message": "left session successfully" }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are the DM, so you cannot leave" }),
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

#[delete("/<session_id>")]
pub fn delete_session(
    auth: Result<Auth, JsonValue>,
    session_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. session does not exist)
            let session_details =
                session::Session::find(session_id, &connection).map_err(|response| response)?;

            if auth.id == session_details.dm {
                session::Session::delete(&session_details, &connection)
                    .map(|_| ApiResponse {
                        json: json!({ "message": "session deleted successfully" }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the DM" }),
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

#[delete("/<session_id>/remove/<user_id>", format = "application/json")]
pub fn remove_user_from_session(
    auth: Result<Auth, JsonValue>,
    session_id: i32,
    user_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. session does not exist)
            let session_details =
                session::Session::find(session_id, &connection).map_err(|response| response)?;

            if auth.id == session_details.dm {
                if user_id != auth.id {
                    session::Session::delete_user(&session_details, user_id, &connection)
                        .map(|_| ApiResponse {
                            json: json!({ "message": "removed user from session successfully" }),
                            status: Status::Ok,
                        })
                        .map_err(|response| response)
                } else {
                    Err(ApiResponse {
                        json: json!({ "error": "you are the DM, so you cannot be removed" }),
                        status: Status::Unauthorized,
                    })
                }
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the DM" }),
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

#[get("/<session_id>/guest_link/<guest_name>")]
pub fn get_guest_link(
    auth: Result<Auth, JsonValue>,
    session_id: i32,
    guest_name: String,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. session does not exist)
            let session_details =
                session::Session::find(session_id, &connection).map_err(|response| response)?;

            if auth.id == session_details.dm {
                session::Session::create_guest_token(session_details.id, &guest_name, &connection)
                    .map(|guest_token| ApiResponse {
                        json: json!({
                            "guest_link":
                                format!(
                                    "http://localhost:8000/#/session/{}?guest={}",
                                    session_details.slug, guest_token
                                )
                        }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the DM" }),
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

#[get("/<session_id>/guest/<guest_token>")]
pub fn get_session_as_guest(
    session_id: i32,
    guest_token: &RawStr,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    if let Some(guest_auth) = GuestAuth::decode_guest_token(guest_token.as_str()) {
        if guest_auth.session_id == session_id {
            session::Session::find(session_id, &connection)
                .map(|session_json| ApiResponse {
                    json: json!({ "session": session_json }),
                    status: Status::Ok,
                })
                .map_err(|response| response)
        } else {
            Err(ApiResponse {
                json: json!({ "error": "this guest link is not valid for that session" }),
                status: Status::Unauthorized,
            })
        }
    } else {
        Err(ApiResponse {
            json: json!({ "error": "not a valid guest link" }),
            status: Status::Unauthorized,
        })
    }
}

#[delete("/<session_id>/guest/<guest_id>")]
pub fn remove_guest_from_session(
    auth: Result<Auth, JsonValue>,
    session_id: i32,
    guest_id: i32,
    connection: DnDAgendaDB,
) -> Result<ApiResponse, ApiResponse> {
    match auth {
        Ok(auth) => {
            // get error if there is any (i.e. session does not exist)
            let session_details =
                session::Session::find(session_id, &connection).map_err(|response| response)?;

            if auth.id == session_details.dm {
                session::Session::delete_guest(&session_details, guest_id, &connection)
                    .map(|_| ApiResponse {
                        json: json!({ "message": "removed guest from session successfully" }),
                        status: Status::Ok,
                    })
                    .map_err(|response| response)
            } else {
                Err(ApiResponse {
                    json: json!({ "error": "you are not the DM" }),
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

#[get("/<session_id>/guests")]
pub fn get_guests(
    auth: Result<Auth, JsonValue>,
    session_id: i32,
    connection: DnDAgendaDB,
) -> ApiResponse {
    match auth {
        Ok(_auth) => match session::Session::read_guests(session_id, &connection) {
            Ok(guests) => ApiResponse {
                json: json!({ "guests": guests }),
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

fn slugify(title: &str) -> String {let dm = User::find(new_session.dm, connection)
                    .map(|user| user.to_profile())
                    .map_err(|response| response);

                populate(&new_session, dm, connection)
                    .map(|session_json| session_json)
                    .map_err(|response| response)
    slug::slugify(title)
}
