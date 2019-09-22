//! Test registration and login

mod common;

use common::*;
use rocket::http::{ContentType, Status};
use rocket::local::LocalResponse;

#[test]
/// Register new user, handling repeated registration as well.
fn test_register() {
    let client = test_client();
    let response = &mut client
        .post("/api/v1/users")
        .header(ContentType::JSON)
        .body(json_string!({ "username": USERNAME, "email": EMAIL, "password": PASSWORD }))
        .dispatch();

    let status = response.status();
    // If user was already created we should get an UnprocessableEntity or Ok otherwise.
    //
    // As tests are ran in an indepent order `login()` probably has already created smoketest user.
    // And so we gracefully handle "user already exists" error here.
    match status {
        Status::Ok => check_user_response(response),
        Status::UnprocessableEntity => check_user_validation_errors(response),
        _ => panic!("Got status: {}", status),
    }
}

// Utility functions

/// Assert that body contains "user" response with expected fields.
fn check_user_response(response: &mut LocalResponse) {
    let value = response_json_value(response);
    let user = value.get("user").expect("must have a 'user' field");

    assert_eq!(user.get("email").expect("user has email"), EMAIL);
    assert_eq!(user.get("username").expect("user has username"), USERNAME);
    assert!(user.get("bio").is_some());
    assert!(user.get("image").is_some());
    assert!(user.get("token").is_some());
}

fn check_user_validation_errors(response: &mut LocalResponse) {
    let value = response_json_value(response);
    let error = value
        .get("error")
        .expect("must have an 'error' field")
        .as_str();

    assert!((error == Some("email already taken")) || (error == Some("username already taken")))
}
