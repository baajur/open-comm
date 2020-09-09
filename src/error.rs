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

use jsonwebtoken::errors::ErrorKind as JWTErrorKind;
use mobc_postgres::tokio_postgres::error::SqlState;
use warp::{http::StatusCode, reject, Rejection, Reply};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DBPoolError(#[from] mobc::Error<mobc_postgres::tokio_postgres::Error>),
    #[error(transparent)]
    DBError(#[from] mobc_postgres::tokio_postgres::Error),
    #[error("unauthorized request")]
    Unauthorized,
    #[error(transparent)]
    JWTError(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

impl reject::Reject for Error {}

impl From<Error> for Rejection {
    fn from(item: Error) -> Rejection {
        reject::custom(item)
    }
}

pub async fn handle_rejects(err: Rejection) -> Result<impl Reply, Infallible> {
    let code = if err.is_not_found() {
        StatusCode::NOT_FOUND
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        StatusCode::BAD_REQUEST
    } else if let Some(e) = err.find::<Error>() {
        match e {
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::JWTError(e) => match e.kind() {
                JWTErrorKind::InvalidIssuer
                | JWTErrorKind::InvalidSignature
                | JWTErrorKind::ExpiredSignature => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Error::DBError(e) => {
                if let Some(code) = e.code() {
                    if *code == SqlState::UNIQUE_VIOLATION {
                        StatusCode::CONFLICT
                    } else {
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        StatusCode::METHOD_NOT_ALLOWED
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    Ok(warp::reply::with_status("", code))
}
