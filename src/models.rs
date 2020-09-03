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

use rocket::FromForm;
use diesel::{Insertable, Queryable};
use serde::{Serialize, Deserialize};

use super::schema::{users, user_auths};

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserToken {
    pub iat: u64,
    pub exp: u64,
    pub username: String,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser {
    pub username: String,
}

#[derive(Queryable)]
pub struct UserAuth {
    pub id: i32,
    pub user_id: i32,
    pub password_hash: String,
    pub salt: String,
}

#[derive(Insertable)]
#[table_name="user_auths"]
pub struct NewUserAuth {
    pub user_id: i32,
    pub password_hash: String,
    pub salt: String,
}

#[derive(FromForm, Deserialize)]
pub struct NewUserForm {
    pub username: String,
    pub password: String,
}

#[derive(FromForm, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub enum AuthError {
    Missing,
    Invalid,
    Expired,
}
