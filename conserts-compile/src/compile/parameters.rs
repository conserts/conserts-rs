// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use super::monitor;
use conserts_elements::consert::Consert;
use conserts_elements::elements::{demands::Demand, services::RequiredService};
use conserts_error::ConSertError;

pub struct CompileParameters {
    path: String,
    providers: Option<Vec<String>>,
    out_path: String,
    filter_configuration: monitor::FilterConfiguration,
}

impl CompileParameters {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: String,
        providers: Option<Vec<String>>,
        out_path: String,
        filter_configuration: monitor::FilterConfiguration,
    ) -> Self {
        Self {
            path,
            providers,
            out_path,
            filter_configuration,
        }
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn providers(&self) -> Vec<String> {
        match &self.providers {
            Some(providers) => providers.clone(),
            None => vec![],
        }
    }

    pub fn base_path(&self, consert: &Consert) -> std::path::PathBuf {
        std::path::Path::new(&self.out_path).join(consert.crate_name())
    }

    pub fn canonical_base_path(
        &self,
        consert: &Consert,
    ) -> Result<std::path::PathBuf, ConSertError<Demand, RequiredService>> {
        // const PATH_PREFIX: &str = r#"\\?\"#;
        Ok(self.base_path(consert).canonicalize()?)
    }

    pub fn filter_configuration(&self) -> monitor::FilterConfiguration {
        self.filter_configuration.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::small_consert;
    use super::*;

    #[test]
    fn test_getters() {
        let current_dir = std::env::current_dir().unwrap();
        let consert = small_consert().build().unwrap();
        std::fs::create_dir_all("./target/consert_Test").unwrap();
        let cp = CompileParameters::new(
            "FooBar".to_string(),
            None,
            "./target/".to_string(),
            monitor::FilterConfiguration::new(5),
        );
        assert_eq!(
            cp.base_path(&consert),
            std::path::PathBuf::new().join(&format!(
                ".{0}target{0}consert_Test",
                std::path::MAIN_SEPARATOR
            ))
        );
        assert_eq!(
            cp.canonical_base_path(&consert).unwrap(),
            current_dir
                .canonicalize()
                .unwrap()
                .join(&format!("target{}consert_Test", std::path::MAIN_SEPARATOR))
        );
        assert_eq!(cp.path(), "FooBar".to_string());
        assert_eq!(
            cp.filter_configuration(),
            monitor::FilterConfiguration::new(5)
        );
        let providers: Vec<String> = vec![];
        assert_eq!(cp.providers(), providers);
    }
}
