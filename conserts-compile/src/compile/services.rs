// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use super::{ProvidedService, RequiredService, Service};
use conserts_elements::elements::guarantees::{Guarantee, Invariant};
use crate::util::Join;
use proc_macro2::{Ident, TokenStream};
use std::rc::Rc;
use std::sync::Arc;

extern crate inflector;
use inflector::Inflector;

fn required_statements(required_services: &[Arc<RequiredService>]) -> Vec<(Ident, TokenStream)> {
    required_services
        .iter()
        .enumerate()
        .map(|(index, service)| {
            let ident = format_ident!("{}", service.identifier().to_pascal_case().replace(" ", ""));
            let binding = format_ident!("r{}", index);
            let code = quote!(
                let #binding = required::#ident::evaluate(&runtime_evidence);
            );
            (binding, code)
        })
        .collect()
}

fn invariant_statements(invariants: &[Arc<Invariant>]) -> Vec<(Ident, TokenStream)> {
    invariants
        .iter()
        .enumerate()
        .map(|(index, invariant)| {
            let ident = format_ident!("{}", invariant.ident);
            let binding = format_ident!("i{}", index);
            let code = quote!(
                let #binding = #ident::evaluate(&runtime_evidence);
            );
            (binding, code)
        })
        .collect()
}

fn guarantee_statements(guarantees: &[Arc<Guarantee>]) -> Vec<(Ident, TokenStream)> {
    guarantees
        .iter()
        .enumerate()
        .map(|(index, guarantee)| {
            let ident = format_ident!("{}", guarantee.ident);
            let binding = format_ident!("g{}", index);
            let code = quote!(
                let #binding = guarantees::#ident::evaluate(&runtime_evidence
            );
            (binding, code)
        })
        .collect()
}

fn required_services_code(required_services: &[Arc<RequiredService>]) -> TokenStream {
    required_services
        .iter()
        .map(|service| {
            let ident = format_ident!("{}", service.identifier().to_pascal_case().replace(" ", ""));
            let all_statements = service
                .demands
                .iter()
                .enumerate()
                .map(|(index, demand)| {
                    let check_logic = demand.lock().unwrap().field_access_code();
                    let binding = format_ident!("c{}", index);
                    let r = quote!(
                        let #binding = #check_logic;
                    );
                    (binding, r)
                })
                .collect::<Vec<_>>();

            let bindings: Vec<Ident> = all_statements.iter().map(|(i, _)| i.clone()).collect();
            let statements = all_statements
                .into_iter()
                .map(|(_, s)| s)
                .collect::<Vec<_>>()
                .join();

            quote!(
                pub struct #ident;

                impl Service for #ident {
                    fn evaluate(runtime_evidence: &RuntimeEvidence) -> bool {
                        #statements
                        #(#bindings)&&*
                    }
                }
            )
        })
        .collect::<Vec<_>>()
        .join()
}

fn provided_services_code(
    provided_services: &[Rc<ProvidedService>],
    required_services: &[Arc<RequiredService>],
    invariants: &[Arc<Invariant>],
) -> TokenStream {
    let required_statements = required_statements(required_services);
    let invariant_statements = invariant_statements(invariants);

    provided_services
        .iter()
        .map(|service| {
            let guarantee_statements = guarantee_statements(&service.guarantees);

            let all_statements : Vec<_> = guarantee_statements
                .into_iter()
                .chain(required_statements.iter().cloned())
                .chain(invariant_statements.iter().cloned())
                .collect();

            let bindings : Vec<_> = all_statements
                .iter()
                .map(|(i, _)| i.clone())
                .collect();

            let statements = all_statements
                .iter()
                .map(|(_, t)| t.clone())
                .collect::<Vec<_>>()
                .join();

            let service_identifier =
                format_ident!("{}", service.identifier().to_pascal_case().replace(" ", ""));

            quote!(
                pub struct #service_identifier;

                impl Service for #service_identifier {
                    fn evaluate(runtime_evidence: &RuntimeEvidence) -> bool {
                        #statements
                        #(#bindings)&&*
                    }
                }
            )
        })
        .collect::<Vec<_>>()
        .join()
}

fn render(
    provided_services: &[Rc<ProvidedService>],
    required_services: &[Arc<RequiredService>],
    invariants: &[Arc<Invariant>],
) -> TokenStream {
    let provided_services_code =
        provided_services_code(provided_services, required_services, invariants);

    let required_services_code = required_services_code(required_services);

    quote!(
        use super::evidence::RuntimeEvidence

        pub trait Service {
            fn evaluate(runtime_evidence: &RuntimeEvidence) -> bool;
        }

        pub mod required {
            use super::{Service, RuntimeEvidence};
            #required_services_code
        }

        pub mod provided {
            use super::{required, Service, RuntimeEvidence};
            use super::super::{guarantees};

            #provided_services_code
        }
    )
}
