//! Panic messages used by the DRC20 reference implementation.

/// The token has already been initialized.
pub const ALREADY_INITIALIZED: &str = "DRC20: already initialized";

/// The account doesn't have enough tokens to perform the requested transfer.
pub const BALANCE_TOO_LOW: &str = "DRC20: balance too low";

/// The spender doesn't have enough allowance to perform the requested transfer.
pub const ALLOWANCE_TOO_LOW: &str = "DRC20: allowance too low";

/// Total supply overflow (only possible at initialization or an extreme transfer).
pub const SUPPLY_OVERFLOW: &str = "DRC20: total supply overflow";

/// Shielded transactions are not supported by this transparent token.
pub const SHIELDED_NOT_SUPPORTED: &str = "DRC20: shielded transactions are not supported";

/// The reserved `ZERO_ADDRESS` must not be used as a recipient/spender/holder.
pub const ZERO_ADDRESS_NOT_ALLOWED: &str = "DRC20: zero address not allowed";
