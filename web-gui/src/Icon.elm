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


module Icon exposing (home, login, register)

import Html exposing (Html)
import Svg as Svg
import Svg.Attributes as Attr


home : Html msg
home =
    Svg.svg
        [ Attr.viewBox "0 0 16 16", Attr.fill "currentColor", Attr.xmlSpace "http://www.w3.org/2000/svg" ]
        [ Svg.path
            [ Attr.fillRule "evenodd"
            , Attr.d "M8 3.293l6 6V13.5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 13.5V9.293l6-6zm5-.793V6l-2-2V2.5a.5.5 0 0 1 .5-.5h1a.5.5 0 0 1 .5.5z"
            ]
            []
        , Svg.path
            [ Attr.fillRule "evenodd"
            , Attr.d "M7.293 1.5a1 1 0 0 1 1.414 0l6.647 6.646a.5.5 0 0 1-.708.708L8 2.207 1.354 8.854a.5.5 0 1 1-.708-.708L7.293 1.5z"
            ]
            []
        ]


login : Html msg
login =
    Svg.svg
        [ Attr.viewBox "0 0 16 16", Attr.fill "currentColor", Attr.xmlSpace "http://www.w3.org/2000/svg" ]
        [ Svg.path
            [ Attr.fillRule "evenodd"
            , Attr.d "M6 3.5a.5.5 0 0 1 .5-.5h8a.5.5 0 0 1 .5.5v9a.5.5 0 0 1-.5.5h-8a.5.5 0 0 1-.5-.5v-2a.5.5 0 0 0-1 0v2A1.5 1.5 0 0 0 6.5 14h8a1.5 1.5 0 0 0 1.5-1.5v-9A1.5 1.5 0 0 0 14.5 2h-8A1.5 1.5 0 0 0 5 3.5v2a.5.5 0 0 0 1 0v-2z"
            ]
            []
        , Svg.path
            [ Attr.fillRule "evenodd"
            , Attr.d "M11.854 8.354a.5.5 0 0 0 0-.708l-3-3a.5.5 0 1 0-.708.708L10.293 7.5H1.5a.5.5 0 0 0 0 1h8.793l-2.147 2.146a.5.5 0 0 0 .708.708l3-3z"
            ]
            []
        ]


register : Html msg
register =
    Svg.svg
        [ Attr.viewBox "0 0 16 16", Attr.fill "currentColor", Attr.xmlSpace "http://www.w3.org/2000/svg" ]
        [ Svg.path
            [ Attr.fillRule "evenodd"
            , Attr.d "M1 14s-1 0-1-1 1-4 6-4 6 3 6 4-1 1-1 1H1zm5-6a3 3 0 1 0 0-6 3 3 0 0 0 0 6zm7.5-3a.5.5 0 0 1 .5.5V7h1.5a.5.5 0 0 1 0 1H14v1.5a.5.5 0 0 1-1 0V8h-1.5a.5.5 0 0 1 0-1H13V5.5a.5.5 0 0 1 .5-.5z"
            ]
            []
        ]
