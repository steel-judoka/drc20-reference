// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Dusk Forge

//! Data-driver for the DRC20 reference contract.

#![no_std]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::pedantic)]
#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use dusk_data_driver::{
    json_to_rkyv,
    rkyv_to_json,
    rkyv_to_json_u64,
    ConvertibleContract,
    Error,
    JsonValue,
};

use drc20_types::{events, Allowance, ApproveCall, BalanceOf, Init, TransferCall, TransferFromCall};

/// Contract driver for encoding and decoding calls + events.
#[derive(Default)]
pub struct ContractDriver;

impl ConvertibleContract for ContractDriver {
    fn encode_input_fn(&self, fn_name: &str, json: &str) -> Result<Vec<u8>, Error> {
        match fn_name {
            "name" | "symbol" | "decimals" | "total_supply" => json_to_rkyv::<()>(json),
            "init" => json_to_rkyv::<Init>(json),
            "balance_of" => json_to_rkyv::<BalanceOf>(json),
            "allowance" => json_to_rkyv::<Allowance>(json),
            "transfer" => json_to_rkyv::<TransferCall>(json),
            "approve" => json_to_rkyv::<ApproveCall>(json),
            "transfer_from" => json_to_rkyv::<TransferFromCall>(json),
            name => Err(Error::Unsupported(format!("fn_name {name}"))),
        }
    }

    fn decode_input_fn(&self, fn_name: &str, rkyv: &[u8]) -> Result<JsonValue, Error> {
        match fn_name {
            "name" | "symbol" | "decimals" | "total_supply" => rkyv_to_json::<()>(rkyv),
            "init" => rkyv_to_json::<Init>(rkyv),
            "balance_of" => rkyv_to_json::<BalanceOf>(rkyv),
            "allowance" => rkyv_to_json::<Allowance>(rkyv),
            "transfer" => rkyv_to_json::<TransferCall>(rkyv),
            "approve" => rkyv_to_json::<ApproveCall>(rkyv),
            "transfer_from" => rkyv_to_json::<TransferFromCall>(rkyv),
            name => Err(Error::Unsupported(format!("fn_name {name}"))),
        }
    }

    fn decode_output_fn(&self, fn_name: &str, rkyv: &[u8]) -> Result<JsonValue, Error> {
        match fn_name {
            // state-changing
            "init" | "transfer" | "approve" | "transfer_from" => Ok(JsonValue::Null),

            // metadata
            "name" | "symbol" => rkyv_to_json::<String>(rkyv),
            "decimals" => rkyv_to_json::<u8>(rkyv),

            // u64 outputs (use u64-safe JSON helper)
            "total_supply" | "balance_of" | "allowance" => rkyv_to_json_u64(rkyv),

            name => Err(Error::Unsupported(format!("fn_name {name}"))),
        }
    }

    fn decode_event(&self, event_name: &str, rkyv: &[u8]) -> Result<JsonValue, Error> {
        match event_name {
            events::Transfer::TOPIC => rkyv_to_json::<events::Transfer>(rkyv),
            events::Approval::TOPIC => rkyv_to_json::<events::Approval>(rkyv),
            event => Err(Error::Unsupported(format!("event {event}"))),
        }
    }

    fn get_schema(&self) -> String {
        // Optional: return schema description for tooling.
        String::new()
    }
}

#[cfg(all(target_family = "wasm", feature = "ffi"))]
dusk_data_driver::generate_wasm_entrypoint!(ContractDriver);
