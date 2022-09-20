#![forbid(unsafe_code)]
#![deny(unused_results)]

// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use conserts_elements::consert::Consert;
use conserts_elements::demands::Demand;
use conserts_elements::dimension::{Dimension, SubsetRelationship};
use conserts_elements::numeric_range::NumericRange;
use conserts_elements::services::RequiredService;
use conserts_error::ConSertError;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

pub fn plot(consert: &Consert) -> Result<String, ConSertError<Demand, RequiredService>> {
    let mut s = Vec::new();
    dot::render(&ConsertWrapper::from_consert(consert), &mut s)?;
    let mut s = String::from_utf8(s)?;
    s = s.replace(
        '{',
        "{ rankdir = BT; node [fontsize=16 shape=box fontname=\"Verdana\"];",
    );
    Ok(post_process(&s))
}

// Make guarantees and TLG on the same level
fn post_process(s: &str) -> String {
    let mut guarantee_lines: String = String::from("{rank=same;");
    let mut gates_lines: String = String::from("{rank=same;");
    let mut processed_s: String = String::from("");
    for line in s.split(';') {
        if line.contains("<Guarantee>") {
            guarantee_lines.push_str(line);
            guarantee_lines.push(';');
        } else if line.contains("<TLG>") {
            gates_lines.push_str(line);
            gates_lines.push(';');
        } else if !line.contains('}') {
            processed_s.push_str(line);
            processed_s.push(';');
        }
    }
    let guarantees_gates = format!("{}}}{}}}}}", guarantee_lines, gates_lines);
    processed_s.push_str(&guarantees_gates);
    processed_s
}

pub struct Node {
    pub id: String,
    pub label: String,
}

pub struct Edge {
    pub from: String,
    pub to: String,
}

pub struct ConsertWrapper {
    pub name: String,
    pub nodes: BTreeMap<String, Node>,
    pub edges: BTreeSet<(String, String)>,
}

fn range_to_string(range: &NumericRange) -> String {
    match range {
        NumericRange::Exclusive(r) => format!("{:?}", r),
        NumericRange::Inclusive(r) => format!("{:?}", r),
    }
}

fn subset_to_string(subset: &SubsetRelationship) -> String {
    match subset {
        SubsetRelationship::Guarantee => "G <= D".into(),
        SubsetRelationship::Demand => "D <= G".into(),
    }
}

fn dim_to_string(dimension: &Dimension) -> String {
    match dimension {
        Dimension::Binary { r#type } => r#type.clone(),
        Dimension::Categorical {
            r#type,
            covered,
            subset,
        } => {
            format!(
                "{}\n[{}] ({})",
                r#type.clone(),
                covered.iter().cloned().collect::<Vec<_>>().join(", "),
                subset_to_string(subset),
            )
        }
        Dimension::Numeric {
            r#type: _,
            covered,
            subset,
            uom,
        } => format!(
            "{} : {}\n{} ({})",
            uom.clone().map_or("".to_string(), |q| q.Quantity()),
            uom.clone()
                .map_or("".to_string(), |q| q.get_unit_singular().to_string()),
            covered
                .iter()
                .map(range_to_string)
                .collect::<Vec<_>>()
                .join("\n"),
            subset_to_string(subset),
        ),
    }
}

impl ConsertWrapper {
    fn from_consert(consert: &Consert) -> Self {
        #![allow(clippy::unwrap_used)]
        let nodes = std::iter::empty()
            .chain(consert.evidence().into_iter().map(|e| {
                (
                    e.id.clone(),
                    Node {
                        id: e.id.clone(),
                        label: format!(
                            "<Evidence>\n\n{}\n\n{}",
                            e.description.clone().unwrap_or_else(|| e.id.clone()),
                            dim_to_string(&e.dimension)
                        ),
                    },
                )
            }))
            .chain(consert.guarantees().into_iter().flat_map(|g| {
                let (gates, _) = g.gates_and_guarantee_propagations();
                std::iter::once((
                    g.id.clone(),
                    Node {
                        id: g.id.clone(),
                        label: format!(
                            "<Guarantee>\n\n{}\n\n{}",
                            g.description.clone().unwrap_or_else(|| g.id.clone()),
                            g.dimensions
                                .iter()
                                .map(dim_to_string)
                                .collect::<Vec<_>>()
                                .join("\n\n")
                        ),
                    },
                ))
                .chain(gates.into_iter().map(|gate| {
                    (
                        gate.id.clone(),
                        Node {
                            id: gate.id,
                            label: match gate.function {
                                conserts_elements::consert_tree::GateFunction::And => {
                                    "<Gate>\n&".into()
                                }
                                conserts_elements::consert_tree::GateFunction::Or => {
                                    "<Gate>\n||".into()
                                }
                            },
                        },
                    )
                }))
                .collect::<Vec<_>>()
            }))
            .chain(consert.demands().into_iter().map(|d| {
                let d = d.lock().unwrap();
                let id = d.id.clone();
                (
                    id.clone(),
                    Node {
                        id: id.clone(),
                        label: format!(
                            "<Demand>\n\n{}\n\n{}",
                            d.description.clone().unwrap_or_else(|| id.clone()),
                            d.dimensions
                                .iter()
                                .map(dim_to_string)
                                .collect::<Vec<_>>()
                                .join("\n\n")
                        ),
                    },
                )
            }))
            .collect::<BTreeMap<_, _>>();
        let edges = std::iter::empty()
            .chain(consert.guarantees().into_iter().flat_map(|g| {
                let (_, propagations) = g.gates_and_guarantee_propagations();
                propagations
                    .into_iter()
                    .map(|g| (g.source(), g.target()))
                    .collect::<Vec<_>>()
            }))
            .collect::<BTreeSet<_>>();

        Self {
            name: consert.crate_name(),
            nodes,
            edges,
        }
    }
}

impl<'a> dot::Labeller<'a, String, (String, String)> for ConsertWrapper {
    fn graph_id(&'a self) -> dot::Id<'a> {
        #![allow(clippy::unwrap_used)]
        dot::Id::new(self.name.clone()).unwrap()
    }

    fn node_id(&'a self, n: &String) -> dot::Id<'a> {
        #![allow(clippy::unwrap_used)]
        dot::Id::new(n.to_string()).unwrap()
    }

    fn node_label(&'a self, n: &String) -> dot::LabelText<'a> {
        #![allow(clippy::unwrap_used)]
        dot::LabelText::LabelStr(Cow::Owned(self.nodes.get(n).unwrap().label.clone()))
    }
}

impl<'a> dot::GraphWalk<'a, String, (String, String)> for ConsertWrapper {
    fn nodes(&'a self) -> dot::Nodes<'a, String> {
        let nodes = self
            .nodes
            .iter()
            .map(|n| n.0.to_string())
            .collect::<Vec<_>>();
        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a, (String, String)> {
        let edges: Vec<(String, String)> = self.edges.iter().cloned().collect();
        Cow::Owned(edges)
    }

    fn source(&'a self, edge: &(String, String)) -> String {
        edge.0.clone()
    }

    fn target(&'a self, edge: &(String, String)) -> String {
        edge.1.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conserts_elements::{
        consert::ConsertBuilder,
        consert_tree::{ConsertTreeElement, GateFunction, Tree},
        dimension::Dimension,
        uom::UnitOfMeasure,
    };
    use pretty_assertions::assert_eq;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_generation() {
        let mut consert = ConsertBuilder::new().name("Test").path("test");
        let rte = consert.add_runtime_evidence(
            "RtE",
            None,
            Dimension::Binary {
                r#type: "Type".into(),
            },
        );
        let demand = Arc::new(Mutex::new(Demand::new(
            "Demand",
            Some("First Demand".into()),
            Dimension::Numeric {
                r#type: "Type".into(),
                covered: vec![NumericRange::Inclusive(0.0..=5.0)],
                subset: SubsetRelationship::Demand,
                uom: Some(UnitOfMeasure::new("meter").unwrap()),
            },
        )));
        let consert = consert.add_demand(demand);
        let cst = Tree::leaf(ConsertTreeElement::RuntimeEvidence(0, rte));
        let cst = Tree::node(
            ConsertTreeElement::Gate("Gate1".into(), 0, GateFunction::And),
            vec![cst],
        );
        let consert = consert.add_guarantee(
            "G1",
            None,
            Dimension::Binary {
                r#type: "Type".into(),
            },
            cst,
        );
        let consert = consert.build().unwrap();
        assert_eq!(
            plot(&consert).unwrap(),
            std::fs::read_to_string("../tests/resources/Consert.dot").unwrap()
        );
    }
}
