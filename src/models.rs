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

use std::{borrow::Cow, path::PathBuf, str::FromStr};

use diesel::{Insertable, Queryable};
use rocket::{http::RawStr, request::FromFormValue, FromForm};
use serde::{Deserialize, Serialize};

use super::schema::*;

/// The request guard for having a database connection.
#[rocket_contrib::database("user_db")]
#[repr(transparent)]
pub struct DbConn(diesel::PgConnection);

#[repr(transparent)]
pub struct DataDir(pub PathBuf);

/// The keys for encoding and decoding JWT keys.
#[derive(Debug)]
pub struct JWTKey<'a> {
    pub encoder: jsonwebtoken::EncodingKey,
    pub decoder: jsonwebtoken::DecodingKey<'a>,
}

/// The type of a JWT bearer token. A request guard implementation is defined in crate::auth.
#[derive(Serialize, Deserialize)]
pub struct BearerToken {
    pub iat: u64,
    pub exp: u64,
    pub username: String,
}

#[repr(transparent)]
pub struct UserToken(pub BearerToken);

#[derive(Debug)]
pub enum AuthError {
    Missing,
    Invalid,
    Expired,
}

/// The query type of a user entry.
#[derive(Identifiable, Queryable, PartialEq, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
}

/// The insertion type of a new user entry.
#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
}

/// The query type of a user authentication entry.
#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(User)]
pub struct UserAuth {
    pub id: i32,
    pub user_id: i32,
    pub password_hash: String,
    pub salt: String,
}

/// The insertion type of a new user authentication entry.
#[derive(Insertable)]
#[table_name = "user_auths"]
pub struct NewUserAuth {
    pub user_id: i32,
    pub password_hash: String,
    pub salt: String,
}

/// The input data type of a register request.
#[derive(FromForm, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub password: String,
}

/// The response type of a register request.
#[derive(Serialize, Deserialize)]
pub struct RegisterResp {
    pub token: String,
}

/// The input data type of a login request.
#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

/// The response type of a login request.
#[derive(Serialize, Deserialize)]
pub struct LoginResp {
    pub token: String,
}

/// A database entry for a public card.
#[derive(Queryable, PartialEq, Debug)]
pub struct Card {
    pub id: i32,
    pub phrase: String,
    pub images: Vec<String>,
    pub categories: Vec<String>,
}

/// A database entry for a user specific card.
#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(User)]
pub struct UserCard {
    pub id: i32,
    pub user_id: i32,
    pub phrase: String,
    pub images: Vec<String>,
    pub categories: Vec<String>,
}

#[derive(Insertable)]
#[table_name = "user_cards"]
pub struct NewUserCard<'a> {
    pub user_id: i32,
    pub phrase: Cow<'a, str>,
    pub images: Vec<Cow<'a, str>>,
    pub categories: Vec<Cow<'a, str>>,
}

/// The response type of a card query.
#[derive(Serialize, Deserialize)]
pub struct CardPageResp<'a> {
    pub cards: Vec<CardEntry<'a>>,
    pub next_page: Option<String>,
}

/// An entry in the response type of card request.
#[derive(Serialize, Deserialize, Debug)]
pub struct CardEntry<'a> {
    pub phrase: Cow<'a, str>,
    pub images: Vec<Cow<'a, str>>,
    pub categories: Vec<Cow<'a, str>>,
}

/// An update to a card entry.
#[derive(Deserialize, AsChangeset, Insertable, Debug)]
#[table_name = "user_cards"]
pub struct UpdateCardEntry {
    pub phrase: Option<String>,
    pub images: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
}

/// A comma seperated value list.
#[derive(Debug)]
pub struct CSVec<T>(pub Vec<T>);

impl<T> CSVec<T> {
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<'v, T> FromFormValue<'v> for CSVec<T>
where
    T: FromStr,
{
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<CSVec<T>, &'v RawStr> {
        match form_value.url_decode() {
            Ok(s) => {
                let mut vs: Vec<T> = vec![];
                for v in s.split(",") {
                    match T::from_str(v) {
                        Ok(decoded) => vs.push(decoded),
                        Err(_) => return Err(form_value),
                    }
                }
                Ok(CSVec(vs))
            }
            Err(_) => Err(form_value),
        }
    }
}
