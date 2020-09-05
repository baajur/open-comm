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

use std::{
    iter,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[macro_use]
extern crate rocket;

use rocket::{
    fairing::AdHoc,
    http::Status,
    request::{FromRequest, Outcome, Request, State},
    response::content::{Html, JavaScript},
};
use rocket_contrib::{databases::database, json::Json};

#[macro_use]
extern crate diesel;

use diesel::{
    prelude::*,
    result::{DatabaseErrorKind::UniqueViolation, Error::DatabaseError},
};

#[macro_use]
extern crate diesel_migrations;

use jsonwebtoken::{
    decode as jwt_decode, encode as jwt_encode, errors::ErrorKind as JWTErrorKind, DecodingKey,
    EncodingKey, Header as JWTHeader, Validation,
};

use crypto::{digest::Digest, sha3::Sha3};

use rand::{distributions::Alphanumeric, thread_rng, Rng};

pub mod models;
pub mod schema;

use models::*;

#[database("user_db")]
#[repr(transparent)]
struct UserDbConn(diesel::PgConnection);

#[get("/account")]
fn user_profile(user_token: UserToken) -> String {
    user_token.username
}

#[post("/register", data = "<new_user_form>")]
fn register(
    db: UserDbConn,
    jwt_key: State<JWTKey>,
    new_user_form: Json<RegisterForm>,
) -> Result<Json<RegisterResp>, Status> {
    use schema::{user_auths, users};

    let mut new_user = new_user_form.into_inner();
    let UserDbConn(conn) = db;

    // Insert the user.
    let user: NewUser = NewUser {
        username: new_user.username.clone(),
    };
    let new_user_res = diesel::insert_into(users::table)
        .values(&user)
        .returning(users::id)
        .get_result(&conn);

    match new_user_res {
        Ok(user_id) => {
            // Hash and insert the user credentials.
            let salt = random_string(10);
            new_user.password.extend(salt.chars());
            let user_auth: NewUserAuth = NewUserAuth {
                user_id,
                salt,
                password_hash: secure_hash(new_user.password),
            };
            diesel::insert_into(user_auths::table)
                .values(&user_auth)
                .execute(&conn)
                .expect("Unable to insert new user auth into database.");

            // Add the JWT as a cookie.
            let token = generate_jwt(user.username.clone(), &jwt_key.inner());
            Ok(Json(RegisterResp { token }))
        }
        Err(DatabaseError(UniqueViolation, _)) => Err(Status::Conflict),
        Err(e) => {
            println!("{:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[post("/login", data = "<login_form>")]
fn login(
    db: UserDbConn,
    jwt_key: State<JWTKey>,
    login_form: Json<LoginForm>,
) -> Result<Json<LoginResp>, Status> {
    use schema::{user_auths, users};

    let mut login = login_form.into_inner();
    let UserDbConn(conn) = db;

    let user: User = users::table
        .filter(users::username.eq(login.username.as_str()))
        .first(&conn)
        .map_err(|_| Status::Unauthorized)?;

    let (password_hash, salt): (String, String) = UserAuth::belonging_to(&user)
        .select((user_auths::password_hash, user_auths::salt))
        .first(&conn)
        .map_err(|_| Status::Unauthorized)?;

    login.password.extend(salt.chars());

    if secure_hash(login.password) == password_hash {
        let token = generate_jwt(login.username.clone(), jwt_key.inner());
        Ok(Json(LoginResp { token }))
    } else {
        Err(Status::Unauthorized)
    }
}

#[derive(Debug)]
struct JWTKey<'a> {
    pub encoder: EncodingKey,
    pub decoder: DecodingKey<'a>,
}

impl<'a, 'r> FromRequest<'a, 'r> for UserToken {
    type Error = AuthError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match request.cookies().get_private("auth-jwt") {
            Some(raw_jwt) => {
                let jwt_key = request.guard::<State<JWTKey>>().unwrap();
                let jwt_validation = Validation {
                    leeway: 60,
                    ..Default::default()
                };
                let decode_res = jwt_decode::<UserToken>(
                    raw_jwt.value(),
                    &jwt_key.inner().decoder,
                    &jwt_validation,
                );
                match decode_res {
                    Ok(token) => Outcome::Success(token.claims),
                    Err(err) => match err.into_kind() {
                        JWTErrorKind::ExpiredSignature => {
                            Outcome::Failure((Status::Unauthorized, AuthError::Expired))
                        }
                        _ => Outcome::Failure((Status::Unauthorized, AuthError::Invalid)),
                    },
                }
            }
            None => Outcome::Failure((Status::Unauthorized, AuthError::Missing)),
        }
    }
}

fn secure_hash(s: String) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input_str(s.as_str());
    hasher.result_str()
}

fn random_string(len: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .collect::<String>()
}

fn generate_jwt(username: String, key: &JWTKey) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let payload = UserToken {
        iat: now,
        exp: now + 604800,
        username,
    };
    jwt_encode(&JWTHeader::default(), &payload, &key.encoder).expect("Unable to encode JWT.")
}

#[get("/elm.js", rank = 1)]
fn gui_lib() -> JavaScript<&'static str> {
    JavaScript(include_str!(env!("GUI_LIB")))
}

#[get("/")]
fn gui_root() -> Html<&'static str> {
    Html(include_str!(env!("GUI_INDEX")))
}

#[get("/<_p..>", rank = 2)]
fn gui(_p: PathBuf) -> Html<&'static str> {
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
        .mount("/", routes![gui_root, gui, gui_lib])
        .mount("/api", routes![user_profile, register, login])
        .manage(jwt_key)
        .launch();
}
