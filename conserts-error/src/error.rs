#![forbid(unsafe_code)]
#![deny(unused_results)]

// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use std::convert::Infallible;
use std::ffi::OsString;
use std::fmt::Debug;
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use std::sync::{Arc, Mutex};

use roxmltree::Error as XMLError;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug, Eq, PartialEq)]
pub enum ConstructionError {
    #[error("Name is missing. Consider calling .name()")]
    MissingName,
    #[error("Path is missing. Consider calling .path()")]
    MissingPath,
}

#[non_exhaustive]
#[derive(Error, Debug, Eq, PartialEq)]
pub enum ParsingError {
    #[error("Invalid OS path: {0:#?}")]
    InvalidPath(OsString),
    #[error("Invalid ConSert Tree path: {0:#?}")]
    InvalidConsertTreePath(String),
    #[error("Invalid path file stem")]
    InvalidPathFileStem(),
    #[error("Unsupported operator: {0}")]
    UnsupportedOperator(String),
    #[error("Unsupported gate: {0}")]
    UnsupportedGate(String),
    #[error("Could not find a {0} with index: {1}")]
    WrongIndex(String, String),
    #[error("Could not parse XML document")]
    Xml(XMLError),
    #[error("Missing XML descendent: {0}")]
    MissingXmlDescendent(String),
    #[error("Missing XML attribute: {0}")]
    MissingXmlAttribute(String),
    #[error("Missing unit in '{0}'. There must be a space between value and unit.")]
    MissingUnit(String),
    #[error("Missing threshold in '{0}'.")]
    MissingThreshold(String),
    #[error("Missing a matching evidence {0}.")]
    MissingEvidence(String),
    #[error("Missing a matching property {0}")]
    MissingProperty(String),
    #[error("Missing element with id: {0}")]
    MissingElement(String),
    #[error("Failed to parse integer: {0}")]
    Integer(ParseIntError),
    #[error("IDs are not unique")]
    NonUniqueIds,
    #[error("Failed parsing: {0}")]
    Other(String),
}
#[non_exhaustive]
#[derive(Error, Debug, Eq, PartialEq)]
pub enum UnitOfMeasureError {
    #[error("Unsupported unit: {0}")]
    UnsupportedUnit(String),
    #[error("Incompatible units")] // consider showing the 2 units
    Incompatible,
}

#[non_exhaustive]
#[derive(Error, Debug, Eq, PartialEq)]
pub enum CompileError {
    #[error("A Rust installation (including cargo) is missing.")]
    MissingRust(),
    #[error("A GraphViz installation (including dot) is missing.")]
    MissingGraphViz(),
    #[error("Failed compiling: {0}")]
    Other(String),
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum CompositionError<Demand: Debug, RequiredService: Debug> {
    #[error(
        "Failed while composing {path} with existing SoS due to\n- unmatched demands: {unmatched_demands:#?}\n- unmatched required services: {unmatched_required_services:#?}\n\n Consider providing additional ConSerts using the --provider option"
    )]
    Incompatible {
        path: String,
        unmatched_demands: Vec<Arc<Mutex<Demand>>>,
        unmatched_required_services: Vec<Arc<RequiredService>>,
    },
    #[error("Consert is not independent")]
    Dependent,
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ConSertError<Demand: 'static, RequiredService: 'static>
where
    Demand: Debug,
    RequiredService: Debug,
{
    #[error("Failed compiling a ConSert: ")]
    Compile {
        #[from]
        source: CompileError,
    },
    #[error("Failed constructing a ConSert: ")]
    Construction {
        #[from]
        source: ConstructionError,
    },
    #[error("IO error occured")]
    Io(#[from] std::io::Error),
    #[error("Failed parsing file")]
    Parsing {
        #[from]
        source: ParsingError,
    },
    #[error("Composition failed")]
    Composition {
        #[from]
        source: CompositionError<Demand, RequiredService>,
    },
    #[error("Unit of measurement error: ")]
    UnitOfMeasure {
        #[from]
        source: UnitOfMeasureError,
    },
    #[error("Askama error:")]
    Askama {
        #[from]
        source: askama::Error,
    },
    #[error("Serde json error:")]
    SerdeJson {
        #[from]
        source: serde_json::Error,
    },
    #[error("Regex error:")]
    Regex {
        #[from]
        source: regex::Error,
    },
    #[error("Serde YML error:")]
    SerdeYaml {
        #[from]
        source: serde_yaml::Error,
    },
    #[error("Infallible")]
    Infallible {
        #[from]
        source: Infallible,
    },
    #[error("UTF8")]
    FromUtf8 {
        #[from]
        source: FromUtf8Error,
    },
}
