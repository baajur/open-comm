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

use std::{str::FromStr, time::Duration};

use mobc::Connection;
use mobc_postgres::{
    tokio_postgres::{Config, NoTls},
    PgConnectionManager,
};

use crate::Error;

pub type Conn = Connection<PgConnectionManager<NoTls>>;
pub type Pool = mobc::Pool<PgConnectionManager<NoTls>>;

const DB_POOL_MAX_OPEN: u64 = 32;
const DB_POOL_MAX_IDLE: u64 = 8;
const DB_POOL_TIMEOUT_SECONDS: u64 = 15;

pub fn create_pool<'a>(db_url: &'a str) -> Result<Pool, Error> {
    let config = Config::from_str(db_url)?;

    let manager = PgConnectionManager::new(config, NoTls);
    Ok(mobc::Pool::builder()
        .max_open(DB_POOL_MAX_OPEN)
        .max_idle(DB_POOL_MAX_IDLE)
        .get_timeout(Some(Duration::from_secs(DB_POOL_TIMEOUT_SECONDS)))
        .build(manager))
}

pub async fn get_db_conn(db_pool: &Pool) -> Result<Conn, Error> {
    Ok(db_pool.get().await?)
}

pub async fn init_db(db_pool: &Pool) -> Result<(), Error> {
    let init_sql = include_str!("init.sql");
    let conn = get_db_conn(db_pool).await?;
    conn.batch_execute(init_sql).await.map_err(Error::DBError)?;
    Ok(())
}

pub async fn uninit_db(db_pool: &Pool) -> Result<(), Error> {
    let uninit_sql = include_str!("uninit.sql");
    let conn = get_db_conn(db_pool).await?;
    conn.batch_execute(uninit_sql)
        .await
        .map_err(Error::DBError)?;
    Ok(())
}
