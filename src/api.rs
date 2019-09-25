use rocket::http::{ContentType, Status};
use rocket::request::{self, FromRequest, Request};
use rocket::response::{self, Responder, Response};
use rocket_contrib::json::JsonValue;
use std::collections::HashMap;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Debug)]
pub struct ApiResponse {
    pub json: JsonValue,
    pub status: Status,
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

use rocket::Outcome;

use frank_jwt as jwt;

use crate::config;

#[derive(Debug, Deserialize, Serialize)]
pub struct Auth {
    /// expiration timestamp
    pub exp: i64,
    /// user id
    pub id: i32,
    pub username: String,
}

impl Auth {
    pub fn token(&self) -> String {
        let headers = json!({});
        let payload = json!(self);
        jwt::encode(
            headers.0,
            &config::SECRET.to_string(),
            &payload,
            jwt::Algorithm::HS256,
        )
        .expect("jwt")
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Auth {
    type Error = JsonValue;

    /// Extract Auth token from the "Authorization" header.
    ///
    /// Handlers with Auth guard will fail with 503 error.
    /// Handlers with Option<Auth> will be called with None.
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Auth, Self::Error> {
        if let Some(auth) = extract_auth_from_request(request) {
            // ie assignment successful
            Outcome::Success(auth)
        } else {
            Outcome::Failure((Status::Unauthorized, json!({"error": "unauthorised"})))
        }
    }
}

fn extract_auth_from_request(request: &Request) -> Option<Auth> {
    request
        .headers()
        .get_one("authorization")
        .and_then(extract_token_from_header)
        .and_then(decode_token)
}

fn extract_token_from_header(header: &str) -> Option<&str> {
    if header.starts_with(config::TOKEN_PREFIX) {
        Some(&header[config::TOKEN_PREFIX.len()..])
    } else {
        None
    }
}

/// Decode token into `Auth` struct. If any error is encountered, log it
/// an return None.
fn decode_token(token: &str) -> Option<Auth> {
    jwt::decode(token, &config::SECRET.to_string(), jwt::Algorithm::HS256, &jwt::ValidationOptions::default())
        .map(|(_, payload)| {
            serde_json::from_value::<Auth>(payload)
                .map_err(|err| {
                    eprintln!("Auth serde decode error: {:?}", err);
                })
                .ok()
        })
        .unwrap_or_else(|err| {
            eprintln!("Auth decode error: {:?}", err);
            None
        })
}

pub struct FieldValidator {
    errors: ValidationErrors,
}

impl Default for FieldValidator {
    fn default() -> Self {
        Self {
            errors: ValidationErrors::new(),
        }
    }
}

impl FieldValidator {
    pub fn validate<T: Validate>(model: &T) -> Self {
        Self {
            errors: model.validate().err().unwrap_or_else(ValidationErrors::new),
        }
    }

    pub fn check(self) -> Result<(), ApiResponse> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            let errors = self
                .errors
                .field_errors()
                .into_iter()
                .map(|(field, errors)| {
                    let messages = errors
                        .clone()
                        .into_iter()
                        .map(|err| (err.code, err.message))
                        .collect();
                    (field, messages)
                })
                .collect::<HashMap<_, Vec<_>>>();
            Err(ApiResponse {
                json: json!({ "errors": errors }),
                status: Status::UnprocessableEntity,
            })
        }
    }

    pub fn extract<T>(&mut self, field_name: &'static str, field: Option<T>) -> T
    where
        T: Default,
    {
        field.unwrap_or_else(|| {
            self.errors
                .add(field_name, ValidationError::new("can't be blank"));
            T::default()
        })
    }
}
