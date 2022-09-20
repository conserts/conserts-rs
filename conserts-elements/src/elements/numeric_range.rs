// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::elements::{demands::Demand, services::RequiredService};
use conserts_error::{ConSertError, ParsingError};
use serde::{Deserialize, Serialize};
use std::ops::{Range, RangeInclusive};

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub enum NumericRange {
    Exclusive(Range<f64>),
    Inclusive(RangeInclusive<f64>),
}

impl NumericRange {
    pub fn from(
        operator: &str,
        threshold: f64,
    ) -> Result<NumericRange, ConSertError<Demand, RequiredService>> {
        match operator {
            "<=" | "bound to" => Ok(NumericRange::Inclusive(0.0..=threshold)),
            ">=" => Ok(NumericRange::Inclusive(threshold..=f64::MAX)),
            "<" => Ok(NumericRange::Exclusive(0.0..threshold)),
            ">" => Ok(NumericRange::Exclusive(threshold..f64::MAX)),
            _ => Err(ParsingError::UnsupportedOperator(operator.to_string()).into()),
        }
    }

    pub fn included_in(&self, other: &NumericRange) -> bool {
        match (self, other) {
            (NumericRange::Exclusive(_s), NumericRange::Exclusive(_o)) => unimplemented!(),
            (NumericRange::Exclusive(_s), NumericRange::Inclusive(_o)) => unimplemented!(),
            (NumericRange::Inclusive(_s), NumericRange::Exclusive(_o)) => unimplemented!(),
            (NumericRange::Inclusive(s), NumericRange::Inclusive(o)) => {
                o.contains(s.start()) && o.contains(s.end())
            }
        }
    }

    pub(crate) fn multiply(&self, f: f64) -> Self {
        match self {
            NumericRange::Exclusive(r) => NumericRange::Exclusive((r.start * f)..(r.end * f)),
            NumericRange::Inclusive(r) => {
                NumericRange::Inclusive(RangeInclusive::new(r.start() * f, r.end() * f))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_from() {
        assert_eq!(
            NumericRange::from("<", 5.7).unwrap(),
            NumericRange::Exclusive(0.0..5.7)
        );
        assert_eq!(
            NumericRange::from(">", 5.7).unwrap(),
            NumericRange::Exclusive(5.7..f64::MAX)
        );
        assert_eq!(
            NumericRange::from("<=", 5.7).unwrap(),
            NumericRange::Inclusive(0.0..=5.7)
        );
        assert_eq!(
            NumericRange::from(">=", 5.7).unwrap(),
            NumericRange::Inclusive(5.7..=f64::MAX)
        );

        let r = NumericRange::from("foo", 5.7).unwrap_err();
        if let ConSertError::Parsing { source } = r {
            assert_eq!(source, ParsingError::UnsupportedOperator("foo".to_string()))
        } else {
            panic!();
        }
    }
}
