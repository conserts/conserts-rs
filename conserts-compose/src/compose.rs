// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use conserts_elements::consert::Consert;
use conserts_elements::elements::{demands::Demand, services::RequiredService};
use conserts_error::{CompositionError, ConSertError};
use std::rc::Rc;

// Extension trait for Consert
pub trait Link {
    fn link(&mut self, sos: &SystemOfSystems);
}

// TODO: write test for this function
impl Link for Consert {
    fn link(&mut self, sos: &SystemOfSystems) {
        #![allow(clippy::unwrap_used)]
        for existing_consert in sos.conserts().iter().filter(|c| c.path() != self.path()) {
            for required_service in self.required_services().iter() {
                for provided_service in existing_consert.provided_services().iter() {
                    if required_service.matches_service_type(provided_service) {
                        for guarantee in provided_service.guarantees() {
                            for demand in required_service.demands().iter_mut() {
                                if guarantee.fulfills(demand) {
                                    demand
                                        .lock()
                                        .unwrap()
                                        .link(existing_consert.crate_name(), guarantee.clone())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SystemOfSystems {
    conserts: Vec<Rc<Consert>>,
}

impl SystemOfSystems {
    pub fn new() -> SystemOfSystems {
        SystemOfSystems::default()
    }

    pub(crate) fn conserts(&self) -> Vec<Rc<Consert>> {
        self.conserts.clone()
    }

    #[allow(dead_code)]
    pub fn from_consert(
        consert: Rc<Consert>,
    ) -> Result<SystemOfSystems, ConSertError<Demand, RequiredService>> {
        if consert.is_independent() {
            Ok(SystemOfSystems {
                conserts: vec![consert],
            })
        } else {
            Err(CompositionError::Dependent.into())
        }
    }

    pub(crate) fn can_consert_be_added(
        &self,
        other: &Rc<Consert>,
    ) -> Result<(), ConSertError<Demand, RequiredService>> {
        #![allow(clippy::unwrap_used)]
        let mut unmatched_demands = other.demands();
        let mut unmatched_required_services = other.required_services();

        for existing_consert in self.conserts.iter() {
            for guarantee in existing_consert.guarantees() {
                // Retain those demands that are not fulfilled
                unmatched_demands = unmatched_demands
                    .into_iter()
                    .filter(|demand| !guarantee.fulfills(demand))
                    .collect();
            }

            for provided_service in existing_consert.provided_services().iter() {
                // Retain those services that are not fulfilled
                unmatched_required_services = unmatched_required_services
                    .into_iter()
                    .filter(|service| !service.matches_service_type(provided_service))
                    .collect();
            }
        }

        let composable = unmatched_demands.is_empty() && unmatched_required_services.is_empty();
        if composable {
            Ok(())
        } else {
            Err(CompositionError::Incompatible {
                path: other.path(),
                unmatched_demands,
                unmatched_required_services,
            }
            .into())
        }
    }

    pub fn add_consert(
        &mut self,
        other: Rc<Consert>,
    ) -> Result<(), ConSertError<Demand, RequiredService>> {
        match self.can_consert_be_added(&other) {
            Ok(()) => {
                self.conserts.push(other);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, iter::FromIterator};

    use super::*;
    use conserts_elements::{
        consert_tree::{ConsertTreeElement, Tree},
        dimension::{Dimension, SubsetRelationship},
        guarantees::Guarantee,
    };
    use conserts_parse::Xml;

    fn _generate_conserts() -> (Rc<Consert>, Rc<Consert>) {
        let leader =
            Rc::new(Consert::from_path_xml(&"../models/DEIS_DemoLeaderTruckSystem.model").unwrap());
        let follower = Rc::new(
            Consert::from_path_xml(&"../models/DEIS_DemoFollowerTruckSystem.model").unwrap(),
        );
        (leader, follower)
    }

    #[test]
    fn test_empty_sos_creation() {
        let sos = SystemOfSystems::new();
        assert_eq!(sos.conserts().len(), 0);
    }

    #[test]
    fn test_creation() {
        let (leader, follower) = _generate_conserts();

        let sos = SystemOfSystems::from_consert(follower);
        assert!(
            sos.is_err(),
            "creating a SoS based on a dependent ConSert MUST fail"
        );

        let sos = SystemOfSystems::from_consert(leader);
        assert!(
            sos.is_ok(),
            "creating a SoS based on an independent ConSert MUST succeed"
        );
        //println!("{:#?}", sos);
    }

    #[test]
    fn test_composition() {
        let (leader, follower) = _generate_conserts();

        let mut sos = SystemOfSystems::from_consert(leader).unwrap();

        let result = sos.add_consert(follower);

        assert!(result.is_ok(), "Leader + follower should be composable");
    }

    #[test]
    fn test_composition_fail() {
        let (_, follower) = _generate_conserts();
        let leader = Rc::new(
            Consert::from_path_xml(&"../models/DEIS_DemoLeaderTruckSystemIncompatible.model")
                .unwrap(),
        );

        let mut sos = SystemOfSystems::from_consert(leader).unwrap();

        let result = sos.add_consert(follower);
        assert!(
            result.is_err(),
            "Leader + follower should not be composable"
        );
        if let Err(e) = result {
            if let ConSertError::Composition {
                source:
                    CompositionError::Incompatible {
                        unmatched_demands, ..
                    },
            } = e
            {
                assert!(unmatched_demands.len() == 1);
            } else {
                panic!()
            }
        }
    }

    #[test]
    fn test_composition_categorical_dimensions() {
        use std::sync::{Arc, Mutex};

        // Same SIL Level
        let demand = Arc::new(Mutex::new(Demand::new(
            "D",
            None,
            Dimension::Categorical {
                r#type: "ASIL".into(),
                covered: BTreeSet::from_iter(vec!["ASIL-B".into()]),
                subset: SubsetRelationship::Demand,
            },
        )));
        let guarantee = Guarantee::new(
            0,
            "G",
            None,
            Dimension::Categorical {
                r#type: "ASIL".into(),
                covered: BTreeSet::from_iter(vec!["ASIL-B".into(), "SIL3".into()]),
                subset: SubsetRelationship::Demand,
            },
            Tree::leaf(ConsertTreeElement::Tautology),
        );

        assert!(guarantee.fulfills(&demand));

        // Higher Guarantee than demanded
        let demand = Arc::new(Mutex::new(Demand::new(
            "D",
            None,
            Dimension::Categorical {
                r#type: "ASIL".into(),
                covered: BTreeSet::from_iter(vec!["ASIL-B".into()]),
                subset: SubsetRelationship::Demand,
            },
        )));
        let guarantee = Guarantee::new(
            0,
            "G",
            None,
            Dimension::Categorical {
                r#type: "ASIL".into(),
                covered: BTreeSet::from_iter(vec![
                    "ASIL-D".into(),
                    "ASIL-C".into(),
                    "ASIL-B".into(),
                    "ASIL-A".into(),
                    "SIL3".into(),
                ]),
                subset: SubsetRelationship::Demand,
            },
            Tree::leaf(ConsertTreeElement::Tautology),
        );

        assert!(guarantee.fulfills(&demand));

        // Fail for multiple
        let demand = Arc::new(Mutex::new(Demand::new(
            "D",
            None,
            Dimension::Categorical {
                r#type: "ASIL".into(),
                covered: BTreeSet::from_iter(vec!["ASIL-B".into(), "SIL3".into()]),
                subset: SubsetRelationship::Demand,
            },
        )));
        let guarantee = Guarantee::new(
            0,
            "G",
            None,
            Dimension::Categorical {
                r#type: "ASIL".into(),
                covered: BTreeSet::from_iter(vec!["ASIL-B".into()]),
                subset: SubsetRelationship::Demand,
            },
            Tree::leaf(ConsertTreeElement::Tautology),
        );

        assert!(!guarantee.fulfills(&demand));

        // Sucess if subset relationship is the other way round
        let demand = Arc::new(Mutex::new(Demand::new(
            "D",
            None,
            Dimension::Categorical {
                r#type: "ASIL".into(),
                covered: BTreeSet::from_iter(vec!["ASIL-B".into(), "SIL3".into()]),
                subset: SubsetRelationship::Guarantee,
            },
        )));
        let guarantee = Guarantee::new(
            0,
            "G",
            None,
            Dimension::Categorical {
                r#type: "ASIL".into(),
                covered: BTreeSet::from_iter(vec!["ASIL-B".into()]),
                subset: SubsetRelationship::Guarantee,
            },
            Tree::leaf(ConsertTreeElement::Tautology),
        );

        assert!(guarantee.fulfills(&demand));
    }
}
