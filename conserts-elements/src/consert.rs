// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::dimension::Dimension;
use crate::elements::{demands::Demand, services::RequiredService};
use crate::{
    elements::{consert_tree, demands, evidence, guarantees, services},
    guarantees::Guarantee,
    services::ProvidedService,
};
use conserts_error::{ConSertError, ConstructionError, ParsingError};
use proc_macro2::Ident;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct ConsertBuilder {
    name: Option<String>,
    path: Option<String>,
    guarantees: Vec<Arc<guarantees::Guarantee>>,
    demands: Vec<Arc<Mutex<demands::Demand>>>,
    provided_services: Vec<Rc<services::ProvidedService>>,
    required_services: Vec<Arc<services::RequiredService>>,
    evidence: Vec<Arc<evidence::Evidence>>,
}

impl ConsertBuilder {
    pub fn new() -> Self {
        Self {
            demands: vec![],
            evidence: vec![],
            guarantees: vec![],
            provided_services: vec![],
            required_services: vec![],
            name: None,
            path: None,
        }
    }

    pub fn name<S>(mut self, name: S) -> Self
    where
        S: Into<String>,
    {
        self.name = Some(name.into());
        self
    }

    pub fn path<S>(mut self, path: S) -> Self
    where
        S: Into<String>,
    {
        self.path = Some(path.into());
        self
    }

    pub fn add_runtime_evidence<S>(
        &mut self,
        name: S,
        description: Option<String>,
        dimension: Dimension,
    ) -> Arc<evidence::Evidence>
    where
        S: Into<String>,
    {
        let name = name.into();
        let evidence = Arc::new(evidence::Evidence::new(
            self.evidence.len(),
            &name,
            description,
            dimension,
        ));
        self.evidence.push(evidence.clone());
        evidence
    }

    pub fn insert_runtime_evidence(mut self, rte: Arc<evidence::Evidence>) -> Self {
        self.evidence.push(rte);
        self
    }

    pub fn add_guarantee<S>(
        mut self,
        name: S,
        description: Option<String>,
        dimension: Dimension,
        cst: consert_tree::Tree<consert_tree::ConsertTreeElement>,
    ) -> Self
    where
        S: Into<String>,
    {
        let name = &name.into();
        let guarantee = Arc::new(guarantees::Guarantee::new(
            self.guarantees.len(),
            name,
            description,
            dimension,
            cst,
        ));
        self.guarantees.push(guarantee);
        self
    }

    pub fn insert_guarantee(mut self, guarantee: Arc<guarantees::Guarantee>) -> Self {
        self.guarantees.push(guarantee);
        self
    }

    pub fn add_demand(mut self, demand: Arc<Mutex<demands::Demand>>) -> ConsertBuilder {
        self.demands.push(demand);
        self
    }

    pub fn build(self) -> Result<Consert, ConSertError<Demand, RequiredService>> {
        Ok(Consert {
            demands: self.demands,
            checksum: "NOT-TRACED-TO-A-XML-MODEL".to_string(),
            evidence: self.evidence,
            guarantees: self.guarantees,
            provided_services: self.provided_services,
            required_services: self.required_services,
            name: self.name.ok_or(ConstructionError::MissingName)?,
            path: self.path.ok_or(ConstructionError::MissingPath)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Consert {
    name: String,
    path: String,
    checksum: String,
    guarantees: Vec<Arc<guarantees::Guarantee>>,
    demands: Vec<Arc<Mutex<demands::Demand>>>,
    provided_services: Vec<Rc<services::ProvidedService>>,
    required_services: Vec<Arc<services::RequiredService>>,
    evidence: Vec<Arc<evidence::Evidence>>,
}

impl Consert {
    pub fn empty() -> Self {
        Self {
            demands: vec![],
            evidence: vec![],
            checksum: "NOT-TRACED-TO-A-XML-MODEL".to_string(),
            guarantees: vec![],
            provided_services: vec![],
            required_services: vec![],
            name: "empty".to_string(),
            path: "empty".to_string(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        path: String,
        checksum: String,
        guarantees: Vec<Arc<Guarantee>>,
        demands: Vec<Arc<Mutex<Demand>>>,
        provided_services: Vec<Rc<ProvidedService>>,
        required_services: Vec<Arc<RequiredService>>,
        evidence: Vec<Arc<evidence::Evidence>>,
    ) -> Self {
        Self {
            name,
            path,
            checksum,
            guarantees,
            demands,
            provided_services,
            required_services,
            evidence,
        }
    }

    // TODO: write test for this function
    pub fn path_to_name<P: AsRef<Path>>(
        path: &P,
    ) -> Result<String, ConSertError<Demand, RequiredService>> {
        Ok(path
            .as_ref()
            .file_stem()
            .ok_or_else(ParsingError::InvalidPathFileStem)?
            .to_owned()
            .into_string()
            .map_err(ParsingError::InvalidPath)?
            .replace('-', "_")
            .to_lowercase())
    }

    pub fn crate_name(&self) -> String {
        format!("consert_{}", self.name)
    }

    #[cfg(not(tarpaulin_include))] // getter
    pub fn checksum(&self) -> String {
        self.checksum.clone()
    }

    pub fn is_independent(&self) -> bool {
        self.required_services.is_empty() && self.demands.is_empty()
    }

    #[cfg(not(tarpaulin_include))] // getter
    pub fn required_services(&self) -> Vec<Arc<services::RequiredService>> {
        self.required_services.clone()
    }

    #[cfg(not(tarpaulin_include))] // getter
    pub fn provided_services(&self) -> Vec<Rc<services::ProvidedService>> {
        self.provided_services.clone()
    }

    #[cfg(not(tarpaulin_include))] // getter
    pub fn demands(&self) -> Vec<Arc<Mutex<demands::Demand>>> {
        self.demands.clone()
    }

    #[cfg(not(tarpaulin_include))] // getter
    pub fn path(&self) -> String {
        self.path.clone()
    }

    #[cfg(not(tarpaulin_include))] // getter
    pub fn evidence(&self) -> Vec<Arc<evidence::Evidence>> {
        self.evidence.clone()
    }

    #[cfg(not(tarpaulin_include))] // getter
    pub fn guarantees(&self) -> Vec<Arc<guarantees::Guarantee>> {
        self.guarantees.clone()
    }

    pub fn crate_ident(&self) -> Ident {
        format_ident!("consert_{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_path_to_name() {
        assert_eq!(
            Consert::path_to_name(&std::path::PathBuf::new().join("C:/Temp/The-Test-Crate.model"))
                .unwrap(),
            "the_test_crate"
        );
    }
}
