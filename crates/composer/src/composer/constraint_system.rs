// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Constraint, witness, and embedded-curve types shared by Composer backends.

pub(crate) mod constraint;
pub(crate) mod ecc;
pub(crate) mod witness;

pub use constraint::Constraint;
pub(crate) use constraint::{Selector, WiredWitness};
pub use ecc::{TorsionFreeWitnessPoint, WitnessPoint};
pub use witness::Witness;
