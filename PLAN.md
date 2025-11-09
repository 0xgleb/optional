# Implementation Plan: OptionsToken Contract Public Interface (Issue #2)

## Overview

Define the complete public API for the OptionsToken contract with stub
implementations. This establishes the custom OptionsToken methods only -
ERC-1155 standard interface is Issue #11.

**Scope**: 9 custom methods, storage structures, error types, test stubs.

## Dependencies

- `stylus-sdk = "0.9.0"` (already present)
- `alloy-primitives = "=0.8.20"` (already present)
- `motsu = "0.1.0"` (dev, already present)
- `proptest = "1.4"` (dev, already present)

## Task 1. Contract Skeleton with Error Types and Enums

**Goal**: Create minimal compiling contract with error handling and option type
foundation.

### Contract Structure

- [x] Replace counter example in `options/src/lib.rs` with minimal OptionsToken
      contract:

  ```rust
  #![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
  #![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
  extern crate alloc;

  use stylus_sdk::prelude::*;
  use alloy_primitives::{Address, U256};
  use alloc::vec::Vec;

  sol_storage! {
      #[entrypoint]
      pub struct OptionsToken {
          // Storage will be added in Task 2
      }
  }

  #[public]
  impl OptionsToken {
      // Methods will be added in later tasks
  }
  ```

### Error Enum

- [x] Create `OptionsError` enum with `Unimplemented` variant:

  ```rust
  #[derive(Debug)]
  pub enum OptionsError {
      /// Stub implementation placeholder - function not yet implemented.
      Unimplemented,
      // Additional error variants will be added as needed during implementation
  }
  ```

- [x] Implement `From<OptionsError> for Vec<u8>` for Stylus error encoding:
  ```rust
  impl From<OptionsError> for Vec<u8> {
      fn from(err: OptionsError) -> Vec<u8> {
          format!("{:?}", err).into_bytes()
      }
  }
  ```

### Option Type Enum

- [x] Create `OptionType` enum with `Call` and `Put` variants
- [x] Add doc comments:
  - Call: Right to BUY underlying at strike
  - Put: Right to SELL underlying at strike

### Basic Test

- [x] Create basic test that instantiates contract:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_contract_instantiates() {
          let contract = OptionsToken {
              // Initialize with default storage
          };
          // Verify contract exists
      }
  }
  ```

### Validation

- [x] `cargo build` succeeds
- [x] `cargo test` passes
- [x] `cargo clippy --all-targets --all-features -- -D clippy::all -D warnings`
      passes
- [x] `cargo fmt` clean

## Task 2. Implement Write Options API (Stubs)

**Goal**: Options minting interface with collateral locking.

### Methods

- [x] Implement `write_call_option()`:

  - Parameters: strike, expiry, quantity, underlying: Token, quote: Token
  - Doc: Locks underlying tokens as collateral (1:1), mints ERC-1155 tokens,
    returns deterministic token ID
  - Body: `Err(OptionsError::Unimplemented(Unimplemented {}))`

- [x] Implement `write_put_option()`:
  - Parameters: strike, expiry, quantity, underlying: Token, quote: Token
  - Doc: Locks quote tokens as collateral (strike \* quantity), mints ERC-1155
    tokens, returns deterministic token ID
  - Body: `Err(OptionsError::Unimplemented(Unimplemented {}))`

### Unit Tests

- [x] Create `test_write_call_option_returns_unimplemented()`
- [x] Create `test_write_put_option_returns_unimplemented()`

### Property Tests

- [x] Create `prop_write_options_unimplemented()` with strategies:
  - Addresses: any [u8; 20]
  - Strike: 1..1_000_000_000u64
  - Expiry: any u64
  - Quantity: 1..1_000_000_000u64
  - Decimals: 0..=18u8
  - Test both functions return `Unimplemented`

### Validation

- [x] `cargo test` passes
- [x] `cargo clippy` passes
- [x] `cargo build --target wasm32-unknown-unknown --release` succeeds

## Task 3. Implement Exercise Signaling API (Stubs)

**Goal**: Exercise intent management with fund locking.

### Methods

- [x] Implement
      `signal_call_exercise(&mut self, token_id: U256, quantity: U256) -> Result<(), OptionsError>`:

  - Doc: Locks quote tokens (strike payment), records intent, makes signaled
    tokens non-transferable, reversible before expiry
  - Body: `Err(OptionsError::Unimplemented(Unimplemented {}))`

- [x] Implement
      `signal_put_exercise(&mut self, token_id: U256, quantity: U256) -> Result<(), OptionsError>`:

  - Doc: Locks underlying tokens, records intent, makes signaled tokens
    non-transferable, reversible before expiry
  - Body: `Err(OptionsError::Unimplemented(Unimplemented {}))`

- [x] Implement
      `cancel_call_exercise_intent(&mut self, token_id: U256, quantity: U256) -> Result<(), OptionsError>`:

  - Doc: Returns locked quote tokens, clears intent, restores transferability,
    only before expiry
  - Body: `Err(OptionsError::Unimplemented(Unimplemented {}))`

- [x] Implement
      `cancel_put_exercise_intent(&mut self, token_id: U256, quantity: U256) -> Result<(), OptionsError>`:
  - Doc: Returns locked underlying tokens, clears intent, restores
    transferability, only before expiry
  - Body: `Err(OptionsError::Unimplemented(Unimplemented {}))`

### Unit Tests

- [x] Create `test_signal_call_exercise_unimplemented()`
- [x] Create `test_signal_put_exercise_unimplemented()`
- [x] Create `test_cancel_call_exercise_unimplemented()`
- [x] Create `test_cancel_put_exercise_unimplemented()`

### Property Tests

- [x] Create `prop_exercise_signaling_unimplemented()` with strategies:
  - Token IDs: any u64
  - Quantities: 1..1_000_000_000u64
  - Test all 4 functions return `Unimplemented`

### Validation

- [x] `cargo test` passes
- [x] `cargo clippy` passes

## Task 4. Implement Settlement API (Stubs)

**Goal**: Expiry finalization and settlement.

### Methods

- [ ] Implement
      `finalize_call_expiry(&mut self, token_id: U256, holder_address: Address) -> Result<(), OptionsError>`:

  - Doc: If exercised: quote → writer, underlying → holder. If not: underlying →
    writer. Burns tokens. Only after expiry.
  - Body: `Err(OptionsError::Unimplemented)`

- [ ] Implement
      `finalize_put_expiry(&mut self, token_id: U256, holder_address: Address) -> Result<(), OptionsError>`:

  - Doc: If exercised: underlying → writer, quote → holder. If not: quote →
    writer. Burns tokens. Only after expiry.
  - Body: `Err(OptionsError::Unimplemented)`

- [ ] Implement
      `finalize_batch_expiry(&mut self, token_id: U256, holder_addresses: Vec<Address>) -> Result<(), OptionsError>`:
  - Doc: Processes multiple holders atomically for gas efficiency. Same
    settlement logic per holder.
  - Body: `Err(OptionsError::Unimplemented)`

### Unit Tests

- [ ] Create `test_finalize_call_expiry_unimplemented()`
- [ ] Create `test_finalize_put_expiry_unimplemented()`
- [ ] Create `test_finalize_batch_expiry_unimplemented()`

### Property Tests

- [ ] Create `prop_finalize_expiry_unimplemented()` with strategies:
  - Token IDs: any U256
  - Addresses: non-zero (1..=u160::MAX)
  - Address vectors: 1..10 addresses
  - Test all 3 functions return `Unimplemented`

### Validation

- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

## Task 5. Final Integration and Verification

**Goal**: Verify complete contract compiles for deployment.

### Build Verification

- [ ] Run full test suite: `cargo test`
- [ ] Run formatter: `cargo fmt`
- [ ] Run clippy:
      `cargo clippy --all-targets --all-features -- -D clippy::all -D warnings`
- [ ] Build WASM: `cargo build --target wasm32-unknown-unknown --release`
- [ ] Verify WASM artifact:
      `ls -lh target/wasm32-unknown-unknown/release/options.wasm`
- [ ] Export ABI: `cargo stylus export-abi > abi.json`
- [ ] Review ABI contains all 9 functions
- [ ] Run Stylus check (optional, may timeout): `timeout 60 cargo stylus check`

### Validation

- [ ] All tests pass ✓
- [ ] No clippy warnings ✓
- [ ] WASM builds successfully ✓
- [ ] ABI exports correctly ✓
- [ ] Contract ready for testnet deployment ✓

## Design Decisions

### Decimal Normalization

All amounts normalized to 18 decimals internally:

```rust
normalized_amount = amount * 10^(18 - decimals)
```

Decimals passed as parameters (not queried from ERC20) because:

- ERC20 `decimals()` is optional
- Gas optimization (avoid external calls)
- Caller responsibility to provide correct decimals

### Token ID Calculation

Deterministic hash ensures same parameters = same token ID:

```rust
token_id = keccak256(abi.encodePacked(
    underlying,
    quote,
    strike,        // 18 decimals normalized
    expiry,
    option_type
))
```

### Storage Key Generation

Deterministic keys for O(1) lookups:

- Position: `keccak256(writer, tokenId)`
- Exercise intent: `keccak256(holder, tokenId)`
- Locked funds: `keccak256(holder, tokenId, token)`
- Collateral: `keccak256(user, token)`

### Separate Call/Put Functions

Instead of `write_option(OptionType)`, we have `write_call_option()` and
`write_put_option()`.

Rationale:

- Clearer intent for users
- Easier to document (different collateral requirements)
- Better error messages
- More explicit ABI for frontend integration
- Follows "make invalid states unrepresentable"

### Error Handling Philosophy

- NEVER panic - all operations return `Result<T, OptionsError>`
- NEVER use `unwrap()` or `expect()` in production code
- ALL arithmetic uses checked operations
- Descriptive error variants with context
- Fail fast with clear errors

## Success Criteria

- Contract compiles at every task checkpoint ✓
- Tests pass after each feature addition ✓
- Clippy clean throughout development ✓
- WASM builds successfully ✓
- ABI exports correctly with all 9 custom functions ✓
- All functions return `Err(OptionsError::Unimplemented)` ✓
- Full documentation on all public functions ✓
- Complete test coverage for all stubs ✓
- Storage structures defined and documented ✓
- Ready for testnet deployment ✓
