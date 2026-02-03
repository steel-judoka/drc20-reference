# DRC20 - Public Fungible Token Standard for DuskDS

> **Status:** Reference implementation / draft: the standard and interface are still open for feedback.
> Please open an issue (or PR) with suggestions, edge-cases, or compatibility concerns.

A minimal ERC20-like fungible token reference implementation for the DuskDS network.

## Repo layout

- `contract/` — the on-chain DRC20 contract (WASM)
- `types/` — shared types (accounts, call args, events, error strings)
- `web/` — minimal admin/management UI (connect + read + send tx)

## Interface

### Views

- `name() -> String`
- `symbol() -> String`
- `decimals() -> u8`
- `total_supply() -> u64`
- `balance_of(BalanceOf) -> u64`
- `allowance(Allowance) -> u64`

### State-changing

- `transfer(TransferCall)`
- `approve(ApproveCall)`
- `transfer_from(TransferFromCall)`

### Events

- `transfer` (`events::Transfer`)
- `approval` (`events::Approval`)

## Build

### Prereqs

- Rust toolchain pinned in `rust-toolchain.toml`
- wasm target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- `jq` is used by Makefiles (`cargo metadata` parsing).
- Optional: `wasm-opt` (from Binaryen) for optimized outputs.

### 1) Build the contract WASM

```bash
make wasm-opt
# output: target/wasm32-unknown-unknown/release/drc20_opt.wasm
```

### 2) Build the data-driver WASM (off-chain)

The data-driver is auto-generated from the contract via Dusk Forge (no separate
`data-driver/` crate) and can be used by any runtime that supports WASM.

```bash
make data-driver
# output: target/data-driver/wasm32-unknown-unknown/release/drc20_opt.wasm
```

This command also copies the WASM into `web/public/data_driver.wasm`.

### 3) Run the web management UI

```bash
cd web
npm i
npm run dev
```

Open:

```text
http://localhost:5173/?network=testnet&contractId=0xYOUR_32B_CONTRACT_ID
```

Optional overrides:

- `&nodeUrl=https://testnet.nodes.dusk.network`
- `&driverUrl=/data_driver.wasm`


## Deploy

### 1) Prepare constructor args (`Init`)

This repo expects an `init.json` in the repo root describing the initial distribution.

- `init.json` is git-ignored (so you can safely keep local addresses / configs out of version control).
- `example.init.json` is committed as a starting point.

Create your local `init.json`:

```bash
cp example.init.json init.json
# edit init.json
```

Example format:

```json
{
  "initial_balances": [
    { "account": { "External": "26brdzqNXEG1jTzCubJAPhks18bSSDY4n21ZW6VLYkCv6bBUdBAZZAbn1Coz1LPBYc4uEekBbzFnZvhL9untGCqRamhZS2cBV51fdZog3qkP3NbMEaqgNMcKEahAFV8t2Cke" }, "amount": "1000" }
  ]
}
```

> Notes:
> - `External` is a public account (base58).
> - Amounts are `u64` and accepted as JSON strings (to avoid JS precision issues).

Generate the `rkyv`-encoded hex string expected by most deploy tooling:

```bash
make init-args FILE=./init.json
# prints: <hex>
```

> Note: `make init-args` defaults to `./init.json` if you don't pass `FILE=...`.

### 2) Deploy the WASM

Deploy using your preferred tooling (e.g. `rusk-wallet contract-deploy`).

Deploy tooling must pass constructor args. To deploy with zero supply, use an
empty `initial_balances` list in `Init`.

## Tests

The repo includes a small spec test-suite that runs the contract in `dusk-vm` and
asserts the classic ERC20 behaviors (balances, allowances, events, failure cases).

It also deploys a tiny helper contract to prove the "contract caller" path works
(i.e. `sender_account()` resolves to `Account::Contract(...)` for ICC calls).

Run the tests:

```bash
make test
```

This will:

1. Build `contract/` to `target/wasm32-unknown-unknown/release/drc20.wasm`
2. Build the helper contract to `target/wasm32-unknown-unknown/release/drc20_test_caller.wasm`
3. Run `cargo test -p drc20-tests`

## Notes

- `init(Init)` is intended to be called at deployment with an initial distribution.
- The contract is transparent only (requires a public sender, shielded tx not supported).
- `ZERO_ADDRESS` is reserved for mint events (`from = ZERO_ADDRESS`).

## License

MIT © Dusk Forge.
