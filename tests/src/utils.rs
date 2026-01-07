// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Dusk Forge

//! Utilities for DRC20 VM tests.
//!
//! These helpers intentionally mimic the serialization/deserialization
//! behavior that Piecrust uses (rkyv, 32-bit archives).

use bytecheck::CheckBytes;
use dusk_core::abi::StandardBufSerializer;
use dusk_core::signatures::bls::PublicKey as AccountPublicKey;
use dusk_core::transfer::moonlight::AccountData;
use dusk_core::transfer::TRANSFER_CONTRACT;
use dusk_vm::{Error as VMError, Session};
use rkyv::ser::serializers::{
    BufferScratch, BufferSerializer, CompositeSerializer,
};
use rkyv::ser::Serializer;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{
    check_archived_root, Archive, Deserialize, Infallible, Serialize,
};

const GAS_LIMIT: u64 = 0x10_000_000;

/// Deserialize function return data using `rkyv`.
///
/// This matches the validation/deserialization behavior used by Piecrust.
pub fn rkyv_deserialize<R>(serialized: impl AsRef<[u8]>) -> R
where
    R: Archive,
    R::Archived: Deserialize<R, Infallible>
        + for<'b> CheckBytes<DefaultValidator<'b>>,
{
    let archived = check_archived_root::<R>(serialized.as_ref())
        .expect("Failed to deserialize archived root");
    archived
        .deserialize(&mut Infallible)
        .expect("Failed to deserialize using rkyv")
}

/// Serialize function call arguments using `rkyv`.
///
/// This matches the argument serialization used by Piecrust.
pub fn rkyv_serialize<A>(fn_arg: &A) -> Vec<u8>
where
    A: for<'b> Serialize<StandardBufSerializer<'b>>,
    A::Archived: for<'b> CheckBytes<DefaultValidator<'b>>,
{
    // Scratch-space and page-size values taken from piecrust-uplink.
    const SCRATCH_SPACE: usize = 1024;
    const PAGE_SIZE: usize = 0x1000;

    let mut scratch_buf = [0u8; SCRATCH_SPACE];
    let scratch = BufferScratch::new(&mut scratch_buf);

    let mut buffer = [0u8; PAGE_SIZE];
    let ser = BufferSerializer::new(&mut buffer[..]);
    let mut ser = CompositeSerializer::new(ser, scratch, Infallible);

    ser.serialize_value(fn_arg)
        .expect("Failed to rkyv serialize fn_arg");
    let pos = ser.pos();

    buffer[..pos].to_vec()
}

/// Query the chain id from the genesis transfer contract.
pub fn chain_id(session: &mut Session) -> Result<u8, VMError> {
    session
        .call(TRANSFER_CONTRACT, "chain_id", &(), GAS_LIMIT)
        .map(|r| r.data)
}

/// Query moonlight account info from the genesis transfer contract.
pub fn account(
    session: &mut Session,
    pk: &AccountPublicKey,
) -> Result<AccountData, VMError> {
    session
        .call(TRANSFER_CONTRACT, "account", pk, GAS_LIMIT)
        .map(|r| r.data)
}
