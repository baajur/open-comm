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

use std::{
    borrow::Cow,
    fs,
    io::{BufRead, BufReader, Write},
};

use rocket::{
    http::uri::Uri,
    response::{status::Created, NamedFile},
    Data, Route, State,
};
use rocket_contrib::json::Json;

use diesel::prelude::*;

use crypto::{digest::Digest, sha3::Sha3};
use tempfile::NamedTempFile;

use crate::{models::*, Error};

const PAGE_LIMIT: i64 = 100;
const IMG_LIMIT: i64 = 1000000;
pub fn routes() -> Vec<Route> {
    routes![
        create_user_img,
        list_user_images,
        read_user_image,
        // CRUD card entries.
        create_user_card,
        list_user_cards,
        update_user_card,
        delete_user_card,
    ]
}

#[post(
    "/api/user/<username>/cards/private/image",
    format = "image/png",
    data = "<img>"
)]
pub fn create_user_img<'a>(
    _user_tok: UserToken,
    data_dir: State<DataDir>,
    username: String,
    img: Data,
) -> Result<Created<()>, Error> {
    let DataDir(data_dir) = data_dir.inner();
    let file = NamedTempFile::new()?;
    let mut hasher = Sha3::sha3_224();
    let mut reader = BufReader::new(img.open());
    loop {
        let consumed = reader.fill_buf().and_then(|bytes| {
            if bytes.len() > 0 {
                hasher.input(bytes);
                file.as_file().write(bytes)
            } else {
                Ok(0)
            }
        })?;
        if consumed == 0 {
            break;
        }
        reader.consume(consumed);
    }
    let parent = data_dir
        .join("images")
        .join("private")
        .join(username.as_str());
    fs::create_dir_all(parent.as_path())?;
    let path = parent.join(hasher.result_str()).with_extension("png");
    if path.exists() {
        Err(Error::Conflict)
    } else {
        fs::copy(file.path(), path.as_path())?;
        Ok(Created(
            uri!(
                read_user_image: username = username,
                name = path.file_name().unwrap().to_str().unwrap()
            )
            .to_string(),
            None,
        ))
    }
}

#[get("/api/user/<username>/cards/private/image")]
fn list_user_images<'a>(
    _user_tok: UserToken,
    data_dir: State<DataDir>,
    username: String,
) -> Result<Json<Vec<String>>, Error> {
    let DataDir(data_dir) = data_dir.inner();
    let path = data_dir
        .join("images")
        .join("private")
        .join(username.as_str());
    match fs::read_dir(path) {
        Ok(iter) => Ok(Json(
            iter.filter_map(Result::ok)
                .filter_map(|entry| {
                    entry.file_name().to_str().map(|n| {
                        uri!(read_user_image: username = username.as_str(), name = n).to_string()
                    })
                })
                .collect(),
        )),
        _ => Err(Error::InternalError),
    }
}

#[get("/api/user/<username>/cards/private/image/<name>")]
fn read_user_image(
    _user_tok: UserToken,
    data_dir: State<DataDir>,
    username: String,
    name: String,
) -> Result<NamedFile, Error> {
    let DataDir(data_dir) = data_dir.inner();
    let path = data_dir
        .join("images")
        .join("private")
        .join(username.as_str())
        .join(name);
    Ok(NamedFile::open(path.as_path())?)
}

#[post("/api/user/<username>/cards/private", data = "<card>")]
pub fn create_user_card<'a>(
    db: DbConn,
    _user_tok: UserToken,
    username: String,
    card: Json<CardEntry>,
) -> Result<Created<Json<CardEntry>>, Error> {
    use crate::schema::{user_cards, users};

    let DbConn(conn) = db;

    let user_id: i32 = users::table
        .filter(users::username.eq(username.as_str()))
        .select(users::id)
        .first(&conn)?;

    let new_card = NewUserCard {
        user_id,
        phrase: card.phrase.clone(),
        images: card.images.clone(),
        categories: card.categories.clone(),
    };
    diesel::insert_into(user_cards::table)
        .values(&new_card)
        .execute(&conn)?;
    let location = uri!(list_user_cards: username, card.phrase.to_string(), _, _, _);
    Ok(Created(location.to_string(), Some(card)))
}

#[get("/api/user/<username>/cards/private?<phrase>&<categories>&<images>&<offset>")]
pub fn list_user_cards<'a>(
    db: DbConn,
    _user_tok: UserToken,
    username: String,
    phrase: Option<String>,
    categories: Option<CSVec<String>>,
    images: Option<CSVec<String>>,
    offset: Option<i64>,
) -> Result<Json<CardPageResp<'a>>, Error> {
    use crate::schema::{user_cards, users};
    let DbConn(conn) = db;

    let user: User = users::table
        .filter(users::username.eq(username.as_str()))
        .first(&conn)?;

    let mut cards_query = UserCard::belonging_to(&user).into_boxed();
    let mut next_page = uri!(
        list_user_cards: username = username,
        offset = _,
        phrase = _,
        categories = _,
        images = _
    )
    .to_string();
    let mut query_parts = vec![];
    if let Some(p) = phrase {
        cards_query = cards_query.filter(user_cards::phrase.eq(p.clone()));
        query_parts.push(format!("phrase={}", Uri::percent_encode(p.as_str())));
    }
    if let Some(CSVec(cats)) = categories {
        cards_query = cards_query.filter(user_cards::categories.contains(cats.clone()));
        query_parts.push(format!(
            "categories={}",
            Uri::percent_encode(cats.join(",").as_str())
        ));
    }
    if let Some(CSVec(imgs)) = images {
        cards_query = cards_query.filter(user_cards::images.contains(imgs.clone()));
        query_parts.push(format!(
            "images={}",
            Uri::percent_encode(imgs.join(",").as_str())
        ));
    }
    if let Some(o) = offset {
        cards_query = cards_query.offset(o);
    }
    let cards: Vec<(Cow<str>, Vec<Cow<str>>, Vec<Cow<str>>)> = cards_query
        .order_by(user_cards::phrase)
        .select((
            user_cards::phrase,
            user_cards::images,
            user_cards::categories,
        ))
        .limit(PAGE_LIMIT)
        .get_results(&conn)?;

    query_parts.push(format!("offset={}", cards.len()));

    next_page.push('?');
    next_page.push_str(query_parts.join("&").as_str());

    Ok(Json(CardPageResp {
        cards: cards
            .iter()
            .map(|(p, i, c)| CardEntry {
                phrase: p.clone(),
                images: i.clone(),
                categories: c.clone(),
            })
            .collect(),
        next_page: if cards.len() as i64 == PAGE_LIMIT {
            Some(next_page)
        } else {
            None
        },
    }))
}

#[patch("/api/user/<username>/cards/private?<phrase>", data = "<update>")]
pub fn update_user_card<'a>(
    db: DbConn,
    _user_tok: UserToken,
    username: String,
    phrase: String,
    update: Json<UpdateCardEntry>,
) -> Result<Json<CardEntry<'a>>, Error> {
    use crate::schema::{user_cards, users};
    let DbConn(conn) = db;

    let user_id: i32 = users::table
        .filter(users::username.eq(username.as_str()))
        .select(users::id)
        .first(&conn)?;

    let (phrase, images, categories): (Cow<str>, Vec<Cow<str>>, Vec<Cow<str>>) =
        diesel::update(user_cards::table)
            .filter(
                user_cards::user_id
                    .eq(user_id)
                    .and(user_cards::phrase.eq(phrase)),
            )
            .set(&update.into_inner())
            .returning((
                user_cards::phrase,
                user_cards::images,
                user_cards::categories,
            ))
            .get_result(&conn)?;

    let update_res = CardEntry {
        phrase,
        images,
        categories,
    };

    Ok(Json(update_res))
}

#[delete("/api/user/<username>/cards/private?<phrase>")]
pub fn delete_user_card<'a>(
    db: DbConn,
    _user_tok: UserToken,
    username: String,
    phrase: String,
) -> Result<(), Error> {
    use crate::schema::{user_cards, users};
    let DbConn(conn) = db;

    let user_id: i32 = users::table
        .filter(users::username.eq(username.as_str()))
        .select(users::id)
        .first(&conn)?;

    diesel::delete(user_cards::table)
        .filter(
            user_cards::user_id
                .eq(user_id)
                .and(user_cards::phrase.eq(phrase)),
        )
        .execute(&conn)?;

    Ok(())
}
