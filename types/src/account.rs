use core::cmp::Ordering;

use alloc::vec::Vec;
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

impl Account {
    /// Canonical byte serialization for digest construction.
    ///
    /// Format: `[discriminant: u8] ++ [inner_bytes]`
    /// - External: `0x00 ++ public_key_raw_bytes (96 bytes)`
    /// - Contract: `0x01 ++ contract_id_bytes (32 bytes)`
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Account::External(pk) => {
                let mut buf = Vec::with_capacity(1 + 96);
                buf.push(0x00);
                buf.extend_from_slice(&pk.to_raw_bytes());
                buf
            }
            Account::Contract(id) => {
                let mut buf = Vec::with_capacity(1 + 32);
                buf.push(0x01);
                buf.extend_from_slice(id.as_bytes());
                buf
            }
        }
    }
}
