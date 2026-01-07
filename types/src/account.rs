use core::cmp::Ordering;

use bytecheck::CheckBytes;
use dusk_core::abi::ContractId;
use dusk_core::signatures::bls::PublicKey;
use rkyv::{Archive, Deserialize, Serialize};

/// A DRC20 account.
///
/// Dusk contracts can be called both by externally owned accounts (BLS public keys)
/// and by other contracts (contract IDs). The DRC20 reference implementation supports both.
///
/// `Ord` is implemented so this type can be used as a `BTreeMap` key in contract state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Account {
    /// An externally owned account.
    External(PublicKey),
    /// A contract account.
    Contract(ContractId),
}

impl From<PublicKey> for Account {
    fn from(pk: PublicKey) -> Self {
        Self::External(pk)
    }
}

impl From<ContractId> for Account {
    fn from(contract: ContractId) -> Self {
        Self::Contract(contract)
    }
}

impl PartialOrd for Account {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Account {
    fn cmp(&self, other: &Self) -> Ordering {
        use Account::{Contract, External};

        match (self, other) {
            (External(lhs), External(rhs)) => lhs.to_raw_bytes().cmp(&rhs.to_raw_bytes()),
            (Contract(lhs), Contract(rhs)) => lhs.cmp(rhs),
            // Define a stable order across variants.
            (External(_), Contract(_)) => Ordering::Less,
            (Contract(_), External(_)) => Ordering::Greater,
        }
    }
}
