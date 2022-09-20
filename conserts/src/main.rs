#![forbid(unsafe_code)]
#![deny(unused_results)]

// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

extern crate pretty_env_logger;
use color_eyre::eyre::Result;

#[cfg(not(tarpaulin_include))] // integration function
fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();
    let matches = conserts_cli::get_cli_parameters();
    conserts_cli::run(&matches)
}
