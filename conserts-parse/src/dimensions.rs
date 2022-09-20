// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use conserts_elements::dimension::SubsetRelationship;
use conserts_elements::elements::services::RequiredService;
use conserts_elements::elements::uom::UnitOfMeasure;
use conserts_elements::numeric_range::NumericRange;
use conserts_elements::{dimension::Dimension, elements::demands::Demand};
use conserts_error::{ConSertError, ParsingError};
use regex::Regex;

extern crate inflector;
use inflector::Inflector;

static OPERATORS: &[&str] = &["<=", ">=", "<", ">", "bound to"];

pub fn text_to_dimension(text: &str) -> Result<Dimension, ConSertError<Demand, RequiredService>> {
    let (dimension, _asil) = split_asil(text)?;
    if is_numeric_dimension(&dimension) {
        Ok(split_numeric_dimension(&dimension)?)
    } else {
        Ok(Dimension::Binary {
            r#type: dimension.to_pascal_case().replace(' ', ""),
        })
    }
}

pub fn split_asil(
    dimension: &str,
) -> Result<(String, String), ConSertError<Demand, RequiredService>> {
    let parts: Vec<&str> = dimension.split(':').collect();
    let asil = parts.get(1).unwrap_or(&"ASIL UNKNOWN");
    let part = parts
        .get(0)
        .ok_or_else(|| ParsingError::Other("No elements in parts".to_string()))?;

    Ok((part.to_string(), asil.to_string()))
}

pub fn is_numeric_dimension(text: &str) -> bool {
    OPERATORS.iter().any(|op| text.contains(op))
}

pub fn split_numeric_dimension(
    text: &str,
) -> Result<Dimension, ConSertError<Demand, RequiredService>> {
    for &op in OPERATORS {
        if text.contains(op) {
            let parts: Vec<&str> = text.split(op).collect();

            let bound = parts[1].trim();

            let re = Regex::new(r"^([\d\.,]+)[\s]*(.+)$")?;
            let caps = re
                .captures(bound)
                .ok_or_else(|| ParsingError::Other("Regex did not match".to_string()))?;

            let threshold = match caps.get(1) {
                Some(t) => t.as_str(),
                None => return Err(ParsingError::MissingThreshold(bound.to_string()).into()),
            };

            let unit = match caps.get(2) {
                Some(u) => u.as_str(),
                None => return Err(ParsingError::MissingUnit(bound.to_string()).into()),
            };

            let threshold = threshold.parse().map_err(|_| {
                ParsingError::Other(format!("Could not parse threshold: {}", threshold))
            })?;

            let dimension = Dimension::Numeric {
                r#type: parts[0].to_pascal_case().replace(' ', ""),
                covered: vec![NumericRange::from(op, threshold)?],
                subset: SubsetRelationship::Guarantee,
                uom: Some(UnitOfMeasure::new(unit)?),
            };

            return Ok(dimension);
        }
    }
    Err(ParsingError::UnsupportedOperator(text.to_string()).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_to_dimension_test() {
        let test_uom = UnitOfMeasure::new("ms").unwrap();
        let test_dimension = Dimension::Numeric {
            r#type: "CommunicationDelayBoundTo".into(),
            covered: vec![NumericRange::from("<", 300.5).unwrap()],
            subset: SubsetRelationship::Guarantee,
            uom: Some(test_uom),
        };

        // Testing with space
        let test_text1 = "Communication Delay bound to < 300.5 ms";
        let res1 = text_to_dimension(test_text1).unwrap();
        assert_eq!(res1, test_dimension);

        //Testing no space
        let test_text2 = "Communication Delay bound to < 300.5ms";
        let res2 = text_to_dimension(test_text2).unwrap();
        assert_eq!(res2, test_dimension);
    }

    #[test]
    fn test_split_numeric_dimension_fail() {
        let r = split_numeric_dimension("FooBar < 5").unwrap_err();
        if let ConSertError::Parsing { source } = r {
            assert_eq!(
                source,
                ParsingError::Other("Regex did not match".to_string())
            );
        } else {
            panic!();
        }
        let r = split_numeric_dimension("FooBar 5").unwrap_err();
        if let ConSertError::Parsing { source } = r {
            assert_eq!(
                source,
                ParsingError::UnsupportedOperator("FooBar 5".to_string())
            );
        } else {
            panic!();
        }
    }
}
