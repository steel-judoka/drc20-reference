// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Dusk Forge

//! DRC20 contract.
//!
//! Minimal ERC20-like reference for Dusk:
//! - transfer / approve / transfer_from
//! - balance_of / allowance / total_supply
//! - metadata getters (name/symbol/decimals)
//!
//! This crate is wasm32-only and compiled to `wasm32-unknown-unknown`.

#![no_std]
#![cfg(target_family = "wasm")]
#![deny(unused_extern_crates)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::pedantic)]

extern crate alloc;

use dusk_core::abi;

pub(crate) mod state;
use state::Drc20;

use drc20_types::{Allowance, ApproveCall, BalanceOf, Init, TransferCall, TransferFromCall};

static mut STATE: Drc20 = Drc20::new();

/// Initializes the contract with an initial distribution.
///
/// Deployment tooling on Dusk can omit constructor args entirely. In that case
/// `arg_len == 0`, and attempting to deserialize a non-unit type will panic.
///
/// To make deployments safer (and cheaper), we treat "no args" as an empty
/// initial distribution (total supply = 0).
#[no_mangle]
unsafe extern "C" fn init(arg_len: u32) -> u32 {
    if arg_len == 0 {
        abi::wrap_call(0, |(): ()| STATE.init(Init { initial_balances: alloc::vec::Vec::new() }))
    } else {
        abi::wrap_call(arg_len, |args: Init| STATE.init(args))
    }
}

/// Token name.
#[no_mangle]
unsafe extern "C" fn name(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(): ()| Drc20::name())
}

/// Token symbol.
#[no_mangle]
unsafe extern "C" fn symbol(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(): ()| Drc20::symbol())
}

/// Token decimals.
#[no_mangle]
unsafe extern "C" fn decimals(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(): ()| Drc20::decimals())
}

/// Total token supply.
#[no_mangle]
unsafe extern "C" fn total_supply(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(): ()| STATE.total_supply())
}

/// Balance of an account.
#[no_mangle]
unsafe extern "C" fn balance_of(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |args: BalanceOf| STATE.balance_of(args))
}

/// Allowance set by owner for spender.
#[no_mangle]
unsafe extern "C" fn allowance(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |args: Allowance| STATE.allowance(args))
}

/// Transfer tokens from the caller to a recipient.
#[no_mangle]
unsafe extern "C" fn transfer(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |args: TransferCall| STATE.transfer(args))
}

/// Approve a spender.
#[no_mangle]
unsafe extern "C" fn approve(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |args: ApproveCall| STATE.approve(args))
}

/// Transfer tokens using allowance.
#[no_mangle]
unsafe extern "C" fn transfer_from(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |args: TransferFromCall| STATE.transfer_from(args))
}
