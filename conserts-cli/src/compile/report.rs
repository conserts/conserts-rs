// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use std::path::Path;

use colored::*;

#[cfg(not(tarpaulin_include))] // IO function
pub(super) fn general_result(base_path: &Path) {
    println!(
        "{}: Compiled your ConSert to {}",
        "Success".bright_green().bold(),
        &base_path.to_string_lossy().bold() // to remove //?/ prefix on windows uncomment the PATH_PREFIX const and use [PATH_PREFIX.len()..]
    );
}

#[cfg(not(tarpaulin_include))] // IO function
pub(crate) fn rust_missing() {
    println!(
        "{}: The compiled crate is not processed further as Rust (including cargo) is missing on this system",
        "Warning".bright_yellow().bold(),
    );
}
