// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://dusk.network/favicon.svg")]
#![doc(html_favicon_url = "https://dusk.network/favicon.png")]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(feature = "std"), no_std)]

// Compile the workspace-level getting-started snippets alongside this crate's
// own README doctests. Groth16's dev-dependencies provide both proof systems.
#[cfg(doctest)]
#[doc = include_str!("../../../README.md")]
mod workspace_readme_doctests {}

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;

#[cfg(feature = "alloc")]
mod compiler;
#[cfg(feature = "alloc")]
mod fft;
#[cfg(feature = "alloc")]
mod keys;
#[cfg(feature = "alloc")]
mod msm;
#[cfg(feature = "alloc")]
mod proof;
#[cfg(feature = "alloc")]
mod qap;

#[cfg(feature = "alloc")]
pub use compiler::{Compiler, Prover, Verifier};
pub use error::Error;
#[cfg(feature = "alloc")]
pub use keys::{PreparedVerifyingKey, ProvingKey, VerifyingKey};
#[cfg(feature = "alloc")]
pub use proof::{PROOF_SIZE, Proof};

/// Common imports for defining and proving Groth16 circuits.
pub mod prelude;
