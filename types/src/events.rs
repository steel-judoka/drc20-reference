use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

use crate::Account;

/// Event emitted when tokens are transferred.
///
/// Mirrors the ERC20 `Transfer` event.
///
/// - minting: `from = ZERO_ADDRESS`
/// - burning: `to = ZERO_ADDRESS` (not implemented in this reference)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transfer {
    /// The account tokens are transferred from.
    pub from: Account,
    /// The account receiving the tokens.
    pub to: Account,
    /// The value transferred.
    #[cfg_attr(feature = "serde", serde(with = "crate::serde_u64"))]
    pub value: u64,
}

impl Transfer {
    /// Event topic used when a transfer occurs.
    pub const TOPIC: &'static str = "transfer";
}

/// Event emitted when an allowance is set.
///
/// Mirrors the ERC20 `Approval` event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Approval {
    /// The account granting the allowance.
    pub owner: Account,
    /// The spender allowed to spend `owner`'s funds.
    pub spender: Account,
    /// The value `spender` is allowed to spend.
    #[cfg_attr(feature = "serde", serde(with = "crate::serde_u64"))]
    pub value: u64,
}

impl Approval {
    /// Event topic used when an approval occurs.
    pub const TOPIC: &'static str = "approval";
}
