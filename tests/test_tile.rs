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

use open_comm::{auth, app, tile, JWTConfig};

mod common;

#[tokio::test]
async fn tile_flow() {
    let api = app(
        common::db_pool().await,
        Some(JWTConfig::Secret(common::secret())),
    )
    .await
    .expect("app initialized");

    // Register a new user.
    let token = {
        let res = warp::test::request()
            .method("POST")
            .path("/api/register")
            .header("Content-Type", "application/json")
            .json(&auth::Register {
                username: "tile_flow".to_string(),
                password: "bar".to_string(),
            })
            .reply(&api)
            .await;
        assert_eq!(res.status(), 201, "registration created new resource");
        let body = String::from_utf8_lossy(res.body());
        let maybe_token = serde_json::from_str::<auth::RegisterResp>(body.as_ref());
        assert!(maybe_token.is_ok(), "register responds with valid data");
        maybe_token.unwrap().token
    };

    let tiles = vec![
        tile::Tile {
            phrase: "Dad".to_string(),
            images: vec!["/img/user/tile_flow/somehash.png".to_string()],
            categories: vec!["Family".to_string()],
        },
        tile::Tile {
            phrase: "pizza".to_string(),
            images: vec!["/img/user/tile_flow/someotherhash.png".to_string()],
            categories: vec!["Food".to_string()],
        }
    ];

    // Test create tiles.
    for tile in tiles.clone() {
        let res = warp::test::request()
            .method("POST")
            .path("/api/user/tile_flow/tiles")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&tile)
            .reply(&api)
            .await;
        assert_eq!(res.status(), 201, "new tile created new resource");
        let body = String::from_utf8_lossy(res.body());
        let maybe_tile = serde_json::from_str::<tile::Tile>(body.as_ref());
        assert!(maybe_tile.is_ok(), "new tile responds with valid data");
        assert_eq!(maybe_tile.unwrap(), tile, "new tile responds with correct tile");
    }

    {
        // Test query phrase.
        let res = warp::test::request()
            .method("GET")
            .path("/api/user/tile_flow/tiles?phrase=pizza")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .reply(&api)
            .await;
        assert_eq!(res.status(), 200, "tile query ok");
        let body = String::from_utf8_lossy(res.body());
        let maybe_tiles = serde_json::from_str::<Vec<tile::Tile>>(body.as_ref());
        assert!(maybe_tiles.is_ok(), "tile query responds with valid data");
        assert_eq!(
            maybe_tiles.unwrap(),
            vec![tiles[1].clone()],
            "tile query responds with correct tile"
        );
    }

    {
        // Test query phrase.
        let res = warp::test::request()
            .method("GET")
            .path("/api/user/tile_flow/tiles?category=Family")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .reply(&api)
            .await;
        assert_eq!(res.status(), 200, "tile query ok");
        let body = String::from_utf8_lossy(res.body());
        let maybe_tiles = serde_json::from_str::<Vec<tile::Tile>>(body.as_ref());
        assert!(maybe_tiles.is_ok(), "tile query responds with valid data");
        assert_eq!(
            maybe_tiles.unwrap(),
            vec![tiles[0].clone()],
            "tile query responds with correct tile"
        );
    }
}
