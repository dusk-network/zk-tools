// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#[cfg(feature = "plonkish")]
use alloc::vec::Vec;

#[cfg(feature = "plonkish")]
use super::compress::CompressedCircuit;
#[cfg(feature = "plonkish")]
use crate::prelude::Plonkish;
use crate::prelude::{Composer, ComposerBackend, Error};

/// Circuit definition that can be built by any [`ComposerBackend`].
///
/// Compilers use [`Default`] to construct the circuit shape independently of
/// a particular witness assignment.
pub trait Circuit: Default {
    /// Build the circuit with the selected backend.
    fn circuit<B: ComposerBackend>(
        &self,
        composer: &mut Composer<B>,
    ) -> Result<(), Error>;

    /// Returns the number of constraints emitted for backend `B`.
    fn size<B: ComposerBackend>(&self) -> usize {
        let mut composer = Composer::<B>::initialized();
        match self.circuit(&mut composer) {
            Ok(_) => composer.constraints(),
            Err(_) => 0,
        }
    }

    /// Return a compressed PLONKish circuit description.
    #[cfg(feature = "plonkish")]
    fn compress() -> Result<Vec<u8>, Error> {
        let mut composer = Composer::<Plonkish>::initialized();
        Self::default().circuit(&mut composer)?;

        let hades_optimization = true;
        Ok(CompressedCircuit::from_composer(
            hades_optimization,
            composer,
        ))
    }
}
