// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use super::io::CrateFile;
use askama::Template;
use conserts_elements::elements::{demands::Demand, services::RequiredService};
use conserts_error::ConSertError;

#[derive(Template)]
#[template(path = "Cargo.toml", escape = "none")]
struct CargoTomlTemplate {
    name: String,
    checksum: String,
}

pub(super) fn generate_cargo_toml(
    consert: &conserts_elements::consert::Consert,
) -> Result<impl Iterator<Item = CrateFile>, ConSertError<Demand, RequiredService>> {
    let content = CargoTomlTemplate {
        name: consert.crate_name(),
        checksum: consert.checksum(),
    }
    .render()?;

    Ok(std::iter::once((
        std::path::PathBuf::new().join("Cargo.toml"),
        content,
    )))
}

#[derive(Template)]
#[template(path = ".gitignore", escape = "none")]
struct GitignoreTemplate {}

pub(super) fn generate_gitignore(
) -> Result<impl Iterator<Item = CrateFile>, ConSertError<Demand, RequiredService>> {
    let data = GitignoreTemplate {}.render()?;
    Ok(std::iter::once((
        std::path::PathBuf::new().join(".gitignore"),
        data,
    )))
}

pub(super) fn generate_dot(
    consert: &conserts_elements::consert::Consert,
) -> Result<impl Iterator<Item = CrateFile>, ConSertError<Demand, RequiredService>> {
    let content = conserts_plot::plot(consert)?;
    Ok(std::iter::once((
        std::path::PathBuf::new().join("Consert.dot"),
        content,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn gitignore() {
        let content = generate_gitignore().unwrap().next().unwrap();
        assert_eq!(
            (
                PathBuf::new().join(".gitignore"),
                std::fs::read_to_string("../tests/resources/.gitignore").unwrap()
            ),
            content
        );
    }
}
