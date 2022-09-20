#![forbid(unsafe_code)]
#![deny(unused_results)]

// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use askama::Template;
use conserts_elements::guarantees::Guarantee;
use conserts_elements::{consert::Consert, demands::Demand, elements, services::RequiredService};
use conserts_error::{ConSertError, ParsingError};
use roxmltree::{self, ExpandedName, Node};
use sha3::{Digest, Sha3_256};
use std::convert::{TryFrom, TryInto};
use std::sync::{Arc, Mutex};
use std::{path::Path, rc::Rc};

mod demands;
mod dimensions;
mod evidence;
mod guarantees;
mod services;
mod templates;
mod yml;

#[derive(Debug)]
pub struct ConsertElements {
    guarantees: Vec<Arc<Guarantee>>,
    demands: Vec<Arc<Mutex<elements::demands::Demand>>>,
    provided_services: elements::services::ProvidedServices,
    required_services: elements::services::RequiredServices,
    evidence: Vec<Arc<elements::evidence::Evidence>>,
}

impl<'a> TryFrom<&roxmltree::Document<'a>> for ConsertElements {
    type Error = ConSertError<elements::demands::Demand, elements::services::RequiredService>;

    fn try_from(doc: &roxmltree::Document<'a>) -> Result<Self, Self::Error> {
        let evidence = evidence::parse(doc)?;
        let demands = demands::parse(doc)?;

        let guarantee_propagations = guarantees::parse_guarantee_propagations(doc)?;
        let gates = guarantees::parse_gates(doc)?;

        let guarantees = guarantees::parse_guarantees(
            doc,
            &evidence,
            &guarantee_propagations,
            &gates,
            &demands,
        )?;

        let (provided_services, required_services) =
            services::parse_services(doc, &guarantees, &demands)?;

        Ok(Self {
            guarantees,
            demands,
            provided_services,
            required_services,
            evidence,
        })
    }
}

impl TryFrom<yml::YamlConsertElements> for ConsertElements {
    type Error = ConSertError<elements::demands::Demand, elements::services::RequiredService>;

    fn try_from(value: yml::YamlConsertElements) -> Result<Self, Self::Error> {
        value.unique_ids()?;

        let evidence = value.evidence();
        let demands = value.demands();
        let guarantees = value.guarantees()?;

        let provided_services = value.provided_services();
        let required_services = value.required_services();

        Ok(Self {
            guarantees,
            demands,
            provided_services,
            required_services,
            evidence,
        })
    }
}

impl ConsertElements {
    pub fn guarantees(&self) -> Vec<Arc<elements::guarantees::Guarantee>> {
        self.guarantees.clone()
    }

    pub fn demands(&self) -> Vec<Arc<Mutex<elements::demands::Demand>>> {
        self.demands.clone()
    }

    pub fn provided_services(&self) -> Vec<Rc<elements::services::ProvidedService>> {
        self.provided_services.clone()
    }

    pub fn required_services(&self) -> Vec<Arc<elements::services::RequiredService>> {
        self.required_services.clone()
    }

    pub fn evidence(&self) -> Vec<Arc<elements::evidence::Evidence>> {
        self.evidence.clone()
    }
}

pub trait GetAttribute {
    fn try_get_attribute(&self, attribute: &str) -> Result<&str, ParsingError>;
}

impl GetAttribute for roxmltree::Node<'_, '_> {
    fn try_get_attribute(&self, attribute: &str) -> Result<&str, ParsingError> {
        self.attribute(attribute)
            .ok_or_else(|| ParsingError::MissingXmlAttribute(attribute.to_string()))
    }
}

pub fn get_refinement(
    g: &Node,
) -> Result<String, ConSertError<elements::demands::Demand, elements::services::RequiredService>> {
    Ok(g.descendants()
        .find(|n| n.tag_name() == ExpandedName::from("refinement"))
        .ok_or_else(|| ParsingError::MissingXmlDescendent("refinement".to_string()))?
        .try_get_attribute("name")?
        .to_string())
}

pub fn get_functional_service_type(
    n: Node,
) -> Result<String, ConSertError<elements::demands::Demand, elements::services::RequiredService>> {
    Ok(n.descendants()
        .find(|n| n.tag_name() == ExpandedName::from("functionalServiceType"))
        .ok_or_else(|| ParsingError::MissingXmlDescendent("functionalServiceType".to_string()))?
        .try_get_attribute("name")?
        .to_string())
}

fn parse_model(xml: &str) -> Result<roxmltree::Document, ConSertError<Demand, RequiredService>> {
    Ok(roxmltree::Document::parse(xml).map_err(ParsingError::Xml)?)
}

fn read_model_file(path: &Path) -> Result<String, ConSertError<Demand, RequiredService>> {
    Ok(std::fs::read_to_string(path)?)
}

fn hash_model(model: &str) -> String {
    // create a SHA3-256 object
    let mut hasher = Sha3_256::new();
    // write input message
    hasher.update(model);
    // read hash digest
    hex::encode(hasher.finalize())
}

pub trait Yaml {
    fn to_yaml(&self) -> Result<String, ConSertError<Demand, RequiredService>>;
    fn from_yaml<P: AsRef<Path>>(
        path: &P,
        checksum: String,
        doc: String,
    ) -> Result<Consert, ConSertError<Demand, RequiredService>>;
    fn from_path_yaml<P: AsRef<Path>>(
        path: &P,
    ) -> Result<Consert, ConSertError<Demand, RequiredService>>;
}

impl Yaml for Consert {
    fn to_yaml(&self) -> Result<String, ConSertError<Demand, RequiredService>> {
        unimplemented!()
    }

    fn from_yaml<P: AsRef<Path>>(
        path: &P,
        checksum: String,
        doc: String,
    ) -> Result<Consert, ConSertError<Demand, RequiredService>> {
        let name = Consert::path_to_name(path)?;
        let path = path.as_ref().to_string_lossy().to_string();

        use std::str::FromStr;
        let elements = yml::YamlConsertElements::from_str(doc.as_str())?;
        let elements: ConsertElements = elements.try_into()?;

        Ok(Consert::new(
            name,
            path,
            checksum,
            elements.guarantees(),
            elements.demands(),
            elements.provided_services(),
            elements.required_services(),
            elements.evidence(),
        ))
    }

    fn from_path_yaml<P: AsRef<Path>>(
        path: &P,
    ) -> Result<Consert, ConSertError<Demand, RequiredService>> {
        let model = read_model_file(path.as_ref())?;

        let checksum = hash_model(&model);

        Consert::from_yaml(path, checksum, model)
    }
}

pub trait Xml {
    fn to_xml(&self) -> Result<String, ConSertError<Demand, RequiredService>>;
    fn from_xml<P: AsRef<Path>>(
        path: &P,
        checksum: String,
        doc: roxmltree::Document,
    ) -> Result<Consert, ConSertError<Demand, RequiredService>>;
    fn from_path_xml<P: AsRef<Path>>(
        path: &P,
    ) -> Result<Consert, ConSertError<Demand, RequiredService>>;
}

impl Xml for Consert {
    fn to_xml(&self) -> Result<String, ConSertError<Demand, RequiredService>> {
        Ok(templates::ConsertTemplate::from(self).render()?)
    }

    fn from_xml<P: AsRef<Path>>(
        path: &P,
        checksum: String,
        doc: roxmltree::Document,
    ) -> Result<Consert, ConSertError<Demand, RequiredService>> {
        let name = Consert::path_to_name(path)?;
        let path = path.as_ref().to_string_lossy().to_string();

        let elements = ConsertElements::try_from(&doc)?;

        Ok(Consert::new(
            name,
            path,
            checksum,
            elements.guarantees(),
            elements.demands(),
            elements.provided_services(),
            elements.required_services(),
            elements.evidence(),
        ))
    }

    fn from_path_xml<P: AsRef<Path>>(
        path: &P,
    ) -> Result<Consert, ConSertError<Demand, RequiredService>> {
        let model = read_model_file(path.as_ref())?;

        let doc = parse_model(&model)?;
        let checksum = hash_model(&model);

        Consert::from_xml(path, checksum, doc)
    }
}

#[cfg(test)]
mod tests {
    use super::Yaml;
    use conserts_elements::consert::Consert;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_yml_input() {
        let s = Consert::from_path_yaml(&"../models/FabOS_Scanner.yml").unwrap();
        assert_eq!(s.demands().len(), 1);
        assert_eq!(s.guarantees().len(), 2);
    }

    #[test]
    fn test_valid_yml() {
        if Consert::from_path_yaml(&"../models/InvalidGuarantee.yml").is_err() {
        } else {
            panic!("Should fail")
        }

        if Consert::from_path_yaml(&"../models/InvalidCovered.yml").is_err() {
        } else {
            panic!("Should fail")
        }
    }
}
