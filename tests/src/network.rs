// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Dusk Forge

//! VM harness for DRC20 tests.
//!
//! The goal of this harness is to run the DRC20 contract in a way that is as
//! close as possible to how it is executed on-chain:
//!
//! - Deploy the genesis transfer + stake contracts
//! - Fund Moonlight accounts so they can pay gas
//! - Execute calls as Moonlight transactions through the transfer contract
//!   so `abi::public_sender()` is available

use bytecheck::CheckBytes;
use dusk_core::abi::{ContractError, StandardBufSerializer};
use dusk_core::abi::{ContractId, CONTRACT_ID_BYTES};
use dusk_core::signatures::bls::{
    PublicKey as AccountPublicKey, SecretKey as AccountSecretKey,
};
use dusk_core::stake::STAKE_CONTRACT;
use dusk_core::transfer::data::ContractCall;
use dusk_core::transfer::moonlight::AccountData;
use dusk_core::transfer::{Transaction, TRANSFER_CONTRACT};
use dusk_core::LUX;
use dusk_vm::{execute, ExecutionConfig};
use dusk_vm::{CallReceipt, ContractData, Error as VMError, Session, VM};
use rkyv::validation::validators::DefaultValidator;
use rkyv::{Archive, Deserialize, Infallible, Serialize};

use crate::utils::{account, chain_id, rkyv_deserialize, rkyv_serialize};

const TRANSFER_BYTECODE: &[u8] =
    include_bytes!("../genesis-contracts/transfer_contract.wasm");
const STAKE_BYTECODE: &[u8] =
    include_bytes!("../genesis-contracts/stake_contract.wasm");

const ZERO_ADDRESS: ContractId =
    ContractId::from_bytes([0; CONTRACT_ID_BYTES]);

const GAS_LIMIT: u64 = 0x10000000;
const CHAIN_ID: u8 = 0x1;
const NO_CONFIG: ExecutionConfig = ExecutionConfig::DEFAULT;

type Result<T, Error = VMError> = core::result::Result<T, Error>;

/// A VM session that behaves similar to a network VM.
///
/// State-changing calls are executed as transactions through the genesis
/// transfer contract.
pub struct NetworkSession {
    pub(crate) session: Session,
    pub(crate) config: ExecutionConfig,
}

impl NetworkSession {
    /// Deploy a contract into the session.
    pub fn deploy<'a, A, D>(
        &mut self,
        bytecode: &[u8],
        deploy_data: D,
    ) -> Result<ContractId>
    where
        A: 'a + for<'b> Serialize<StandardBufSerializer<'b>>,
        D: Into<ContractData<'a, A>>,
    {
        self.session.deploy(bytecode, deploy_data, u64::MAX)
    }

    /// Directly call a contract (no gas payment, bypass transfer contract).
    ///
    /// Suitable for getters (`name`, `balance_of`, etc.).
    pub fn direct_call<A, R>(
        &mut self,
        contract: ContractId,
        fn_name: &str,
        fn_arg: &A,
    ) -> Result<CallReceipt<R>, ContractError>
    where
        A: for<'b> Serialize<StandardBufSerializer<'b>>,
        A::Archived: for<'b> CheckBytes<DefaultValidator<'b>>,
        R: Archive,
        R::Archived: Deserialize<R, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        self.session
            .call::<_, R>(contract, fn_name, fn_arg, u64::MAX)
            .map_err(|e| match e {
                VMError::Panic(panic_msg) => ContractError::Panic(panic_msg),
                VMError::OutOfGas => ContractError::OutOfGas,
                other => panic!("Unknown VM error: {other}"),
            })
    }

    /// Execute a contract call as a Moonlight transaction via the transfer contract.
    ///
    /// This is the standard on-chain execution path and provides `public_sender`.
    pub fn icc_transaction<A, R>(
        &mut self,
        moonlight_sk: &AccountSecretKey,
        contract: ContractId,
        fn_name: &str,
        fn_arg: &A,
    ) -> Result<CallReceipt<R>, ContractError>
    where
        A: for<'b> Serialize<StandardBufSerializer<'b>>,
        A::Archived: for<'b> CheckBytes<DefaultValidator<'b>>,
        R: Archive,
        R::Archived: Deserialize<R, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        let contract_call = ContractCall {
            contract,
            fn_name: String::from(fn_name),
            fn_args: rkyv_serialize(fn_arg),
        };

        let moonlight_pk = AccountPublicKey::from(moonlight_sk);

        let AccountData { nonce, .. } =
            account(&mut self.session, &moonlight_pk)
                .expect("Getting moonlight account should succeed");

        let tx = Transaction::moonlight(
            moonlight_sk,
            None,
            0,
            0,
            GAS_LIMIT,
            LUX,
            nonce + 1,
            CHAIN_ID,
            Some(contract_call),
        )
        .expect("Creating moonlight transaction should succeed");

        let receipt = execute(&mut self.session, &tx, &self.config)
            .unwrap_or_else(|e| {
                panic!("Executing tx should succeed: {e:?}")
            });

        match receipt.data {
            Ok(serialized) => Ok(CallReceipt {
                gas_limit: receipt.gas_limit,
                gas_spent: receipt.gas_spent,
                events: receipt.events,
                call_tree: receipt.call_tree,
                data: rkyv_deserialize(&serialized),
            }),
            Err(e) => Err(e),
        }
    }

    /// Create a fresh VM and deploy the genesis transfer + stake contracts.
    ///
    /// The given public keys are funded with the specified balance so they can
    /// pay for contract calls.
    pub fn instantiate(pks_to_fund: Vec<(&AccountPublicKey, u64)>) -> Self {
        let vm = VM::ephemeral().expect("Creating VM should succeed");

        let mut session = VM::genesis_session(&vm, 1);

        // Deploy transfer contract
        session
            .deploy(
                TRANSFER_BYTECODE,
                ContractData::builder()
                    .owner(ZERO_ADDRESS.to_bytes())
                    .contract_id(TRANSFER_CONTRACT),
                GAS_LIMIT,
            )
            .expect("Deploying transfer contract should succeed");

        // Deploy stake contract
        session
            .deploy(
                STAKE_BYTECODE,
                ContractData::builder()
                    .owner(ZERO_ADDRESS.to_bytes())
                    .contract_id(STAKE_CONTRACT),
                GAS_LIMIT,
            )
            .expect("Deploying stake contract should succeed");

        // Fund public keys with DUSK so they can pay gas
        for (&pk, val) in &pks_to_fund {
            session
                .call::<_, ()>(
                    TRANSFER_CONTRACT,
                    "add_account_balance",
                    &(pk, *val),
                    GAS_LIMIT,
                )
                .expect("Funding moonlight account should succeed");
        }

        // Commit the first block. This sets the block height for subsequent ops.
        let base = session.commit().expect("Committing should succeed");

        let mut session = vm
            .session(base, CHAIN_ID, 1)
            .expect("Instantiating new session should succeed");

        // Sanity: assert balances and chain id
        for (pk, val) in pks_to_fund {
            let acc = account(&mut session, pk)
                .expect("Getting moonlight account should succeed");
            assert_eq!(acc.balance, val);
            assert_eq!(acc.nonce, 0);
        }

        let cid = chain_id(&mut session)
            .expect("Getting chain id should succeed");
        assert_eq!(cid, CHAIN_ID);

        // Enable public sender tracking.
        let mut config = NO_CONFIG;
        config.with_public_sender = true;

        Self { session, config }
    }
}
