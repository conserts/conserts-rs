// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::dimensions::text_to_dimension;
use crate::{
    elements::{demands::Demand, services::RequiredService},
    get_refinement, GetAttribute,
};
use conserts_error::ConSertError;
use std::sync::{Arc, Mutex};

pub(super) fn parse(
    doc: &roxmltree::Document,
) -> Result<Vec<Arc<Mutex<Demand>>>, ConSertError<Demand, RequiredService>> {
    doc.descendants()
        .filter(|node| node.tag_name() == roxmltree::ExpandedName::from("demands"))
        .enumerate()
        .map(|(index, node)| {
            let refinement = get_refinement(&node)?;
            Ok(Arc::new(Mutex::new(Demand::from_index_and_node(
                index,
                node.try_get_attribute("name")?.to_string(),
                text_to_dimension(&refinement)?,
            )?)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::ops::RangeInclusive;

    use super::*;
    use conserts_elements::dimension::{Dimension, SubsetRelationship};
    use conserts_elements::elements::numeric_range::NumericRange;
    use conserts_elements::elements::uom::UnitOfMeasure;

    #[test]
    fn test_parse() {
        let model =
            &std::fs::read_to_string("../models/DEIS_DemoFollowerTruckSystem.model").unwrap();
        let doc = roxmltree::Document::parse(model).unwrap();
        let demand = doc
            .descendants()
            .find(|node| node.tag_name() == roxmltree::ExpandedName::from("demands"))
            .unwrap();

        match Demand::from_index_and_node(
            0,
            demand.try_get_attribute("name").unwrap().to_string(),
            text_to_dimension(&get_refinement(&demand).unwrap()).unwrap(),
        ) {
            Ok(d) => {
                assert_eq!(d.index, 0);
                assert_eq!(d.identifier(), "D0");
                assert_eq!(d.description, Some("SD1".to_string()));
                assert_eq!(
                    d.dimensions,
                    vec![Dimension::Numeric {
                        r#type: "SpeedDeviationIsBoundTo".into(),
                        covered: vec![NumericRange::Inclusive(RangeInclusive::new(0.0, 2.0))],
                        subset: SubsetRelationship::Guarantee,
                        uom: Some(UnitOfMeasure::new("km/h").unwrap()),
                    }]
                )
            }
            Err(e) => panic!("{:#?}", e),
        }
    }
}
