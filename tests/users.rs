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
        Status::Created => check_user_response(response),
        Status::UnprocessableEntity => check_user_validation_errors(response),
        _ => panic!("Got status: {}", status),
    }
}

#[test]
/// Registration with the same email must fail
fn test_register_with_duplicated_email() {
    let client = test_client();
    register(client, "tester", "tester@test.com", PASSWORD);

    let response = &mut client
        .post("/api/v1/users")
        .header(ContentType::JSON)
        .body(json_string!({
                "username": "tester_1",
                "email": "tester@test.com",
                "password": PASSWORD,
        }))
        .dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);

    let value = response_json_value(response);
    let error = value
        .get("errors")
        .expect("must have a 'errors' field")
        .get("email")
        .expect("must have a 'email' field")
        .get(0)
        .and_then(|error| error.as_str());

    assert_eq!(error, Some("has already been taken"))
}

#[test]
/// Login with wrong password must fail.
fn test_incorrect_login() {
    let client = test_client();
    let response = &mut client
        .post("/api/v1/users/login")
        .header(ContentType::JSON)
        .body(json_string!({"email": EMAIL, "password": "foo"}))
        .dispatch();

    assert_eq!(response.status(), Status::Unauthorized);

    let value = response_json_value(response);
    let login_error = value
        .get("error")
        .expect("must have an 'error' field")
        .as_str();

    assert_eq!(login_error, Some("incorrect email/password"));
}

#[test]
/// Try login checking that access Token is present.
fn test_login() {
    let client = test_client();
    let response = &mut client
        .post("/api/v1/users/login")
        .header(ContentType::JSON)
        .body(json_string!({"email": EMAIL, "password": PASSWORD}))
        .dispatch();

    let value = response_json_value(response);
    value
        .get("user")
        .expect("must have a 'user' field")
        .get("token")
        .expect("user has token")
        .as_str()
        .expect("token must be a string");
}

#[test]
/// Check that `/users` endpoint returns expected data.
fn test_get_users() {
    let client = test_client();
    let token = login(&client);
    let response = &mut client
        .get("/api/v1/users")
        .header(token_header(token))
        .dispatch();

    let value = response_json_value(response);
    value
        .get("users")
        .expect("must have a 'users' field")
        .as_array()
        .expect("users array must be an array");
}

#[test]
/// Check that `/users` endpoint returns expected data.
fn test_get_users_no_token() {
    let client = test_client();
    let response = &mut client.get("/api/v1/users").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);

    let value = response_json_value(response);
    let token_error = value
        .get("error")
        .expect("must have an 'error' field")
        .as_str();

    assert_eq!(token_error, Some("unauthorised"));
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
        .get("errors")
        .expect("must have a 'errors' field")
        .get("username") // if duplicate user, the username error is output first so check for that
        .expect("must have a 'username' field")
        .get(0)
        .and_then(|error| error.as_str());

    assert_eq!(error, Some("has already been taken"))
}
