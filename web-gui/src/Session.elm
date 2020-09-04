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


module Session exposing
    ( Session(..)
    , fromUser
    , navKey
    , onChange
    , upgrade
    , user
    )

import Api exposing (User)
import Browser.Navigation as Nav


type Session
    = LoggedIn Nav.Key User
    | Guest Nav.Key


navKey : Session -> Nav.Key
navKey s =
    case s of
        LoggedIn k _ ->
            k

        Guest k ->
            k


user : Session -> Maybe User
user s =
    case s of
        LoggedIn _ v ->
            Just v

        Guest _ ->
            Nothing


fromUser : Nav.Key -> Maybe User -> Session
fromUser key maybeUser =
    case maybeUser of
        Just u ->
            LoggedIn key u

        Nothing ->
            Guest key


upgrade : Session -> User -> Session
upgrade session u =
    LoggedIn (navKey session) u


onChange : (Session -> msg) -> Nav.Key -> Sub msg
onChange toMsg key =
    Api.onUserChange
        (\maybeUser -> toMsg (fromUser key maybeUser))
