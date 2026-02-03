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

#[dusk_forge::contract]
mod drc20 {
    use alloc::collections::BTreeMap;
    use alloc::string::String;

    use dusk_core::abi;

    use drc20_types::{
        error,
        events,
        Account,
        Allowance,
        ApproveCall,
        BalanceOf,
        Init,
        TransferCall,
        TransferFromCall,
        ZERO_ADDRESS,
    };

    /// DRC20 contract state.
    pub struct Drc20 {
        initialized: bool,
        balances: BTreeMap<Account, u64>,
        allowances: BTreeMap<Account, BTreeMap<Account, u64>>,
        supply: u64,
    }

    impl Drc20 {
        /// Create a new (empty) token state.
        pub const fn new() -> Self {
            Self {
                initialized: false,
                balances: BTreeMap::new(),
                allowances: BTreeMap::new(),
                supply: 0,
            }
        }

        /// Initialize the token with an initial distribution.
        ///
        /// Intended to be called once at deployment. To deploy with an empty
        /// distribution, pass an `Init` value with an empty `initial_balances`.
        pub fn init(&mut self, args: Init) {
            assert!(!self.initialized, "{}", error::ALREADY_INITIALIZED);

            for entry in args.initial_balances {
                if entry.amount == 0 {
                    continue;
                }

                assert!(
                    entry.account != ZERO_ADDRESS,
                    "{}",
                    error::ZERO_ADDRESS_NOT_ALLOWED
                );

                let bal = self.balances.entry(entry.account).or_insert(0);
                *bal = bal.checked_add(entry.amount).expect(error::SUPPLY_OVERFLOW);

                self.supply = self
                    .supply
                    .checked_add(entry.amount)
                    .expect(error::SUPPLY_OVERFLOW);

                // ERC20-style mint event: from ZERO_ADDRESS.
                abi::emit(
                    events::Transfer::TOPIC,
                    events::Transfer {
                        from: ZERO_ADDRESS,
                        to: entry.account,
                        value: entry.amount,
                    },
                );
            }

            self.initialized = true;
        }

        // --- Metadata (compile-time constants in this reference) ---

        /// Token name.
        pub fn name() -> String {
            String::from("DRC20 Reference Token")
        }

        /// Token symbol.
        pub fn symbol() -> String {
            String::from("DRC20")
        }

        /// Token decimals.
        pub fn decimals() -> u8 {
            18
        }

        // --- Views ---

        /// Total supply.
        pub fn total_supply(&self) -> u64 {
            self.supply
        }

        /// Balance of an account.
        pub fn balance_of(&self, args: BalanceOf) -> u64 {
            self.balances.get(&args.account).copied().unwrap_or(0)
        }

        /// Allowance from `owner` to `spender`.
        pub fn allowance(&self, args: Allowance) -> u64 {
            self.allowances
                .get(&args.owner)
                .and_then(|m| m.get(&args.spender).copied())
                .unwrap_or(0)
        }

        // --- State changes ---

        /// Transfer from caller.
        pub fn transfer(&mut self, args: TransferCall) {
            let from = sender_account();
            self.transfer_internal(from, args.to, args.value);
        }

        /// Approve allowance.
        pub fn approve(&mut self, args: ApproveCall) {
            let owner = sender_account();

            assert!(
                args.spender != ZERO_ADDRESS,
                "{}",
                error::ZERO_ADDRESS_NOT_ALLOWED
            );

            let owner_allowances = self.allowances.entry(owner).or_default();
            owner_allowances.insert(args.spender, args.value);

            abi::emit(
                events::Approval::TOPIC,
                events::Approval {
                    owner,
                    spender: args.spender,
                    value: args.value,
                },
            );
        }

        /// Transfer using allowance.
        pub fn transfer_from(&mut self, args: TransferFromCall) {
            let spender = sender_account();

            let current = self.allowance(Allowance {
                owner: args.owner,
                spender,
            });

            assert!(current >= args.value, "{}", error::ALLOWANCE_TOO_LOW);

            // Decrease allowance.
            let owner_allowances = self.allowances.entry(args.owner).or_default();
            owner_allowances.insert(spender, current - args.value);

            self.transfer_internal(args.owner, args.to, args.value);
        }

        fn transfer_internal(&mut self, from: Account, to: Account, value: u64) {
            assert!(to != ZERO_ADDRESS, "{}", error::ZERO_ADDRESS_NOT_ALLOWED);

            // ERC20 allows zero-value transfers and expects a Transfer event.
            let from_balance = self.balances.get(&from).copied().unwrap_or(0);
            assert!(from_balance >= value, "{}", error::BALANCE_TOO_LOW);

            if value != 0 {
                // Update sender balance.
                if let Some(bal) = self.balances.get_mut(&from) {
                    *bal = from_balance - value;
                    if *bal == 0 {
                        self.balances.remove(&from);
                    }
                }

                // Update receiver balance.
                let to_entry = self.balances.entry(to).or_insert(0);
                *to_entry = to_entry.checked_add(value).expect(error::SUPPLY_OVERFLOW);
            }

            abi::emit(events::Transfer::TOPIC, events::Transfer { from, to, value });
        }
    }

    /// Determines the "msg.sender" of the current call.
    ///
    /// - If called directly by an external account: returns `Account::External(public_sender)`
    /// - If called by another contract: returns `Account::Contract(caller)`
    ///
    /// # Panics
    ///
    /// - If no public sender is available (shielded transactions are not supported)
    /// - If a contract call has no caller (should be impossible)
    fn sender_account() -> Account {
        // Note: query calls have an empty callstack and should never reach here.
        if abi::callstack().len() == 1 {
            Account::External(abi::public_sender().expect(error::SHIELDED_NOT_SUPPORTED))
        } else {
            Account::Contract(abi::caller().expect("missing caller"))
        }
    }
}
