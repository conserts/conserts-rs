// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::elements::demands::Demand;
use crate::elements::evidence::Evidence;
use crate::elements::guarantees::GuaranteePropagation;
use crate::elements::services::RequiredService;
use conserts_error::{ConSertError, ParsingError};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

pub type ConsertTree = Tree<ConsertTreeElement>;

impl Default for ConsertTree {
    fn default() -> Self {
        Tree::leaf(ConsertTreeElement::Tautology)
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Gate {
    pub id: String,
    #[serde(skip)]
    pub index: usize,
    pub function: GateFunction,
}

impl Gate {
    pub fn new(id: String, index: usize, function: GateFunction) -> Self {
        Self {
            id,
            index,
            function,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn function(&self) -> GateFunction {
        self.function
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GateFunction {
    And,
    Or,
}

#[derive(Debug)]
pub struct Tree<T> {
    pub data: Box<TreeNode<T>>,
}

impl<T> Tree<T> {
    pub fn leaf(element: T) -> Self {
        Self {
            data: Box::new(TreeNode {
                element,
                children: vec![],
            }),
        }
    }

    pub fn node(element: T, children: Vec<Tree<T>>) -> Self {
        Self {
            data: Box::new(TreeNode { element, children }),
        }
    }
}

#[derive(Debug)]
pub struct TreeNode<T> {
    pub element: T,
    pub children: Vec<Tree<T>>,
}

pub enum ConsertTreeElement {
    RuntimeEvidence(usize, Arc<Evidence>),
    Demand(usize, Arc<Mutex<Demand>>),
    Gate(String, usize, GateFunction),
    Tautology,
    Contradiction,
}

impl ConsertTreeElement {
    pub fn from_path(
        path: &str,
        runtime_evidence: &[Arc<Evidence>],
        gates: &[Gate],
        demands: &[Arc<Mutex<Demand>>],
    ) -> Result<ConsertTreeElement, ConSertError<Demand, RequiredService>> {
        #![allow(clippy::unwrap_used)]
        let segments: Vec<&str> = path.split('@').collect();
        let last = segments[segments.len() - 1];
        let segments: Vec<&str> = last.split('.').collect();
        let index: usize = segments[1]
            .parse()
            .map_err(|e| ConSertError::from(ParsingError::Integer(e)))?;
        match segments[0] {
            "gates" => {
                let gate = gates.iter().find(|g| g.index == index).ok_or_else(|| {
                    ParsingError::WrongIndex("gate".to_string(), index.to_string())
                })?;
                Ok(ConsertTreeElement::Gate(
                    format!("G{}", index),
                    index,
                    gate.function,
                ))
            }
            "runtimeEvidence" => {
                let rte = runtime_evidence
                    .iter()
                    .find(|rte| rte.index == index)
                    .ok_or_else(|| {
                        ParsingError::WrongIndex("runtime evidence".to_string(), index.to_string())
                    })?;
                Ok(ConsertTreeElement::RuntimeEvidence(index, rte.clone()))
            }
            "demands" => {
                let demand = demands
                    .iter()
                    .find(|demand| demand.lock().unwrap().index == index)
                    .ok_or_else(|| {
                        ParsingError::WrongIndex("demand".to_string(), index.to_string())
                    })?;
                Ok(ConsertTreeElement::Demand(index, demand.clone()))
            }
            t => Err(ParsingError::InvalidConsertTreePath(t.into()).into()),
        }
    }

    pub fn to_path_name(&self) -> Option<String> {
        match self {
            ConsertTreeElement::RuntimeEvidence(index, _) => {
                Some(format!("runtimeEvidence.{}", index))
            }
            ConsertTreeElement::Demand(index, _) => Some(format!("demands.{}", index)),
            ConsertTreeElement::Gate(_, index, _) => Some(format!("gates.{}", index)),
            ConsertTreeElement::Tautology => None,
            ConsertTreeElement::Contradiction => None,
        }
    }

    pub fn id(&self) -> Option<String> {
        #![allow(clippy::unwrap_used)]
        match self {
            ConsertTreeElement::RuntimeEvidence(_, evidence) => Some(evidence.id.clone()),
            ConsertTreeElement::Demand(_, demand) => Some(demand.lock().unwrap().id.clone()),
            ConsertTreeElement::Gate(id, _, _) => Some(id.to_string()),
            ConsertTreeElement::Tautology => None,
            ConsertTreeElement::Contradiction => None,
        }
    }
}

impl From<Gate> for ConsertTreeElement {
    fn from(gate: Gate) -> Self {
        ConsertTreeElement::Gate(gate.id, gate.index, gate.function)
    }
}

impl From<Arc<Evidence>> for ConsertTreeElement {
    fn from(runtime_evidence: Arc<Evidence>) -> Self {
        ConsertTreeElement::RuntimeEvidence(runtime_evidence.index, runtime_evidence)
    }
}

#[cfg(not(tarpaulin_include))] // debug formatting
impl Debug for ConsertTreeElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #![allow(clippy::unwrap_used)]
        match self {
            ConsertTreeElement::RuntimeEvidence(index, evidence) => f
                .debug_struct("ConsertTreeElement::RuntimeEvidence")
                .field("index", index)
                .field("evidence", &evidence.description)
                .finish(),
            ConsertTreeElement::Gate(id, index, function) => f
                .debug_struct("ConsertTreeElement::Gate")
                .field("id", id)
                .field("index", index)
                .field("function", function)
                .finish(),
            ConsertTreeElement::Demand(index, demand) => f
                .debug_struct("ConsertTreeElement::Demand")
                .field("index", index)
                .field("demand", &demand.lock().unwrap().description)
                .finish(),
            ConsertTreeElement::Tautology => {
                f.debug_struct("ConsertTreeElement::Tautology").finish()
            }
            ConsertTreeElement::Contradiction => {
                f.debug_struct("ConsertTreeElement::Contradiction").finish()
            }
        }
    }
}

pub fn grow_cst(
    path: &str,
    propagations: &[GuaranteePropagation],
    runtime_evidence: &[Arc<Evidence>],
    gates: &[Gate],
    demands: &[Arc<Mutex<Demand>>],
) -> Result<ConsertTree, ConSertError<Demand, RequiredService>> {
    let element = ConsertTreeElement::from_path(path, runtime_evidence, gates, demands)?;

    let children = propagations
        .iter()
        .filter(|gp| gp.target_path().eq(path))
        .map(|gp| {
            grow_cst(
                &gp.source_path(),
                propagations,
                runtime_evidence,
                gates,
                demands,
            )
        })
        .collect::<Result<Vec<ConsertTree>, ConSertError<Demand, RequiredService>>>()?;

    Ok(Tree {
        data: Box::new(TreeNode { element, children }),
    })
}
