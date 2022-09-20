// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::dimension::Dimension;
use crate::elements::guarantees::Guarantee;
use crate::elements::services::RequiredService;
use conserts_error::ConSertError;
use inflector::Inflector;
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Demand {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(deserialize_with = "ensure_non_empty")]
    pub dimensions: Vec<Dimension>,
    #[serde(skip)]
    pub index: usize,
    #[serde(skip)]
    pub linked_guarantees: Vec<(String, Arc<Guarantee>)>,
}

fn ensure_non_empty<'de, D>(deserializer: D) -> Result<Vec<Dimension>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = Vec::<Dimension>::deserialize(deserializer)?;
    if buf.is_empty() {
        Err(serde::de::Error::custom(
            "A demand must have at least one dimension.",
        ))
    } else {
        Ok(buf)
    }
}

impl Demand {
    pub fn new<S>(id: S, description: Option<String>, dimension: Dimension) -> Self
    where
        S: Into<String>,
    {
        Self {
            index: 0,
            id: id.into(),
            description,
            linked_guarantees: vec![],
            dimensions: vec![dimension],
        }
    }

    pub fn from_index_and_node(
        index: usize,
        node_name: String,
        dimension: Dimension,
    ) -> Result<Self, ConSertError<Demand, RequiredService>> {
        let description = Some(node_name);

        Ok(Self {
            index,
            id: format!("D{}", index),
            description,
            linked_guarantees: vec![],
            dimensions: vec![dimension],
        })
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn identifier(&self) -> String {
        self.id.to_pascal_case()
    }

    pub fn link(&mut self, crate_name: String, guarantee: Arc<Guarantee>) {
        self.linked_guarantees.push((crate_name, guarantee));
    }

    pub fn guarantees(&self) -> Vec<(String, Arc<Guarantee>)> {
        self.linked_guarantees.clone()
    }
}
