// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::Program;
use snarkvm_circuits::{algorithms::Pedersen512 as Pedersen512Gadget, CommitmentScheme, Literal, ToBits};

const SETUP_MESSAGE: &str = "PedersenCommitment512";

#[derive(Clone)]
pub struct Pedersen512<P: Program>(Pedersen512Gadget<P::Environment>);

impl<P: Program> Default for Pedersen512<P> {
    fn default() -> Self {
        Self::new(SETUP_MESSAGE)
    }
}

impl<P: Program> Pedersen512<P> {
    /// Creates a new instance of `Pedersen512`.
    pub fn new(message: &str) -> Self {
        Self(Pedersen512Gadget::setup(message))
    }

    /// Commit to the given `value` and `randomness`. Halts if given a value that
    /// exceeds the input size, or if `randomness` is not a scalar.
    pub fn commit(
        &self,
        value: Literal<P::Environment>,
        randomness: Literal<P::Environment>,
    ) -> Literal<P::Environment> {
        if let Literal::Scalar(r) = randomness {
            match value {
                Literal::Address(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::Boolean(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::Field(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::Group(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::I8(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::I16(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::I32(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::I64(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::I128(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::U8(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::U16(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::U32(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::U64(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::U128(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::Scalar(a) => Literal::Group(self.0.commit(&a.to_bits_le(), &r.to_bits_le())),
                Literal::String(a) => {
                    let bits = a.to_bits_le();
                    if bits.len() > 512 {
                        P::halt("Invalid input size for Pedersen512 commitment")
                    } else {
                        Literal::Group(self.0.commit(&bits, &r.to_bits_le()))
                    }
                }
            }
        } else {
            P::halt("Invalid randomness type for Pedersen512 commitment")
        }
    }
}
