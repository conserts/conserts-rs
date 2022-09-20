// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::{
    demands::Demand,
    dimension::{Dimension, SubsetResult},
    services::RequiredService,
};
use conserts_error::{ConSertError, UnitOfMeasureError};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Evidence {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub dimension: Dimension,
    #[serde(skip)]
    pub index: usize,
}

impl Evidence {
    pub fn new<S>(index: usize, id: S, description: Option<String>, dimension: Dimension) -> Self
    where
        S: Into<String>,
    {
        Self {
            index,
            id: id.into(),
            description,
            dimension,
        }
    }

    pub fn fulfills(&self, other: &Self) -> Result<bool, ConSertError<Demand, RequiredService>> {
        match self.dimension.subset_of(&other.dimension) {
            SubsetResult::True => Ok(true),
            SubsetResult::False => Ok(false),
            SubsetResult::Incompatible => Err(UnitOfMeasureError::Incompatible.into()),
        }
    }
}
