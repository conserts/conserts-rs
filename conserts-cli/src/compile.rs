// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use color_eyre::{
    eyre::{anyhow, WrapErr},
    Result,
};
use conserts_compile::compile::monitor::FilterConfiguration;
use conserts_compile::compile::CompileParameters;
use conserts_elements::consert::Consert;

pub(crate) mod report;

#[cfg(not(tarpaulin_include))] // IO function
pub(super) fn parse_args(args: &clap::ArgMatches) -> Result<CompileParameters> {
    let path = args
        .value_of("input")
        .ok_or_else(|| anyhow!("Missing input"))?
        .to_string();

    let providers = args
        .values_of("provider")
        .map(|values| values.map(|v| v.to_string()).collect());

    let out_path = args
        .value_of("output")
        .ok_or_else(|| anyhow!("Missing output"))?
        .to_string();

    let filter_configuration = FilterConfiguration::new(
        args.value_of("filter-depth")
            .ok_or_else(|| anyhow!("Missing filter-depth"))?
            .parse()
            .wrap_err("Filter depth has to be a number")?,
    );

    Ok(CompileParameters::new(
        path,
        providers,
        out_path,
        filter_configuration,
    ))
}

#[cfg(not(tarpaulin_include))] // integration function
pub(super) fn test_composition(_: &mut Consert, _: &CompileParameters) -> Result<()> {
    Ok(())
}

#[cfg(not(tarpaulin_include))] // integration function
pub(super) fn report_result(consert: &Consert, parameters: &CompileParameters) -> Result<()> {
    let base_path = parameters.canonical_base_path(consert)?;
    report::general_result(&base_path);
    Ok(())
}
