// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! This module holds the components needed in the Constraint System.
//! The components used are Variables, Witness and Wires.

/// Allocated witness in the constraint system.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Witness {
    index: usize,
}

impl Default for Witness {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Witness {
    /// A `1` witness representation.
    pub const ONE: Witness = Witness::new(1);
    /// A `0` witness representation.
    pub const ZERO: Witness = Witness::new(0);

    /// Generate a new [`Witness`]
    pub(crate) const fn new(index: usize) -> Self {
        Self { index }
    }

    /// Index of the allocated witness in the composer
    pub const fn index(&self) -> usize {
        self.index
    }
}

#[cfg(feature = "zeroize")]
impl zeroize::DefaultIsZeroes for Witness {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witness_constants_and_default_are_consistent() {
        assert_eq!(Witness::ZERO.index(), 0);
        assert_eq!(Witness::ONE.index(), 1);

        // `Default` is implemented as `Witness::ZERO`.
        assert_eq!(Witness::default(), Witness::ZERO);
    }
}
