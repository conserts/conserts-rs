// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use conserts_elements::{consert::Consert, demands::Demand, services::RequiredService};
use conserts_error::ConSertError;
use proc_macro2::TokenStream;

mod check_logic;
mod crate_files;
mod evidence;
mod guarantees;
#[cfg(not(tarpaulin_include))]
mod io;
pub mod monitor;
mod parameters;
mod properties;
mod render;
pub use parameters::*;

#[cfg(not(tarpaulin_include))] // integration function
pub fn export(
    parameters: &crate::compile::CompileParameters,
    consert: &Consert,
) -> Result<(), ConSertError<Demand, RequiredService>> {
    let base_path = parameters.base_path(consert);
    io::create_directories(&base_path, parameters)?;
    let files = generate_all_crate_files(parameters, consert)?;
    io::write_files(&base_path, files.collect())?;
    io::format_crate(&base_path)?;
    io::convert_dot(&base_path)?;
    Ok(())
}

#[cfg(not(tarpaulin_include))] // integration function
fn generate_all_crate_files(
    parameters: &CompileParameters,
    consert: &Consert,
) -> Result<impl Iterator<Item = io::CrateFile>, ConSertError<Demand, RequiredService>> {
    let filter_configuration = parameters.filter_configuration();

    let files = std::iter::empty()
        .chain(render_crate_code(consert, filter_configuration)?)
        .chain(crate_files::generate_cargo_toml(consert)?)
        .chain(crate_files::generate_gitignore()?)
        .chain(crate_files::generate_dot(consert)?);
    Ok(files)
}

#[cfg(not(tarpaulin_include))] // integration function
fn render_crate_code(
    consert: &conserts_elements::consert::Consert,
    configuration: monitor::FilterConfiguration,
) -> Result<
    impl Iterator<Item = crate::compile::io::CrateFile>,
    ConSertError<Demand, RequiredService>,
> {
    Ok(std::iter::empty()
        .chain(evidence::render(consert.evidence(), consert.demands())?)
        .chain(guarantees::render(&consert.guarantees(), false))
        .chain(monitor::render(
            consert.evidence(),
            consert.demands(),
            configuration,
        ))
        .chain(properties::render(consert))
        .chain(render_lib(consert))
        .chain(render_prelude()))
}

fn render_lib(_: &Consert) -> impl Iterator<Item = crate::compile::io::CrateFile> {
    let mut code = quote!(
        //#![deny(warnings)]
    );
    code.extend(quote!(
        pub mod evidence;
        pub mod guarantees;
        pub mod monitor;
        pub mod prelude;
        pub mod properties;
        pub use uom;
    ));
    std::iter::once((
        std::path::PathBuf::new().join("src/lib.rs"),
        code.to_string(),
    ))
}

fn render_prelude() -> std::iter::Once<crate::compile::io::CrateFile> {
    let code = quote!(
        pub use crate::evidence::RuntimeEvidence;
        pub use crate::guarantees;
        pub use crate::properties::*;
        pub use crate::monitor::Monitor;
    );

    std::iter::once((
        std::path::PathBuf::new().join("src/prelude.rs"),
        code.to_string(),
    ))
}

trait TokenStreamJoin {
    fn join(self) -> TokenStream;
    fn join_with(self, separator: TokenStream) -> TokenStream;
}

impl TokenStreamJoin for Vec<TokenStream> {
    fn join(self) -> TokenStream {
        self.into_iter().fold(TokenStream::new(), |mut a, b| {
            a.extend(b);
            a
        })
    }

    fn join_with(self, separator: TokenStream) -> TokenStream {
        let mut iter = self.into_iter();
        let first = iter.next().unwrap_or_default();
        iter.fold(first, |mut a, b| {
            a.extend(separator.clone());
            a.extend(b);
            a
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conserts_elements::consert::ConsertBuilder;
    use conserts_elements::dimension::Dimension;
    use conserts_elements::elements::consert_tree::*;
    use conserts_elements::elements::demands::Demand;
    use conserts_elements::elements::guarantees::Guarantee;
    use pretty_assertions::assert_eq;
    use std::path;
    use std::sync::Arc;

    pub(super) fn small_consert() -> ConsertBuilder {
        let mut consert = ConsertBuilder::new().name("Test").path("Test");

        let evidence = consert.add_runtime_evidence(
            "Evidence",
            None,
            Dimension::Binary {
                r#type: "Type".into(),
            },
        );

        let cst = Tree::leaf(ConsertTreeElement::Tautology);
        let mut demand = Demand::new(
            "D0",
            None,
            Dimension::Binary {
                r#type: "Type".into(),
            },
        );
        demand.linked_guarantees.push((
            "other_crate".to_string(),
            Arc::new(Guarantee::new(
                0,
                "ExternalGuarantee",
                None,
                Dimension::Binary {
                    r#type: "Type".into(),
                },
                cst,
            )),
        ));
        let demand = Arc::new(std::sync::Mutex::new(demand));
        let cst1 = Tree::leaf(evidence.into());
        let cst2 = Tree::leaf(ConsertTreeElement::Demand(0, demand.clone()));
        let cst = Tree::node(
            ConsertTreeElement::Gate("Gate0".into(), 0, GateFunction::And),
            vec![cst1, cst2],
        );

        let consert = consert.add_guarantee(
            "Guarantee5",
            None,
            Dimension::Binary {
                r#type: "Type".into(),
            },
            cst,
        );
        consert.add_demand(demand)
    }

    #[test]
    fn test_render_prelude() {
        assert_eq!(
            render_prelude().next().unwrap(),
            (
                path::PathBuf::new().join("src/prelude.rs"),
                quote!(
                    pub use crate::evidence::RuntimeEvidence;
                    pub use crate::guarantees;
                    pub use crate::properties::*;
                    pub use crate::monitor::Monitor;
                )
                .to_string()
            )
        );
    }
    #[test]
    fn test_join() {
        let a = quote!(let foo = 8;);
        let b = quote!(let bar = 15);
        let v = vec![a.clone(), b.clone()];

        let expected = quote!(
            #a
            #b
        );
        assert_eq!(v.join().to_string(), expected.to_string());

        let v = vec![a.clone(), b.clone()];

        let expected = quote!(
            #a || #b
        );
        assert_eq!(v.join_with(quote!(||)).to_string(), expected.to_string());
    }
}
