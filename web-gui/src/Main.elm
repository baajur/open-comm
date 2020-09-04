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


module Main exposing (..)

import Api exposing (User)
import Browser
import Browser.Navigation as Nav
import Html
import Json.Decode as Decode exposing (Value)
import Page
import Page.Blank as Blank
import Page.Home as Home
import Page.Login as Login
import Page.NotFound as NotFound
import Page.Register as Register
import Route exposing (Route)
import Session exposing (Session)
import Url exposing (Url)



-- Model


type Model
    = Redirect Session
    | NotFound Session
    | Login Login.Model
    | Register Register.Model
    | Home Home.Model


init : Maybe User -> Url -> Nav.Key -> ( Model, Cmd Msg )
init maybeUser url navKey =
    changeRouteTo (Route.fromUrl url)
        (Redirect (Session.fromUser navKey maybeUser))


view : Model -> Browser.Document Msg
view model =
    let
        user =
            toSession model
                |> Session.user

        viewPage page toMsg config =
            let
                { title, body } =
                    Page.view user page config
            in
            { title = title
            , body = List.map (Html.map toMsg) body
            }
    in
    case model of
        Redirect _ ->
            Page.view user Page.Other Blank.view

        NotFound _ ->
            Page.view user Page.Other NotFound.view

        Login login ->
            viewPage Page.Login GotLoginMsg (Login.view login)

        Register register ->
            viewPage Page.Register GotRegisterMsg (Register.view register)

        Home home ->
            viewPage Page.Home GotHomeMsg (Home.view home)


type Msg
    = ChangeUrl Url
    | ClickedLink Browser.UrlRequest
    | GotLoginMsg Login.Msg
    | GotRegisterMsg Register.Msg
    | GotHomeMsg Home.Msg
    | GotSession Session


toSession : Model -> Session
toSession model =
    case model of
        Redirect s ->
            s

        NotFound s ->
            s

        Login m ->
            Login.toSession m

        Register m ->
            Register.toSession m

        Home m ->
            Home.toSession m


changeRouteTo : Maybe Route -> Model -> ( Model, Cmd Msg )
changeRouteTo maybeRoute model =
    let
        session =
            toSession model
    in
    case maybeRoute of
        Nothing ->
            ( NotFound session, Cmd.none )

        Just Route.Login ->
            Login.init session
                |> updateWith Login GotLoginMsg model

        Just Route.Register ->
            Register.init session
                |> updateWith Register GotRegisterMsg model

        Just Route.Home ->
            Home.init session
                |> updateWith Home GotHomeMsg model


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model ) of
        ( ClickedLink (Browser.Internal url), _ ) ->
            ( model
            , Nav.pushUrl (Session.navKey (toSession model)) (Url.toString url)
            )

        ( ClickedLink (Browser.External href), _ ) ->
            ( model
            , Nav.load href
            )

        ( GotLoginMsg subMsg, Login subModel ) ->
            Login.update subMsg subModel
                |> updateWith Login GotLoginMsg model

        ( GotRegisterMsg subMsg, Register subModel ) ->
            Register.update subMsg subModel
                |> updateWith Register GotRegisterMsg model

        ( GotHomeMsg subMsg, Home subModel ) ->
            Home.update subMsg subModel
                |> updateWith Home GotHomeMsg model

        ( ChangeUrl url, _ ) ->
            changeRouteTo (Route.fromUrl url) model

        ( GotSession session, Redirect _ ) ->
            ( Redirect session
            , Route.pushUrl (Session.navKey session) Route.Home
            )

        ( _, _ ) ->
            ( model, Cmd.none )


updateWith :
    (subModel -> Model)
    -> (subMsg -> Msg)
    -> Model
    -> ( subModel, Cmd subMsg )
    -> ( Model, Cmd Msg )
updateWith toModel toMsg model ( subModel, subCmd ) =
    ( toModel subModel
    , Cmd.map toMsg subCmd
    )


subscriptions : Model -> Sub Msg
subscriptions model =
    case model of
        NotFound _ ->
            Sub.none

        Redirect _ ->
            Session.onChange GotSession (Session.navKey (toSession model))

        Login subModel ->
            Sub.map GotLoginMsg (Login.subscriptions subModel)

        Register subModel ->
            Sub.map GotRegisterMsg (Register.subscriptions subModel)

        Home subModel ->
            Sub.map GotHomeMsg (Home.subscriptions subModel)


main : Program Value Model Msg
main =
    Api.application
        { init = init
        , onUrlChange = ChangeUrl
        , onUrlRequest = ClickedLink
        , subscriptions = subscriptions
        , update = update
        , view = view
        }
