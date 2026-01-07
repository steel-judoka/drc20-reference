// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Dusk Forge

//! Helper contract used only by the DRC20 test-suite.
//!
//! It allows exercising the "contract caller" path of `sender_account()`.

#![no_std]
#![cfg(target_family = "wasm")]

use dusk_core::abi;
use dusk_core::abi::ContractId;

use drc20_types::{Account, TransferCall};

/// No-op init.
#[no_mangle]
unsafe extern "C" fn init(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(): ()| ())
}

/// Call `transfer` on a DRC20 token contract.
///
/// Args: `(token_contract_id, to, value)`
#[no_mangle]
unsafe extern "C" fn call_transfer(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(token, to, value): (ContractId, Account, u64)| {
        let args = TransferCall { to, value };
        abi::call::<_, ()>(token, "transfer", &args)
            .expect("DRC20 test-caller: transfer call failed");
    })
}
