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

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::{
    fairing::AdHoc,
    response::content::{Html, JavaScript},
};

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use jsonwebtoken::{DecodingKey, EncodingKey};

pub mod auth;
pub mod models;
pub mod schema;

use models::*;

#[get("/elm.js")]
fn gui_lib() -> JavaScript<&'static str> {
    JavaScript(include_str!(env!("GUI_LIB")))
}

#[get("/")]
fn gui() -> Html<&'static str> {
    Html(include_str!(env!("GUI_INDEX")))
}

embed_migrations!("migrations");

fn main() {
    let secret = b"secret";
    let jwt_key = JWTKey {
        encoder: EncodingKey::from_secret(secret),
        decoder: DecodingKey::from_secret(secret),
    };
    rocket::ignite()
        .attach(UserDbConn::fairing())
        .attach(AdHoc::on_attach("Run migrations", |r| {
            if let Some(conn) = UserDbConn::get_one(&r) {
                match embedded_migrations::run(&conn.0) {
                    Ok(_) => Ok(r),
                    Err(_) => Err(r),
                }
            } else {
                Ok(r)
            }
        }))
        .mount("/", routes![gui, gui_lib])
        .mount("/api", routes![auth::register, auth::login])
        .manage(jwt_key)
        .launch();
}
