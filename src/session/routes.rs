use crate::database::DnDAgendaDB;
use crate::session;

use rocket_contrib::json::Json;
use rocket_contrib::json::JsonError;
use rocket_contrib::json::JsonValue;

use crate::api::ApiResponse;
use crate::api::Auth;
use rocket::http::Status;

// use crate::api::validate_session_date;
use crate::api::FieldValidator;
use validator::Validate;

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use slug;

use regex::Regex;

lazy_static! {
    static ref SESSION_DATE_FORMAT: Regex =
        Regex::new(r"\d{4,}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{3}Z").unwrap();
}

#[derive(Deserialize)]
pub struct NewSession {
    session: NewSessionData,
}

#[derive(Deserialize, Validate)]
pub struct NewSessionData {
    #[validate(length(min = 1, message = "Title must be at least 1 character long"))]
    pub title: Option<String>,
    #[validate(length(min = 1, message = "Description must be at least 1 character long"))]
    pub description: Option<String>,
    pub dm: Option<i32>,
    #[validate(regex = "SESSION_DATE_FORMAT")]
    pub session_date: Option<String>,
    #[validate(custom = "check_session_date")]
    pub colour: Option<String>,
}

#[post("/", format = "application/json", data = "<session>")] // data attribute tells rocket to expect Body Data - then map the body to a parameter
pub fn create(
    session: Result<Json<NewSession>, JsonError>,
    connection: DnDAgendaDB,
) -> ApiResponse {
    match session {
        Ok(json_session) => {
            let new_session = json_session.into_inner().session;

            let mut extractor = FieldValidator::validate(&new_session);
            let title = extractor.extract("title", new_session.title);
            let description = extractor.extract("description", new_session.description);
            let session_date = extractor.extract("session_date", new_session.session_date);

            let check = extractor.check();
            match check {
                Ok(_) => {
                    let insertable_session = session::InsertableSession {
                        slug: &slugify(title),
                        title,
                        description,
                        dm: Some(Auth),
                        session_date,
                        colour
                    };
                    session::InsertableUser::create(insertable_user, &connection)
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
    }
}

pub fn slugify(title: &str) -> String {
    if cfg!(feature = "random-suffix") {
        format!("{}-{}", slug::slugify(title), generate_suffix(6))
    } else {
        slug::slugify(title)
    }
}

fn generate_suffix(len: usize) -> String {
    let mut rng = thread_rng();
    (0..len).map(|_| rng.sample(Alphanumeric)).collect()
}