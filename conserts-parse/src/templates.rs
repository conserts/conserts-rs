// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use askama::Template;
use conserts_elements::consert::Consert;
use conserts_elements::elements::{consert_tree, evidence, guarantees};
use evidence::Evidence;
use std::borrow::Borrow;

struct GateTemplate {
    index: String,
    function: String,
}

impl<T> From<T> for GateTemplate
where
    T: Borrow<consert_tree::Gate>,
{
    fn from(gate: T) -> Self {
        Self {
            index: format!("{}", gate.borrow().index()),
            function: format!("{:?}", gate.borrow().function()),
        }
    }
}

struct EvidenceTemplate {
    name: String,
    description: String,
}

impl<T> From<T> for EvidenceTemplate
where
    T: AsRef<Evidence>,
{
    fn from(evidence: T) -> Self {
        Self {
            name: evidence.as_ref().id.clone(),
            description: evidence.as_ref().description.clone().unwrap_or_default(),
        }
    }
}

struct GuaranteeTemplate {
    name: String,
    description: String,
}

impl<T> From<T> for GuaranteeTemplate
where
    T: AsRef<guarantees::Guarantee>,
{
    fn from(guarantee: T) -> Self {
        Self {
            name: guarantee.as_ref().id.clone(),
            description: guarantee.as_ref().description.clone().unwrap_or_default(),
        }
    }
}

struct GuaranteePropagationTemplate {
    source: String,
    target: String,
}

impl<T> From<T> for GuaranteePropagationTemplate
where
    T: Borrow<guarantees::GuaranteePropagation>,
{
    fn from(guarantee_propagation: T) -> Self {
        Self {
            source: guarantee_propagation.borrow().source_path(),
            target: guarantee_propagation.borrow().target_path(),
        }
    }
}

#[derive(Template)]
#[template(path = "consert.xml")]
pub(crate) struct ConsertTemplate {
    gates: Vec<GateTemplate>,
    guarantee_propagations: Vec<GuaranteePropagationTemplate>,
    guarantees: Vec<GuaranteeTemplate>,
    evidence: Vec<EvidenceTemplate>,
}

impl<T> From<T> for ConsertTemplate
where
    T: Borrow<Consert>,
{
    fn from(consert: T) -> Self {
        let consert = consert.borrow();

        let (gates, guarantee_propagations): (Vec<_>, Vec<_>) = consert
            .guarantees()
            .iter()
            .map(|g| g.gates_and_guarantee_propagations())
            .unzip();

        let gates = gates
            .into_iter()
            .flatten()
            .map(|g| GateTemplate::from(&g))
            .collect();

        let guarantee_propagations = guarantee_propagations
            .into_iter()
            .flatten()
            .map(|gp| GuaranteePropagationTemplate::from(&gp))
            .collect();

        let guarantees = consert
            .guarantees()
            .iter()
            .map(GuaranteeTemplate::from)
            .collect();

        let evidence = consert
            .evidence()
            .iter()
            .map(EvidenceTemplate::from)
            .collect();

        Self {
            gates,
            guarantee_propagations,
            guarantees,
            evidence,
        }
    }
}
