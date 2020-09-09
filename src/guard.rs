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

use std::convert::Infallible;

use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection};

use crate::{auth::BearerToken, db, Error};

pub fn with_db(
    db_pool: db::Pool,
) -> impl Filter<Extract = (db::Pool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

pub fn with_jwt_priv_key(
    priv_key: EncodingKey,
) -> impl Filter<Extract = (EncodingKey,), Error = Infallible> + Clone {
    warp::any().map(move || priv_key.clone())
}

pub fn authentic_user_header(
    pub_key: DecodingKey<'static>,
) -> impl Filter<Extract = (BearerToken,), Error = Rejection> + Clone {
    warp::header("Authorization").and_then(move |h: String| {
        let k = pub_key.clone();
        async move {
            BearerToken::verify_token(&k, &h.as_str()["Bearer ".len()..])
                .map_err(|e| Rejection::from(e))
        }
    })
}

#[derive(Serialize, Deserialize)]
pub struct TokenQuery {
    access_token: String,
}

pub fn authentic_token_query(
    pub_key: DecodingKey<'static>,
) -> impl Filter<Extract = (BearerToken,), Error = Rejection> + Clone {
    warp::query().and_then(move |q: TokenQuery| {
        let k = pub_key.clone();
        async move {
            BearerToken::verify_token(&k, &q.access_token.as_str()).map_err(|e| Rejection::from(e))
        }
    })
}

async fn user_and_token_match(user: String, tok: BearerToken) -> Result<String, Rejection> {
    if user == tok.username {
        Ok(tok.username)
    } else {
        Err(Rejection::from(Error::Unauthorized))
    }
}

pub fn user_resource(
    pub_key: DecodingKey<'static>,
) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::path::param()
        .and(authentic_user_header(pub_key.clone()))
        .and_then(user_and_token_match)
}

pub fn user_resource_query(
    pub_key: DecodingKey<'static>,
) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::path::param()
        .and(authentic_user_header(pub_key.clone()))
        .and_then(user_and_token_match)
}
