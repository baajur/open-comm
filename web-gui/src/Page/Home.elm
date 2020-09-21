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


module Page.Home exposing
    ( Model
    , Msg
    , init
    , subscriptions
    , toSession
    , update
    , view
    )

import Api
import BoundedDeque exposing (BoundedDeque)
import Dict exposing (Dict)
import Html exposing (Html)
import Html.Attributes as Attr
import Html.Events as Events
import Http
import Icon
import Page.AddTile as AddTile
import Session exposing (Session)


type alias Model =
    { session : Session
    , addTile : Maybe AddTile.Model
    , categories : Dict String (List Api.Tile)
    , category : String
    , speaking : Maybe String
    , speechQueue : BoundedDeque String
    , problems : List String
    }


toSession : Model -> Session
toSession { session } =
    session


init : Session -> Maybe String -> ( Model, Cmd Msg )
init session category =
    let
        cat =
            Maybe.withDefault "" category
    in
    ( { session = session
      , addTile = Nothing
      , categories = Dict.empty
      , category = cat
      , speaking = Nothing
      , speechQueue = BoundedDeque.empty 10
      , problems = []
      }
    , case session of
        Session.LoggedIn _ user ->
            Api.getTiles user category (GotTiles cat)

        Session.Guest _ ->
            Cmd.none
    )


view : Model -> { title : String, content : Html Msg }
view { session, addTile, categories, category, speaking, speechQueue } =
    let
        addTileView =
            case ( session, addTile ) of
                ( Session.LoggedIn _ _, Just t ) ->
                    [ AddTile.view t
                        |> Html.map GotAddTileMsg
                    ]

                ( Session.LoggedIn _ _, Nothing ) ->
                    [ Html.div
                        [ Events.onClick StartAddingTile
                        , Attr.class "tile"
                        ]
                        [ Icon.add ]
                    ]

                _ ->
                    []

        speakingView =
            case speaking of
                Just phrase ->
                    Html.div [ Attr.class "speech-text" ]
                        [ Html.strong []
                            [ Html.text phrase ]
                        , BoundedDeque.foldl
                            (\s a -> " " ++ s ++ a)
                            ""
                            speechQueue
                            |> Html.text
                        ]

                Nothing ->
                    Html.text ""
    in
    { title = "Home"
    , content =
        Html.div []
            [ Html.div [ Attr.class "tiles" ]
                ((Maybe.withDefault [] (Dict.get category categories)
                    |> List.map (viewTile (Session.user session))
                 )
                    ++ addTileView
                )
            , speakingView
            ]
    }


viewTile : Maybe Api.User -> Api.Tile -> Html Msg
viewTile user tile =
    let
        imgSrc =
            case user of
                Just u ->
                    Api.userImg u tile.image

                Nothing ->
                    tile.image
    in
    Html.div
        [ Attr.class "tile"
        , Events.onClick (SpeakPhrase tile.phrase)
        ]
        [ Html.h1 [ Attr.class "tile-phrase" ] [ Html.text tile.phrase ]
        , Html.img
            [ Attr.class "tile-img"
            , Attr.src imgSrc
            ]
            []
        ]


type Msg
    = GotSession Session
    | SpeakPhrase String
    | FinishedSpeaking String
    | GotAddTileMsg AddTile.Msg
    | GotTiles String (Result Http.Error (List Api.Tile))
    | AddedTile (Result Http.Error Api.Tile)
    | StartAddingTile


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotSession s ->
            ( { model | session = s }, Cmd.none )

        SpeakPhrase phrase ->
            case model.speaking of
                Just _ ->
                    ( { model
                        | speechQueue =
                            BoundedDeque.pushBack phrase model.speechQueue
                      }
                    , Cmd.none
                    )

                Nothing ->
                    ( { model | speaking = Just phrase }
                    , Api.speakText phrase
                    )

        FinishedSpeaking _ ->
            let
                ( next, queue ) =
                    BoundedDeque.popFront model.speechQueue
            in
            ( { model | speechQueue = queue, speaking = next }
            , case next of
                Just phrase ->
                    Api.speakText phrase

                Nothing ->
                    Cmd.none
            )

        GotAddTileMsg subMsg ->
            case ( model.session, model.addTile ) of
                ( Session.LoggedIn _ user, Just subModel ) ->
                    AddTile.update GotAddTileMsg AddedTile user subMsg subModel
                        |> updateWith
                            (\s -> { model | addTile = Just s })

                ( _, Nothing ) ->
                    ( model, Cmd.none )

                _ ->
                    ( { model | addTile = Nothing }, Cmd.none )

        GotTiles cat (Ok tiles) ->
            ( { model
                | categories = Dict.insert cat tiles model.categories
              }
            , Cmd.none
            )

        GotTiles _ (Err _) ->
            let
                errMsg =
                    "unable to query tiles"
            in
            ( { model | problems = [ errMsg ] }
            , Cmd.none
            )

        StartAddingTile ->
            ( { model | addTile = Just AddTile.init }
            , Cmd.none
            )

        AddedTile (Ok tile) ->
            let
                addToCategory c acc =
                    Dict.update
                        c
                        (\l ->
                            Just
                                (List.sortBy .phrase
                                    (tile :: Maybe.withDefault [] l)
                                )
                        )
                        acc
            in
            ( { model
                | categories =
                    List.foldl
                        addToCategory
                        model.categories
                        ("" :: tile.categories)
                , addTile = Nothing
              }
            , Cmd.none
            )

        AddedTile (Err e) ->
            let
                errMsg =
                    case e of
                        Http.BadStatus 409 ->
                            "There is already a tile for that phrase."

                        Http.BadStatus n ->
                            "Internal error (" ++ String.fromInt n ++ ")."

                        _ ->
                            "Unknown error."
            in
            ( { model | problems = [ errMsg ] }
            , Cmd.none
            )


updateWith :
    (subModel -> Model)
    -> ( subModel, Cmd Msg )
    -> ( Model, Cmd Msg )
updateWith toModel ( subModel, subCmd ) =
    ( toModel subModel
    , subCmd
    )


subscriptions : Model -> Sub Msg
subscriptions { session } =
    Sub.batch
        [ Session.onChange GotSession (Session.navKey session)
        , Api.onSpeechEnd FinishedSpeaking
        ]
