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

use std::fmt::Display;

use futures::stream::TryStreamExt;
use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};
use warp::{
    http::StatusCode,
    multipart::FormData,
    reply::{json, with_status, Json, WithStatus},
    Filter, Rejection, Reply,
};

use crate::{auth::BearerToken, db, guard, util, Error};

pub fn api(
    db_pool: db::Pool,
    jwt_key: DecodingKey<'static>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let create_user_tile = warp::post()
        .and(guard::user_resource(jwt_key.clone()))
        .and(warp::path("tiles"))
        .and(warp::path::end())
        .and(warp::multipart::form())
        .and(guard::with_db(db_pool.clone()))
        .and_then(create_user_tile);

    let list_tiles = warp::get()
        .and(guard::optional_user_resource(jwt_key.clone()))
        .and(warp::path("tiles"))
        .and(warp::path::end())
        .and(warp::query())
        .and(guard::with_db(db_pool.clone()))
        .and_then(list_tiles);

    let image = warp::get()
        .and(warp::path("image"))
        .and(guard::optional_authentic_token(jwt_key.clone()))
        .and(warp::path::param())
        .and(guard::with_db(db_pool.clone()))
        .and_then(read_image);

    let update_user_tile = warp::patch()
        .and(guard::user_resource(jwt_key.clone()))
        .and(warp::path("tiles"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::multipart::form())
        .and(guard::with_db(db_pool.clone()))
        .and_then(update_user_tile);

    let delete_user_tile = warp::delete()
        .and(guard::user_resource(jwt_key))
        .and(warp::path("tiles"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and(guard::with_db(db_pool))
        .and_then(delete_user_tile);

    create_user_tile
        .or(list_tiles)
        .or(image)
        .or(update_user_tile)
        .or(delete_user_tile)
}

#[inline(always)]
fn image_path<T: Display>(filename: T) -> String {
    format!("/api/image/{}", filename)
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub phrase: String,
    pub image: String,
    pub categories: Vec<String>,
}

#[derive(Default)]
struct TileForm {
    pub phrase: Option<String>,
    pub image: Option<(Vec<u8>, String)>,
    pub categories: Option<Vec<String>>,
}

async fn decode_tile_form(mut form_data: FormData) -> Result<TileForm, Error> {
    let mut form: TileForm = Default::default();
    while let Ok(Some(part)) = form_data.try_next().await {
        match (part.name(), part.content_type()) {
            ("phrase", _) => {
                if let Ok(bytes) = util::stream_bytes(part.stream()).await {
                    form.phrase =
                        Some(String::from_utf8(bytes).map_err(|_| Error::MalformedRequest)?);
                }
            }
            ("categories", _) => {
                if let Ok(bytes) = util::stream_bytes(part.stream()).await {
                    let raw_str = String::from_utf8(bytes).map_err(|_| Error::MalformedRequest)?;
                    form.categories = Some(
                        serde_json::from_str::<Vec<String>>(raw_str.as_ref())
                            .map_err(|_| Error::MalformedRequest)?,
                    );
                }
            }
            ("image", Some(format)) => {
                let maybe_ext = match format {
                    "image/svg+xml" => Some("svg".to_string()),
                    f if f.starts_with("image/") => {
                        Some(f.strip_prefix("image/").unwrap().to_string())
                    }
                    _ => None,
                };
                if let Ok(bytes) = util::stream_bytes(part.stream()).await {
                    if let Some(ext) = maybe_ext {
                        let hash = util::hash(bytes.as_slice());
                        form.image = Some((bytes, format!("{}.{}", hash, ext)));
                    }
                }
            }
            _ => (),
        }
    }
    Ok(form)
}

pub async fn create_user_tile(
    username: String,
    form: FormData,
    pool: db::Pool,
) -> Result<WithStatus<Json>, Rejection> {
    let tile = decode_tile_form(form).await?;
    match (tile.phrase, tile.image, tile.categories) {
        (Some(phrase), Some((image, filename)), Some(categories)) => {
            let conn = db::get_db_conn(&pool).await?;
            let uid: i32 = {
                let row = conn
                    .query_one("SELECT id FROM users WHERE username = $1", &[&username])
                    .await
                    .map_err(|_| Error::NotFound)?;
                row.get::<_, Option<i32>>("id").ok_or(Error::NotFound)?
            };
            conn.query(
                r#"
                INSERT INTO tiles (user_id, phrase, image, image_filename, categories)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                &[&uid, &phrase, &image, &filename, &categories],
            )
            .await
            .map_err(Error::DBError)?;

            let tile = Tile {
                phrase: phrase.to_string(),
                image: image_path(filename),
                categories,
            };

            Ok(with_status(json(&tile), StatusCode::CREATED))
        }
        _ => Err(Rejection::from(Error::MalformedRequest)),
    }
}

#[derive(Serialize, Deserialize)]
pub struct TileQuery {
    phrase: Option<String>,
    category: Option<String>,
}

pub async fn list_tiles(
    maybe_user: Option<String>,
    query: TileQuery,
    pool: db::Pool,
) -> Result<Json, Rejection> {
    let conn = db::get_db_conn(&pool).await?;
    let uid: Option<i32> = if let Some(user) = maybe_user {
        let row = conn
            .query_one("SELECT id FROM users WHERE username = $1", &[&user])
            .await
            .map_err(Error::DBError)?;
        Some(row.get("id"))
    } else {
        None
    };
    Ok(json(
        &conn
            .query(
                r#"
                SELECT phrase, image_filename, categories
                FROM tiles
                WHERE (user_id IS NULL OR user_id = $1)
                    AND ($2::TEXT IS NULL OR phrase LIKE $2)
                    AND ($3::TEXT IS NULL OR $3 = ANY(categories))
                "#,
                &[&uid, &query.phrase, &query.category],
            )
            .await
            .map_err(Error::DBError)?
            .iter()
            .map(|row| {
                let filename: String = row.get("image_filename");

                Tile {
                    phrase: row.get("phrase"),
                    image: image_path(filename),
                    categories: row.get("categories"),
                }
            })
            .collect::<Vec<Tile>>(),
    ))
}

pub async fn read_image(
    maybe_tok: Option<BearerToken>,
    filename: String,
    pool: db::Pool,
) -> Result<Vec<u8>, Rejection> {
    let conn = db::get_db_conn(&pool).await?;
    let row = conn
        .query_one(
            r#"
            SELECT image FROM tiles
            WHERE (user_id IS NULL OR user_id = $1)
                AND image_filename = $2
            "#,
            &[&maybe_tok.map(|tok| tok.username), &filename],
        )
        .await
        .map_err(Error::DBError)?;
    Ok(row.get("image"))
}

pub async fn update_user_tile(
    username: String,
    phrase: String,
    form: FormData,
    pool: db::Pool,
) -> Result<Json, Rejection> {
    let tile = decode_tile_form(form).await?;
    let conn = db::get_db_conn(&pool).await?;
    let uid: i32 = {
        let row = conn
            .query_one("SELECT id FROM users WHERE username = $1", &[&username])
            .await
            .map_err(|_| Error::NotFound)?;
        row.get("id")
    };
    let (image, image_filename) = match tile.image {
        Some((bytes, filename)) => (Some(bytes), Some(filename)),
        None => (None, None),
    };
    let row = conn
        .query_one(
            r#"
            UPDATE tiles
            SET phrase = COALESCE($1, phrase),
                image = COALESCE($2, image),
                image_filename = COALESCE($3, image_filename),
                categories = COALESCE($4, categories)
            WHERE user_id = $5 AND phrase = $6
            RETURNING phrase, image_filename, categories
            "#,
            &[
                &tile.phrase,
                &image,
                &image_filename,
                &tile.categories,
                &uid,
                &phrase,
            ],
        )
        .await
        .map_err(Error::DBError)?;

    let tile = Tile {
        phrase: row.get("phrase"),
        image: image_path::<String>(row.get("image_filename")),
        categories: row.get("categories"),
    };

    Ok(json(&tile))
}

pub async fn delete_user_tile(
    username: String,
    phrase: String,
    pool: db::Pool,
) -> Result<StatusCode, Rejection> {
    let conn = db::get_db_conn(&pool).await?;
    let uid: i32 = {
        let row = conn
            .query_one("SELECT id FROM users WHERE username = $1", &[&username])
            .await
            .map_err(|_| Error::NotFound)?;
        row.get("id")
    };
    conn.query(
        r#"
        DELETE FROM tiles
        WHERE user_id = $1 AND phrase = $2
        "#,
        &[&uid, &phrase],
    )
    .await
    .map_err(Error::DBError)?;

    Ok(StatusCode::OK)
}
