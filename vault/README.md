# OptionVault Contract

ERC-4626-based vault contract for managing collateral for option series on Arbitrum Stylus.

## Overview

The OptionVault contract is a critical component of the options trading system. Each option series has its own dedicated vault that:

- Holds 100% collateral for the option series
- Issues vault shares (ERC-20) to writers proportional to their deposits
- Tracks deposits with FIFO ordering for fair assignment on exercise
- Handles exercise withdrawals that reduce vault assets
- Enables post-expiry claims with FIFO assignment (assigned writers get strike payments, unassigned get collateral)
- Supports early redemption by burning shares + options together

## Key Features

### ERC-4626 Compliance
- Standard vault interface for maximum DeFi composability
- Hardcoded `decimals_offset=3` for ERC-4626 inflation attack protection
- Provides 1000x security multiplier without requiring pricing oracles

### FIFO Assignment Tracking
- Each deposit creates a checkpoint with cumulative total
- Writers assigned in deposit order (first deposits assigned first)
- After expiry, `claim()` calculates assignment on-demand
- Batched transfers (one for strike payments, one for collateral)

### Two-Token Architecture
- **Vault Shares (ERC-20)**: Represent short position + collateral claim
- **Option Tokens (ERC-20)**: Represent long position (right to exercise)
- Both independently transferrable for maximum flexibility

## Current Status

**Implementation**: Stub implementations with error types defined

All core functions return `Unimplemented` error. Full implementation pending.

### Implemented
- ✅ Storage structure with FIFO checkpoint tracking
- ✅ Error types for validation
- ✅ Event definitions
- ✅ Public interface signatures
- ✅ View functions for checkpoint data
- ✅ Basic test coverage for stubs

### TODO
- ⏳ `deposit()` - Create checkpoints, mint shares
- ⏳ `exercise_withdraw()` - Reduce assets on exercise
- ⏳ `claim()` - Calculate FIFO assignment, distribute strike/collateral
- ⏳ `burn_shares_with_options()` - Early redemption path
- ⏳ ERC-4626 standard function implementations
- ⏳ Comprehensive test coverage
- ⏳ Integration tests with OptionToken contract

## Security Considerations

### Hardcoded Inflation Protection
The vault constructor hardcodes `decimals_offset=3`, providing:
- 1000x security multiplier against ERC-4626 inflation attacks
- Uniform protection across all option series
- No pricing oracle required
- No configuration needed (simpler, safer)

See SPEC.md for detailed security analysis.

## Building

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Testing

```bash
cargo test
```

## Related Contracts

- **OptionToken**: ERC-20 option tokens that interact with this vault
- **OptionFactory**: Deploys OptionToken + OptionVault pairs

## Documentation

See the main [SPEC.md](../SPEC.md) for the complete system architecture and vault design rationale.
