import { createDuskApp, DUSK_CHAIN_PRESETS } from "@mochavi/connect";
import { defineMochaviConnectButton } from "@mochavi/connect/ui";

defineMochaviConnectButton();

const qs = new URLSearchParams(location.search);

// --- Config ---
const NETWORK = (qs.get("network") || "testnet").toLowerCase();
const NODE_URL =
  qs.get("nodeUrl") ||
  (NETWORK === "mainnet"
    ? "https://nodes.dusk.network"
    : NETWORK === "devnet"
    ? "https://devnet.nodes.dusk.network"
    : "https://testnet.nodes.dusk.network");

const CONTRACT_ID = qs.get("contractId") || "0x0000000000000000000000000000000000000000000000000000000000000000";
const DRIVER_URL =
  (qs.get("driverUrl") || "/data_driver.wasm") +
  (qs.get("driverUrl") ? "" : `?v=${Date.now()}`);

const CHAIN_PRESET =
  NETWORK === "mainnet"
    ? DUSK_CHAIN_PRESETS.mainnet
    : NETWORK === "devnet"
    ? DUSK_CHAIN_PRESETS.devnet
    : NETWORK === "local"
    ? DUSK_CHAIN_PRESETS.local
    : DUSK_CHAIN_PRESETS.testnet;

const dusk = createDuskApp({
  nodeUrl: NODE_URL,
  chain: { chainId: CHAIN_PRESET },
  autoConnect: true,
  contracts: {
    token: {
      contractId: CONTRACT_ID,
      driverUrl: DRIVER_URL,
      name: "DRC20",
      methodSigs: {
        name: "name() -> String",
        symbol: "symbol() -> String",
        decimals: "decimals() -> u8",
        total_supply: "total_supply() -> u64",
        balance_of: "balance_of(BalanceOf) -> u64",
        allowance: "allowance(Allowance) -> u64",
        transfer: "transfer(TransferCall)",
        approve: "approve(ApproveCall)",
        transfer_from: "transfer_from(TransferFromCall)",
      },
    },
  },
});

const wallet = dusk.wallet;
const c = dusk.contract("token");

// --- DOM ---
const $ = (id) => document.getElementById(id);

const ui = {
  connect: $("connect"),
  cfg: $("cfg"),
  snack: $("snack"),

  btnRefresh: $("btn-refresh"),

  tName: $("t-name"),
  tSymbol: $("t-symbol"),
  tDecimals: $("t-decimals"),
  tSupply: $("t-supply"),

  me: $("me"),
  meBal: $("me-balance"),

  balAccount: $("bal-account"),
  btnBal: $("btn-bal"),
  balOut: $("bal-out"),

  txTo: $("tx-to"),
  txValue: $("tx-value"),
  btnTransfer: $("btn-transfer"),

  apSpender: $("ap-spender"),
  apValue: $("ap-value"),
  btnApprove: $("btn-approve"),

  alOwner: $("al-owner"),
  alSpender: $("al-spender"),
  btnAllowance: $("btn-allowance"),
  alOut: $("al-out"),

  tfOwner: $("tf-owner"),
  tfTo: $("tf-to"),
  tfValue: $("tf-value"),
  btnTransferFrom: $("btn-transferfrom"),
};

ui.connect.wallet = wallet;

ui.cfg.textContent = `network=${NETWORK}  nodeUrl=${NODE_URL}  contractId=${CONTRACT_ID}  driverUrl=${DRIVER_URL}`;

// --- Helpers ---
function isZeroContractId(hex) {
  return /^0x0+$/i.test(String(hex || "").trim());
}

function big(v) {
  try {
    return BigInt(v ?? 0);
  } catch {
    return 0n;
  }
}

function snack(msg, kind = null) {
  ui.snack.textContent = msg;
  ui.snack.classList.toggle("hidden", !msg);
  ui.snack.style.borderColor =
    kind === "ok"
      ? "rgba(110,231,255,0.35)"
      : kind === "warn"
      ? "rgba(250,204,21,0.35)"
      : kind === "danger"
      ? "rgba(248,113,113,0.35)"
      : "rgba(255,255,255,0.14)";
}

function parseU64(s) {
  const raw = String(s || "").trim();
  if (!raw) throw new Error("Missing value");
  const n = BigInt(raw);
  if (n < 0n || n > 18446744073709551615n) throw new Error("Out of range for u64");
  return raw; // keep as string to avoid JS precision loss
}

function parseAccount(s) {
  const raw = String(s || "").trim();
  if (!raw) throw new Error("Missing account");
  // ContractId is typically 32-byte hex (0x + 64 hex chars)
  if (/^0x[0-9a-f]{64}$/i.test(raw)) return { Contract: raw.toLowerCase() };
  // Otherwise treat as External (wallet addresses are commonly base58).
  return { External: raw };
}

async function ensureConnected() {
  if (!wallet.state.authorized) await wallet.connect();
  const addr = wallet.state.selectedAddress;
  if (addr) ui.me.textContent = addr;
  return addr;
}

async function refresh() {
  if (isZeroContractId(CONTRACT_ID)) {
    snack("Configure ?contractId=0x… first", "warn");
    return;
  }

  snack("Refreshing…");

  try {
    const [name, symbol, decimals, supply] = await Promise.all([
      c.call.name(),
      c.call.symbol(),
      c.call.decimals(),
      c.call.total_supply(),
    ]);

    ui.tName.textContent = String(name ?? "—");
    ui.tSymbol.textContent = String(symbol ?? "—");
    ui.tDecimals.textContent = String(decimals ?? "—");
    ui.tSupply.textContent = big(supply).toString();

    const me = wallet.state.selectedAddress;
    if (me) {
      ui.me.textContent = me;
      const bal = await c.call.balance_of({ account: { External: me } });
      ui.meBal.textContent = big(bal).toString();
    } else {
      ui.me.textContent = "—";
      ui.meBal.textContent = "—";
    }

    snack("Up to date", "ok");
    setTimeout(() => snack(""), 1200);
  } catch (e) {
    console.error(e);
    snack(String(e?.message || e), "danger");
  }
}

async function sendTx(fnName, args, display) {
  if (isZeroContractId(CONTRACT_ID)) throw new Error("Missing contractId");
  await ensureConnected();
  snack("Confirm in your wallet…");

  const tx = await c.write[fnName](args, {
    amount: "0",
    deposit: "0",
    display,
  });

  const unsub = tx.onStatus(({ status }) => {
    if (status === "submitted") snack("Sent", "ok");
    else if (status === "executing") snack("Processing…", "warn");
    else if (status === "executed") snack("Executed", "ok");
    else if (status === "failed") snack("Failed", "danger");
    else if (status === "timeout") snack("Timeout", "warn");
  });

  try {
    await tx.wait({ timeoutMs: 90_000 });
  } finally {
    unsub();
  }
}

// --- Wire up UI ---
ui.btnRefresh.onclick = refresh;

ui.btnBal.onclick = async () => {
  try {
    const acct = parseAccount(ui.balAccount.value);
    const bal = await c.call.balance_of({ account: acct });
    ui.balOut.textContent = big(bal).toString();
    snack("OK", "ok");
    setTimeout(() => snack(""), 800);
  } catch (e) {
    snack(String(e?.message || e), "danger");
  }
};

ui.btnTransfer.onclick = async () => {
  try {
    const to = parseAccount(ui.txTo.value);
    const value = parseU64(ui.txValue.value);
    await sendTx("transfer", { to, value }, { title: "DRC20 transfer", fields: { to: ui.txTo.value, value } });
    await refresh();
  } catch (e) {
    snack(String(e?.message || e), "danger");
  }
};

ui.btnApprove.onclick = async () => {
  try {
    const spender = parseAccount(ui.apSpender.value);
    const value = parseU64(ui.apValue.value);
    await sendTx("approve", { spender, value }, { title: "DRC20 approve", fields: { spender: ui.apSpender.value, value } });
    snack("Approved", "ok");
    await refresh();
  } catch (e) {
    snack(String(e?.message || e), "danger");
  }
};

ui.btnAllowance.onclick = async () => {
  try {
    const owner = parseAccount(ui.alOwner.value);
    const spender = parseAccount(ui.alSpender.value);
    const out = await c.call.allowance({ owner, spender });
    ui.alOut.textContent = big(out).toString();
    snack("OK", "ok");
    setTimeout(() => snack(""), 800);
  } catch (e) {
    snack(String(e?.message || e), "danger");
  }
};

ui.btnTransferFrom.onclick = async () => {
  try {
    const owner = parseAccount(ui.tfOwner.value);
    const to = parseAccount(ui.tfTo.value);
    const value = parseU64(ui.tfValue.value);
    await sendTx(
      "transfer_from",
      { owner, to, value },
      { title: "DRC20 transfer_from", fields: { owner: ui.tfOwner.value, to: ui.tfTo.value, value } }
    );
    await refresh();
  } catch (e) {
    snack(String(e?.message || e), "danger");
  }
};

refresh();
