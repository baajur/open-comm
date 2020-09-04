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


module Page.Login exposing (..)

import Api exposing (LoginForm, login)
import Browser exposing (Document)
import Html exposing (Html)
import Html.Attributes as Attr
import Html.Events as Events
import Http
import Route exposing (Route)
import Session exposing (Session)


type alias Model =
    { form : LoginForm
    , session : Session
    , problems : List Problem
    }


type Problem
    = InvalidEntry ValidatedField String
    | ServerError String


type ValidatedField
    = Username
    | Password


showProblem : Problem -> String
showProblem p =
    case p of
        InvalidEntry _ s ->
            s

        ServerError s ->
            s


toSession : Model -> Session
toSession { session } =
    session


init : Session -> ( Model, Cmd Msg )
init session =
    ( { form = { username = "", password = "" }
      , session = session
      , problems = []
      }
    , Cmd.none
    )


view : Model -> { title : String, content : Html Msg }
view { form, problems } =
    { title = "Login"
    , content =
        Html.form [ Events.onSubmit SubmittedForm ]
            ([ Html.input
                [ Attr.placeholder "Username"
                , Events.onInput EnteredUsername
                , Attr.value form.username
                ]
                []
             , Html.input
                [ Attr.placeholder "Password"
                , Events.onInput EnteredPassword
                , Attr.value form.password
                ]
                []
             , Html.button []
                [ Html.text "Sign in" ]
             ]
                ++ List.map
                    (\s -> Html.p [] [ Html.text (showProblem s) ])
                    problems
            )
    }


type Msg
    = SubmittedForm
    | EnteredUsername String
    | EnteredPassword String
    | CompletedLogin (Result Http.Error Api.User)
    | GotSession Session


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        SubmittedForm ->
            case validate model.form of
                Ok validForm ->
                    ( { model | problems = [] }
                    , Api.login validForm CompletedLogin
                    )

                Err problems ->
                    ( { model | problems = problems }
                    , Cmd.none
                    )

        EnteredUsername username ->
            updateForm (\form -> { form | username = username }) model

        EnteredPassword password ->
            updateForm (\form -> { form | password = password }) model

        CompletedLogin (Err error) ->
            ( { model | problems = ServerError "Login failed." :: model.problems }
            , Cmd.none
            )

        CompletedLogin (Ok user) ->
            ( { model | session = Session.upgrade model.session user }
            , Api.storeUser user
            )

        GotSession s ->
            ( { model | session = s }
            , Route.pushUrl (Session.navKey s) Route.Home
            )


fieldsToValidate : List ValidatedField
fieldsToValidate =
    [ Username, Password ]


validate : LoginForm -> Result (List Problem) LoginForm
validate form =
    let
        trimmedForm =
            trimForm form
    in
    case List.concatMap (validateField trimmedForm) fieldsToValidate of
        [] ->
            Ok trimmedForm

        problems ->
            Err problems


validateField : LoginForm -> ValidatedField -> List Problem
validateField { username, password } field =
    List.map (InvalidEntry field) <|
        case field of
            Username ->
                if String.isEmpty username then
                    [ "username can't be blank." ]

                else
                    []

            Password ->
                if String.isEmpty password then
                    [ "password can't be blank." ]

                else
                    []


updateForm : (LoginForm -> LoginForm) -> Model -> ( Model, Cmd Msg )
updateForm transform model =
    ( { model | form = transform model.form }, Cmd.none )


trimForm : LoginForm -> LoginForm
trimForm { username, password } =
    { username = String.trim username
    , password = String.trim password
    }


subscriptions : Model -> Sub Msg
subscriptions { session } =
    Session.onChange GotSession (Session.navKey session)
