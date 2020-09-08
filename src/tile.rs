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

use crate::{auth::BearerToken, db, guard, Error};

pub fn api(
    db_pool: db::Pool,
    jwt_key: DecodingKey<'static>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::get()
        .and(guard::user_resource(jwt_key))
        .and(warp::path("tiles"))
        .and(warp::path::end())
        .and(guard::with_db(db_pool))
        .and_then(list_tiles_handler)
}

#[derive(Serialize, Deserialize)]
pub struct Tile {
    phrase: String,
    images: Vec<String>,
    categories: Vec<String>,
}

impl<'a> From<&'a Row> for Tile {
    fn from(item: &'a Row) -> Self {
        Self {
            phrase: item.get("phrase"),
            images: item.get("images"),
            categories: item.get("categories"),
        }
    }
}

pub async fn list_tiles_handler(tok: BearerToken, pool: db::Pool) -> Result<Json, Rejection> {
    Ok(json(
        &db::get_db_conn(&pool)
            .await?
            .query(
                r#"
                SELECT phrase, images, categories
                FROM user_tiles
                INNER JOIN users
                ON users.id = user_tiles.user_id
                WHERE users.username = $1
                "#,
                &[&tok.username],
            )
            .await
            .map_err(Error::DBError)?
            .iter()
            .map(Tile::from)
            .collect::<Vec<Tile>>(),
    ))
}