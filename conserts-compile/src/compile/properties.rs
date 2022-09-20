// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use super::render::RenderProperty;
use crate::compile::TokenStreamJoin;
use conserts_elements::consert::Consert;
use proc_macro2::TokenStream;
use std::sync::Arc;

pub(super) fn render(consert: &Consert) -> impl Iterator<Item = crate::compile::io::CrateFile> {
    let properties = consert
        .evidence()
        .into_iter()
        .map(|evidence| evidence as Arc<dyn RenderProperty>)
        .chain(
            consert
                .demands()
                .into_iter()
                .map(|demand| demand as Arc<dyn RenderProperty>),
        )
        .collect::<Vec<_>>();

    let property_declarations = render_property_declarations(&properties);
    let property_field_declarations = render_property_field_declarations(&properties);
    let unknown_property_inits = render_unknown_property_inits(&properties);
    render_properties_module(
        property_declarations,
        property_field_declarations,
        unknown_property_inits,
    )
}

fn render_properties_module(
    property_declarations: TokenStream,
    property_field_declarations: TokenStream,
    unknown_property_inits: TokenStream,
) -> impl Iterator<Item = crate::compile::io::CrateFile> {
    std::iter::once((
        std::path::PathBuf::new().join("src/properties.rs"),
        quote!(
            #property_declarations

            //#[repr(C)]
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            pub struct RuntimeProperties {
                #property_field_declarations
            }

            impl RuntimeProperties {
                pub fn unknown() -> RuntimeProperties {
                    RuntimeProperties {
                        #unknown_property_inits
                    }
                }
            }
        )
        .to_string(),
    ))
}

fn render_property_declarations(properties: &[Arc<dyn RenderProperty>]) -> TokenStream {
    properties
        .iter()
        .map(|property| {
            let property_declaration = property.render_type_declaration();
            let documentation = property.documentation();
            let documentation = match documentation {
                Some(documentation) => quote!(#[doc = #documentation]),
                None => quote!(),
            };
            quote!(
                //#[repr(C)]
                #documentation
                #[derive(Clone, Copy, Debug, PartialEq)]
                #property_declaration
            )
        })
        .collect::<Vec<_>>()
        .join()
}

fn render_property_field_declarations(properties: &[Arc<dyn RenderProperty>]) -> TokenStream {
    properties
        .iter()
        .map(|property| {
            let (name, t) = property.render_field_declaration();
            quote!(
                pub #name: #t,
            )
        })
        .collect::<Vec<_>>()
        .join()
}

fn render_unknown_property_inits(properties: &[Arc<dyn RenderProperty>]) -> TokenStream {
    properties
        .iter()
        .map(|property| {
            let (name, t) = property.render_field_declaration();
            quote!(
                #name: #t::Unknown,
            )
        })
        .collect::<Vec<_>>()
        .join()
}

#[cfg(test)]
mod tests {
    use super::*;
    use conserts_elements::{
        consert::ConsertBuilder,
        consert_tree::{ConsertTreeElement, Tree},
        dimension::Dimension,
        evidence::Evidence,
        guarantees::Guarantee,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_render() {
        let runtime_evidence = Arc::new(Evidence::new(
            0,
            "FooBar",
            None,
            Dimension::Binary {
                r#type: "Value".into(),
            },
        ));

        let guarantee = Arc::new(Guarantee::new(
            0,
            "Guarantee",
            None,
            Dimension::Binary {
                r#type: "Value".into(),
            },
            Tree::leaf(ConsertTreeElement::Tautology),
        ));

        let consert = ConsertBuilder::new()
            .name("Test")
            .path("Foo")
            .insert_guarantee(guarantee)
            .insert_runtime_evidence(runtime_evidence);

        assert_eq!(
            render(&consert.build().unwrap()).next().unwrap(),
            (
                std::path::PathBuf::new().join("src/properties.rs"),
                quote!(
                    #[derive(Clone, Copy, Debug, PartialEq)]
                    #[derive(Eq)]
                    pub enum FooBar {
                        Unknown,
                        Known(bool),
                    }

                    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
                    pub struct RuntimeProperties {
                        pub foo_bar: FooBar,
                    }

                    impl RuntimeProperties {
                        pub fn unknown() -> RuntimeProperties {
                            RuntimeProperties {
                                foo_bar: FooBar::Unknown,
                            }
                        }
                    }
                )
                .to_string()
            )
        );
    }
}
