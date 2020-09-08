/*
 * Copyright (C) 2020 Oakes, Gregory <gregoryoakes@fastmail.com>
 * Author: Oakes, Gregory <gregory.oakes@fastmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use warp::Filter;

use open_comm::{auth, handle_rejects, user};

mod common;

#[tokio::test]
async fn auth_flow() {
    let pool = common::db_pool().await;
    let auth_api = auth::api(pool.clone(), common::jwt_encoder()).recover(handle_rejects);
    let user_api = user::api(pool, common::jwt_decoder()).recover(handle_rejects);

    // Register a new user.
    let res = warp::test::request()
        .method("POST")
        .path("/register")
        .header("Content-Type", "application/json")
        .json(&auth::Register {
            username: "foo".to_string(),
            password: "bar".to_string(),
        })
        .reply(&auth_api)
        .await;
    assert_eq!(res.status(), 201, "registration created new resource");

    // Attempt to repeat the registration. This request should fail.
    let res = warp::test::request()
        .method("POST")
        .path("/register")
        .header("Content-Type", "application/json")
        .json(&auth::Register {
            username: "foo".to_string(),
            password: "bar".to_string(),
        })
        .reply(&auth_api)
        .await;
    assert_eq!(
        res.status(),
        409,
        "repeat registration responds with conflict"
    );

    // Login to the new user's account.
    let res = warp::test::request()
        .method("POST")
        .path("/login")
        .header("Content-Type", "application/json")
        .json(&auth::Login {
            username: "foo".to_string(),
            password: "bar".to_string(),
        })
        .reply(&auth_api)
        .await;
    assert_eq!(res.status(), 200, "login is allowed for the new user");
    let body = String::from_utf8_lossy(res.body());
    let maybe_token = serde_json::from_str::<auth::LoginResp>(body.as_ref());
    assert!(maybe_token.is_ok(), "login responds with valid data");
    let token = maybe_token.unwrap().token;

    // Access a protected resource.
    let res = warp::test::request()
        .method("GET")
        .path("/user/foo")
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .reply(&user_api)
        .await;
    assert_eq!(
        res.status(),
        200,
        "can access protected resource with token"
    );
    let body = String::from_utf8_lossy(res.body());
    let user = serde_json::from_str::<user::User>(body.as_ref());
    assert!(user.is_ok(), "user endpoint responds with valid data");
    assert_eq!(
        "foo",
        user.unwrap().username,
        "user endpoint responds with the correct user"
    );

    // Register a new user.
    let res = warp::test::request()
        .method("POST")
        .path("/register")
        .header("Content-Type", "application/json")
        .json(&auth::Register {
            username: "bar".to_string(),
            password: "foo".to_string(),
        })
        .reply(&auth_api)
        .await;
    assert_eq!(res.status(), 201, "registration created new resource");
    let body = String::from_utf8_lossy(res.body());
    let maybe_token = serde_json::from_str::<auth::RegisterResp>(body.as_ref());
    assert!(maybe_token.is_ok(), "register responds with valid data");
    let token = maybe_token.unwrap().token;

    // Attempt to access a protected resource of a different user.
    let res = warp::test::request()
        .method("GET")
        .path("/user/foo")
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .reply(&user_api)
        .await;
    assert_eq!(
        res.status(),
        401,
        "cannot access unauthorized protected resource with token"
    );
    let body = String::from_utf8_lossy(res.body());
    let user = serde_json::from_str::<user::User>(body.as_ref());
    assert!(user.is_err(), "user endpoint responds with valid data");
}
