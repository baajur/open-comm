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
    http::StatusCode,
    reply::{json, with_status, Json, WithStatus},
    Filter, Rejection, Reply,
};

use crate::{db, guard, Error};

pub fn api(
    db_pool: db::Pool,
    jwt_key: DecodingKey<'static>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let create_user_tile = warp::post()
        .and(warp::path("user"))
        .and(guard::user_resource(jwt_key.clone()))
        .and(warp::path("tiles"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guard::with_db(db_pool.clone()))
        .and_then(create_user_tile);

    let list_tiles = warp::get()
        .and(warp::path("tiles"))
        .and(warp::path::end())
        .and(warp::query())
        .and(guard::with_db(db_pool.clone()))
        .and_then(list_tiles_handler);

    let list_user_tiles = warp::get()
        .and(warp::path("user"))
        .and(guard::user_resource(jwt_key))
        .and(warp::path("tiles"))
        .and(warp::path::end())
        .and(warp::query())
        .and(guard::with_db(db_pool))
        .and_then(list_user_tiles_handler);

    create_user_tile.or(list_tiles).or(list_user_tiles)
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub phrase: String,
    pub images: Vec<String>,
    pub categories: Vec<String>,
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

pub async fn create_user_tile(
    username: String,
    tile: Tile,
    pool: db::Pool,
) -> Result<WithStatus<Json>, Rejection> {
    Ok(with_status(
        json(&Tile::from(
            &db::get_db_conn(&pool)
                .await?
                .query_one(
                    r#"
                INSERT INTO tiles (user_id, phrase, images, categories)
                VALUES ((SELECT id FROM users WHERE username = $1), $2, $3, $4)
                RETURNING phrase, images, categories
                "#,
                    &[&username, &tile.phrase, &tile.images, &tile.categories],
                )
                .await
                .map_err(Error::DBError)?,
        )),
        StatusCode::CREATED,
    ))
}

#[derive(Serialize, Deserialize)]
pub struct TileQuery {
    phrase: Option<String>,
    category: Option<String>,
}

pub async fn list_tiles_handler(query: TileQuery, pool: db::Pool) -> Result<Json, Rejection> {
    Ok(json(
        &db::get_db_conn(&pool)
            .await?
            .query(
                r#"
                SELECT phrase, images, categories
                FROM tiles
                WHERE user_id IS NULL
                AND ($1 IS NULL OR phrase LIKE $1)
                AND ($2 IS NULL OR categories CONTAINS $2)
                "#,
                &[&query.phrase, &query.category],
            )
            .await
            .map_err(Error::DBError)?
            .iter()
            .map(Tile::from)
            .collect::<Vec<Tile>>(),
    ))
}

pub async fn list_user_tiles_handler(
    username: String,
    query: TileQuery,
    pool: db::Pool,
) -> Result<Json, Rejection> {
    Ok(json(
        &db::get_db_conn(&pool)
            .await?
            .query(
                r#"
                SELECT phrase, images, categories
                FROM tiles
                INNER JOIN users
                ON users.id = tiles.user_id
                WHERE (users.username = $1 OR tiles.user_id IS NULL)
                AND ($2 OR phrase LIKE $3)
                AND ($4 OR $5 = ANY(categories))
                "#,
                &[
                    &username,
                    &query.phrase.is_none(),
                    &query.phrase.unwrap_or_else(|| "".to_string()),
                    &query.category.is_none(),
                    &query.category.unwrap_or_else(|| "".to_string()),
                ],
            )
            .await
            .map_err(Error::DBError)?
            .iter()
            .map(Tile::from)
            .collect::<Vec<Tile>>(),
    ))
}
