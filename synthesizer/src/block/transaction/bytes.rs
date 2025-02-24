// Copyright (C) 2019-2023 Aleo Systems Inc.
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

use super::*;

impl<N: Network> FromBytes for Transaction<N> {
    /// Reads the transaction from the buffer.
    #[inline]
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        // Read the version.
        let version = u8::read_le(&mut reader)?;
        // Ensure the version is valid.
        if version != 0 {
            return Err(error("Invalid transaction version"));
        }

        // Read the variant.
        let variant = u8::read_le(&mut reader)?;
        // Match the variant.
        let (id, transaction) = match variant {
            0 => {
                // Read the ID.
                let id = N::TransactionID::read_le(&mut reader)?;
                // Read the owner.
                let owner = ProgramOwner::read_le(&mut reader)?;
                // Read the deployment.
                let deployment = Deployment::read_le(&mut reader)?;
                // Read the fee.
                let fee = Fee::read_le(&mut reader)?;

                // Initialize the transaction.
                let transaction = Self::from_deployment(owner, deployment, fee).map_err(|e| error(e.to_string()))?;
                // Return the ID and the transaction.
                (id, transaction)
            }
            1 => {
                // Read the ID.
                let id = N::TransactionID::read_le(&mut reader)?;
                // Read the execution.
                let execution = Execution::read_le(&mut reader)?;

                // Read the fee variant.
                let fee_variant = u8::read_le(&mut reader)?;
                // Read the fee.
                let fee = match fee_variant {
                    0u8 => None,
                    1u8 => Some(Fee::read_le(&mut reader)?),
                    _ => return Err(error("Invalid fee variant")),
                };

                // Initialize the transaction.
                let transaction = Self::from_execution(execution, fee).map_err(|e| error(e.to_string()))?;
                // Return the ID and the transaction.
                (id, transaction)
            }
            _ => return Err(error("Invalid transaction variant")),
        };

        // Ensure the transaction ID matches.
        match transaction.id() == id {
            // Return the transaction.
            true => Ok(transaction),
            false => Err(error("Transaction ID mismatch")),
        }
    }
}

impl<N: Network> ToBytes for Transaction<N> {
    /// Writes the transaction to the buffer.
    #[inline]
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        // Write the version.
        0u8.write_le(&mut writer)?;

        // Write the transaction.
        match self {
            Self::Deploy(id, owner, deployment, fee) => {
                // Write the variant.
                0u8.write_le(&mut writer)?;
                // Write the ID.
                id.write_le(&mut writer)?;
                // Write the owner.
                owner.write_le(&mut writer)?;
                // Write the deployment.
                deployment.write_le(&mut writer)?;
                // Write the fee.
                fee.write_le(&mut writer)
            }
            Self::Execute(id, execution, fee) => {
                // Write the variant.
                1u8.write_le(&mut writer)?;
                // Write the ID.
                id.write_le(&mut writer)?;
                // Write the execution.
                execution.write_le(&mut writer)?;
                // Write the fee.
                match fee {
                    None => 0u8.write_le(&mut writer),
                    Some(fee) => {
                        1u8.write_le(&mut writer)?;
                        fee.write_le(&mut writer)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::network::Testnet3;

    type CurrentNetwork = Testnet3;

    #[test]
    fn test_bytes() -> Result<()> {
        let rng = &mut TestRng::default();

        for expected in [
            crate::vm::test_helpers::sample_deployment_transaction(rng),
            crate::vm::test_helpers::sample_execution_transaction_with_fee(rng),
        ]
        .into_iter()
        {
            // Check the byte representation.
            let expected_bytes = expected.to_bytes_le()?;
            assert_eq!(expected, Transaction::read_le(&expected_bytes[..])?);
            assert!(Transaction::<CurrentNetwork>::read_le(&expected_bytes[1..]).is_err());
        }
        Ok(())
    }
}
