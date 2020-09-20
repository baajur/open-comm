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

use open_comm::{app, auth, tile, JWTConfig};

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

    // Test create tiles.
    {
        let res = warp::test::request()
            .method("POST")
            .path("/api/user/tile_flow/tiles")
            .header(
                "Content-Type",
                "multipart/form-data; boundary=------------------------0af30d233b54bac0",
            )
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(include_bytes!("tile_create.bin"))
            .reply(&api)
            .await;
        assert_eq!(res.status(), 201, "new tile created new resource");
        let body = String::from_utf8_lossy(res.body());
        let maybe_tile = serde_json::from_str::<tile::Tile>(body.as_ref());
        assert!(maybe_tile.is_ok(), "new tile responds with valid data");
        let tile = maybe_tile.unwrap();
        assert_eq!(
            tile.phrase, "pizza",
            "new tile responds with correct phrase"
        );
        assert_eq!(
            tile.categories,
            vec!["food", "favorite"],
            "new tile responds with correct categories"
        );
    }

    // Test create tiles.
    {
        let res = warp::test::request()
            .method("POST")
            .path("/api/user/tile_flow/tiles")
            .header(
                "Content-Type",
                "multipart/form-data; boundary=------------------------0b56506eb827d2ac",
            )
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(include_bytes!("tile_create2.bin"))
            .reply(&api)
            .await;
        assert_eq!(res.status(), 201, "new tile created new resource");
        let body = String::from_utf8_lossy(res.body());
        let maybe_tile = serde_json::from_str::<tile::Tile>(body.as_ref());
        assert!(maybe_tile.is_ok(), "new tile responds with valid data");
        let tile = maybe_tile.unwrap();
        assert_eq!(
            tile.phrase, "spinach",
            "new tile responds with correct phrase"
        );
        assert_eq!(
            tile.categories,
            vec!["food", "not favorite"],
            "new tile responds with correct categories"
        );
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
        let tile = maybe_tiles.unwrap();
        assert_eq!(
            tile[0].phrase, "pizza",
            "tile query responds with correct phrase"
        );
        assert_eq!(
            tile[0].categories,
            vec!["food", "favorite"],
            "tile query responds with correct categories"
        );
    }

    {
        // Test query phrase.
        let res = warp::test::request()
            .method("GET")
            .path("/api/user/tile_flow/tiles?category=favorite")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .reply(&api)
            .await;
        assert_eq!(res.status(), 200, "tile query ok");
        let body = String::from_utf8_lossy(res.body());
        let maybe_tiles = serde_json::from_str::<Vec<tile::Tile>>(body.as_ref());
        assert!(maybe_tiles.is_ok(), "tile query responds with valid data");
        let tile = maybe_tiles.unwrap();
        assert_eq!(
            tile[0].phrase, "pizza",
            "tile query responds with correct phrase"
        );
        assert_eq!(
            tile[0].categories,
            vec!["food", "favorite"],
            "tile query responds with correct categories"
        );
    }

    {
        // Test query phrase unauth.
        let res = warp::test::request()
            .method("GET")
            .path("/api/user/tile_flow/tiles?category=favorite")
            .header("Accept", "application/json")
            .reply(&api)
            .await;
        assert_eq!(res.status(), 405, "tile query should be unauthorized");
    }

    {
        // Test update tile.
        let res = warp::test::request()
            .method("PATCH")
            .path("/api/user/tile_flow/tiles/pizza")
            .header(
                "Content-Type",
                "multipart/form-data; boundary=------------------------85d9e2b74277c596",
            )
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(include_bytes!("tile_update.bin"))
            .reply(&api)
            .await;
        assert_eq!(res.status(), 200, "tile update ok");
        let body = String::from_utf8_lossy(res.body());
        let maybe_tile = serde_json::from_str::<tile::Tile>(body.as_ref());
        assert!(maybe_tile.is_ok(), "tile query responds with valid data");
        let tile = maybe_tile.unwrap();
        assert_eq!(
            tile.phrase, "pie",
            "tile query responds with correct phrase"
        );
        assert_eq!(
            tile.categories,
            vec!["food", "favorite"],
            "tile query responds with correct categories"
        );
    }

    {
        // Test delete tile.
        let res = warp::test::request()
            .method("DELETE")
            .path("/api/user/tile_flow/tiles/pie")
            .header("Authorization", format!("Bearer {}", token))
            .reply(&api)
            .await;
        assert_eq!(res.status(), 200, "tile delete ok");
    }
}
