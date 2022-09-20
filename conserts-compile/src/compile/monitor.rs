// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::compile::render::RenderEvidence;
use crate::compile::TokenStreamJoin;
use conserts_elements::demands::Demand;
use conserts_elements::evidence::Evidence;
use proc_macro2::TokenStream;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilterConfiguration {
    depth: usize,
}

impl FilterConfiguration {
    pub fn new(depth: usize) -> Self {
        Self { depth }
    }
}

pub(super) fn render(
    evidence: Vec<Arc<Evidence>>,
    demands: Vec<Arc<Mutex<Demand>>>,
    filter_configuration: FilterConfiguration,
) -> impl Iterator<Item = crate::compile::io::CrateFile> {
    let evidence = std::iter::empty()
        .chain(evidence.into_iter().map(|e| e as Arc<dyn RenderEvidence>))
        .chain(demands.into_iter().map(|d| d as Arc<dyn RenderEvidence>))
        .collect::<Vec<_>>();

    let pop_count_code = pop_count_code(&evidence);

    render_monitor_module(pop_count_code, filter_configuration.depth)
}

fn generate_per_evidence<F: FnMut(&Arc<dyn RenderEvidence>) -> TokenStream>(
    evidence: &[Arc<dyn RenderEvidence>],
    f: F,
) -> TokenStream {
    evidence.iter().map(f).collect::<Vec<_>>().join()
}

fn pop_count_code(evidence: &[Arc<dyn RenderEvidence>]) -> TokenStream {
    let field_declarations = generate_per_evidence(evidence, |evidence| {
        let field = evidence.field_identifier();
        quote!(
            #field: PopCount,
        )
    });

    let update_statements = generate_per_evidence(evidence, |evidence| {
        let field = evidence.field_identifier();
        quote!(
            if evidence.#field {
                pop_count.#field.t += 1;
            } else {
                pop_count.#field.f += 1;
            }
        )
    });

    let vote_fields = generate_per_evidence(evidence, |evidence| {
        let field = evidence.field_identifier();
        quote!(
            #field: (self.#field.t > self.#field.f),
        )
    });

    quote!(
        #[derive(Debug, Default)]
        struct PopCount {
            t: usize,
            f: usize
        }

        #[derive(Debug, Default)]
        pub struct RuntimeEvidencePopCount {
            #field_declarations
        }

        impl RuntimeEvidencePopCount {
            fn from(evidence: &[RuntimeEvidence]) -> RuntimeEvidencePopCount {
                let mut pop_count = RuntimeEvidencePopCount::default();
                for evidence in evidence {
                    #update_statements
                }
                pop_count
            }

            fn majority_vote(&self) -> RuntimeEvidence {
                RuntimeEvidence {
                    #vote_fields
                }
            }
        }
    )
}

fn render_monitor_module(
    pop_count_code: TokenStream,
    depth: usize,
) -> impl Iterator<Item = super::io::CrateFile> {
    std::iter::once((
        std::path::PathBuf::new().join("src/monitor.rs"),
        quote!(
            use crate::evidence::RuntimeEvidence;
            use crate::properties::*;
            use heapless::HistoryBuffer;

            #pop_count_code

            pub struct Monitor {
                values: HistoryBuffer<RuntimeEvidence, #depth>
            }

            impl Default for Monitor {
                fn default() -> Self {
                    Self {
                        values: HistoryBuffer::new()
                    }
                }
            }

            impl Monitor {
                pub fn new() -> Monitor {
                    Monitor::default()
                }
                pub fn add_sample(&mut self, value: RuntimeProperties) {
                    let evidence = RuntimeEvidence::from(&value);
                    self.values.write(evidence);
                }

                pub fn get_sample(&mut self) -> RuntimeEvidence {
                    let evidence = self.values.as_slice();
                    RuntimeEvidencePopCount::from(evidence).majority_vote()
                }
            }
        )
        .to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use conserts_elements::dimension::Dimension;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_render() {
        use conserts_elements::elements::evidence::Evidence;
        use std::sync::Arc;

        let filter_configuration = crate::compile::monitor::FilterConfiguration::new(5);

        let evidence = Arc::new(Evidence::new(
            0,
            "EvidenceFoo",
            None,
            Dimension::Binary {
                r#type: "Type".into(),
            },
        ));

        assert_eq!(
            render(vec![evidence], vec![], filter_configuration).next().unwrap(),
            (
                std::path::PathBuf::new().join("src/monitor.rs"),
                quote!(
                    use crate::evidence::RuntimeEvidence;
                    use crate :: properties :: * ;
                    use heapless :: HistoryBuffer ;
                    # [derive (Debug , Default)]
                    struct PopCount { t : usize , f : usize }
                    # [derive (Debug , Default)]
                    pub struct RuntimeEvidencePopCount { evidence_foo : PopCount , }
                    impl RuntimeEvidencePopCount {
                        fn from (evidence : & [RuntimeEvidence]) -> RuntimeEvidencePopCount {
                            let mut pop_count = RuntimeEvidencePopCount :: default () ;
                            for evidence in evidence {
                                if evidence . evidence_foo {
                                    pop_count . evidence_foo . t += 1 ;
                                } else {
                                    pop_count . evidence_foo . f += 1 ;
                                }
                            }
                            pop_count
                        }
                        fn majority_vote (& self) -> RuntimeEvidence {
                            RuntimeEvidence {
                                evidence_foo : (self . evidence_foo . t > self . evidence_foo . f),
                            }
                        }
                    }
                    pub struct Monitor { values : HistoryBuffer < RuntimeEvidence , 5usize > }

                    impl Default for Monitor {
                        fn default () -> Self {
                            Self {
                                values : HistoryBuffer :: new ()
                            }
                        }
                    }

                    impl Monitor {
                        pub fn new () -> Monitor {
                            Monitor :: default ()
                        }
                        pub fn add_sample (& mut self , value : RuntimeProperties) {
                            let evidence = RuntimeEvidence :: from (& value) ; self . values . write (evidence) ;
                        }
                        pub fn get_sample (& mut self) -> RuntimeEvidence {
                            let evidence = self . values . as_slice () ; RuntimeEvidencePopCount :: from (evidence) . majority_vote ()
                        }
                    }
                )
                .to_string()
            )
        );
    }
}
