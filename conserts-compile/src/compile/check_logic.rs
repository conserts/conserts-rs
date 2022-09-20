// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use std::sync::Mutex;

use super::render::{Render, RenderEvidence, RenderProperty};
use crate::compile::TokenStreamJoin;
use conserts_elements::demands::Demand;
use proc_macro2::TokenStream;

pub(crate) trait CheckLogic: RenderEvidence {
    fn check_logic(&self) -> proc_macro2::TokenStream;
}

impl CheckLogic for conserts_elements::elements::evidence::Evidence {
    fn check_logic(&self) -> proc_macro2::TokenStream {
        let documentation = match self.documentation() {
            Some(doc) => quote!(#[doc = #doc]),
            None => quote!(),
        };
        match self.dimension.clone() {
            conserts_elements::dimension::Dimension::Binary { r#type: _ }
            | conserts_elements::dimension::Dimension::Categorical {
                r#type: _,
                covered: _,
                subset: _,
            } => {
                let t = self.type_identifier();
                let field = self.field_identifier();
                quote!(
                    {
                        use crate::properties::#t::*;
                        #documentation
                        match &runtime_properties.#field {
                            Unknown => false,
                            Known(value) => *value,
                        }
                    }
                )
            }
            conserts_elements::dimension::Dimension::Numeric {
                r#type: _,
                covered,
                subset: _,
                uom,
            } => {
                let t = self.type_identifier();
                let field = self.field_identifier();

                match uom {
                    Some(uom) => {
                        let quantity_ident = format_ident!("{}", uom.quantity());
                        let measurement_unit_ident = format_ident!("{}", uom.measurement_unit());
                        let range_check = covered
                            .into_iter()
                            .map(|r| {
                            let tokens = r.render();
                            quote!(
                                #tokens.contains(&value.get::<uom::si::#quantity_ident::#measurement_unit_ident>())
                            )
                            }
                            )
                            .collect::<Vec<TokenStream>>()
                            .join_with(quote!(||));
                        quote!(
                        {
                            use crate::properties::#t::*;
                            #documentation
                            match &runtime_properties.#field {
                                Unknown => false,
                                Known(value) => {
                                    #range_check
                                }
                            }
                        })
                    }
                    None => todo!(),
                }
            }
        }
    }
}

impl CheckLogic for Demand {
    fn check_logic(&self) -> proc_macro2::TokenStream {
        let t = self.type_identifier();
        let field = self.field_identifier();
        let documentation = match self.documentation() {
            Some(doc) => quote!(#[doc = #doc]),
            None => quote!(),
        };
        quote!(
            {
                use crate::properties::#t::*;
                #documentation
                match &runtime_properties.#field {
                    Unknown => false,
                    Known(value) => *value,
                }
            }
        )
    }
}

impl CheckLogic for Mutex<Demand> {
    fn check_logic(&self) -> proc_macro2::TokenStream {
        #![allow(clippy::unwrap_used)]
        self.lock().unwrap().check_logic()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conserts_elements::dimension::{Dimension, SubsetRelationship};
    use conserts_elements::elements::evidence::Evidence;
    use conserts_elements::elements::uom;
    use conserts_elements::numeric_range::NumericRange;
    use pretty_assertions::assert_eq;
    use std::collections::BTreeSet;
    use std::iter::FromIterator;
    use std::ops::RangeInclusive;

    #[test]
    fn test_check_logic_boolean() {
        let evidence = Evidence::new(
            0,
            "BooleanEvidence",
            None,
            Dimension::Binary {
                r#type: "Type".into(),
            },
        );
        assert_eq!(
            evidence.check_logic().to_string(),
            quote!({
                use crate::properties::BooleanEvidence::*;
                match &runtime_properties.boolean_evidence {
                    Unknown => false,
                    Known(value) => *value,
                }
            })
            .to_string()
        );
    }

    #[test]
    fn test_check_logic_categorical() {
        let evidence = Evidence::new(
            0,
            "CategoricalEvidence",
            None,
            Dimension::Categorical {
                r#type: "Type".into(),
                covered: BTreeSet::from_iter(vec!["ASIL-B".into()]),
                subset: SubsetRelationship::Demand,
            },
        );
        assert_eq!(
            evidence.check_logic().to_string(),
            quote!({
                use crate::properties::CategoricalEvidence::*;
                match &runtime_properties.categorical_evidence {
                    Unknown => false,
                    Known(value) => *value,
                }
            })
            .to_string()
        );
    }

    #[test]
    fn test_check_logic_numeric() {
        let evidence = Evidence::new(
            0,
            "NumericEvidence",
            None,
            Dimension::Numeric {
                r#type: "Type".into(),
                covered: vec![
                    NumericRange::Inclusive(RangeInclusive::new(0.0, 40.0)),
                    NumericRange::Inclusive(RangeInclusive::new(50.0, 60.0)),
                ],
                subset: SubsetRelationship::Demand,
                uom: Some(uom::UnitOfMeasure::new("ms").unwrap()),
            },
        );
        assert_eq!(
            evidence.check_logic().to_string(),
            quote!({
                use crate::properties::NumericEvidence::*;
                match &runtime_properties.numeric_evidence {
                    Unknown => false,
                    Known(value) => {
                        (0f64..=40f64).contains(&value.get::<uom::si::time::millisecond>())
                            || (50f64..=60f64).contains(&value.get::<uom::si::time::millisecond>())
                    }
                }
            })
            .to_string()
        );
    }
    #[test]
    fn test_check_logic_demand() {
        let evidence = Evidence::new(
            0,
            "DemandEvidence",
            None,
            Dimension::Binary {
                r#type: "Type".into(),
            },
        );
        assert_eq!(
            evidence.check_logic().to_string(),
            quote!({
                use crate::properties::DemandEvidence::*;
                match &runtime_properties.demand_evidence {
                    Unknown => false,
                    Known(value) => *value,
                }
            })
            .to_string()
        );
    }
}
