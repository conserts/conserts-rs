// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::compile::TokenStreamJoin;
use conserts_elements::{demands::Demand, elements::evidence::Evidence, services::RequiredService};
use conserts_error::ConSertError;
use proc_macro2::TokenStream;
use std::sync::{Arc, Mutex};

use super::check_logic::CheckLogic;

pub(super) fn render(
    evidence: Vec<Arc<Evidence>>,
    demands: Vec<Arc<Mutex<Demand>>>,
) -> Result<
    impl Iterator<Item = crate::compile::io::CrateFile>,
    ConSertError<Demand, RequiredService>,
> {
    let render_evidence = std::iter::empty()
        .chain(evidence.into_iter().map(|e| e as Arc<dyn CheckLogic>))
        .chain(demands.into_iter().map(|d| d as Arc<dyn CheckLogic>))
        .collect::<Vec<_>>();

    let field_declarations = render_field_declarations(&render_evidence);
    let evaluations = render_evaluations(&render_evidence)?;
    let assignments = render_assignments(&render_evidence);
    Ok(render_evidence_module(
        field_declarations,
        evaluations,
        assignments,
    ))
}

fn render_evidence_module(
    field_declarations: TokenStream,
    evaluations: TokenStream,
    assignments: TokenStream,
) -> impl Iterator<Item = crate::compile::io::CrateFile> {
    std::iter::once((
        std::path::PathBuf::new().join("src/evidence.rs"),
        quote!(
            #![allow(unused_doc_comments)]
            use crate::properties::*;

            #[derive(Debug, Copy, Clone)]
            pub struct RuntimeEvidence {
                #field_declarations
            }

            impl RuntimeEvidence {
                pub fn from(runtime_properties: &RuntimeProperties) -> RuntimeEvidence {
                    #evaluations
                    RuntimeEvidence {
                        #assignments
                    }
                }
            }

            impl core::default::Default for RuntimeEvidence {
                fn default() -> Self {
                    RuntimeEvidence::from(&RuntimeProperties::unknown())
                    }
            }
        )
        .to_string(),
    ))
}

fn render_field_declarations(evidence: &[Arc<dyn CheckLogic>]) -> TokenStream {
    evidence
        .iter()
        .map(|evidence| {
            let field = evidence.field_identifier();
            quote!(
                pub(crate) #field: bool,
            )
        })
        .collect::<Vec<_>>()
        .join()
}

fn render_evaluations(
    evidence: &[Arc<dyn CheckLogic>],
) -> Result<TokenStream, ConSertError<Demand, RequiredService>> {
    Ok(evidence
        .iter()
        .map(|evidence| {
            let field = evidence.field_identifier();
            let check_logic = evidence.check_logic();
            Ok(quote!(
                let #field = #check_logic;
            ))
        })
        .collect::<Result<Vec<_>, ConSertError<Demand, RequiredService>>>()?
        .join())
}

fn render_assignments(evidence: &[Arc<dyn CheckLogic>]) -> TokenStream {
    evidence
        .iter()
        .map(|evidence| {
            let field = evidence.field_identifier();
            quote!(
                #field,
            )
        })
        .collect::<Vec<_>>()
        .join()
}

#[cfg(test)]
mod tests {
    use super::*;
    use conserts_elements::dimension::Dimension;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_render() {
        let evidence = Arc::new(Evidence::new(
            0,
            "FirstEvidence",
            None,
            Dimension::Binary {
                r#type: "PropertyType".into(),
            },
        ));

        let demand = Arc::new(Mutex::new(Demand::new(
            "D0",
            Some("Foo".to_string()),
            Dimension::Binary {
                r#type: "Value".into(),
            },
        )));

        assert_eq!(
            render(vec![evidence], vec![demand])
                .unwrap()
                .next()
                .unwrap(),
            (
                std::path::PathBuf::new().join("src/evidence.rs"),
                quote!(
                    #![allow(unused_doc_comments)]
                    use crate::properties::*;

                    #[derive(Debug, Copy, Clone)]
                    pub struct RuntimeEvidence {
                        pub(crate) first_evidence: bool,
                        pub(crate) d0: bool,
                    }

                    impl RuntimeEvidence {
                        pub fn from(runtime_properties: &RuntimeProperties) -> RuntimeEvidence {
                            let first_evidence = {
                                use crate::properties::FirstEvidence::*;
                                match &runtime_properties.first_evidence {
                                    Unknown => false,
                                    Known(value) => *value,
                                }
                            };
                            let d0 = {
                                use crate::properties::D0::*;
                                #[doc = "Foo"]
                                match &runtime_properties.d0 {
                                    Unknown => false,
                                    Known(value) => *value,
                                }
                            };
                            RuntimeEvidence {
                                first_evidence,
                                d0,
                            }
                        }
                    }

                    impl core::default::Default for RuntimeEvidence {
                        fn default() -> Self {
                            RuntimeEvidence::from(&RuntimeProperties::unknown())
                            }
                    }
                )
                .to_string()
            )
        );
    }
}
