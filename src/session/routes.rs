use crate::database::DnDAgendaDB;
use crate::session;

use rocket_contrib::json::Json;
use rocket_contrib::json::JsonError;
use rocket_contrib::json::JsonValue;

use crate::api::ApiResponse;
use crate::api::Auth;
use rocket::http::Status;

use crate::api::validate_colour;
use crate::api::validate_user_exists;
use crate::api::FieldValidator;
use validator::Validate;

use chrono::{DateTime, Utc};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
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
        Ok(_auth) => match session::Session::find(session_id, &connection) {
            Ok(session) => ApiResponse {
                json: json!({ "session": session }),
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
        code = "must be valid ouput of JS toISOString()"
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
        Ok(_auth) => match session {
            Ok(json_session) => {
                let new_session = json_session.into_inner();

                let mut extractor = FieldValidator::validate(&new_session);
                let title = extractor.extract("title", new_session.title);
                let description = extractor.extract("description", new_session.description);
                let dm = extractor.extract("dm", new_session.dm);
                let session_date_str = extractor.extract("session_date", new_session.session_date);
                let colour = extractor.extract("colour", new_session.colour);

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
                        match session::InsertableSession::create(insertable_session, &connection) {
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

fn slugify(title: &str) -> String {
    slug::slugify(title)
}
