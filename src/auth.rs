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

use std::{
    iter,
    time::{SystemTime, UNIX_EPOCH},
};

use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request, State},
    Route,
};
use rocket_contrib::json::Json;

use diesel::{
    prelude::*,
    result::{DatabaseErrorKind::UniqueViolation, Error::DatabaseError},
};

use jsonwebtoken::{
    decode as jwt_decode, encode as jwt_encode, errors::ErrorKind as JWTErrorKind,
    Header as JWTHeader, Validation,
};

use crypto::{digest::Digest, sha3::Sha3};

use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::models::*;

pub fn routes() -> Vec<Route> {
    routes![register, login]
}

#[post("/api/register", data = "<new_user_form>")]
pub fn register(
    db: DbConn,
    jwt_key: State<JWTKey>,
    new_user_form: Json<RegisterForm>,
) -> Result<Json<RegisterResp>, Status> {
    use crate::schema::{user_auths, users};

    let mut new_user = new_user_form.into_inner();
    let DbConn(conn) = db;

    // Insert the user.
    let user: NewUser = NewUser {
        username: new_user.username.clone(),
    };
    let user_id = diesel::insert_into(users::table)
        .values(&user)
        .returning(users::id)
        .get_result(&conn)
        .map_err(|e| match e {
            DatabaseError(UniqueViolation, _) => Status::Conflict,
            _ => Status::InternalServerError,
        })?;

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
        .map_err(|_| Status::InternalServerError)?;

    // Add the JWT as a cookie.
    let token = generate_jwt(user.username.clone(), &jwt_key.inner());
    Ok(Json(RegisterResp { token }))
}

#[post("/api/login", data = "<login_form>")]
pub fn login(
    db: DbConn,
    jwt_key: State<JWTKey>,
    login_form: Json<LoginForm>,
) -> Result<Json<LoginResp>, Status> {
    use crate::schema::{user_auths, users};

    let mut login = login_form.into_inner();
    let DbConn(conn) = db;

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

impl BearerToken {
    fn verify_token<'a>(request: &'a Request<'_>, raw_jwt: &'a str) -> Result<Self, AuthError> {
        let jwt_key = request.guard::<State<JWTKey>>().unwrap();
        let jwt_validation = Validation {
            leeway: 60,
            ..Default::default()
        };
        let decode_res =
            jwt_decode::<BearerToken>(raw_jwt, &jwt_key.inner().decoder, &jwt_validation);
        match decode_res {
            Ok(token) => Ok(token.claims),
            Err(err) => match err.into_kind() {
                JWTErrorKind::ExpiredSignature => Err(AuthError::Expired),
                _ => Err(AuthError::Invalid),
            },
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for BearerToken {
    type Error = AuthError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let mut res: Result<Self, Self::Error> = Err(AuthError::Missing);
        for bearer in request.headers().get("Authorization") {
            res = match bearer.get("Bearer ".len()..) {
                Some(raw_jwt) => match BearerToken::verify_token(request, raw_jwt) {
                    Ok(verified) => Ok(verified),
                    Err(e) => Err(e),
                },
                None => Err(AuthError::Invalid),
            };
        }
        if res.is_err() {
            res = match request.get_query_value::<String>("access_token") {
                Some(Ok(raw_jwt)) => match BearerToken::verify_token(request, raw_jwt.as_str()) {
                    Ok(verified) => Ok(verified),
                    Err(e) => Err(e),
                },
                _ => Err(AuthError::Missing),
            };
        }
        match res {
            Ok(verified) => Outcome::Success(verified),
            Err(e) => Outcome::Failure((Status::Unauthorized, e)),
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for UserToken {
    type Error = AuthError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match BearerToken::from_request(request) {
            Outcome::Success(verified) => {
                let path = request.uri().path();
                if path.starts_with(format!("/api/user/{}", verified.username).as_str()) {
                    Outcome::Success(UserToken(verified))
                } else if path.starts_with("/api/user") {
                    Outcome::Failure((Status::Unauthorized, AuthError::Invalid))
                } else {
                    panic!("Attempted to use a `UserToken` for a non-user API.");
                }
            }
            Outcome::Failure(e) => Outcome::Failure(e),
            Outcome::Forward(e) => Outcome::Forward(e),
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
    let payload = BearerToken {
        iat: now,
        exp: now + 604800,
        username,
    };
    jwt_encode(&JWTHeader::default(), &payload, &key.encoder).expect("Unable to encode JWT.")
}
