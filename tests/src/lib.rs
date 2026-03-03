// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Dusk Forge

//! DRC20 spec tests.

pub mod network;
pub mod utils;

#[cfg(test)]
mod spec {
	    use std::cell::RefCell;
    use std::fs;
    use std::path::PathBuf;

    use dusk_core::abi::{ContractError, ContractId};
    use dusk_core::dusk;
    use dusk_core::signatures::bls::{
        PublicKey as AccountPublicKey, SecretKey as AccountSecretKey,
    };
    use dusk_core::BlsScalar;
    use dusk_vm::host_queries::hash;
    use dusk_vm::ContractData;

    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use drc20_types::{
        error,
        events,
        permit,
        Account,
        Allowance,
        ApproveCall,
        BalanceOf,
        Init,
        InitBalance,
        PermitCall,
        TransferCall,
        TransferFromCall,
        ZERO_ADDRESS,
    };

    use super::network::NetworkSession;

    const DEPLOYER: [u8; 64] = [0u8; 64];

    const TOKEN_ID: ContractId = ContractId::from_bytes([1; 32]);
    const CALLER_ID: ContractId = ContractId::from_bytes([2; 32]);

    const CHAIN_ID: u8 = 0x1;
    const MOONLIGHT_BALANCE: u64 = dusk(1_000.0);

    const INITIAL_ALICE: u64 = 1_000;

    /// Load a wasm artifact from `target/wasm32-unknown-unknown/release/`.
    ///
    /// Tries `*_opt.wasm` first, falling back to the non-optimized `*.wasm`.
    fn load_wasm(crate_base_name: &str) -> Vec<u8> {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("tests crate should live one level under workspace root")
            .to_path_buf();

        let release_dir = workspace_root
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release");

        let opt = release_dir.join(format!("{crate_base_name}_opt.wasm"));
        let plain = release_dir.join(format!("{crate_base_name}.wasm"));

        if opt.exists() {
            fs::read(&opt).unwrap_or_else(|e| {
                panic!("Failed to read wasm artifact {opt:?}: {e}")
            })
        } else if plain.exists() {
            fs::read(&plain).unwrap_or_else(|e| {
                panic!("Failed to read wasm artifact {plain:?}: {e}")
            })
        } else {
            panic!(
                "Missing wasm artifact for `{crate_base_name}`.\n\n\
                 Build the wasm first, for example:\n\
                   make wasm\n\
                 or (with optimization):\n\
                   make wasm-opt\n\n\
                 Also build the test caller contract:\n\
                   make test-caller-wasm\n"
            );
        }
    }

    struct TestContext {
	        /// VM session wrapped in `RefCell` so test helpers can take `&self`.
	        ///
	        /// This avoids borrow-checker issues in tests where we want to pass
	        /// references to keys stored in the context (e.g. `&ctx.sk_alice`) at
	        /// the same time as performing state changes.
	        net: RefCell<NetworkSession>,

        sk_alice: AccountSecretKey,
        pk_alice: AccountPublicKey,

        sk_bob: AccountSecretKey,
        pk_bob: AccountPublicKey,

        sk_carol: AccountSecretKey,
        pk_carol: AccountPublicKey,
    }

    impl TestContext {
        fn new() -> Self {
            let mut rng = StdRng::seed_from_u64(0xA11CE);
            let sk_alice = AccountSecretKey::random(&mut rng);
            let pk_alice = AccountPublicKey::from(&sk_alice);

            let mut rng = StdRng::seed_from_u64(0xB0B);
            let sk_bob = AccountSecretKey::random(&mut rng);
            let pk_bob = AccountPublicKey::from(&sk_bob);

            let mut rng = StdRng::seed_from_u64(0xCA901);
            let sk_carol = AccountSecretKey::random(&mut rng);
            let pk_carol = AccountPublicKey::from(&sk_carol);

            let mut net = NetworkSession::instantiate(vec![
                (&pk_alice, MOONLIGHT_BALANCE),
                (&pk_bob, MOONLIGHT_BALANCE),
                (&pk_carol, MOONLIGHT_BALANCE),
            ]);

            // Deploy DRC20
            let token_wasm = load_wasm("drc20");
            net.deploy(
                &token_wasm,
                ContractData::builder()
                    .owner(DEPLOYER)
                    .init_arg(&Init {
                        initial_balances: vec![InitBalance {
                            account: Account::External(pk_alice),
                            amount: INITIAL_ALICE,
                        }],
                    })
                    .contract_id(TOKEN_ID),
            )
            .expect("Deploying DRC20 should succeed");

            // Deploy test-caller contract (used to test contract-origin calls)
            let caller_wasm = load_wasm("drc20_test_caller");
            net.deploy(
                &caller_wasm,
                ContractData::builder()
                    .owner(DEPLOYER)
                    .init_arg(&())
                    .contract_id(CALLER_ID),
            )
            .expect("Deploying DRC20 test-caller should succeed");

	            Self {
	                net: RefCell::new(net),
                sk_alice,
                pk_alice,
                sk_bob,
                pk_bob,
                sk_carol,
                pk_carol,
            }
        }

        fn alice(&self) -> Account {
            Account::External(self.pk_alice)
        }

        fn bob(&self) -> Account {
            Account::External(self.pk_bob)
        }

        fn carol(&self) -> Account {
            Account::External(self.pk_carol)
        }

        fn caller_contract(&self) -> Account {
            Account::Contract(CALLER_ID)
        }

        // --- Views ---

	        fn name(&self) -> String {
	            self.net
	                .borrow_mut()
	                .direct_call::<(), String>(TOKEN_ID, "name", &())
                .expect("name() call should succeed")
                .data
        }

	        fn symbol(&self) -> String {
	            self.net
	                .borrow_mut()
	                .direct_call::<(), String>(TOKEN_ID, "symbol", &())
                .expect("symbol() call should succeed")
                .data
        }

	        fn decimals(&self) -> u8 {
	            self.net
	                .borrow_mut()
	                .direct_call::<(), u8>(TOKEN_ID, "decimals", &())
                .expect("decimals() call should succeed")
                .data
        }

	        fn total_supply(&self) -> u64 {
	            self.net
	                .borrow_mut()
	                .direct_call::<(), u64>(TOKEN_ID, "total_supply", &())
                .expect("total_supply() call should succeed")
                .data
        }

	        fn balance_of(&self, account: Account) -> u64 {
	            self.net
	                .borrow_mut()
	                .direct_call::<BalanceOf, u64>(
                    TOKEN_ID,
                    "balance_of",
                    &BalanceOf { account },
                )
                .expect("balance_of() call should succeed")
                .data
        }

	        fn allowance(&self, owner: Account, spender: Account) -> u64 {
	            self.net
	                .borrow_mut()
	                .direct_call::<Allowance, u64>(
                    TOKEN_ID,
                    "allowance",
                    &Allowance { owner, spender },
                )
                .expect("allowance() call should succeed")
                .data
        }

        fn permit_nonces(&self, owner: Account) -> u64 {
            self.net
                .borrow_mut()
                .direct_call::<Account, u64>(TOKEN_ID, "permit_nonces", &owner)
                .expect("permit_nonces() call should succeed")
                .data
        }

        fn contract_domain_separator(&self) -> BlsScalar {
            self.net
                .borrow_mut()
                .direct_call::<(), BlsScalar>(TOKEN_ID, "domain_separator", &())
                .expect("domain_separator() call should succeed")
                .data
        }

        // --- State changes ---

	        fn transfer(
	            &self,
            sk: &AccountSecretKey,
            to: Account,
            value: u64,
        ) -> Result<dusk_vm::CallReceipt<()>, ContractError> {
	            self.net
	                .borrow_mut()
                .icc_transaction(sk, TOKEN_ID, "transfer", &TransferCall { to, value })
        }

	        fn approve(
	            &self,
            sk: &AccountSecretKey,
            spender: Account,
            value: u64,
        ) -> Result<dusk_vm::CallReceipt<()>, ContractError> {
	            self.net
	                .borrow_mut()
                .icc_transaction(
                    sk,
                    TOKEN_ID,
                    "approve",
                    &ApproveCall { spender, value },
                )
        }

	        fn transfer_from(
	            &self,
            sk: &AccountSecretKey,
            owner: Account,
            to: Account,
            value: u64,
        ) -> Result<dusk_vm::CallReceipt<()>, ContractError> {
	            self.net.borrow_mut().icc_transaction(
                sk,
                TOKEN_ID,
                "transfer_from",
                &TransferFromCall { owner, to, value },
            )
        }

	        fn call_caller_transfer(
	            &self,
            sk: &AccountSecretKey,
            token: ContractId,
            to: Account,
            value: u64,
        ) -> Result<dusk_vm::CallReceipt<()>, ContractError> {
	            self.net
	                .borrow_mut()
                .icc_transaction(sk, CALLER_ID, "call_transfer", &(token, to, value))
        }

        fn permit(
            &self,
            sk: &AccountSecretKey,
            owner: dusk_core::signatures::bls::PublicKey,
            spender: Account,
            value: u64,
            deadline: u64,
            signature: dusk_core::signatures::bls::Signature,
        ) -> Result<dusk_vm::CallReceipt<()>, ContractError> {
            self.net
                .borrow_mut()
                .icc_transaction(
                    sk,
                    TOKEN_ID,
                    "permit",
                    &PermitCall {
                        owner,
                        spender,
                        value,
                        deadline,
                        signature,
                    },
                )
        }
    }

    fn domain_separator() -> BlsScalar {
        hash(permit::domain_separator_bytes(&TOKEN_ID, CHAIN_ID))
    }

    fn build_permit_digest(
        domain_sep: &BlsScalar,
        owner: &AccountPublicKey,
        spender: &Account,
        value: u64,
        nonce: u64,
        deadline: u64,
    ) -> Vec<u8> {
        hash(permit::permit_digest_bytes(domain_sep, owner, spender, value, nonce, deadline))
            .to_bytes()
            .to_vec()
    }

    // ---------------------------------------------------------------------
    // Spec tests
    // ---------------------------------------------------------------------

    #[test]
    fn metadata_getters_work() {
	        let ctx = TestContext::new();
        assert_eq!(ctx.name(), "DRC20 Reference Token");
        assert_eq!(ctx.symbol(), "DRC20");
        assert_eq!(ctx.decimals(), 18);
    }

    #[test]
    fn init_distribution_sets_balances_and_total_supply() {
	        let ctx = TestContext::new();

        assert_eq!(ctx.total_supply(), INITIAL_ALICE);
        assert_eq!(ctx.balance_of(ctx.alice()), INITIAL_ALICE);
        assert_eq!(ctx.balance_of(ctx.bob()), 0);
        assert_eq!(ctx.balance_of(ctx.caller_contract()), 0);

        // Unknown accounts should return 0
        let mut rng = StdRng::seed_from_u64(0xD00D);
        let sk = AccountSecretKey::random(&mut rng);
        let pk = AccountPublicKey::from(&sk);
        assert_eq!(ctx.balance_of(Account::External(pk)), 0);

        // Default allowance should be 0
        assert_eq!(ctx.allowance(ctx.alice(), ctx.bob()), 0);
    }

    #[test]
    fn transfer_updates_balances_and_emits_transfer_event() {
	        let ctx = TestContext::new();

        let amount = 250;
        let receipt = ctx
            .transfer(&ctx.sk_alice, ctx.bob(), amount)
            .expect("transfer() should succeed");

        assert_eq!(ctx.balance_of(ctx.alice()), INITIAL_ALICE - amount);
        assert_eq!(ctx.balance_of(ctx.bob()), amount);

        // Find the DRC20 Transfer event
        let mut seen = false;
        for event in receipt.events.iter() {
            if event.topic == events::Transfer::TOPIC {
                let e = rkyv::from_bytes::<events::Transfer>(&event.data)
                    .expect("transfer event should deserialize");

                assert_eq!(e.from, ctx.alice());
                assert_eq!(e.to, ctx.bob());
                assert_eq!(e.value, amount);
                seen = true;
            }
        }
        assert!(seen, "expected a transfer event");
    }

    #[test]
    fn zero_value_transfer_is_allowed_and_emits_event() {
	        let ctx = TestContext::new();

        // Bob has 0 tokens but can still do a 0-value transfer.
        let receipt = ctx
            .transfer(&ctx.sk_bob, ctx.alice(), 0)
            .expect("0-value transfer should succeed");

        assert_eq!(ctx.balance_of(ctx.alice()), INITIAL_ALICE);
        assert_eq!(ctx.balance_of(ctx.bob()), 0);

        let mut seen = false;
        for event in receipt.events.iter() {
            if event.topic == events::Transfer::TOPIC {
                let e = rkyv::from_bytes::<events::Transfer>(&event.data)
                    .expect("transfer event should deserialize");

                assert_eq!(e.from, ctx.bob());
                assert_eq!(e.to, ctx.alice());
                assert_eq!(e.value, 0);
                seen = true;
            }
        }
        assert!(seen, "expected a transfer event for 0-value transfer");
    }

    #[test]
    fn transfer_fails_if_balance_too_low() {
	        let ctx = TestContext::new();

        let receipt = ctx.transfer(&ctx.sk_bob, ctx.alice(), 1);

        if let ContractError::Panic(msg) = receipt.unwrap_err() {
            assert_eq!(msg, error::BALANCE_TOO_LOW);
        } else {
            panic!("Expected a panic error");
        }
    }

    #[test]
    fn approve_sets_allowance_and_emits_approval_event() {
	        let ctx = TestContext::new();

        let allowance = 123;
        let receipt = ctx
            .approve(&ctx.sk_alice, ctx.bob(), allowance)
            .expect("approve() should succeed");

        assert_eq!(ctx.allowance(ctx.alice(), ctx.bob()), allowance);

        let mut seen = false;
        for event in receipt.events.iter() {
            if event.topic == events::Approval::TOPIC {
                let e = rkyv::from_bytes::<events::Approval>(&event.data)
                    .expect("approval event should deserialize");

                assert_eq!(e.owner, ctx.alice());
                assert_eq!(e.spender, ctx.bob());
                assert_eq!(e.value, allowance);
                seen = true;
            }
        }
        assert!(seen, "expected an approval event");
    }

    #[test]
    fn approve_overwrites_existing_allowance() {
	        let ctx = TestContext::new();

        ctx.approve(&ctx.sk_alice, ctx.bob(), 10)
            .expect("approve should succeed");
        assert_eq!(ctx.allowance(ctx.alice(), ctx.bob()), 10);

        ctx.approve(&ctx.sk_alice, ctx.bob(), 999)
            .expect("approve overwrite should succeed");
        assert_eq!(ctx.allowance(ctx.alice(), ctx.bob()), 999);
    }

    #[test]
    fn transfer_from_spends_allowance_and_decreases_it() {
	        let ctx = TestContext::new();

        // Alice approves Bob
        ctx.approve(&ctx.sk_alice, ctx.bob(), 100)
            .expect("approve should succeed");

        // Bob transfers 60 from Alice to Carol
        let receipt = ctx
            .transfer_from(&ctx.sk_bob, ctx.alice(), ctx.carol(), 60)
            .expect("transfer_from should succeed");

        assert_eq!(ctx.balance_of(ctx.alice()), INITIAL_ALICE - 60);
        assert_eq!(ctx.balance_of(ctx.carol()), 60);
        assert_eq!(ctx.allowance(ctx.alice(), ctx.bob()), 40);

        // Ensure the Transfer event is (from=Alice, to=Carol)
        let mut seen = false;
        for event in receipt.events.iter() {
            if event.topic == events::Transfer::TOPIC {
                let e = rkyv::from_bytes::<events::Transfer>(&event.data)
                    .expect("transfer event should deserialize");

                assert_eq!(e.from, ctx.alice());
                assert_eq!(e.to, ctx.carol());
                assert_eq!(e.value, 60);
                seen = true;
            }
        }
        assert!(seen, "expected a transfer event");
    }

    #[test]
    fn transfer_from_fails_if_allowance_too_low() {
	        let ctx = TestContext::new();

        ctx.approve(&ctx.sk_alice, ctx.bob(), 10)
            .expect("approve should succeed");

        let receipt = ctx.transfer_from(&ctx.sk_bob, ctx.alice(), ctx.carol(), 11);

        if let ContractError::Panic(msg) = receipt.unwrap_err() {
            assert_eq!(msg, error::ALLOWANCE_TOO_LOW);
        } else {
            panic!("Expected a panic error");
        }

        // Allowance should remain unchanged after failure
        assert_eq!(ctx.allowance(ctx.alice(), ctx.bob()), 10);
    }

    #[test]
    fn zero_address_is_rejected_for_transfer_and_approve() {
	        let ctx = TestContext::new();

        // transfer(to = ZERO_ADDRESS)
        let receipt = ctx.transfer(&ctx.sk_alice, ZERO_ADDRESS, 1);
        if let ContractError::Panic(msg) = receipt.unwrap_err() {
            assert_eq!(msg, error::ZERO_ADDRESS_NOT_ALLOWED);
        } else {
            panic!("Expected a panic error");
        }

        // approve(spender = ZERO_ADDRESS)
        let receipt = ctx.approve(&ctx.sk_alice, ZERO_ADDRESS, 1);
        if let ContractError::Panic(msg) = receipt.unwrap_err() {
            assert_eq!(msg, error::ZERO_ADDRESS_NOT_ALLOWED);
        } else {
            panic!("Expected a panic error");
        }
    }

    #[test]
    fn contract_calls_are_accounted_as_contract_sender() {
	        let ctx = TestContext::new();

        // Fund the caller contract with 100 tokens from Alice
        ctx.transfer(&ctx.sk_alice, ctx.caller_contract(), 100)
            .expect("funding caller contract should succeed");

        assert_eq!(ctx.balance_of(ctx.caller_contract()), 100);

        // Call the caller contract, which calls DRC20.transfer(...) internally.
        let receipt = ctx
            .call_caller_transfer(&ctx.sk_alice, TOKEN_ID, ctx.bob(), 60)
            .expect("caller contract transfer should succeed");

        assert_eq!(ctx.balance_of(ctx.caller_contract()), 40);
        assert_eq!(ctx.balance_of(ctx.bob()), 60);

        // Ensure the DRC20 Transfer event has from = caller contract
        let mut seen = false;
        for event in receipt.events.iter() {
            if event.topic == events::Transfer::TOPIC {
                let e = rkyv::from_bytes::<events::Transfer>(&event.data)
                    .expect("transfer event should deserialize");

                assert_eq!(e.from, ctx.caller_contract());
                assert_eq!(e.to, ctx.bob());
                assert_eq!(e.value, 60);
                seen = true;
            }
        }
        assert!(seen, "expected a transfer event from the caller contract");
    }

    #[test]
    fn panic_if_permit_expired() {
        let ctx = TestContext::new();

        let spender = ctx.bob();
        let value: u64 = 100;
        let nonce: u64 = 0;
        // block height 1 > deadline 0 -> should panic
        let deadline: u64 = 0;

        let digest = build_permit_digest(
            &domain_separator(),
            &ctx.pk_alice,
            &spender,
            value,
            nonce,
            deadline,
        );

        let sig = ctx.sk_alice.sign(&digest);

        let receipt = ctx.permit(
            &ctx.sk_alice,
            ctx.pk_alice,
            spender,
            value,
            deadline,
            sig,
        );

        if let ContractError::Panic(msg) = receipt.unwrap_err() {
            assert_eq!(msg, error::PERMIT_EXPIRED);
        } else {
            panic!("Expected a panic error");
        }
    }
    
    #[test]
    fn panic_if_permit_signature_is_invalid() {
        let ctx = TestContext::new();

        let spender = ctx.bob();
        let value: u64 = 100;
        let nonce: u64 = 0;
        let deadline: u64 = 100;

        let digest = build_permit_digest(
            &domain_separator(),
            &ctx.pk_alice,
            &spender,
            value,
            nonce,
            deadline,
        );

        // Original digest owner is Alice, but we sign with Bob's key.
        let sig = ctx.sk_bob.sign(&digest);

        // gas payer can be anyone, we use Alice for simplicity
        let receipt = ctx.permit(
            &ctx.sk_alice,
            ctx.pk_alice,
            spender,
            value,
            deadline,
            sig,
        );

        if let ContractError::Panic(msg) = receipt.unwrap_err() {
            assert_eq!(msg, error::INVALID_PERMIT_SIGNATURE);
        } else {
            panic!("Expected a panic error");
        }
    }

    #[test]
    fn permit_works_with_valid_signature() {
        let ctx = TestContext::new();

        let spender = ctx.bob();
        let value: u64 = 100;
        let nonce: u64 = 0;
        let deadline: u64 = 100;

        let digest = build_permit_digest(
            &domain_separator(),
            &ctx.pk_alice,
            &spender,
            value,
            nonce,
            deadline,
        );

        let sig = ctx.sk_alice.sign(&digest);

        let receipt = ctx.permit(
            &ctx.sk_alice,
            ctx.pk_alice,
            spender,
            value,
            deadline,
            sig,
        ).expect("permit should succeed");
        
        assert_eq!(ctx.allowance(ctx.alice(), ctx.bob()), value);
    }

    #[test]
    fn permit_nonces_is_zero_for_new_owner_and_increments_after_permit() {
        let ctx = TestContext::new();

        assert_eq!(ctx.permit_nonces(ctx.alice()), 0);
        assert_eq!(ctx.permit_nonces(ctx.bob()), 0);

        let spender = ctx.bob();
        let value: u64 = 50;
        let nonce: u64 = 0;
        let deadline: u64 = 100;

        let digest = build_permit_digest(
            &domain_separator(),
            &ctx.pk_alice,
            &spender,
            value,
            nonce,
            deadline,
        );
        let sig = ctx.sk_alice.sign(&digest);

        ctx.permit(
            &ctx.sk_alice,
            ctx.pk_alice,
            spender,
            value,
            deadline,
            sig,
        )
        .expect("permit should succeed");

        assert_eq!(ctx.permit_nonces(ctx.alice()), 1);
        assert_eq!(ctx.permit_nonces(ctx.bob()), 0);
    }

    #[test]
    fn domain_separator_view_matches_expected() {
        let ctx = TestContext::new();

        let from_contract = ctx.contract_domain_separator();
        let expected = domain_separator();

        assert_eq!(from_contract.to_bytes(), expected.to_bytes());
    }
}
