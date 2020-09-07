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

#[macro_use]
extern crate rocket;

use rocket::{
    http::{Accept, ContentType, Header, Status},
    local::Client,
};

use open_comm::{auth, card, construct_rocket, models::*};

mod common;

#[test]
fn auth_flow() -> Result<(), Box<dyn std::error::Error>> {
    common::setup();
    let client = Client::untracked(construct_rocket()).expect("valid rocket instance");

    // Invalid login.
    let failed_login = client
        .post(uri!(auth::login).to_string())
        .body(r#"{"username":"foo","password":"bar"}"#)
        .header(ContentType::JSON)
        .header(Accept::JSON)
        .dispatch();
    assert_eq!(
        failed_login.status(),
        Status::Unauthorized,
        "Invalid login unauthorized."
    );

    // Unauthenticated and unauthorized request.
    let failed_card_resp = client
        .get(uri!(card::list_user_cards: "foo", _, _, _, _).to_string())
        .header(Accept::JSON)
        .dispatch();
    assert_eq!(
        failed_card_resp.status(),
        Status::Unauthorized,
        "Tokenless request unauthorized."
    );

    // Register user.rust lang orm
    let mut reg_resp = client
        .post(uri!(auth::register).to_string())
        .body(r#"{"username":"foo","password":"bar"}"#)
        .header(ContentType::JSON)
        .header(Accept::JSON)
        .dispatch();
    assert_eq!(reg_resp.status(), Status::Ok, "Create user failed.");
    let _reg_body: RegisterResp = serde_json::from_str(reg_resp.body_string().unwrap().as_str())?;

    // Authenticate user.
    let mut login_resp = client
        .post(uri!(auth::login).to_string())
        .body(r#"{"username":"foo","password":"bar"}"#)
        .header(ContentType::JSON)
        .header(Accept::JSON)
        .dispatch();
    assert_eq!(login_resp.status(), Status::Ok, "Token issue failed.");
    let login_body: LoginResp = serde_json::from_str(login_resp.body_string().unwrap().as_str())?;

    // Authentic user can access authorized resource.
    let mut card_resp = client
        .get(uri!(card::list_user_cards: "foo", _, _, _, _).to_string())
        .header(Accept::JSON)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", login_body.token),
        ))
        .dispatch();
    assert_eq!(
        card_resp.status(),
        Status::Ok,
        "Authentic user cannot access authorized resource."
    );
    let _card_body: CardPageResp = serde_json::from_str(card_resp.body_string().unwrap().as_str())?;

    // Authentic user can access authorized resource using param token.
    let mut card_uri = uri!(card::list_user_cards: "foo", _, _, _, _).to_string();
    card_uri.push_str(format!("?access_token={}", login_body.token).as_str());
    let mut card_resp = client
        .get(card_uri)
        .header(Accept::JSON)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", login_body.token),
        ))
        .dispatch();
    assert_eq!(
        card_resp.status(),
        Status::Ok,
        "Authentic user cannot access authorized resource using param token."
    );
    let _card_body: CardPageResp = serde_json::from_str(card_resp.body_string().unwrap().as_str())?;

    // Register a different user.
    let mut other_reg_resp = client
        .post(uri!(auth::register).to_string())
        .body(r#"{"username":"bar","password":"foo"}"#)
        .header(ContentType::JSON)
        .header(Accept::JSON)
        .dispatch();
    assert_eq!(other_reg_resp.status(), Status::Ok, "Create user failed.");
    let other_reg_body: RegisterResp =
        serde_json::from_str(other_reg_resp.body_string().unwrap().as_str())?;

    // Authentic user cannot access unauthorized resource.
    let other_card_resp = client
        .get(uri!(card::list_user_cards: "foo", _, _, _, _).to_string())
        .header(Accept::JSON)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", other_reg_body.token),
        ))
        .dispatch();
    assert_eq!(
        other_card_resp.status(),
        Status::Unauthorized,
        "Authentic user can access unauthorized resource."
    );
    Ok(())
}
