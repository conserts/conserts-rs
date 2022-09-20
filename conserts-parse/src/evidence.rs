// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::dimensions::text_to_dimension;
use crate::GetAttribute;
use conserts_elements::elements::evidence::Evidence;
use conserts_elements::elements::{demands::Demand, services::RequiredService};
use conserts_error::ConSertError;
use roxmltree::{self, Document, ExpandedName, Node};
use std::sync::Arc;

pub(super) fn parse(
    doc: &Document,
) -> Result<Vec<Arc<Evidence>>, ConSertError<Demand, RequiredService>> {
    let rtes = collect_nodes(doc);
    let evidence = collect_evidence(&rtes)?;
    Ok(evidence)
}

fn collect_nodes<'a>(doc: &'a Document) -> Vec<Node<'a, 'a>> {
    doc.descendants()
        .filter(|node| node.tag_name() == ExpandedName::from("runtimeEvidence"))
        .collect()
}

fn collect_evidence(
    rtes: &[Node],
) -> Result<Vec<Arc<Evidence>>, ConSertError<Demand, RequiredService>> {
    let evidence = std::iter::empty()
        .chain(collect_rte_evidence(rtes))
        .collect::<Result<Vec<_>, ConSertError<Demand, RequiredService>>>()?;
    Ok(evidence)
}

fn collect_rte_evidence(
    rtes: &[Node],
) -> Vec<Result<Arc<Evidence>, ConSertError<Demand, RequiredService>>> {
    rtes.iter()
        .enumerate()
        .map(|(index, rte)| {
            let name = rte.try_get_attribute("name")?.to_owned();

            let description = rte.try_get_attribute("description")?.to_owned();

            let dimension = text_to_dimension(&description)?;

            Ok(Arc::new(Evidence::new(
                index,
                name,
                Some(description),
                dimension,
            )))
        })
        .collect()
}
