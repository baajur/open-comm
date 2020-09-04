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

use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let profile = env::var_os("PROFILE").unwrap().into_string().unwrap();
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let gui_lib_path = out_dir.join("elm.js");
    let gui_lib_path_str = gui_lib_path.to_str().unwrap();

    let output_arg = format!("--output={}", gui_lib_path_str);
    let profile_flag = match profile.as_str() {
        "release" => "--optimize",
        _ => "--debug",
    };
    Command::new("elm")
        .args(&["make", profile_flag, "src/Main.elm", output_arg.as_str()])
        .current_dir("web-gui")
        .status()
        .expect("Failed to build web-gui.");
    println!("cargo:rustc-env=GUI_LIB={}", gui_lib_path_str);

    let gui_index_path = out_dir.join("index.html");
    fs::copy("web-gui/index.html", gui_index_path.clone())
        .expect("Failed to copy web-gui entry point to out directory.");
    println!("cargo:rustc-env=GUI_INDEX={}", gui_index_path.to_str().unwrap());
}
