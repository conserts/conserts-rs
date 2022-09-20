// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::demands::Demand;
use crate::uom::UnitOfMeasure;
use crate::{elements::numeric_range::NumericRange, services::RequiredService};
use conserts_error::{ConSertError, UnitOfMeasureError};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Dimension {
    Binary {
        r#type: String,
    },
    Categorical {
        r#type: String,
        #[serde(deserialize_with = "ensure_non_empty_categorical")]
        covered: BTreeSet<String>,
        subset: SubsetRelationship,
    },
    Numeric {
        r#type: String,
        #[serde(deserialize_with = "ensure_non_empty_numeric")]
        covered: Vec<NumericRange>,
        subset: SubsetRelationship,
        #[serde(skip_serializing_if = "Option::is_none")]
        uom: Option<UnitOfMeasure>,
    },
}

fn ensure_non_empty_categorical<'de, D>(deserializer: D) -> Result<BTreeSet<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = BTreeSet::<String>::deserialize(deserializer)?;
    if buf.is_empty() {
        Err(serde::de::Error::custom(
            "A covered set of categories must be non-empty.",
        ))
    } else {
        Ok(buf)
    }
}

fn ensure_non_empty_numeric<'de, D>(deserializer: D) -> Result<Vec<NumericRange>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = Vec::<NumericRange>::deserialize(deserializer)?;
    if buf.is_empty() {
        Err(serde::de::Error::custom(
            "A covered set of ranges must be non-empty.",
        ))
    } else {
        Ok(buf)
    }
}

impl Dimension {
    pub fn subset(&self) -> Option<SubsetRelationship> {
        match self {
            Dimension::Binary { r#type: _ } => None,
            Dimension::Categorical {
                r#type: _,
                covered: _,
                subset,
            }
            | Dimension::Numeric {
                r#type: _,
                covered: _,
                subset,
                uom: _,
            } => Some(*subset),
        }
    }

    pub fn subset_of(&self, other: &Dimension) -> SubsetResult {
        match (self, other) {
            (Dimension::Binary { r#type: l_type }, Dimension::Binary { r#type: r_type }) => {
                if l_type == r_type {
                    SubsetResult::True
                } else {
                    SubsetResult::Incompatible
                }
            }
            (
                Dimension::Categorical {
                    r#type: l_type,
                    covered,
                    subset: _,
                },
                Dimension::Categorical {
                    r#type: r_type,
                    covered: o_covered,
                    subset: _,
                },
            ) => {
                if l_type == r_type {
                    match covered.iter().all(|s| o_covered.contains(s)) {
                        true => SubsetResult::True,
                        false => SubsetResult::False,
                    }
                } else {
                    SubsetResult::Incompatible
                }
            }
            (
                Dimension::Numeric {
                    r#type: l_type,
                    covered,
                    subset: _,
                    uom,
                },
                Dimension::Numeric {
                    r#type: r_type,
                    covered: o_covered,
                    subset: _,
                    uom: o_uom,
                },
            ) => {
                if l_type == r_type {
                    if let Ok(true) = Self::compatible(uom, o_uom) {
                        let f = uom.clone().map(|uom| uom.factor()).unwrap_or_else(|| 1.0);
                        let o_f = o_uom.clone().map(|uom| uom.factor()).unwrap_or_else(|| 1.0);

                        match covered.iter().all(|r| {
                            o_covered
                                .iter()
                                .any(|o_r| r.multiply(f).included_in(&o_r.multiply(o_f)))
                        }) {
                            true => SubsetResult::True,
                            false => SubsetResult::False,
                        }
                    } else {
                        SubsetResult::Incompatible
                    }
                } else {
                    SubsetResult::Incompatible
                }
            }
            _ => SubsetResult::Incompatible,
        }
    }

    fn compatible(
        uom: &Option<UnitOfMeasure>,
        o_uom: &Option<UnitOfMeasure>,
    ) -> Result<bool, ConSertError<Demand, RequiredService>> {
        match (uom, o_uom) {
            (None, None) => Ok(true),
            (Some(uom), Some(o_uom)) => uom.compatible(o_uom),
            _ => Err(UnitOfMeasureError::Incompatible.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SubsetRelationship {
    Guarantee,
    Demand,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SubsetResult {
    True,
    False,
    Incompatible,
}

#[cfg(test)]
mod tests {
    use std::ops::RangeInclusive;

    use crate::{
        dimension::{SubsetRelationship, SubsetResult},
        numeric_range::NumericRange,
        uom::UnitOfMeasure,
    };

    use super::Dimension;

    #[test]
    fn test_subset_no_uom() {
        let d1 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 5.0))],
            subset: SubsetRelationship::Demand,
            uom: None,
        };
        let d2 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 10.0))],
            subset: SubsetRelationship::Demand,
            uom: None,
        };
        assert_eq!(d1.subset_of(&d2), SubsetResult::True);
    }

    #[test]
    fn test_subset_compatible_uom() {
        let d1 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 5.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("mm").unwrap()),
        };
        let d2 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 10.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("mm").unwrap()),
        };
        assert_eq!(d1.subset_of(&d2), SubsetResult::True);
    }
    #[test]
    fn test_subset_compatible_uom_factors() {
        let d1_ms_1000 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 1000.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("ms").unwrap()),
        };
        let d1_ms_999 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 999.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("ms").unwrap()),
        };
        let d1_ms_10000 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 10000.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("ms").unwrap()),
        };
        let d2_s = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 1.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("s").unwrap()),
        };
        let d2_s_9 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 9.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("s").unwrap()),
        };
        let d2_s_10 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 10.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("s").unwrap()),
        };
        assert_eq!(d2_s.subset_of(&d1_ms_1000), SubsetResult::True);
        assert_eq!(d1_ms_10000.subset_of(&d2_s_10), SubsetResult::True);
        assert_eq!(d2_s.subset_of(&d1_ms_999), SubsetResult::False);
        assert_eq!(d1_ms_10000.subset_of(&d2_s_9), SubsetResult::False);
    }

    #[test]
    fn test_non_subset_compatible_uom() {
        let d1 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 5.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("m").unwrap()),
        };
        let d2 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 10.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("mm").unwrap()),
        };
        assert_eq!(d1.subset_of(&d2), SubsetResult::False);
    }

    #[test]
    fn test_incompatible_uom() {
        let d1 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 5.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("m").unwrap()),
        };
        let d2 = Dimension::Numeric {
            r#type: "Value".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 10.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("m/s").unwrap()),
        };
        assert_eq!(d1.subset_of(&d2), SubsetResult::Incompatible);
    }

    #[test]
    fn test_different_types() {
        let d1 = Dimension::Numeric {
            r#type: "FrontDistance".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 5.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("m").unwrap()),
        };
        let d2 = Dimension::Numeric {
            r#type: "BackDistance".into(),
            covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 10.0))],
            subset: SubsetRelationship::Demand,
            uom: Some(UnitOfMeasure::new("m").unwrap()),
        };
        assert_eq!(d1.subset_of(&d2), SubsetResult::Incompatible);
    }
}
