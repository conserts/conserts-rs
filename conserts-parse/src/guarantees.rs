// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use super::get_refinement;
use crate::dimensions::text_to_dimension;
use crate::GetAttribute;
use conserts_elements::elements::consert_tree::*;
use conserts_elements::elements::demands::Demand;
use conserts_elements::elements::evidence::Evidence;
use conserts_elements::elements::guarantees::{Guarantee, GuaranteePropagation};
use conserts_elements::elements::services::RequiredService;
use conserts_error::{ConSertError, ParsingError};
use roxmltree::{Document, ExpandedName};
use std::sync::{Arc, Mutex};

pub(super) fn parse_guarantees(
    doc: &Document,
    runtime_evidence: &[Arc<Evidence>],
    guarantee_propagations: &[GuaranteePropagation],
    gates: &[Gate],
    demands: &[Arc<Mutex<Demand>>],
) -> Result<Vec<Arc<Guarantee>>, ConSertError<Demand, RequiredService>> {
    doc.descendants()
        .filter(|n| n.tag_name() == ExpandedName::from("guarantees"))
        .enumerate()
        .map(|(index, g)| {
            let path = format!("guarantees.{}", index);
            let ident = g.try_get_attribute("name")?;
            let cst = extract_cst(
                guarantee_propagations,
                &path,
                runtime_evidence,
                gates,
                demands,
            )?;

            let refinement = get_refinement(&g)?;

            Ok(Arc::new(Guarantee::new(
                index,
                ident,
                Some(refinement.clone()),
                text_to_dimension(&refinement)?,
                cst,
            )))
        })
        .collect()
}

pub(super) fn parse_guarantee_propagations(
    doc: &Document,
) -> Result<Vec<GuaranteePropagation>, ConSertError<Demand, RequiredService>> {
    doc.descendants()
        .filter(|n| n.tag_name() == ExpandedName::from("guaranteePropagations"))
        .map(|n| {
            Ok(GuaranteePropagation::new(
                "",
                "",
                n.try_get_attribute("sourceElement")?,
                n.try_get_attribute("targetElement")?,
            ))
        })
        .collect()
}

// Consider a test
pub(super) fn parse_gates(
    doc: &Document,
) -> Result<Vec<Gate>, ConSertError<Demand, RequiredService>> {
    doc.descendants()
        .filter(|n| n.tag_name() == ExpandedName::from("gates"))
        .enumerate()
        .map(|(index, node)| {
            let function = match &(node.try_get_attribute("gateType")?.to_lowercase())[..] {
                "and" => GateFunction::And,
                "or" => GateFunction::Or,
                function => return Err(ParsingError::UnsupportedGate(function.to_string()).into()),
            };
            Ok(Gate::new(format!("Gate{}", index), index, function))
        })
        .collect()
}

fn extract_cst(
    guarantee_propagations: &[GuaranteePropagation],
    path: &str,
    runtime_evidence: &[Arc<Evidence>],
    gates: &[Gate],
    demands: &[Arc<Mutex<Demand>>],
) -> Result<ConsertTree, ConSertError<Demand, RequiredService>> {
    let root = guarantee_propagations
        .iter()
        .find(|gp| gp.target_path().ends_with(&path));

    match root {
        Some(root) => grow_cst(
            &root.source_path(),
            guarantee_propagations,
            runtime_evidence,
            gates,
            demands,
        ),
        None => Ok(Tree {
            data: Box::new(TreeNode {
                element: ConsertTreeElement::Tautology,
                children: vec![],
            }),
        }),
    }
}
