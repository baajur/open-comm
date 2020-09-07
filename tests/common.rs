/*
 * Copyright (C) 2020 Oakes, Gregory <gregoryoakes@fastmail.com>
 * Author: Oakes, Gregory <gregory.oakes@fastmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::{env, process::Command};

pub fn setup() {
    env::set_var(
        "ROCKET_DATABASES",
        format!(
            "{{user_db={{url=\"{}\"}}}}",
            String::from_utf8_lossy(
                Command::new("pg_tmp")
                    .output()
                    .expect("valid pg_tmp instance")
                    .stdout
                    .as_ref()
            )
        ),
    );
}
