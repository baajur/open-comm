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


module Page exposing (Page(..), view, viewErrors)

import Api exposing (User)
import Browser exposing (Document)
import Html exposing (Html)
import Html.Attributes as Attr
import Html.Events as Events
import Route exposing (Route)


type Page
    = Other
    | Login
    | Register
    | Home


view : Maybe User -> Page -> { title : String, content : Html msg } -> Document msg
view maybeUser page { title, content } =
    { title = title ++ " - Open Communication"
    , body = [ viewHeader page maybeUser, content, viewFooter ]
    }


viewHeader : Page -> Maybe User -> Html msg
viewHeader page maybeUser =
    Html.nav []
        [ Html.div []
            [ Html.a [ Route.href Route.Home ]
                [ Html.text "Open Communication" ]
            , Html.ul [] <|
                navbarLink page Route.Home [ Html.text "Home" ]
                    :: viewMenu page maybeUser
            ]
        ]


viewMenu : Page -> Maybe User -> List (Html msg)
viewMenu page maybeUser =
    let
        linkTo =
            navbarLink page
    in
    case maybeUser of
        Just user ->
            let
                username =
                    Api.username user
            in
            [ linkTo Route.Home [ Html.text username ]
            ]

        Nothing ->
            [ linkTo Route.Login [ Html.text "Sign in" ]
            , linkTo Route.Register [ Html.text "Sign up" ]
            ]


navbarLink : Page -> Route -> List (Html msg) -> Html msg
navbarLink page route linkContent =
    Html.li [ Attr.classList [ ( "active", isActive page route ) ] ]
        [ Html.a [ Route.href route ] linkContent ]


viewFooter : Html msg
viewFooter =
    Html.footer []
        [ Html.div []
            [ Html.span []
                [ Html.text
                    "Code & design licensed under Affero General Public License."
                ]
            ]
        ]


isActive : Page -> Route -> Bool
isActive page route =
    case ( page, route ) of
        ( Home, Route.Home ) ->
            True

        ( Login, Route.Login ) ->
            True

        ( Register, Route.Register ) ->
            True

        _ ->
            False


{-| Render dismissable errors. We use this all over the place!
-}
viewErrors : msg -> List String -> Html msg
viewErrors dismissErrors errors =
    if List.isEmpty errors then
        Html.text ""

    else
        Html.div
            [ Attr.class "error-messages"
            , Attr.style "position" "fixed"
            , Attr.style "top" "0"
            , Attr.style "background" "rgb(250, 250, 250)"
            , Attr.style "padding" "20px"
            , Attr.style "border" "1px solid"
            ]
        <|
            List.map (\error -> Html.p [] [ Html.text error ]) errors
                ++ [ Html.button
                        [ Events.onClick dismissErrors ]
                        [ Html.text "Ok" ]
                   ]
