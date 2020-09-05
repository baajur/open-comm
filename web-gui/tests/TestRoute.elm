module TestRoute exposing (..)

import Expect exposing (Expectation)
import Fuzz exposing (Fuzzer, int, list, string)
import Maybe
import Route exposing (Route)
import Test exposing (..)
import Url
import Url.Builder


suite : Test
suite =
    describe "The Route module"
        [ describe "Route.fromUrl"
            [ test "parses login" <|
                \_ ->
                    Expect.equal
                        (Maybe.andThen Route.fromUrl (Url.fromString "https://example.com/#/login"))
                        (Just Route.Login)
            , test "parses register" <|
                \_ ->
                    Expect.equal
                        (Maybe.andThen Route.fromUrl (Url.fromString "https://example.com/#/register"))
                        (Just Route.Register)
            , test "parses home" <|
                \_ ->
                    Expect.equal
                        (Maybe.andThen Route.fromUrl (Url.fromString "https://example.com/#/"))
                        (Just Route.Home)
            ]
        ]
