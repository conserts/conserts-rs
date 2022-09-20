#![forbid(unsafe_code)]
#![deny(unused_results)]

// SPDX-FileCopyrightText: 2022 Fraunhofer Institute for Experimental Software Engineering IESE
//
// SPDX-License-Identifier: MIT

#[macro_use]
extern crate quote;

pub mod consert;
pub mod dimension;
pub mod elements;

pub use elements::consert_tree;
pub use elements::demands;
pub use elements::evidence;
pub use elements::guarantees;
pub use elements::numeric_range;
pub use elements::services;
pub use elements::uom;

#[cfg(test)]
pub use elements::guarantees::tests;
