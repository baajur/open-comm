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

use std::{convert::Infallible, env, fs};

use jsonwebtoken::{DecodingKey, EncodingKey};
use warp::{Filter, Reply};

pub mod guard;

pub mod auth;
pub mod tile;

pub mod db;

mod error;
pub use error::Error;

const DEFAULT_DATABASE_URL: &'static str = "postgres://postgres@0.0.0.0:5432";

pub async fn app() -> Result<impl Filter<Extract = impl Reply, Error = Infallible> + Clone, Error> {
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    let db_pool = db::create_pool(db_url.as_str())?;
    db::init_db(&db_pool).await?;

    let (jwt_priv, jwt_pub): (EncodingKey, DecodingKey<'static>) = match env::var("JWT_SECRET") {
        Ok(secret) => (
            EncodingKey::from_secret(secret.as_bytes()),
            DecodingKey::from_secret(secret.as_bytes()).into_static(),
        ),
        _ => match (env::var("JWT_PRIVATE_KEY"), env::var("JWT_PUBLIC_KEY")) {
            (Ok(private), Ok(public)) => (
                EncodingKey::from_rsa_pem(fs::read(private)?.as_ref())?,
                DecodingKey::from_rsa_pem(fs::read(public)?.as_ref())?.into_static(),
            ),
            _ => {
                let secret = auth::random_string(32);
                (
                    EncodingKey::from_secret(secret.as_bytes()),
                    DecodingKey::from_secret(secret.as_bytes()).into_static(),
                )
            }
        },
    };

    let auth_api = auth::api(db_pool.clone(), jwt_priv);
    let tile_api = tile::api(db_pool, jwt_pub);

    tracing_subscriber::fmt::init();
    let route = warp::path("api")
        .and(auth_api.or(tile_api))
        .with(warp::filters::trace::request())
        .recover(error::handle_rejects);
    Ok(route)
}
