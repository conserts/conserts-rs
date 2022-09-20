#![forbid(unsafe_code)]
#![deny(unused_results)]

// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use color_eyre::eyre::{anyhow, Result};
use colored::*;
use compose::SystemOfSystems;
use conserts_compose::compose;
use conserts_elements::consert::Consert;
use conserts_error::{self, CompileError, ConSertError};
use std::rc::Rc;

mod compile;

#[cfg(not(tarpaulin_include))] // IO function
pub fn get_cli_parameters() -> clap::ArgMatches<'static> {
    let app = App::new("conserts")
        .about("Parse, compile, or compose ConSert files")
        .version(crate_version!())
        .subcommand(
            SubCommand::with_name("parse")
                .about("Parses a ConSert and prints it on the CLI")
                .arg(
                    Arg::with_name("input")
                        .help("Input ConSert file")
                        .required(true)
                        .short("i")
                        .takes_value(true)
                        .value_name("FILE"),
                ),
        )
        .subcommand(
            SubCommand::with_name("plot").about("Plots a ConSert").arg(
                Arg::with_name("input")
                    .help("Input ConSert file")
                    .required(true)
                    .short("i")
                    .takes_value(true)
                    .value_name("FILE"),
            ),
        )
        .subcommand(
            SubCommand::with_name("compile")
                .about("Compiles a ConSert to a Rust crate")
                .arg(
                    Arg::with_name("input")
                        .help("Input ConSert file")
                        .required(true)
                        .short("i")
                        .takes_value(true)
                        .value_name("FILE"),
                )
                .arg(
                    Arg::with_name("provider")
                        .help("Providing ConSert file")
                        .long("provider")
                        .multiple(true)
                        .takes_value(true)
                        .value_name("FILE"),
                )
                .arg(
                    Arg::with_name("output")
                        .help("Output ConSert base folder (default: ./target)")
                        .short("o")
                        .takes_value(true)
                        .default_value("./target")
                        .value_name("FILE"),
                )
                .arg(
                    Arg::with_name("filter-depth")
                        .help("Depth of the monitor's filter")
                        .default_value("1")
                        .value_name("FILTER-DEPTH"),
                ),
        )
        .subcommand(
            SubCommand::with_name("compose")
                .about("Composes ConSerts in the order they are provided")
                .arg(
                    Arg::with_name("input")
                        .help("input ConSert file")
                        .required(true)
                        .multiple(true)
                        .short("i")
                        .takes_value(true)
                        .value_name("FILE"),
                ),
        );
    app.get_matches()
}

#[cfg(not(tarpaulin_include))] // integration function
pub fn run(matches: &ArgMatches) -> Result<()> {
    if let Some(matches) = matches.subcommand_matches("parse") {
        parse(matches)?;
    } else if let Some(matches) = matches.subcommand_matches("plot") {
        plot(matches)?;
    } else if let Some(matches) = matches.subcommand_matches("compile") {
        compile(matches)?;
    } else if let Some(matches) = matches.subcommand_matches("compose") {
        compose(matches)?;
    } else {
        println!("{}", matches.usage());
    }
    Ok(())
}

#[cfg(not(tarpaulin_include))] // integration function
fn plot(matches: &ArgMatches) -> Result<()> {
    let path = matches
        .value_of("input")
        .ok_or_else(|| anyhow!("No input provided"))?;
    let consert = consert_from_path(path)?;
    let dot = conserts_plot::plot(&consert)?;
    println!("{}", dot);
    Ok(())
}

#[cfg(not(tarpaulin_include))] // integration function
fn parse(matches: &ArgMatches) -> Result<()> {
    let path = matches
        .value_of("input")
        .ok_or_else(|| anyhow!("No input provided"))?;
    let consert = consert_from_path(path)?;
    println!("{:#?}", consert);
    Ok(())
}

#[cfg(not(tarpaulin_include))] // integration function
fn compile(args: &ArgMatches) -> Result<()> {
    let args = compile::parse_args(args)?;

    let mut consert = consert_from_path(&args.path())?;

    compile::test_composition(&mut consert, &args)?;

    match conserts_compile::compile::export(&args, &consert) {
        Ok(_) => {}
        Err(error) => match error {
            ConSertError::Compile { source } => match source {
                CompileError::MissingRust() => {
                    compile::report::rust_missing();
                }
                _ => return Err(source.into()),
            },
            _ => return Err(error.into()),
        },
    }
    compile::report_result(&consert, &args)
}

#[cfg(not(tarpaulin_include))] // IO function
fn compose(matches: &ArgMatches) -> Result<()> {
    let files = matches
        .values_of("input")
        .ok_or_else(|| anyhow!("Expected parameter"))?;
    let mut sos = SystemOfSystems::new();

    for file in files {
        let consert = Rc::new(consert_from_path(file)?);
        sos.add_consert(consert)?;
    }
    println!("{}: Composition possible.", "Success".bright_green().bold(),);
    Ok(())
}

#[cfg(not(tarpaulin_include))] // IO function
fn consert_from_path(path: &str) -> Result<Consert> {
    use conserts_parse::{Xml, Yaml};
    let consert = if path.ends_with(".xml") || path.ends_with(".model") {
        Consert::from_path_xml::<&str>(&path)?
    } else if path.ends_with(".yml") {
        Consert::from_path_yaml::<&str>(&path)?
    } else {
        return Err(anyhow!("Only .xml /.model and .yml supported"));
    };
    Ok(consert)
}
