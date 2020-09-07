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
    let failed_card_resp = client
        .get(uri!(card::list_user_cards: "foo", _, _, _, _).to_string())
        .header(Accept::JSON)
        .dispatch();
    assert_eq!(
        failed_card_resp.status(),
        Status::Unauthorized,
        "Tokenless request unauthorized."
    );
    let mut reg_resp = client
        .post(uri!(auth::register).to_string())
        .body(r#"{"username":"foo","password":"bar"}"#)
        .header(ContentType::JSON)
        .header(Accept::JSON)
        .dispatch();
    assert_eq!(reg_resp.status(), Status::Ok, "New registration created.");
    let _reg_body: RegisterResp = serde_json::from_str(reg_resp.body_string().unwrap().as_str())?;
    let mut login_resp = client
        .post(uri!(auth::login).to_string())
        .body(r#"{"username":"foo","password":"bar"}"#)
        .header(ContentType::JSON)
        .header(Accept::JSON)
        .dispatch();
    assert_eq!(login_resp.status(), Status::Ok, "Login ok.");
    let login_body: RegisterResp =
        serde_json::from_str(login_resp.body_string().unwrap().as_str())?;
    let card_resp = client
        .get(uri!(card::list_user_cards: "foo", _, _, _, _).to_string())
        .header(Accept::JSON)
        .header(Header::new(
            "Authorization",
            format!("Bearer {}", login_body.token),
        ))
        .dispatch();
    assert_eq!(card_resp.status(), Status::Ok, "Read cards ok.");
    let mut other_reg_resp = client
        .post(uri!(auth::register).to_string())
        .body(r#"{"username":"bar","password":"foo"}"#)
        .header(ContentType::JSON)
        .header(Accept::JSON)
        .dispatch();
    assert_eq!(
        other_reg_resp.status(),
        Status::Ok,
        "Other new registration created."
    );
    let other_reg_body: RegisterResp =
        serde_json::from_str(other_reg_resp.body_string().unwrap().as_str())?;
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
        "Read cards from other user unauthorized."
    );
    Ok(())
}
