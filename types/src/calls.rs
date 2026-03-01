use alloc::vec::Vec;

use bytecheck::CheckBytes;
use dusk_core::signatures::bls::{PublicKey as BlsPublicKey, Signature as BlsSignature};
use rkyv::{Archive, Deserialize, Serialize};

use crate::Account;

/// One entry in the initial distribution.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitBalance {
    /// Account receiving minted tokens.
    pub account: Account,
    /// Amount minted (smallest units).
    #[cfg_attr(feature = "serde", serde(with = "crate::serde_u64"))]
    pub amount: u64,
}

/// Input for `init(Init)`.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Init {
    /// Initial distribution entries.
    pub initial_balances: Vec<InitBalance>,
}

/// Input for `balance_of(BalanceOf)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BalanceOf {
    /// Account to query.
    pub account: Account,
}

/// Input for `allowance(Allowance)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Allowance {
    /// Token owner.
    pub owner: Account,
    /// Spender.
    pub spender: Account,
}

/// Input for `transfer(TransferCall)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransferCall {
    /// Recipient.
    pub to: Account,
    /// Amount to transfer.
    #[cfg_attr(feature = "serde", serde(with = "crate::serde_u64"))]
    pub value: u64,
}

/// Input for `approve(ApproveCall)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApproveCall {
    /// Spender.
    pub spender: Account,
    /// Allowance amount.
    #[cfg_attr(feature = "serde", serde(with = "crate::serde_u64"))]
    pub value: u64,
}

/// Input for `transfer_from(TransferFromCall)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransferFromCall {
    /// Owner whose allowance is being used.
    pub owner: Account,
    /// Recipient.
    pub to: Account,
    /// Amount to transfer.
    #[cfg_attr(feature = "serde", serde(with = "crate::serde_u64"))]
    pub value: u64,
}

/// Input for `permit(PermitCall)`.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PermitCall {
    /// Token owner (signer).
    pub owner: BlsPublicKey,
    /// Spender being approved.
    pub spender: Account,
    /// Allowance amount.
    #[cfg_attr(feature = "serde", serde(with = "crate::serde_u64"))]
    pub value: u64,
    /// Block height after which the permit expires.
    pub deadline: u64,
    /// BLS signature over the permit digest.
    pub signature: BlsSignature,
}
