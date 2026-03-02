//! Shared types for the DRC20 reference implementation.
//!
//! This crate is intentionally small and contains no token logic.
//! It exists so the contract, data-driver, and off-chain tooling stay in lockstep.

#![no_std]

extern crate alloc;

#[cfg(feature = "serde")]
mod serde_u64;

pub mod account;
pub mod calls;
pub mod error;
pub mod events;
pub mod permit;

pub use account::Account;
pub use calls::{
    Allowance,
    ApproveCall,
    BalanceOf,
    Init,
    InitBalance,
    TransferCall,
    TransferFromCall,
    PermitCall,
};

use dusk_core::abi::ContractId;

/// Reserved zero address.
///
/// Used to represent minting in the `Transfer` event (`from = ZERO_ADDRESS`).
pub const ZERO_ADDRESS: Account = Account::Contract(ContractId::from_bytes([0u8; 32]));
