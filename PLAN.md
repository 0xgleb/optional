# Implementation Plan: OptionsToken Contract Public Interface (Issue #2)

## Overview

Define the complete public API for the OptionsToken contract with stub
implementations. This establishes the custom OptionsToken methods only -
ERC-1155 standard interface is Issue #11.

**Scope**: 5 custom methods (American-style options), storage structures, error
types, test stubs.

**American Options**: This implementation uses American-style exercise
(immediate settlement any time before expiry) rather than European-style (signal
then finalize). Benefits: simpler state machine, better UX, no
signaling/cancellation complexity, more valuable product.

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

## Task 3. Implement Exercise API - American Style (Stubs)

**Goal**: Immediate exercise functionality (any time before expiry).

### Methods

- [ ] Implement
      `exercise_call(&mut self, token_id: B256, quantity: U256) -> Result<(), OptionsError>`:

  - Doc: Immediate atomic settlement. Holder pays strike (quote tokens) to
    writer, receives underlying from collateral, burns option tokens. Only
    before expiry.
  - Body: `Err(OptionsError::Unimplemented(Unimplemented {}))`

- [ ] Implement
      `exercise_put(&mut self, token_id: B256, quantity: U256) -> Result<(), OptionsError>`:

  - Doc: Immediate atomic settlement. Holder delivers underlying to writer,
    receives strike (quote tokens) from collateral, burns option tokens. Only
    before expiry.
  - Body: `Err(OptionsError::Unimplemented(Unimplemented {}))`

### Unit Tests

- [ ] Create `test_exercise_call_unimplemented()`
- [ ] Create `test_exercise_put_unimplemented()`

### Property Tests

- [ ] Create `prop_exercise_unimplemented()` with strategies:
  - Token IDs: any [u8; 32] (B256)
  - Quantities: 1..1_000_000_000u64
  - Test both functions return `Unimplemented`

### Validation

- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

## Task 4. Implement Collateral Withdrawal API (Stubs)

**Goal**: Writers reclaim collateral for expired unexercised options.

### Methods

- [ ] Implement
      `withdraw_expired_collateral(&mut self, token_id: B256, quantity: U256) -> Result<(), OptionsError>`:

  - Doc: Writers reclaim collateral for unexercised options after expiry.
    Returns underlying for calls, quote for puts. Reduces/closes writer
    position. Only after expiry.
  - Body: `Err(OptionsError::Unimplemented(Unimplemented {}))`

### Unit Tests

- [ ] Create `test_withdraw_expired_collateral_unimplemented()`

### Property Tests

- [ ] Create `prop_withdraw_expired_collateral_unimplemented()` with strategies:
  - Token IDs: any [u8; 32] (B256)
  - Quantities: 1..1_000_000_000u64
  - Test function returns `Unimplemented`

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

- [ ] All tests pass
- [ ] No clippy warnings
- [ ] WASM builds successfully
- [ ] ABI exports correctly
- [ ] Contract ready for testnet deployment

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
- Collateral: `keccak256(user, token)`

Note: With American options, no need for exercise intent or locked funds
storage - exercise is immediate.

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

- Contract compiles at every task checkpoint
- Tests pass after each feature addition
- Clippy clean throughout development
- WASM builds successfully
- ABI exports correctly with all 5 custom functions
- All functions return `Err(OptionsError::Unimplemented)`
- Full documentation on all public functions
- Complete test coverage for all stubs
- Storage structures defined and documented
- Ready for testnet deployment
