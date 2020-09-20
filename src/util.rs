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

use std::iter::Extend;

use bytes::buf::Buf;
use crypto::{digest::Digest, sha3::Sha3};
use futures::stream::TryStreamExt;
use tokio::stream::Stream;
use warp::Error;

pub fn hash(bytes: &[u8]) -> String {
    let mut hasher = Sha3::sha3_224();
    hasher.input(bytes);
    hasher.result_str()
}

pub async fn stream_bytes<T, U>(stream: T) -> Result<Vec<u8>, Error>
where
    T: Stream<Item = Result<U, Error>>,
    U: Buf,
{
    Ok(stream
        .try_fold(Vec::new(), |mut acc, x| async move {
            acc.extend(x.bytes());
            Ok(acc)
        })
        .await?)
}
