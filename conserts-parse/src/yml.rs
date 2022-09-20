// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use conserts_elements::{
    consert_tree::{ConsertTree, ConsertTreeElement, Gate, GateFunction, Tree, TreeNode},
    demands::Demand,
    elements,
    evidence::Evidence,
    guarantees::Guarantee,
};
use conserts_error::{ConSertError, ParsingError};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    rc::Rc,
    str::FromStr,
    sync::{Arc, Mutex},
};

fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Ord,
{
    let mut uniq = BTreeSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequiredService {
    pub id: String,
    pub functional_service_type: String,
    pub demands: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProvidedService {
    pub id: String,
    pub functional_service_type: String,
    pub guarantees: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YamlConsertElements {
    pub guarantees: Vec<Arc<Guarantee>>,
    pub evidence: Vec<Arc<Evidence>>,
    pub demands: Vec<Arc<Mutex<Demand>>>,
    pub gates: Vec<Gate>,
    pub tree_propagations: Vec<Propagation>,
    pub required_services: Vec<RequiredService>,
    pub provided_services: Vec<ProvidedService>,
}

impl YamlConsertElements {
    pub(crate) fn evidence(&self) -> Vec<Arc<Evidence>> {
        self.evidence.clone()
    }

    pub(crate) fn demands(&self) -> Vec<Arc<Mutex<Demand>>> {
        self.demands.clone()
    }

    pub(crate) fn guarantees(
        &self,
    ) -> Result<
        Vec<Arc<Guarantee>>,
        ConSertError<
            conserts_elements::demands::Demand,
            conserts_elements::services::RequiredService,
        >,
    > {
        self.guarantees
            .iter()
            .enumerate()
            .map(|(index, g)| {
                Ok(Arc::new(Guarantee {
                    id: g.id.clone(),
                    description: g.description.clone(),
                    dimensions: g.dimensions.clone(),
                    index,
                    cst: grow_cst(&g.id, self)?,
                }))
            })
            .collect()
    }

    pub(crate) fn provided_services(
        &self,
    ) -> Vec<Rc<conserts_elements::services::ProvidedService>> {
        self.provided_services
            .iter()
            .map(|p| {
                Rc::new(conserts_elements::services::ProvidedService {
                    ident: p.id.clone(),
                    guarantees: self
                        .guarantees
                        .iter()
                        .filter(|g| p.guarantees.contains(&g.id))
                        .cloned()
                        .collect(),
                    functional_service_type: p.functional_service_type.clone(),
                })
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn required_services(
        &self,
    ) -> Vec<Arc<conserts_elements::services::RequiredService>> {
        #![allow(clippy::unwrap_used)]
        self.required_services
            .iter()
            .map(|r| {
                Arc::new(conserts_elements::services::RequiredService {
                    ident: r.id.clone(),
                    demands: self
                        .demands
                        .iter()
                        .filter(|d| r.demands.contains(&d.lock().unwrap().id))
                        .cloned()
                        .collect(),
                    functional_service_type: r.functional_service_type.clone(),
                })
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn unique_ids(
        &self,
    ) -> Result<
        (),
        ConSertError<
            conserts_elements::demands::Demand,
            conserts_elements::services::RequiredService,
        >,
    > {
        #![allow(clippy::unwrap_used)]
        let ids = self
            .evidence
            .iter()
            .map(|e| e.id.clone())
            .chain(self.demands.iter().map(|d| d.lock().unwrap().id.clone()))
            .chain(self.guarantees.iter().map(|g| g.id.clone()))
            .chain(self.gates.iter().map(|g| g.id.clone()));
        if has_unique_elements(ids) {
            Ok(())
        } else {
            Err(ParsingError::NonUniqueIds.into())
        }
    }
}

pub fn grow_cst(
    id: &str,
    cse: &YamlConsertElements,
) -> Result<
    ConsertTree,
    ConSertError<conserts_elements::demands::Demand, conserts_elements::services::RequiredService>,
> {
    let element = create_element(id, cse)?;

    let children = cse
        .tree_propagations
        .iter()
        .filter(|gp| gp.to.eq(id))
        .map(|gp| grow_cst(&gp.from, cse))
        .collect::<Result<Vec<ConsertTree>, ConSertError<_, _>>>()?;

    if let ConsertTreeElement::Gate(_, _, gate_function) = element {
        match children.len() {
            0 => match gate_function {
                GateFunction::And => {
                    return Ok(Tree {
                        data: Box::new(TreeNode {
                            element: ConsertTreeElement::Tautology,
                            children: vec![],
                        }),
                    })
                }
                GateFunction::Or => {
                    return Ok(Tree {
                        data: Box::new(TreeNode {
                            element: ConsertTreeElement::Contradiction,
                            children: vec![],
                        }),
                    })
                }
            },
            #[allow(clippy::unwrap_used)]
            1 => return Ok(children.into_iter().next().unwrap()),
            _ => {}
        }
    }

    Ok(Tree {
        data: Box::new(TreeNode { element, children }),
    })
}

pub fn create_element(
    id: &str,
    cse: &YamlConsertElements,
) -> Result<
    ConsertTreeElement,
    ConSertError<conserts_elements::demands::Demand, conserts_elements::services::RequiredService>,
> {
    #![allow(clippy::unwrap_used)]
    if cse.guarantees.iter().any(|g| g.id == id) {
        Ok(ConsertTreeElement::Gate(
            format!("TLG_{}", id),
            0,
            GateFunction::And,
        ))
    } else if let Some((idx, d)) = cse
        .demands
        .iter()
        .enumerate()
        .find(|(_, d)| d.lock().unwrap().id == id)
    {
        Ok(ConsertTreeElement::Demand(idx, d.clone()))
    } else if let Some((idx, g)) = cse.gates.iter().enumerate().find(|(_, g)| g.id == id) {
        Ok(ConsertTreeElement::Gate(id.to_string(), idx, g.function))
    } else if let Some((idx, e)) = cse.evidence.iter().enumerate().find(|(_, e)| e.id == id) {
        Ok(ConsertTreeElement::RuntimeEvidence(idx, e.clone()))
    } else {
        Err(ParsingError::MissingElement(id.to_string()).into())
    }
}

impl FromStr for YamlConsertElements {
    type Err = ConSertError<Demand, elements::services::RequiredService>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cse = serde_yaml::from_str(s)?;
        Ok(cse)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Propagation {
    pub from: String,
    pub to: String,
}

#[cfg(test)]
mod tests {
    use std::ops::RangeInclusive;

    use super::*;
    use conserts_elements::{
        dimension::{Dimension, SubsetRelationship},
        guarantees::Guarantee,
        numeric_range::NumericRange,
        uom::UnitOfMeasure,
    };

    #[test]
    fn test_yml_input() {
        let data = NumericRange::Inclusive(RangeInclusive::new(0.0, 5.0));
        let data = vec![data];
        let demands = vec![Arc::new(Mutex::new(Demand {
            index: 0,
            id: "Latency".into(),
            description: Some("Transmission Latency <= 5ms".into()),
            linked_guarantees: vec![],
            dimensions: vec![Dimension::Numeric {
                r#type: "TransmissionLatency".into(),
                covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 5.0))],
                subset: SubsetRelationship::Guarantee,
                uom: Some(UnitOfMeasure::new("ms").unwrap()),
            }],
        }))];
        let guarantees = vec![
            Arc::new(Guarantee {
                index: 0,
                id: "G_Distance".into(),
                description: Some("Distance is kept".into()),
                dimensions: vec![Dimension::Numeric {
                    r#type: "DistanceIsKept".into(),
                    covered: data,
                    subset: SubsetRelationship::Demand,
                    uom: Some(UnitOfMeasure::new("mm").unwrap()),
                }],
                cst: Default::default(),
            }),
            Arc::new(Guarantee {
                index: 1,
                id: "G_Approved".into(),
                description: Some("Installation Approved".into()),
                dimensions: vec![Dimension::Binary {
                    r#type: "InstallationApproved".into(),
                }],
                cst: Default::default(),
            }),
        ];
        let evidence_dist = Arc::new(Evidence {
            index: 0,
            id: "E_DistanceBound".into(),
            description: Some("Distance <= 50m".into()),
            dimension: Dimension::Numeric {
                r#type: "Distance".into(),
                covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 50.0))],
                subset: SubsetRelationship::Guarantee,
                uom: Some(UnitOfMeasure::new("m").unwrap()),
            },
        });
        let evidence_approved = Arc::new(Evidence {
            index: 1,
            id: "E_Approved".into(),
            description: Some("HSE Approved Setup".into()),
            dimension: Dimension::Binary {
                r#type: "HSEApprovedSetup".into(),
            },
        });
        let evidence_ratio = Arc::new(Evidence {
            id: "Force".into(),
            description: None,
            dimension: Dimension::Numeric {
                r#type: "Force".into(),
                covered: vec![],
                subset: SubsetRelationship::Demand,
                uom: Some(UnitOfMeasure::new("N").unwrap()),
            },
            index: 2,
        });
        let evidence = vec![evidence_dist, evidence_approved, evidence_ratio];
        let gates = vec![];
        let tree_propagations = vec![
            Propagation {
                from: "E_DistanceBound".into(),
                to: "G_Distance".into(),
            },
            Propagation {
                from: "E_Approved".into(),
                to: "G_Distance".into(),
            },
            Propagation {
                from: "Latency".into(),
                to: "G_Distance".into(),
            },
            Propagation {
                from: "E_Approved".into(),
                to: "G_Approved".into(),
            },
            Propagation {
                from: "Force".into(),
                to: "G_Distance".into(),
            },
        ];

        let provided_services = vec![ProvidedService {
            id: "DistanceService".into(),
            functional_service_type: "Distance".into(),
            guarantees: vec!["G_Distance".into()],
        }];

        let required_services = vec![RequiredService {
            id: "LatencyService".into(),
            functional_service_type: "Latency".into(),
            demands: vec!["Latency".into()],
        }];

        let _data = YamlConsertElements {
            guarantees,
            evidence,
            tree_propagations,
            demands,
            gates,
            provided_services,
            required_services,
        };
        /*std::fs::write(
            "../models/FabOS_Scanner.yml",
            serde_yaml::to_string(&data).unwrap(),
        )
        .unwrap();*/

        let content = std::fs::read_to_string("../models/FabOS_Scanner.yml").unwrap();
        let cse: YamlConsertElements = serde_yaml::from_str(&content).unwrap();
        assert!(cse.demands.iter().any(|d| {
            let d = d.lock().unwrap();
            d.id == "Latency"
                && d.dimensions
                    .iter()
                    .any(|dim| dim.subset() == Some(SubsetRelationship::Guarantee))
        }));
        dbg!(cse);
    }
}
