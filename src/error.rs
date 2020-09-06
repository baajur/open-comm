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

use std::io;

use diesel::result::{DatabaseErrorKind as DBErrorKind, Error as DBError};
use rocket::{http::Status, request::Request, response};

#[derive(Debug)]
pub enum Error {
    NotFound,
    Conflict,
    InternalError,
}

impl<'r> response::Responder<'r> for Error {
    fn respond_to(self, _request: &Request) -> response::Result<'r> {
        Err(match self {
            Error::NotFound => Status::NotFound,
            Error::Conflict => Status::Conflict,
            Error::InternalError => Status::InternalServerError,
        })
    }
}

impl From<io::Error> for Error {
    fn from(item: io::Error) -> Self {
        match item.kind() {
            io::ErrorKind::NotFound => Error::NotFound,
            _ => Error::InternalError,
        }
    }
}

impl From<DBError> for Error {
    fn from(item: DBError) -> Self {
        match item {
            DBError::NotFound => Error::NotFound,
            DBError::DatabaseError(DBErrorKind::UniqueViolation, _) => Error::Conflict,
            _ => Error::InternalError,
        }
    }
}
