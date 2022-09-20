// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::elements::demands::Demand;
use crate::elements::guarantees::Guarantee;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub type ProvidedServices = Vec<Rc<ProvidedService>>;
pub type RequiredServices = Vec<Arc<RequiredService>>;

#[derive(Debug)]
pub struct ProvidedService {
    pub ident: String,
    pub guarantees: Vec<Arc<Guarantee>>,
    pub functional_service_type: String,
}

impl ProvidedService {
    pub fn new<S1, S2>(
        ident: S1,
        guarantees: Vec<Arc<Guarantee>>,
        functional_service_type: S2,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            ident: ident.into(),
            guarantees,
            functional_service_type: functional_service_type.into(),
        }
    }

    pub fn guarantees(&self) -> Vec<Arc<Guarantee>> {
        self.guarantees.clone()
    }
}

#[derive(Debug)]
pub struct RequiredService {
    pub ident: String,
    pub demands: Vec<Arc<Mutex<Demand>>>,
    pub functional_service_type: String,
}

impl RequiredService {
    pub fn new<S1, S2>(
        ident: S1,
        demands: Vec<Arc<Mutex<Demand>>>,
        functional_service_type: S2,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            ident: ident.into(),
            demands,
            functional_service_type: functional_service_type.into(),
        }
    }

    pub fn demands(&self) -> Vec<Arc<Mutex<Demand>>> {
        self.demands.clone()
    }
}

impl RequiredService {
    pub fn matches_service_type(&self, provided_service: &ProvidedService) -> bool {
        self.functional_service_type
            .eq(&provided_service.functional_service_type)
    }
}

pub trait Service {
    fn identifier(&self) -> String;
}

impl Service for ProvidedService {
    fn identifier(&self) -> String {
        self.ident.clone()
    }
}

impl Service for RequiredService {
    fn identifier(&self) -> String {
        self.ident.clone()
    }
}
