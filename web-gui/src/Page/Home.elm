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


module Page.Home exposing (..)

import Api exposing (User)
import Html exposing (Html)
import Html.Events as Events
import Session exposing (Session)


type alias Model =
    { session : Session }


toSession : Model -> Session
toSession { session } =
    session


init : Session -> ( Model, Cmd Msg )
init session =
    ( { session = session }, Cmd.none )


type Msg
    = Logout
    | GotSession Session


view : Model -> { title : String, content : Html Msg }
view model =
    { title = "Home"
    , content =
        case model.session of
            Session.LoggedIn _ user ->
                Html.div []
                    [ Html.h1 [] [ Html.text (Api.username user) ]
                    , Html.button [ Events.onClick Logout ] [ Html.text "Sign out" ]
                    ]

            Session.Guest _ ->
                Html.div []
                    [ Html.h1 [] [ Html.text "Guest" ] ]
    }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Logout ->
            ( model, Api.logout )

        GotSession s ->
            ( { model | session = s }, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions { session } =
    Session.onChange GotSession (Session.navKey session)
