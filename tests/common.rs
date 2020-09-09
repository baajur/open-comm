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

use std::{env, sync::Mutex};

use open_comm::{auth::random_string, db};

pub fn db_url<'a>() -> String {
    lazy_static::lazy_static! {
        static ref DATABASE_URL: String = {
            env::var("DATABASE_URL").unwrap()
        };
    }
    DATABASE_URL.clone()
}

pub async fn db_pool() -> db::Pool {
    lazy_static::lazy_static! {
        static ref DATABASE_POOL: db::Pool = {
            db::create_pool(db_url().as_ref()).unwrap()
        };
        static ref DATABASE_INIT: Mutex<bool> = Mutex::new(false);
    }
    let pool = DATABASE_POOL.clone();
    unsafe {
        // Attempt to acquire the lock.
        if let Ok(mut guard) = DATABASE_INIT.lock() {
            // If the initialization hasn't already completed on another thread.
            if !*guard {
                db::uninit_db(&pool).await.unwrap();
                db::init_db(&pool).await.unwrap();
                *guard = true;
            }
        }
    }
    pool
}

pub fn secret() -> String {
    lazy_static::lazy_static! {
        static ref SECRET: String = random_string(32);
    }
    SECRET.clone()
}
