// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

use crate::elements::{demands::Demand, services::RequiredService};
use conserts_error::{ConSertError, UnitOfMeasureError};
use inflector::cases::titlecase;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::mem::discriminant;

#[derive(Debug, PartialEq, Clone, Eq, Ord, PartialOrd)]
pub enum Dimension {
    Unitless,
    Force,
    Time,
    Length,
    Velocity,
}

#[derive(Debug, PartialEq, Clone, Eq, Ord, PartialOrd)]
pub struct Unit {
    pub abbreviation: String,
    pub singular: String,
    pub plural: String,
}

/// The encapsulated string represents the order of magnitude in the usual SI prefixes (milli, kilo, ...)
#[derive(Debug, PartialEq, Clone, PartialOrd, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct UnitOfMeasure {
    dimension: Dimension,
    unit: Unit,
    conversion_factor: f64,
}

macro_rules! unit {
    (  $x:path ) => {{
        let (abbreviation, singular, plural) = {
            use uom::si::Unit;
            use $x as base;
            (
                base::abbreviation().to_string(),
                base::singular().to_string(),
                base::plural().to_string(),
            )
        };
        Unit {
            abbreviation,
            singular,
            plural,
        }
    }};
}

impl From<UnitOfMeasure> for String {
    #[cfg(not(tarpaulin_include))] // trivial
    fn from(s: UnitOfMeasure) -> Self {
        s.quantity()
    }
}

impl TryFrom<String> for UnitOfMeasure {
    type Error = ConSertError<Demand, RequiredService>;

    #[cfg(not(tarpaulin_include))] // trivial
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (dimension, unit, conversion_factor) = match value.as_ref() {
            "" => (Dimension::Unitless, unit!(uom::si::ratio::ratio), 1.0),
            // Length
            "kilometer" | "km" => (
                Dimension::Length,
                unit!(uom::si::length::kilometer),
                <uom::si::length::kilometer as uom::Conversion<f64>>::coefficient(),
            ),
            "meter" | "m" => (
                Dimension::Length,
                unit!(uom::si::length::meter),
                <uom::si::length::meter as uom::Conversion<f64>>::coefficient(),
            ),
            "millimeter" | "mm" => (
                Dimension::Length,
                unit!(uom::si::length::millimeter),
                <uom::si::length::millimeter as uom::Conversion<f64>>::coefficient(),
            ),
            // Time
            "second" | "s" => (
                Dimension::Time,
                unit!(uom::si::time::second),
                <uom::si::time::second as uom::Conversion<f64>>::coefficient(),
            ),
            "millisecond" | "ms" => (
                Dimension::Time,
                unit!(uom::si::time::millisecond),
                <uom::si::time::millisecond as uom::Conversion<f64>>::coefficient(),
            ),
            "microsecond" | "us" => (
                Dimension::Time,
                unit!(uom::si::time::microsecond),
                <uom::si::time::microsecond as uom::Conversion<f64>>::coefficient(),
            ),
            "nanosecond" | "ns" => (
                Dimension::Time,
                unit!(uom::si::time::nanosecond),
                <uom::si::time::nanosecond as uom::Conversion<f64>>::coefficient(),
            ),
            // Velocity
            "kilometer_per_hour" | "km/h" => (
                Dimension::Velocity,
                unit!(uom::si::velocity::kilometer_per_hour),
                <uom::si::velocity::kilometer_per_hour as uom::Conversion<f64>>::coefficient(),
            ),
            "meter_per_second" | "m/s" => (
                Dimension::Velocity,
                unit!(uom::si::velocity::meter_per_second),
                <uom::si::velocity::meter_per_second as uom::Conversion<f64>>::coefficient(),
            ),
            // Force
            "newton" | "N" => (
                Dimension::Force,
                unit!(uom::si::force::newton),
                <uom::si::force::newton as uom::Conversion<f64>>::coefficient(),
            ),
            // Unsupported
            _ => return Err(UnitOfMeasureError::UnsupportedUnit(value.to_string()).into()),
        };
        Ok(Self {
            dimension,
            unit,
            conversion_factor,
        })
    }
}

impl UnitOfMeasure {
    pub fn new(uom: &str) -> Result<Self, ConSertError<Demand, RequiredService>> {
        TryFrom::try_from(uom.to_string())
    }
    pub fn get_unit_ab(&self) -> &str {
        &self.unit.abbreviation
    }

    pub fn get_unit_singular(&self) -> &str {
        &self.unit.singular
    }

    pub fn get_unit_plural(&self) -> &str {
        &self.unit.plural
    }
    #[cfg(not(tarpaulin_include))] // trivial
    pub fn compatible(
        &self,
        other: &UnitOfMeasure,
    ) -> Result<bool, ConSertError<Demand, RequiredService>> {
        if discriminant(&self.dimension) == discriminant(&other.dimension) {
            Ok(true)
        } else {
            Err(UnitOfMeasureError::Incompatible.into())
        }
    }

    #[cfg(not(tarpaulin_include))] // trivial
    pub fn quantity(&self) -> String {
        match self.dimension {
            Dimension::Unitless => uom::si::ratio::description().to_string(),
            Dimension::Force => uom::si::force::description().to_string(),
            Dimension::Length => uom::si::length::description().to_string(),
            Dimension::Time => uom::si::time::description().to_string(),
            Dimension::Velocity => uom::si::velocity::description().to_string(),
        }
    }

    #[allow(non_snake_case)]
    #[cfg(not(tarpaulin_include))] // trivial
    pub fn Quantity(&self) -> String {
        titlecase::to_title_case(&self.quantity())
    }

    #[cfg(not(tarpaulin_include))] // trivial
    pub fn measurement_unit(&self) -> String {
        self.get_unit_singular().to_string()
    }
    #[cfg(not(tarpaulin_include))] // trivial
    pub(crate) fn factor(&self) -> f64 {
        self.conversion_factor
    }
}
