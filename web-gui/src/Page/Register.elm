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


module Page.Register exposing
    ( Model
    , Msg
    , init
    , subscriptions
    , toSession
    , update
    , view
    )

import Api exposing (RegisterForm, register)
import Html exposing (Html)
import Html.Attributes as Attr
import Html.Events as Events
import Http
import Page
import Route
import Session exposing (Session)


type alias Model =
    { form : RegisterEntryForm
    , session : Session
    , problems : List Problem
    }


type alias RegisterEntryForm =
    { username : String
    , password : String
    , passwordRepeat : String
    }


type Problem
    = InvalidEntry ValidatedField String
    | ServerError String


type ValidatedField
    = Username
    | Password
    | RepeatPassword


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
    ( { form = { username = "", password = "", passwordRepeat = "" }
      , session = session
      , problems = []
      }
    , Cmd.none
    )


view : Model -> { title : String, content : Html Msg }
view { form, problems } =
    { title = "Register"
    , content =
        Html.div []
            [ Html.form [ Events.onSubmit SubmittedForm ]
                [ Html.input
                    [ Attr.placeholder "Username"
                    , Events.onInput EnteredUsername
                    , Attr.value form.username
                    ]
                    []
                , Html.input
                    [ Attr.placeholder "Password"
                    , Events.onInput EnteredPassword
                    , Attr.value form.password
                    , Attr.type_ "password"
                    ]
                    []
                , Html.input
                    [ Attr.placeholder "Repeat Password"
                    , Events.onInput EnteredRepeatPassword
                    , Attr.value form.passwordRepeat
                    , Attr.type_ "password"
                    ]
                    []
                , Html.button []
                    [ Html.text "Register" ]
                ]
            , List.map showProblem problems
                |> Page.viewErrors ClickedDismissErrors
            ]
    }


type Msg
    = SubmittedForm
    | EnteredUsername String
    | EnteredPassword String
    | EnteredRepeatPassword String
    | CompletedRegistration (Result Http.Error Api.User)
    | GotSession Session
    | ClickedDismissErrors


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        SubmittedForm ->
            case validate model.form of
                Ok validForm ->
                    ( { model | problems = [] }
                    , Api.register validForm CompletedRegistration
                    )

                Err problems ->
                    ( { model | problems = problems }
                    , Cmd.none
                    )

        EnteredUsername username ->
            updateForm (\form -> { form | username = username }) model

        EnteredPassword password ->
            updateForm (\form -> { form | password = password }) model

        EnteredRepeatPassword passwordRepeat ->
            updateForm (\form -> { form | passwordRepeat = passwordRepeat }) model

        CompletedRegistration (Err e) ->
            let
                errMsg =
                    case e of
                        Http.BadStatus 409 ->
                            "Username is taken."

                        Http.BadStatus n ->
                            "Internal error (" ++ String.fromInt n ++ ")."

                        _ ->
                            "Unknown error."
            in
            ( { model | problems = ServerError errMsg :: model.problems }
            , Cmd.none
            )

        CompletedRegistration (Ok user) ->
            ( { model | session = Session.upgrade model.session user }
            , Api.storeUser user
            )

        GotSession s ->
            ( { model | session = s }
            , Route.pushUrl (Session.navKey s) Route.Home
            )

        ClickedDismissErrors ->
            ( { model | problems = [] }
            , Cmd.none
            )


fieldsToValidate : List ValidatedField
fieldsToValidate =
    [ Username, Password, RepeatPassword ]


validate : RegisterEntryForm -> Result (List Problem) RegisterForm
validate form =
    let
        trimmedForm =
            trimForm form
    in
    case List.concatMap (validateField trimmedForm) fieldsToValidate of
        [] ->
            Ok
                { username = trimmedForm.username
                , password = trimmedForm.password
                }

        problems ->
            Err problems


validateField : RegisterEntryForm -> ValidatedField -> List Problem
validateField { username, password, passwordRepeat } field =
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

            RepeatPassword ->
                if passwordRepeat == password then
                    []

                else
                    [ "passwords must match." ]


updateForm : (RegisterEntryForm -> RegisterEntryForm) -> Model -> ( Model, Cmd Msg )
updateForm transform model =
    ( { model | form = transform model.form }, Cmd.none )


trimForm : RegisterEntryForm -> RegisterEntryForm
trimForm { username, password, passwordRepeat } =
    { username = String.trim username
    , password = String.trim password
    , passwordRepeat = String.trim passwordRepeat
    }


subscriptions : Model -> Sub Msg
subscriptions { session } =
    Session.onChange GotSession (Session.navKey session)
