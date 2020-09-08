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

use crypto::{digest::Digest, sha3::Sha3};
use jsonwebtoken::{
    decode as jwt_decode, encode as jwt_encode, DecodingKey, EncodingKey, Header as JWTHeader,
    Validation,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use warp::{
    reply::{json, Json},
    Filter, Rejection, Reply,
};

use crate::{db, guard, Error};

const TOKEN_EXPIRATION: u64 = 604800;
const SALT_SIZE: usize = 16;

pub fn api(
    db_pool: db::Pool,
    jwt_key: EncodingKey,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(guard::with_db(db_pool.clone()))
        .and(guard::with_jwt_priv_key(jwt_key.clone()))
        .and(warp::body::json())
        .and_then(login_handler);
    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::path::end())
        .and(guard::with_db(db_pool.clone()))
        .and(guard::with_jwt_priv_key(jwt_key.clone()))
        .and(warp::body::json())
        .and_then(register_handler);

    register.or(login)
}

#[derive(Deserialize)]
pub struct Register {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterResp {
    pub token: String,
}

async fn register_handler(
    pool: db::Pool,
    jwt_key: EncodingKey,
    mut login: Login,
) -> Result<Json, Rejection> {
    let conn = db::get_db_conn(&pool).await?;

    let user_ins = conn
        .query_one(
            "INSERT INTO users (username) VALUES ($1) RETURNING (id)",
            &[&login.username],
        )
        .await
        .map_err(Error::DBError)?;

    let user_id: i32 = user_ins.get("id");

    let salt = random_string(SALT_SIZE);
    login.password.extend(salt.chars());
    let password_hash = secure_hash(login.password);

    conn.execute(
        "INSERT INTO user_auths (user_id, password_hash, salt) VALUES ($1, $2, $3)",
        &[&user_id, &password_hash, &salt],
    )
    .await
    .map_err(Error::DBError)?;

    Ok(json(&RegisterResp {
        token: generate_jwt(login.username, &jwt_key)?,
    }))
}

#[derive(Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResp {
    pub token: String,
}

async fn login_handler(
    pool: db::Pool,
    jwt_key: EncodingKey,
    mut login: Login,
) -> Result<Json, Rejection> {
    let conn = db::get_db_conn(&pool).await?;

    let query = conn
        .query_one(
            r#"
            SELECT password_hash, salt
            FROM user_auths
            INNER JOIN users
            ON users.id = user_auths.user_id
            WHERE users.username = $1
            "#,
            &[&login.username],
        )
        .await
        .map_err(Error::DBError)?;
    let password_hash: String = query.get("password_hash");
    let salt: String = query.get("salt");
    login.password.extend(salt.chars());
    if secure_hash(login.password) == password_hash {
        Ok(json(&LoginResp {
            token: generate_jwt(login.username, &jwt_key)?,
        }))
    } else {
        Err(Rejection::from(Error::Unauthorized))
    }
}

#[derive(Deserialize, Serialize)]
pub struct BearerToken {
    pub iat: u64,
    pub exp: u64,
    pub username: String,
}

impl BearerToken {
    pub fn verify_token<'a>(jwt_key: &'a DecodingKey, raw_jwt: &'a str) -> Result<Self, Error> {
        let jwt_validation = Validation {
            leeway: 60,
            ..Default::default()
        };
        Ok(jwt_decode::<BearerToken>(raw_jwt, &jwt_key, &jwt_validation)?.claims)
    }
}

fn secure_hash(s: String) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input_str(s.as_str());
    hasher.result_str()
}

pub fn random_string(len: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .collect::<String>()
}

fn generate_jwt(username: String, encoder: &EncodingKey) -> Result<String, Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let payload = BearerToken {
        iat: now,
        exp: now + TOKEN_EXPIRATION,
        username,
    };
    Ok(jwt_encode(&JWTHeader::default(), &payload, &encoder)?)
}
