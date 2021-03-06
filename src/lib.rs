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

use std::{convert::Infallible, fs};

use jsonwebtoken::{DecodingKey, EncodingKey};
use warp::{Filter, Reply};

pub mod guard;
pub mod util;

pub mod auth;
pub mod tile;
pub mod user;

pub mod db;

mod error;
pub use error::{handle_rejects, Error};

pub enum JWTConfig {
    Secret(String),
    RSAFiles { private: String, public: String },
}

pub async fn app(
    db_pool: db::Pool,
    maybe_jwt: Option<JWTConfig>,
) -> Result<impl Filter<Extract = impl Reply, Error = Infallible> + Clone, Error> {
    db::init_db(&db_pool).await?;

    let jwt = maybe_jwt.unwrap_or_else(|| JWTConfig::Secret(auth::random_string(32)));
    let (jwt_priv, jwt_pub): (EncodingKey, DecodingKey<'static>) = match jwt {
        JWTConfig::Secret(secret) => (
            EncodingKey::from_secret(secret.as_bytes()),
            DecodingKey::from_secret(secret.as_bytes()).into_static(),
        ),
        JWTConfig::RSAFiles { private, public } => (
            EncodingKey::from_rsa_pem(fs::read(private)?.as_ref())?,
            DecodingKey::from_rsa_pem(fs::read(public)?.as_ref())?.into_static(),
        ),
    };

    let auth_api = auth::api(db_pool.clone(), jwt_priv);
    let tile_api = tile::api(db_pool.clone(), jwt_pub.clone());
    let user_api = user::api(db_pool, jwt_pub);

    let api = warp::path("api")
        // Limit to 4MiB
        // .and(warp::body::content_length_limit(4194304))
        .and(auth_api.or(tile_api).or(user_api));

    let gui_lib = warp::path!("elm.js").map(|| {
        warp::reply::with_header(
            include_str!(env!("GUI_LIB")),
            "Content-Type",
            "text/javascript",
        )
    });
    let gui_index =
        warp::path::end().map(|| warp::reply::html(include_str!("../web-gui/index.html")));
    let gui_style = warp::path!("style.css").map(|| {
        warp::reply::with_header(
            include_str!("../web-gui/style.css"),
            "Content-Type",
            "text/css",
        )
    });

    let gui = gui_lib.or(gui_index).or(gui_style);

    tracing_subscriber::fmt::init();
    let routes = api
        .or(gui)
        .with(warp::filters::trace::request())
        .recover(error::handle_rejects);

    Ok(routes)
}
