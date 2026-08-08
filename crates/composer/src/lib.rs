// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://dusk.network/favicon.svg")]
#![doc(html_favicon_url = "https://dusk.network/favicon.png")]
#![allow(clippy::suspicious_arithmetic_impl)]
#![allow(clippy::suspicious_op_assign_impl)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::match_bool)]
#![allow(clippy::too_many_arguments)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        #[cfg_attr(not(feature = "std"), macro_use)]
        extern crate alloc;

        mod bit_iterator;
        mod composer;
        mod runtime;

        /// Canonical Turbo Plonkish gate identities.
        #[cfg(feature = "plonkish")]
        pub mod identities;
        #[cfg(all(feature = "r1cs", not(feature = "plonkish")))]
        mod identities;
    }
}

#[cfg(feature = "debug")]
mod debugger;

mod error;

#[cfg(all(feature = "alloc", feature = "plonkish", feature = "test-api"))]
pub use composer::test_support;
#[cfg(feature = "alloc")]
pub use composer::{
    Circuit, Composer, ComposerBackend, Constraint, TorsionFreeWitnessPoint,
    Witness, WitnessPoint,
};
#[cfg(all(feature = "alloc", feature = "plonkish"))]
pub use composer::{CircuitShape, Gate, Plonkish};
#[cfg(all(feature = "alloc", feature = "r1cs"))]
pub use composer::{
    LinearCombination, R1cs, R1csAssignment, R1csCircuit, R1csConstraint,
    R1csShape, Variable,
};
pub use error::{Error, Error as CircuitError};

/// Common circuit-construction imports.
pub mod prelude;
