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

use std::env;

use open_comm::{app, db, JWTConfig};

const DEFAULT_DATABASE_URL: &'static str = "postgres://postgres@0.0.0.0:5432";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    let db_pool = db::create_pool(db_url.as_str())?;

    let jwt = match env::var("JWT_SECRET") {
        Ok(secret) => Some(JWTConfig::Secret(secret)),
        _ => match (env::var("JWT_PRIVATE_KEY"), env::var("JWT_PUBLIC_KEY")) {
            (Ok(private), Ok(public)) => Some(JWTConfig::RSAFiles { private, public }),
            _ => None,
        },
    };

    let app_routes = app(db_pool, jwt).await.expect("app initialized properly");

    warp::serve(app_routes).run(([0, 0, 0, 0], 8080)).await;
    Ok(())
}
