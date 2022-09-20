// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use conserts_elements::{
    demands::Demand, dimension::Dimension, evidence::Evidence, guarantees::Guarantee,
    numeric_range::NumericRange,
};
use proc_macro2::{Ident, TokenStream};
use std::{ops::Range, sync::Mutex};

extern crate inflector;
use inflector::Inflector;

pub(crate) trait Render {
    fn render(&self) -> proc_macro2::TokenStream;
}

impl Render for NumericRange {
    fn render(&self) -> proc_macro2::TokenStream {
        match &self {
            NumericRange::Exclusive(Range { start, end }) => quote!(
                (#start..#end)
            ),
            NumericRange::Inclusive(range) => {
                let start = range.start();
                let end = range.end();
                quote!(
                    (#start..=#end)
                )
            }
        }
    }
}

pub(crate) trait RenderEvidence {
    fn type_identifier(&self) -> Ident;
    fn field_identifier(&self) -> Ident;
}

impl RenderEvidence for Evidence {
    fn type_identifier(&self) -> Ident {
        format_ident!("{}", self.id.to_pascal_case())
    }

    fn field_identifier(&self) -> Ident {
        format_ident!("{}", self.id.to_snake_case())
    }
}

impl RenderEvidence for Guarantee {
    fn type_identifier(&self) -> Ident {
        format_ident!("{}", self.id.to_pascal_case())
    }

    fn field_identifier(&self) -> Ident {
        format_ident!("{}", self.id.to_snake_case())
    }
}

impl RenderEvidence for Demand {
    fn type_identifier(&self) -> Ident {
        format_ident!("{}", self.id.to_pascal_case())
    }

    fn field_identifier(&self) -> Ident {
        format_ident!("{}", self.id.to_snake_case())
    }
}

impl RenderEvidence for Mutex<Demand> {
    fn type_identifier(&self) -> Ident {
        #![allow(clippy::unwrap_used)]
        self.lock().unwrap().type_identifier()
    }

    fn field_identifier(&self) -> Ident {
        #![allow(clippy::unwrap_used)]
        self.lock().unwrap().field_identifier()
    }
}

pub(crate) trait RenderProperty {
    fn render_type_declaration(&self) -> TokenStream;
    fn render_field_declaration(&self) -> (Ident, Ident);
    fn documentation(&self) -> Option<String>;
}

impl RenderProperty for Evidence {
    fn render_type_declaration(&self) -> TokenStream {
        match &self.dimension {
            Dimension::Binary { r#type: _ } => {
                let property_ident = format_ident!("{}", self.id.replace(' ', "").to_pascal_case());
                quote!(
                    #[derive(Eq)]
                    pub enum #property_ident {
                        Unknown,
                        Known(bool),
                    }
                )
            }
            Dimension::Categorical {
                r#type: _,
                covered,
                subset: _,
            } => {
                let property_ident = format_ident!("{}", self.id.replace(' ', "").to_pascal_case());
                let covered_ident =
                    format_ident!("{}", self.id.replace(' ', "").to_screaming_snake_case());

                quote!(
                    #[derive(Eq)]
                    pub enum #property_ident {
                        Unknown,
                        Known(bool)
                    }
                    const #covered_ident: &[&str] = &[#(#covered),*];
                    impl #property_ident {
                        pub fn from_str(value: &str) -> Self {
                            Self::Known(#covered_ident.contains(&value))
                        }
                    }
                )
            }
            Dimension::Numeric {
                r#type: _,
                covered: _,
                subset: _,
                uom,
            } => match uom {
                Some(uom) => {
                    let property_ident =
                        format_ident!("{}", self.id.replace(' ', "").to_pascal_case());
                    let quantity_ident = format_ident!("{}", uom.Quantity());
                    quote!(
                        pub enum #property_ident {
                            Unknown,
                            Known(uom::si::f64::#quantity_ident),
                        }
                        impl Eq for #property_ident { }
                    )
                }
                None => {
                    let property_ident =
                        format_ident!("{}", self.id.replace(' ', "").to_pascal_case());
                    quote!(
                        pub enum #property_ident {
                            Unknown,
                            Known(bool),
                        }
                    )
                }
            },
        }
    }

    fn render_field_declaration(&self) -> (Ident, Ident) {
        let name = format_ident!("{}", self.id.replace(' ', "").to_snake_case());
        let t = format_ident!("{}", self.id.replace(' ', "").to_pascal_case());
        (name, t)
    }

    fn documentation(&self) -> Option<String> {
        self.description.clone()
    }
}

impl RenderProperty for Demand {
    fn render_type_declaration(&self) -> TokenStream {
        let property_ident = format_ident!("{}", self.id.replace(' ', "").to_pascal_case());
        quote!(
            #[derive(Eq)]
            pub enum #property_ident {
                Unknown,
                Known(bool),
            }
        )
    }

    fn render_field_declaration(&self) -> (Ident, Ident) {
        let name = format_ident!("{}", self.id.replace(' ', "").to_snake_case());
        let t = format_ident!("{}", self.id.replace(' ', "").to_pascal_case());
        (name, t)
    }

    fn documentation(&self) -> Option<String> {
        self.description.clone()
    }
}

impl RenderProperty for Mutex<Demand> {
    fn render_type_declaration(&self) -> TokenStream {
        #![allow(clippy::unwrap_used)]
        self.lock().unwrap().render_type_declaration()
    }

    fn render_field_declaration(&self) -> (Ident, Ident) {
        #![allow(clippy::unwrap_used)]
        self.lock().unwrap().render_field_declaration()
    }

    fn documentation(&self) -> Option<String> {
        #![allow(clippy::unwrap_used)]
        self.lock().unwrap().documentation()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conserts_elements::dimension::{Dimension, SubsetRelationship};
    use conserts_elements::numeric_range::NumericRange;
    use conserts_elements::uom::UnitOfMeasure;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_numeric_range() {
        let r = NumericRange::Inclusive(0.0..=5.7);
        assert_eq!(r.render().to_string(), quote!((0f64..=5.7f64)).to_string());
        let r = NumericRange::Exclusive(0.0..5.7);
        assert_eq!(r.render().to_string(), quote!((0f64..5.7f64)).to_string());
    }

    #[test]
    fn test_demand() {
        let dimension = Dimension::Numeric {
            r#type: "Type".into(),
            covered: vec![],
            subset: SubsetRelationship::Demand,
            uom: None,
        };

        let d = Demand::from_index_and_node(0, "SD1".into(), dimension).unwrap();
        assert_eq!(d.type_identifier(), format_ident!("D0"));
        assert_eq!(d.field_identifier(), format_ident!("d0"));
    }

    #[test]
    fn test_property_render_field_declaration() {
        let runtime_evidence = Evidence::new(
            0,
            "FooBar",
            None,
            Dimension::Binary {
                r#type: "Value".into(),
            },
        );
        let (name, t) = runtime_evidence.render_field_declaration();
        assert_eq!(name, format_ident!("foo_bar"),);
        assert_eq!(t, format_ident!("FooBar"),);

        let demand = Demand::new(
            "D0",
            None,
            Dimension::Binary {
                r#type: "Value".into(),
            },
        );

        let (name, t) = demand.render_field_declaration();
        assert_eq!(name, format_ident!("d0"),);
        assert_eq!(t, format_ident!("D0"),);
    }

    #[test]
    fn test_property_render_type_declaration() {
        let runtime_evidence = Evidence::new(
            0,
            "FooBar",
            None,
            Dimension::Binary {
                r#type: "Value".into(),
            },
        );
        let declaration = runtime_evidence.render_type_declaration();
        assert_eq!(
            declaration.to_string(),
            quote!(
                #[derive(Eq)]
                pub enum FooBar {
                    Unknown,
                    Known(bool),
                }
            )
            .to_string()
        );
        let runtime_evidence_numeric = Evidence::new(
            0,
            "FooBar",
            None,
            Dimension::Numeric {
                r#type: "Distance".into(),
                covered: vec![],
                subset: SubsetRelationship::Demand,
                uom: Some(UnitOfMeasure::new("km").unwrap()),
            },
        );
        let declaration = runtime_evidence_numeric.render_type_declaration();
        assert_eq!(
            declaration.to_string(),
            quote!(
                pub enum FooBar {
                    Unknown,
                    Known(uom::si::f64::Length),
                }
                impl Eq for FooBar {}
            )
            .to_string()
        );

        let demand = Demand::new(
            "D0",
            None,
            Dimension::Binary {
                r#type: "Value".into(),
            },
        );
        let declaration = demand.render_type_declaration();
        assert_eq!(
            declaration.to_string(),
            quote!(
                #[derive(Eq)]
                pub enum D0 {
                    Unknown,
                    Known(bool),
                }
            )
            .to_string()
        )
    }
}
