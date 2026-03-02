//! Permit digest helpers.
//!
//! These functions produce the canonical byte sequences that must be hashed
//! (via `abi::hash` on-chain, or Poseidon off-chain) to build a permit digest.
//! Keeping the layout here guarantees that the contract, data-driver, and any
//! off-chain signer always agree on the same encoding.

use alloc::vec::Vec;
use dusk_core::abi::ContractId;
use dusk_core::signatures::bls::PublicKey as BlsPublicKey;
use dusk_core::BlsScalar;

use crate::Account;

/// Raw bytes whose hash is the EIP-712-style domain separator for permits.
///
/// `hash(domain_separator_bytes(contract_id, chain_id))` produces the
/// `BlsScalar` domain separator used inside [`permit_digest_bytes`].
pub fn domain_separator_bytes(
    contract_id: &ContractId,
    chain_id: u8,
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend(contract_id.as_bytes());
    buf.push(chain_id);
    buf.extend(b"DRC20Permit");
    buf
}

/// Raw bytes whose hash is the permit digest that the owner signs.
///
/// `domain_sep` must be the **hashed** domain separator, i.e.
/// `hash(domain_separator_bytes(...))`.
///
/// The resulting digest is `hash(permit_digest_bytes(...)).to_bytes()`.
pub fn permit_digest_bytes(
    domain_sep: &BlsScalar,
    owner: &BlsPublicKey,
    spender: &Account,
    value: u64,
    nonce: u64,
    deadline: u64,
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend(&domain_sep.to_bytes());
    buf.extend(&owner.to_raw_bytes());
    buf.extend(&spender.to_bytes());
    buf.extend(&value.to_le_bytes());
    buf.extend(&nonce.to_le_bytes());
    buf.extend(&deadline.to_le_bytes());
    buf
}
