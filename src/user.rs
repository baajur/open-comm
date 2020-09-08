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

use jsonwebtoken::DecodingKey;
use mobc_postgres::tokio_postgres::row::Row;
use serde::{Deserialize, Serialize};
use warp::{
    reply::{json, Json},
    Filter, Rejection, Reply,
};

use crate::{db, guard, Error};

pub fn api(
    db_pool: db::Pool,
    jwt_key: DecodingKey<'static>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::get()
        .and(warp::path("user"))
        .and(guard::user_resource(jwt_key))
        .and(warp::path::end())
        .and(guard::with_db(db_pool))
        .and_then(read_user)
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub username: String,
}

impl<'a> From<&'a Row> for User {
    fn from(item: &'a Row) -> Self {
        User {
            username: item.get("username"),
        }
    }
}

async fn read_user(username: String, pool: db::Pool) -> Result<Json, Rejection> {
    let query = db::get_db_conn(&pool)
        .await?
        .query_one(
            "SELECT username FROM users WHERE username = $1",
            &[&username],
        )
        .await;

    match query {
        Ok(row) => Ok(json(&User::from(&row))),
        Err(e) => Err(Rejection::from(Error::DBError(e))),
    }
}
