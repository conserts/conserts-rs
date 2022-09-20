// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::compile::render::RenderEvidence;
use crate::compile::TokenStreamJoin;
use conserts_elements::elements::consert_tree::*;
use conserts_elements::elements::guarantees::{ConsertTreeRoot, Guarantee};
use proc_macro2::TokenStream;
use std::sync::Arc;

extern crate inflector;
use inflector::Inflector;

pub(super) fn render(
    guarantees: &[Arc<Guarantee>],
    failure: bool,
) -> impl Iterator<Item = crate::compile::io::CrateFile> {
    let csts = collect_csts(guarantees);
    let guarantees = render_guarantees(csts, failure);
    render_guarantees_module(guarantees, failure)
}

fn render_guarantees_module(
    guarantees: TokenStream,
    failure: bool,
) -> impl Iterator<Item = crate::compile::io::CrateFile> {
    let additional_code = if failure {
        quote!(
            use crate::Failure;
            use std::collections::BTreeSet;
        )
    } else {
        quote!()
    };
    std::iter::once((
        std::path::PathBuf::new().join("src/guarantees.rs"),
        quote!(
            #![allow(unused_doc_comments)]
            #additional_code
            use super::evidence::RuntimeEvidence;

            #guarantees
        )
        .to_string(),
    ))
}

fn collect_csts(
    guarantees: &[Arc<Guarantee>],
) -> impl Iterator<Item = (Option<String>, Arc<dyn ConsertTreeRoot>)> + '_ {
    guarantees
        .iter()
        .map(|g| (g.description.clone(), g.clone().into_rc_cst()))
}

fn render_guarantees<I>(csts: I, failure: bool) -> TokenStream
where
    I: Iterator<Item = (Option<String>, Arc<dyn ConsertTreeRoot>)>,
{
    csts.map(|(doc, cst_root)| render_cst_root(doc, cst_root, failure))
        .collect::<Vec<_>>()
        .join()
}

fn render_cst_root(
    doc: Option<String>,
    cst_root: Arc<dyn ConsertTreeRoot>,
    failure: bool,
) -> TokenStream {
    let cst_top = cst_root.cst();
    let (description, cst) = render_cst(cst_top);
    let identifier = format_ident!("{}", cst_root.identifier().to_pascal_case());
    let rte_identifier = format_ident!(
        "{}",
        match cst_top.data.element {
            ConsertTreeElement::Tautology => "_runtime_evidence",
            _ => "runtime_evidence",
        }
    );
    let documentation = match doc {
        Some(doc) => quote!(#[doc = #doc]),
        None => quote!(),
    };
    let failure_code = if failure {
        let identifier_str = identifier.to_string();
        let rtes = render_cst_failure(cst_top);
        quote!(
            pub fn failure(#rte_identifier: &RuntimeEvidence) -> Failure {
                let mut failures = BTreeSet::new();
                #rtes
                Failure {
                    description: #identifier_str.to_string(),
                    inner: failures,
                }
            }
        )
    } else {
        quote!()
    };
    quote!(
        #documentation
        pub struct #identifier;
        impl #identifier {
            pub fn evaluate(#rte_identifier: &RuntimeEvidence) -> bool {
                #description
                #cst
            }
            #failure_code
        }
    )
}

fn render_cst_failure(cst: &ConsertTree) -> TokenStream {
    #![allow(clippy::unwrap_used)]
    let node = &cst.data;
    let (element, rest) = match &node.element {
        ConsertTreeElement::RuntimeEvidence(_, evidence) => {
            let field = evidence.field_identifier();
            let description = match evidence.description.clone() {
                Some(description) => description,
                None => evidence.id.clone(),
            };
            (Some((field, description)), quote!())
        }
        ConsertTreeElement::Demand(_, demand) => {
            let demand = demand.lock().unwrap();
            let description = demand.description.clone();
            let description = match description {
                Some(description) => description,
                None => demand.id.clone(),
            };
            let field = demand.field_identifier();
            (Some((field, description)), quote!())
        }
        ConsertTreeElement::Tautology => (None, quote!()),
        ConsertTreeElement::Contradiction => (None, quote!()),
        ConsertTreeElement::Gate(_, _, _) => {
            let subtrees = node
                .children
                .iter()
                .map(render_cst_failure)
                .collect::<Vec<_>>()
                .join();
            (None, subtrees)
        }
    };
    if let Some((field, description)) = element {
        quote!(
            #rest
            if !runtime_evidence.#field {
                failures.insert(Failure::new(#description));
            }
        )
    } else {
        rest
    }
}

fn render_cst(cst: &ConsertTree) -> (TokenStream, TokenStream) {
    #![allow(clippy::unwrap_used)]
    let node = &cst.data;
    match &node.element {
        ConsertTreeElement::RuntimeEvidence(_, evidence) => {
            let field = evidence.field_identifier();
            (
                match evidence.description.clone() {
                    Some(description) => quote!(
                        #[doc = #description]
                    ),
                    None => quote!(),
                },
                quote!(
                    runtime_evidence.#field
                ),
            )
        }
        ConsertTreeElement::Demand(_, demand) => {
            let description = demand.lock().unwrap().description.clone();
            let check_logic = demand.lock().unwrap().field_identifier();
            (
                match description {
                    Some(description) => quote!(
                        #[doc = #description]
                    ),
                    None => quote!(),
                },
                quote!(
                    runtime_evidence.#check_logic
                ),
            )
        }
        ConsertTreeElement::Tautology => (quote!(), quote!(true)),
        ConsertTreeElement::Contradiction => (quote!(), quote!(false)),
        ConsertTreeElement::Gate(_, _, function) => {
            if node.children.is_empty() {
                (
                    quote!(),
                    match function {
                        GateFunction::And => quote!(true),
                        GateFunction::Or => quote!(false),
                    },
                )
            } else {
                let (assignments, idents): (Vec<_>, Vec<_>) = node
                    .children
                    .iter()
                    .enumerate()
                    .map(|(index, cst)| {
                        let (description, subtree_tokens) = render_cst(cst);
                        let ident = format_ident!("c{}", index);
                        (
                            quote!(
                                #description
                                let #ident = #subtree_tokens;
                            ),
                            ident,
                        )
                    })
                    .unzip();

                let operations = match function {
                    GateFunction::And => quote!(
                    #(#idents)&&*
                    ),
                    GateFunction::Or => quote!(
                    #(#idents)||*
                    ),
                };
                let assignments = assignments.join();
                (
                    quote!(),
                    quote!(
                        {
                        #assignments
                        #operations
                        }
                    ),
                )
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use conserts_elements::dimension::Dimension;
    use conserts_elements::elements::demands::Demand;
    use conserts_elements::evidence::Evidence;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    pub fn generate_tree(name: &str) -> Arc<Evidence> {
        Arc::new(Evidence::new(
            0,
            format!("{}Evidence", name),
            Some(format!("{}EvidenceDescription", name)),
            Dimension::Binary {
                r#type: "Property".into(),
            },
        ))
    }

    #[test]
    fn test_render() {
        let evidence = generate_tree("First");
        let cst = Tree::leaf(evidence.into());
        let guarantee = Arc::new(Guarantee::new(
            0,
            "FirstGuarantee",
            None,
            Dimension::Binary {
                r#type: "Type".into(),
            },
            cst,
        ));

        let guarantees = vec![guarantee];
        assert_eq!(
            render(&guarantees, false).next().unwrap(),
            (
                std::path::PathBuf::new().join("src/guarantees.rs"),
                quote!(
                    #![allow(unused_doc_comments)]
                    use super::evidence::RuntimeEvidence;
                    pub struct FirstGuarantee;
                    impl FirstGuarantee {
                        pub fn evaluate(runtime_evidence: &RuntimeEvidence) -> bool {
                            #[doc = "FirstEvidenceDescription"]
                            runtime_evidence.first_evidence
                        }
                    }
                )
                .to_string()
            )
        )
    }

    #[test]
    fn test_render_default() {
        let guarantee = Arc::new(Guarantee::new(
            0,
            "FirstGuarantee",
            None,
            Dimension::Binary {
                r#type: "Type".into(),
            },
            Default::default(),
        ));

        let guarantees = vec![guarantee];
        assert_eq!(
            render(&guarantees, false).next().unwrap(),
            (
                std::path::PathBuf::new().join("src/guarantees.rs"),
                quote!(
                    #![allow(unused_doc_comments)]
                    use super::evidence::RuntimeEvidence;
                    pub struct FirstGuarantee;
                    impl FirstGuarantee {
                        pub fn evaluate(_runtime_evidence: &RuntimeEvidence) -> bool {
                            true
                        }
                    }
                )
                .to_string()
            )
        )
    }

    #[test]
    fn test_render_complex() {
        let evidence1 = generate_tree("First");
        let demand = Arc::new(std::sync::Mutex::new(Demand::new(
            "D0",
            None,
            Dimension::Binary {
                r#type: "Type".into(),
            },
        )));

        let cst_1 = Tree::leaf(evidence1.into());
        let cst_2 = Tree::leaf(ConsertTreeElement::Tautology);
        let cst_3 = Tree::leaf(ConsertTreeElement::Demand(0, demand));
        let cst = Tree::node(
            ConsertTreeElement::Gate("Gate0".into(), 0, GateFunction::And),
            vec![cst_1, cst_2],
        );
        let cst = Tree::node(
            ConsertTreeElement::Gate("Gate1".into(), 1, GateFunction::Or),
            vec![cst, cst_3],
        );

        let guarantee = Arc::new(Guarantee::new(
            0,
            "FirstGuarantee",
            None,
            Dimension::Binary {
                r#type: "Type".into(),
            },
            cst,
        ));
        let guarantees = vec![guarantee];
        assert_eq!(
            render(&guarantees, true).next().unwrap(),
            (
                std::path::PathBuf::new().join("src/guarantees.rs"),
                quote!(
                    #![allow(unused_doc_comments)]
                    use crate::Failure;
                    use std::collections::BTreeSet;
                    use super::evidence::RuntimeEvidence;
                    pub struct FirstGuarantee;
                    impl FirstGuarantee {
                        pub fn evaluate(runtime_evidence: &RuntimeEvidence) -> bool {
                            {
                                let c0 = {
                                    #[doc = "FirstEvidenceDescription"]
                                    let c0 = runtime_evidence.first_evidence;
                                    let c1 = true;
                                    c0 && c1
                                };
                                let c1 = runtime_evidence.d0;
                                c0 || c1
                            }
                        }
                        pub fn failure(runtime_evidence: &RuntimeEvidence) -> Failure {
                            let mut failures = BTreeSet::new();
                            if !runtime_evidence.first_evidence {
                                failures.insert(Failure::new("FirstEvidenceDescription"));
                            }
                            if !runtime_evidence.d0 {
                                failures.insert(Failure::new("D0"));
                            }
                            Failure {
                                description: "FirstGuarantee".to_string(),
                                inner: failures,
                            }
                        }
                    }
                )
                .to_string()
            )
        )
    }
}
