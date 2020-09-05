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


module Route exposing
    ( Route(..)
    , fromUrl
    , href
    , pushUrl
    , replaceUrl
    , showRoute
    )

import Browser.Navigation as Nav
import Html exposing (Attribute)
import Html.Attributes as Attr
import Url exposing (Url)
import Url.Builder as Builder
import Url.Parser as Parser exposing (Parser, oneOf, s)


type Route
    = Login
    | Register
    | Home


href : Route -> Attribute msg
href targetRoute =
    Attr.href (showRoute targetRoute)


replaceUrl : Nav.Key -> Route -> Cmd msg
replaceUrl key route =
    Nav.replaceUrl key (showRoute route)


pushUrl : Nav.Key -> Route -> Cmd msg
pushUrl key route =
    Nav.pushUrl key (showRoute route)


parser : Parser (Route -> a) a
parser =
    oneOf
        [ Parser.map Login (s "login")
        , Parser.map Register (s "register")
        , Parser.map Home Parser.top
        ]


fromUrl : Url -> Maybe Route
fromUrl =
    Parser.parse parser


showRoute : Route -> String
showRoute route =
    "/" ++ Builder.relative (breakRoute route) []


breakRoute : Route -> List String
breakRoute route =
    case route of
        Home ->
            []

        Login ->
            [ "login" ]

        Register ->
            [ "register" ]
