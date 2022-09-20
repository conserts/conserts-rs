// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::dimension::{Dimension, SubsetRelationship, SubsetResult};
use crate::elements::consert_tree::*;
use crate::elements::demands::Demand;
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Deserialize, Serialize)]
pub struct Guarantee {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(deserialize_with = "ensure_non_empty")]
    pub dimensions: Vec<Dimension>,
    #[serde(skip)]
    pub index: usize,
    #[serde(skip)]
    pub cst: ConsertTree,
}

fn ensure_non_empty<'de, D>(deserializer: D) -> Result<Vec<Dimension>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = Vec::<Dimension>::deserialize(deserializer)?;
    if buf.is_empty() {
        Err(serde::de::Error::custom(
            "A guarantee must have at least one dimension.",
        ))
    } else {
        Ok(buf)
    }
}

impl Guarantee {
    pub fn new<S>(
        index: usize,
        id: S,
        description: Option<String>,
        dimension: Dimension,
        cst: ConsertTree,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: id.into(),
            index,
            description,
            dimensions: vec![dimension],
            cst,
        }
    }

    #[cfg(not(tarpaulin_include))] // getter
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn fulfills(&self, demand: &Arc<Mutex<Demand>>) -> bool {
        #![allow(clippy::unwrap_used)]

        demand
            .lock()
            .unwrap()
            .dimensions
            .iter()
            .all(|demand_dimension| {
                self.dimensions.iter().any(|guarantee_dimension| {
                    if guarantee_dimension.subset().eq(&demand_dimension.subset()) {
                        match guarantee_dimension.subset() {
                            Some(s) => {
                                let (lop, rop) = match s {
                                    SubsetRelationship::Guarantee => {
                                        (guarantee_dimension, demand_dimension)
                                    }
                                    SubsetRelationship::Demand => {
                                        (demand_dimension, guarantee_dimension)
                                    }
                                };
                                match Dimension::subset_of(lop, rop) {
                                    SubsetResult::True => true,
                                    SubsetResult::False | SubsetResult::Incompatible => false,
                                }
                            }
                            None => true,
                        }
                    } else {
                        false
                    }
                })
            })
    }

    pub fn gates_and_guarantee_propagations(&self) -> (Vec<Gate>, Vec<GuaranteePropagation>) {
        #![allow(clippy::unwrap_used)]
        let (gates, mut guarantee_propagations) =
            Guarantee::collect_gates_and_guarantee_propagations(&self.cst);
        let source = self.cst.data.element.id();
        let source_path = self.cst.data.element.to_path_name();
        if let (Some(source), Some(source_path)) = (source, source_path) {
            guarantee_propagations.push(GuaranteePropagation {
                source,
                target: self.id.clone(),
                source_path: format!("@{}", source_path),
                target_path: format!("@guarantees.{}", self.index),
            });
        }
        (gates, guarantee_propagations)
    }

    fn collect_gates_and_guarantee_propagations(
        cst: &ConsertTree,
    ) -> (Vec<Gate>, Vec<GuaranteePropagation>) {
        let (gates, guarantee_propagation): (Vec<_>, Vec<_>) = cst
            .data
            .children
            .iter()
            .map(Guarantee::collect_gates_and_guarantee_propagations)
            .unzip();

        let gates: Vec<Gate> = gates
            .into_iter()
            .flatten()
            .chain(
                match &cst.data.element {
                    ConsertTreeElement::Tautology
                    | ConsertTreeElement::Contradiction
                    | ConsertTreeElement::Demand(_, _)
                    | ConsertTreeElement::RuntimeEvidence(_, _) => vec![],
                    ConsertTreeElement::Gate(id, index, function) => {
                        vec![Gate::new(id.clone(), *index, *function)]
                    }
                }
                .into_iter(),
            )
            .collect();

        let guarantee_propagation: Vec<GuaranteePropagation> = guarantee_propagation
            .into_iter()
            .flatten()
            .chain(
                if let ConsertTreeElement::Gate(id, index, _) = &cst.data.element {
                    cst.data
                        .children
                        .iter()
                        .map(|child| {
                            Self::child_to_guarantee_propagation(child, id.clone(), *index)
                        })
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                },
            )
            .collect();

        (gates, guarantee_propagation)
    }

    fn child_to_guarantee_propagation(
        child: &Tree<ConsertTreeElement>,
        id: String,
        index: usize,
    ) -> GuaranteePropagation {
        #![allow(clippy::unwrap_used)]
        let child_name = child.data.element.id().unwrap();
        let child_path = child.data.element.to_path_name().unwrap();
        // TODO: update base path when multiple configurations are allowed
        GuaranteePropagation {
            source: child_name,
            target: id,
            source_path: format!("@{}", child_path),
            target_path: format!("@gates.{}", index),
        }
    }
}

impl ConsertTreeRoot for Guarantee {
    #[cfg(not(tarpaulin_include))] // getter
    fn identifier(&self) -> String {
        self.id.clone()
    }

    #[cfg(not(tarpaulin_include))] // getter
    fn cst(&self) -> &ConsertTree {
        &self.cst
    }

    #[cfg(not(tarpaulin_include))] // getter
    fn into_rc_cst(self: Arc<Self>) -> Arc<dyn ConsertTreeRoot> {
        self
    }
}

pub trait ConsertTreeRoot {
    fn identifier(&self) -> String;
    fn cst(&self) -> &ConsertTree;
    fn into_rc_cst(self: Arc<Self>) -> Arc<dyn ConsertTreeRoot>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct GuaranteePropagation {
    source: String,
    target: String,
    source_path: String,
    target_path: String,
}

impl GuaranteePropagation {
    pub fn new<S>(source: S, target: S, source_path: S, target_path: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            source: source.into(),
            target: target.into(),
            source_path: source_path.into(),
            target_path: target_path.into(),
        }
    }

    pub fn source(&self) -> String {
        self.source.clone()
    }

    pub fn target(&self) -> String {
        self.target.clone()
    }

    pub fn source_path(&self) -> String {
        self.source_path.clone()
    }

    pub fn target_path(&self) -> String {
        self.target_path.clone()
    }
}

#[cfg(test)]
pub mod tests {
    use std::{collections::BTreeSet, iter::FromIterator};

    use super::*;
    use crate::{evidence::Evidence, numeric_range::NumericRange};
    use pretty_assertions::assert_eq;

    pub fn generate_tree(name: &str) -> Arc<Evidence> {
        Arc::new(Evidence::new(
            0,
            format!("{}Evidence", name),
            Some(format!("{}EvidenceDescription", name)),
            Dimension::Binary {
                r#type: "Property".into(),
            },
        ))
    }

    #[test]
    fn test_collect() {
        let evidence = generate_tree("First");
        let cst = Tree::leaf(ConsertTreeElement::RuntimeEvidence(0, evidence.clone()));
        let guarantee = Arc::new(Guarantee::new(
            0,
            "FirstGuarantee",
            None,
            evidence.dimension.clone(),
            cst,
        ));

        let (gates, guarantees) = guarantee.gates_and_guarantee_propagations();
        assert_eq!(gates, vec![]);
        assert_eq!(
            guarantees,
            vec![GuaranteePropagation::new(
                "FirstEvidence",
                "FirstGuarantee",
                "@runtimeEvidence.0",
                "@guarantees.0"
            )]
        );
    }

    #[test]
    fn test_collect_complex() {
        let evidence1 = generate_tree("First");
        let demand = Arc::new(std::sync::Mutex::new(Demand::new(
            "D0",
            Some("Demand".into()),
            evidence1.dimension.clone(),
        )));

        let cst_1 = Tree::leaf(ConsertTreeElement::RuntimeEvidence(0, evidence1.clone()));
        let cst_2 = Tree::leaf(ConsertTreeElement::RuntimeEvidence(1, evidence1.clone()));
        let cst_3 = Tree::leaf(ConsertTreeElement::Demand(0, demand));
        let cst = Tree::node(
            ConsertTreeElement::Gate("Gate0".into(), 0, GateFunction::And),
            vec![cst_1, cst_2],
        );
        let cst = Tree::node(
            ConsertTreeElement::Gate("Gate1".into(), 1, GateFunction::Or),
            vec![cst, cst_3],
        );

        let guarantee = Arc::new(Guarantee::new(
            0,
            "FirstGuarantee",
            None,
            evidence1.dimension.clone(),
            cst,
        ));
        let (gates, guarantees) = guarantee.gates_and_guarantee_propagations();
        assert_eq!(
            gates,
            vec![
                Gate::new("Gate0".into(), 0, GateFunction::And),
                Gate::new("Gate1".into(), 1, GateFunction::Or)
            ]
        );
        assert_eq!(
            guarantees,
            vec![
                GuaranteePropagation::new(
                    "FirstEvidence",
                    "Gate0",
                    "@runtimeEvidence.0",
                    "@gates.0"
                ),
                GuaranteePropagation::new(
                    "FirstEvidence",
                    "Gate0",
                    "@runtimeEvidence.1",
                    "@gates.0"
                ),
                GuaranteePropagation::new("Gate0", "Gate1", "@gates.0", "@gates.1"),
                GuaranteePropagation::new("D0", "Gate1", "@demands.0", "@gates.1"),
                GuaranteePropagation::new("Gate1", "FirstGuarantee", "@gates.1", "@guarantees.0"),
            ]
        );
    }

    #[test]
    fn test_fulfill_dimensions() {
        let demand = &Arc::new(Mutex::new(Demand {
            id: "D".into(),
            description: None,
            dimensions: vec![
                Dimension::Numeric {
                    r#type: "Speed".into(),
                    covered: vec![NumericRange::Inclusive(0.0..=5.0)],
                    subset: SubsetRelationship::Demand,
                    uom: Some(crate::uom::UnitOfMeasure::new("m/s").unwrap()),
                },
                Dimension::Categorical {
                    r#type: "SIL".into(),
                    covered: BTreeSet::from_iter(["SIL4".into()]),
                    subset: SubsetRelationship::Demand,
                },
            ],
            index: 0,
            linked_guarantees: vec![],
        }));

        let guarantee_no_dimension = Guarantee {
            id: "G".into(),
            description: None,
            dimensions: vec![],
            index: 0,
            cst: Tree::leaf(ConsertTreeElement::Tautology),
        };

        assert!(!guarantee_no_dimension.fulfills(demand));

        let guarantee_one_dimension = Guarantee {
            id: "G".into(),
            description: None,
            dimensions: vec![Dimension::Numeric {
                r#type: "Speed".into(),
                covered: vec![NumericRange::Inclusive(0.0..=5.0)],
                subset: SubsetRelationship::Demand,
                uom: Some(crate::uom::UnitOfMeasure::new("m/s").unwrap()),
            }],
            index: 0,
            cst: Tree::leaf(ConsertTreeElement::Tautology),
        };

        assert!(!guarantee_one_dimension.fulfills(demand));

        let guarantee_wrong_dimension = Guarantee {
            id: "G".into(),
            description: None,
            dimensions: vec![
                Dimension::Numeric {
                    r#type: "Speed".into(),
                    covered: vec![NumericRange::Inclusive(0.0..=5.0)],
                    subset: SubsetRelationship::Demand,
                    uom: Some(crate::uom::UnitOfMeasure::new("m/s").unwrap()),
                },
                Dimension::Categorical {
                    r#type: "SIL".into(),
                    covered: BTreeSet::from_iter(["SIL3".into()]),
                    subset: SubsetRelationship::Demand,
                },
            ],
            index: 0,
            cst: Tree::leaf(ConsertTreeElement::Tautology),
        };

        assert!(!guarantee_wrong_dimension.fulfills(demand));

        let guarantee_matching_dimension = Guarantee {
            id: "G".into(),
            description: None,
            dimensions: vec![
                Dimension::Numeric {
                    r#type: "Speed".into(),
                    covered: vec![NumericRange::Inclusive(0.0..=5.0)],
                    subset: SubsetRelationship::Demand,
                    uom: Some(crate::uom::UnitOfMeasure::new("m/s").unwrap()),
                },
                Dimension::Categorical {
                    r#type: "SIL".into(),
                    covered: BTreeSet::from_iter([
                        "SIL4".into(),
                        "SIL3".into(),
                        "SIL2".into(),
                        "SIL1".into(),
                    ]),
                    subset: SubsetRelationship::Demand,
                },
            ],
            index: 0,
            cst: Tree::leaf(ConsertTreeElement::Tautology),
        };

        assert!(guarantee_matching_dimension.fulfills(demand));

        let guarantee_more_dimension = Guarantee {
            id: "G".into(),
            description: None,
            dimensions: vec![
                Dimension::Numeric {
                    r#type: "Speed".into(),
                    covered: vec![NumericRange::Inclusive(0.0..=5.0)],
                    subset: SubsetRelationship::Demand,
                    uom: Some(crate::uom::UnitOfMeasure::new("m/s").unwrap()),
                },
                Dimension::Categorical {
                    r#type: "SIL".into(),
                    covered: BTreeSet::from_iter([
                        "SIL4".into(),
                        "SIL3".into(),
                        "SIL2".into(),
                        "SIL1".into(),
                    ]),
                    subset: SubsetRelationship::Demand,
                },
                Dimension::Numeric {
                    r#type: "Latency".into(),
                    covered: vec![NumericRange::Inclusive(0.0..=1.0)],
                    subset: SubsetRelationship::Demand,
                    uom: Some(crate::uom::UnitOfMeasure::new("s").unwrap()),
                },
            ],
            index: 0,
            cst: Tree::leaf(ConsertTreeElement::Tautology),
        };

        assert!(guarantee_more_dimension.fulfills(demand));
    }
}
