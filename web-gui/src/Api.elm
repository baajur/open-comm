{-
   Copyright (C) 2020 Oakes, Gregory <gregoryoakes@fastmail.com>
   Author: Oakes, Gregory <gregory.oakes@fastmail.com>

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU Affero General Public License as
   published by the Free Software Foundation, either version 3 of the
   License, or (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU Affero General Public License for more details.

   You should have received a copy of the GNU Affero General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>.
-}


port module Api exposing
    ( LoginForm
    , RegisterForm
    , User
    , application
    , login
    , logout
    , onUserChange
    , register
    , storeUser
    , username
    )

import Api.Endpoint as Endpoint exposing (Endpoint)
import Browser
import Browser.Navigation as Nav
import Http exposing (Body)
import Json.Decode as Decode exposing (Decoder, Value)
import Json.Decode.Pipeline exposing (required)
import Json.Encode as Encode exposing (object, string)
import Url exposing (Url)


type User
    = User String Token


type Token
    = Token String


username : User -> String
username (User u _) =
    u


authHeader : User -> Http.Header
authHeader (User _ (Token tok)) =
    Http.header "Authorization" ("Bearer " ++ tok)



-- APPLICATION


application :
    { init : Maybe User -> Url -> Nav.Key -> ( model, Cmd msg )
    , onUrlChange : Url -> msg
    , onUrlRequest : Browser.UrlRequest -> msg
    , subscriptions : model -> Sub msg
    , update : msg -> model -> ( model, Cmd msg )
    , view : model -> Browser.Document msg
    }
    -> Program Value model msg
application config =
    let
        init flags url navKey =
            let
                maybeUser =
                    Decode.decodeValue Decode.string flags
                        |> Result.andThen (Decode.decodeString userDecoder)
                        |> Result.toMaybe
            in
            config.init maybeUser url navKey
    in
    Browser.application
        { init = init
        , onUrlChange = config.onUrlChange
        , onUrlRequest = config.onUrlRequest
        , subscriptions = config.subscriptions
        , update = config.update
        , view = config.view
        }



-- API


{-| Clear credentials from local storage cache.
-}
logout : Cmd msg
logout =
    storeCache Nothing


type alias LoginForm =
    { username : String
    , password : String
    }


login : LoginForm -> CmdMsg User msg -> Cmd msg
login form msgFromHttp =
    let
        decoder =
            Decode.succeed (\tok -> User form.username (Token tok))
                |> required "token" Decode.string

        body =
            Encode.object
                [ ( "username", Encode.string form.username )
                , ( "password", Encode.string form.password )
                ]
                |> Http.jsonBody
    in
    post Endpoint.login Nothing body decoder msgFromHttp


type alias RegisterForm =
    { username : String
    , password : String
    }


register : RegisterForm -> CmdMsg User msg -> Cmd msg
register form msgFromHttp =
    let
        decoder =
            Decode.succeed (\tok -> User form.username (Token tok))
                |> required "token" Decode.string

        body =
            Encode.object
                [ ( "username", Encode.string form.username )
                , ( "password", Encode.string form.password )
                ]
                |> Http.jsonBody
    in
    post Endpoint.register Nothing body decoder msgFromHttp



-- DECODERS


userDecoder : Decoder User
userDecoder =
    Decode.succeed User
        |> required "username" Decode.string
        |> required "token" tokenDecoder


tokenDecoder : Decoder Token
tokenDecoder =
    Decode.map Token Decode.string


decodeFromChange : Value -> Maybe User
decodeFromChange val =
    Decode.decodeValue userDecoder val
        |> Result.toMaybe



-- ENCODERS


encodeUser : User -> Encode.Value
encodeUser (User uname (Token tok)) =
    Encode.object
        [ ( "username", Encode.string uname )
        , ( "token", Encode.string tok )
        ]



-- STORAGE


port onStoreChange : (Value -> msg) -> Sub msg


port storeCache : Maybe Value -> Cmd msg


{-| Encode the credential into the local storage cache.
-}
storeUser : User -> Cmd msg
storeUser user =
    storeCache (Just (encodeUser user))


{-| Create a subscription for changes to the storage.
-}
onUserChange : (Maybe User -> msg) -> Sub msg
onUserChange toMsg =
    onStoreChange (\val -> toMsg (decodeFromChange val))



-- RAW API


type alias CmdMsg a msg =
    Result Http.Error a -> msg


post : Endpoint -> Maybe User -> Body -> Decoder a -> CmdMsg a msg -> Cmd msg
post url maybeUser body decoder msgFromHttp =
    Endpoint.request
        { method = "POST"
        , url = url
        , expect = Http.expectJson msgFromHttp decoder
        , headers =
            case maybeUser of
                Just tok ->
                    [ authHeader tok ]

                Nothing ->
                    []
        , body = body
        , timeout = Nothing
        , tracker = Nothing
        }



-- get : Endpoint -> Maybe User -> Decoder a -> CmdMsg a msg -> Cmd msg
-- get url maybeUser decoder msgFromHttp =
--     Endpoint.request
--         { method = "GET"
--         , url = url
--         , expect = Http.expectJson msgFromHttp decoder
--         , headers =
--             case maybeUser of
--                 Just cred ->
--                     [ authHeader cred ]
--
--                 Nothing ->
--                     []
--         , body = Http.emptyBody
--         , timeout = Nothing
--         , tracker = Nothing
--         }
-- put : Endpoint -> User -> Body -> Decoder a -> CmdMsg a msg -> Cmd msg
-- put url tok body decoder msgFromHttp =
--     Endpoint.request
--         { method = "PUT"
--         , url = url
--         , expect = Http.expectJson msgFromHttp decoder
--         , headers = [ authHeader tok ]
--         , body = body
--         , timeout = Nothing
--         , tracker = Nothing
--         }
-- delete : Endpoint -> User -> Body -> Decoder a -> CmdMsg a msg -> Cmd msg
-- delete url tok body decoder msgFromHttp =
--     Endpoint.request
--         { method = "DELETE"
--         , url = url
--         , expect = Http.expectJson msgFromHttp decoder
--         , headers = [ authHeader tok ]
--         , body = body
--         , timeout = Nothing
--         , tracker = Nothing
--         }
