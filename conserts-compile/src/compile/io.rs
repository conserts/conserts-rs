// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use conserts_elements::{demands::Demand, services::RequiredService};
use conserts_error::{CompileError, ConSertError};
use std::fs;
use std::path;

pub(super) type CrateFile = (path::PathBuf, String);

#[cfg(not(tarpaulin_include))] // io function
pub(super) fn write_files<T>(
    base_path: T,
    files: Vec<CrateFile>,
) -> Result<(), ConSertError<Demand, RequiredService>>
where
    T: AsRef<path::Path>,
{
    for (file_path, content) in files {
        fs::write(base_path.as_ref().join(&file_path), content)?;
    }
    Ok(())
}

#[cfg(not(tarpaulin_include))] // io function
pub(super) fn create_directories<T: AsRef<path::Path>>(
    base_path: T,
    _: &crate::compile::CompileParameters,
) -> Result<(), ConSertError<Demand, RequiredService>> {
    fs::create_dir_all(&base_path.as_ref().join("src"))?;
    Ok(())
}

#[cfg(not(tarpaulin_include))] // io function
pub(super) fn format_crate<T: AsRef<path::Path>>(
    base_path: T,
) -> Result<(), ConSertError<Demand, RequiredService>> {
    let cargo = "cargo";
    let _ = which::which(cargo).map_err(|_| CompileError::MissingRust())?;
    let _ = std::process::Command::new(cargo)
        .arg("fmt")
        .current_dir(base_path)
        .output()?;
    Ok(())
}

#[cfg(not(tarpaulin_include))] // io function
pub(super) fn convert_dot<T: AsRef<path::Path>>(
    base_path: T,
) -> Result<(), ConSertError<Demand, RequiredService>> {
    let dot = "dot";
    let _ = which::which(dot).map_err(|_| CompileError::MissingGraphViz())?;
    let _ = std::process::Command::new(dot)
        .arg("-Tsvg")
        .arg("Consert.dot")
        .arg("-o")
        .arg("Consert.svg")
        .current_dir(base_path)
        .output()?;
    Ok(())
}
