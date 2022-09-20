// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::get_functional_service_type;
use crate::GetAttribute;
use conserts_elements::elements::demands::Demand;
use conserts_elements::elements::guarantees::Guarantee;
use conserts_elements::elements::services::{
    ProvidedService, ProvidedServices, RequiredService, RequiredServices,
};
use conserts_error::{ConSertError, ParsingError};
use roxmltree::{Document, ExpandedName};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub fn parse_services(
    doc: &Document,
    guarantees: &[Arc<Guarantee>],
    demands: &[Arc<Mutex<Demand>>],
) -> Result<(ProvidedServices, RequiredServices), ConSertError<Demand, RequiredService>> {
    let provided = parse_provided_services(doc, guarantees)?;
    let required = parse_required_services(doc, demands)?;

    Ok((provided, required))
}

fn parse_provided_services(
    doc: &Document,
    guarantees: &[Arc<Guarantee>],
) -> Result<ProvidedServices, ConSertError<Demand, RequiredService>> {
    doc.descendants()
        .filter(|n| n.tag_name() == ExpandedName::from("providedServices"))
        .map(|n| {
            let related_guarantuees: Vec<&str> =
                n.try_get_attribute("guarantees")?.split(' ').collect();

            let guarantees = related_guarantuees
                .iter()
                .map(|related| {
                    let v: Vec<&str> = related.split('.').collect();
                    let idx: usize = v
                        .last()
                        .ok_or_else(|| ParsingError::Other("Missing guarantees index".to_string()))?
                        .parse()
                        .map_err(|_| {
                            ParsingError::Other(format!(
                                "Cannot parse guarantees index from: {}",
                                related
                            ))
                        })?;
                    guarantees
                        .iter()
                        .find(|g| g.index().eq(&idx))
                        .ok_or_else(|| {
                            ParsingError::WrongIndex("guarantee".to_string(), idx.to_string())
                                .into()
                        })
                        .map(|d| d.clone())
                })
                .collect::<Result<Vec<Arc<Guarantee>>, ConSertError<Demand, RequiredService>>>()?;
            let functional_service_type = get_functional_service_type(n)?;
            Ok(Rc::new(ProvidedService::new(
                n.try_get_attribute("name")?,
                guarantees,
                functional_service_type,
            )))
        })
        .collect()
}

fn parse_required_services(
    doc: &Document,
    demands: &[Arc<Mutex<Demand>>],
) -> Result<RequiredServices, ConSertError<Demand, RequiredService>> {
    #![allow(clippy::unwrap_used)]
    doc.descendants()
        .filter(|n| n.tag_name() == ExpandedName::from("requiredServices"))
        .map(|n| {
            let related_demands: Vec<&str> = n.try_get_attribute("demands")?.split(' ').collect();

            let demands = related_demands
                .iter()
                .map(|related| {
                    let v: Vec<&str> = related.split('.').collect();
                    let idx: usize = v
                        .last()
                        .ok_or_else(|| ParsingError::Other("Cannot demands index".to_string()))?
                        .parse()
                        .map_err(|_| {
                            ParsingError::Other(format!(
                                "Cannot parse demands index from: {}",
                                related
                            ))
                        })?;
                    demands
                        .iter()
                        .find(|d| d.lock().unwrap().index == idx)
                        .ok_or_else(|| {
                            ParsingError::WrongIndex("demand".to_string(), idx.to_string()).into()
                        })
                        .map(|d| d.clone())
                })
                .collect::<Result<Vec<Arc<Mutex<Demand>>>, ConSertError<Demand, RequiredService>>>(
                )?;

            let functional_service_type = get_functional_service_type(n)?;
            Ok(Arc::new(RequiredService::new(
                n.try_get_attribute("name")?,
                demands,
                functional_service_type,
            )))
        })
        .collect()
}
