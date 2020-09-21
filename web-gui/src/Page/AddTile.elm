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


module Page.AddTile exposing
    ( Model
    , Msg
    , clearForm
    , init
    , update
    , view
    )

import Api
import Category
import File exposing (File)
import File.Select
import Html exposing (Html)
import Html.Attributes as Attr
import Html.Events as Events
import Http
import Icon
import Page
import Set exposing (Set)
import Task


type alias Model =
    { phrase : String
    , categories : Set String
    , image : Maybe File
    , imgUrl : Maybe String
    , problems : List String
    }


init : Model
init =
    { phrase = ""
    , categories = Set.empty
    , image = Nothing
    , imgUrl = Nothing
    , problems = []
    }


view : Model -> Html Msg
view { phrase, categories, image, imgUrl, problems } =
    Html.div [ Attr.class "tile" ]
        [ Html.form
            [ Events.onSubmit SubmittedForm ]
            [ Html.input
                [ Attr.placeholder "Phrase"
                , Events.onInput EnteredPhrase
                , Attr.value phrase
                , Attr.class "tile-phrase"
                ]
                []
            , Html.br [] []
            , viewImgUrl image imgUrl
            , Html.br [] []
            , Html.label [] [ Html.text "Categories" ]
            , Html.select
                [ Events.onInput SelectedCategory
                , Attr.multiple True
                ]
                (List.map
                    (\c ->
                        Html.option
                            [ Attr.value c
                            , Attr.selected (Set.member c categories)
                            ]
                            [ Html.text c ]
                    )
                    Category.categories
                )
            , Html.input [ Attr.type_ "submit" ] [ Html.text "Create" ]
            ]
        , Page.viewErrors ClickedDismissErrors problems
        ]


viewImgUrl : Maybe File -> Maybe String -> Html Msg
viewImgUrl img imgUrl =
    case ( img, imgUrl ) of
        ( Just i, Just u ) ->
            Html.img
                [ Attr.class "tile-img"
                , Attr.alt (File.name i)
                , Attr.src u
                ]
                []

        _ ->
            Html.div [ Events.onClick SelectImage ] [ Icon.open ]


type Msg
    = SubmittedForm
    | EnteredPhrase String
    | SelectedCategory String
    | SelectImage
    | SelectedImage File
    | GotImageUrl String
    | ClickedDismissErrors


clearForm : Model -> Model
clearForm model =
    { model | phrase = "", categories = Set.empty, image = Nothing }


update :
    (Msg -> msg)
    -> (Result Http.Error Api.Tile -> msg)
    -> Api.User
    -> Msg
    -> Model
    -> ( Model, Cmd msg )
update toMsg newTileMsg user msg model =
    case msg of
        SubmittedForm ->
            case validateForm model of
                Ok form ->
                    ( model
                    , Api.addTile user form newTileMsg
                    )

                Err problems ->
                    ( { model | problems = problems }, Cmd.none )

        EnteredPhrase s ->
            ( { model | phrase = s }
            , Cmd.none
            )

        SelectedCategory s ->
            ( { model | categories = Set.insert s model.categories }
            , Cmd.none
            )

        SelectImage ->
            ( model
            , File.Select.file [ "image/svg+xml", "image/png" ]
                (\f -> toMsg (SelectedImage f))
            )

        SelectedImage i ->
            ( { model | image = Just i }
            , File.toUrl i
                |> Task.perform (\u -> toMsg (GotImageUrl u))
            )

        GotImageUrl u ->
            ( { model | imgUrl = Just u }
            , Cmd.none
            )

        ClickedDismissErrors ->
            ( { model | problems = [] }, Cmd.none )


validateForm : Model -> Result (List String) Api.TileForm
validateForm { phrase, categories, image } =
    case image of
        Just i ->
            Ok (Api.TileForm phrase (Set.toList categories) i)

        _ ->
            Err []
