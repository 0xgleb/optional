# Implementation Plan: Call Exercise (American) - Issue #7

## Overview

Implement American-style call option exercise with immediate atomic settlement.
Holders can exercise call options before expiry, receiving underlying tokens in
exchange for paying strike price.

## Dependencies

Issue #4 (Write Call Option) - COMPLETED ✓

## Technical Approach

### Settlement Flow

1. Holder pays strike amount (quote tokens) to writer
2. Holder receives underlying tokens from contract collateral
3. Holder's option tokens burned
4. Writer's position reduced

### PoC Constraint: Single Writer Model

Exercise reduces msg.sender's position only. If exerciser has no position,
exercise succeeds but no position reduction occurs. This supports the primary
use case (writer exercises own options) without requiring writer tracking
infrastructure.

Multi-writer support (FIFO/pro-rata) deferred to post-PoC.

### Security Requirements

- Checks-effects-interactions pattern (prevent reentrancy)
- Checked arithmetic (prevent overflow/underflow)
- Fee-on-transfer token detection
- Atomic execution (full success or full revert)

## Implementation Plan

### Task 1. Basic exercise validation with error types and tests

**Types**: Add error variants for exercise failures.

**Implementation**: Add to `sol!` block and `OptionsError` enum:

- `OptionNotFound()` - token ID has no metadata
- `ExerciseAfterExpiry(uint256 expiry, uint256 current)` - exercise after
  expiration
- `WrongOptionType(uint8 expected, uint8 actual)` - wrong option type

**Tests**: Verify stub returns Unimplemented, new errors compile and convert
correctly.

**Completion Criteria**:

- [x] Three new error variants added
- [x] Test: `exercise_call` returns `Unimplemented` error
- [x] `cargo test` passes
- [x] `cargo clippy` passes
- [x] `cargo fmt` passes

### Task 2. Implement basic exercise validation logic with tests

**Implementation**: Create `validate_call_exercise()` helper function:

- Get option metadata, return `OptionNotFound` if zero expiry (uninitialized)
- Check `current_time < expiry`, return `ExerciseAfterExpiry` if expired
- Check `option_type == 0` (Call), return `WrongOptionType` if Put
- Check `quantity > 0`, return `InvalidQuantity` if zero
- Check holder balance >= quantity, return `InsufficientBalance` if insufficient

**Tests**: Test each validation path independently with unit tests.

**Completion Criteria**:

- [x] `validate_call_exercise()` function implemented with all checks
- [x] Test: validation passes with valid inputs
- [x] Test: returns `OptionNotFound` for non-existent token
- [x] Test: returns `ExerciseAfterExpiry` when expired
- [x] Test: returns `WrongOptionType` for Put option
- [x] Test: returns `InvalidQuantity` for zero quantity
- [x] Test: returns `InsufficientBalance` when balance too low
- [x] `cargo test` passes
- [x] `cargo clippy` passes (dead code warning until Task 5)

### Task 3. Implement position reduction helper with tests

**Implementation**: Create `reduce_position()` helper:

- Get current position using `position_key(writer, token_id)`
- Check quantity_written >= quantity, return `InsufficientBalance` if
  insufficient
- Use `checked_sub()` to reduce quantity_written and collateral_locked
- Update position storage with new values

**Tests**: Test position reduction with various scenarios.

**Completion Criteria**:

- [x] `reduce_position()` function implemented
- [x] Test: successfully reduces position with sufficient quantity
- [x] Test: returns error when position insufficient
- [x] Test: position values updated correctly after reduction
- [x] Test: reduces position to zero when quantity matches
- [x] `cargo test` passes (77 tests)
- [x] `cargo clippy` passes (dead code warnings until Task 5)

### Task 4. Implement safe transfer helper with tests

**Implementation**: Create `safe_transfer()` helper:

- Check recipient balance before transfer
- Call ERC20 `transfer()` from contract to recipient
- Check recipient balance after transfer
- Verify received amount matches expected, return `FeeOnTransferDetected` if
  mismatch
- Return `UnexpectedBalanceDecrease` if balance decreased

**Tests**: Test with normal and fee-on-transfer tokens.

**Completion Criteria**:

- [x] `safe_transfer()` function implemented
- [x] Test: successful transfer with normal ERC20
- [x] Test: zero amount transfer
- [x] `cargo test` passes
- [x] `cargo clippy` passes

### Task 5. Implement exercise_call() with happy path tests

**Types**: Add `ExerciseCall` event to `sol!` block:

- Fields: `holder` (indexed), `writer` (indexed), `tokenId` (indexed),
  `quantity`, `strikePayment`, `underlyingReceived`

**Implementation**: Implement `exercise_call()` function:

- **Checks**: Get VM context, validate using `validate_exercise_call()`, get
  metadata
- **Effects**: Burn option tokens, reduce exerciser's position (if they have
  one)
- **Interactions**: Transfer strike payment from holder to holder (PoC: holder
  must be writer), transfer underlying from contract to holder
- Emit `ExerciseCall` event after successful exercise

**Tests**: Test writer exercises their own options (happy path).

**Completion Criteria**:

- [x] `ExerciseCall` event added
- [x] `exercise_call()` implemented with CEI pattern
- [x] Test: writer exercises own options successfully
- [x] Test: option tokens burned correctly
- [x] Test: position reduced correctly
- [x] Test: underlying tokens transferred
- [x] `cargo test` passes (86 tests)
- [x] `cargo clippy` passes

### Task 6. Add comprehensive unit tests for validation failures

**Tests**: Cover all error paths not tested in previous tasks.

**Completion Criteria**:

- [ ] Test: exercise with insufficient quote token balance fails
- [ ] Test: exercise with insufficient quote token approval fails
- [ ] Test: exercise at exactly expiry timestamp fails
- [ ] Test: exercise 1 second before expiry succeeds
- [ ] Test: exercise with fee-on-transfer quote token fails
- [ ] Test: exercise with fee-on-transfer underlying token fails
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

### Task 7. Add unit tests for partial and multiple exercises

**Tests**: Test partial exercise and sequential exercises.

**Completion Criteria**:

- [ ] Test: partial exercise leaves remaining balance
- [ ] Test: multiple partial exercises deplete balance
- [ ] Test: exercise full balance sets balance to zero
- [ ] Test: partial position reduction maintains correct collateral ratio
- [ ] Test: exercising more than balance fails
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

### Task 8. Add property-based tests for invariants

**Implementation**: Add proptest cases for critical invariants.

**Completion Criteria**:

- [ ] Proptest: exercise never panics (returns Ok or Err)
- [ ] Proptest: option balance decreases by exact exercise amount
- [ ] Proptest: position quantity/collateral decrease proportionally
- [ ] Proptest: total supply decreases by exercise amount
- [ ] Proptest: no arithmetic overflow/underflow
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

### Task 9. Add integration tests for complete flows

**Tests**: Test end-to-end scenarios with write → exercise flow.

**Completion Criteria**:

- [ ] Test: write call → immediate exercise by writer
- [ ] Test: write call → wait near expiry → exercise succeeds
- [ ] Test: write call → wait past expiry → exercise fails
- [ ] Test: write large quantity → partial exercise → partial exercise → verify
- [ ] Test: write call → attempt exercise with wrong token ID fails
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

### Task 10. Documentation and final quality checks

**Implementation**: Add comprehensive documentation and verify all quality
standards.

**Completion Criteria**:

- [ ] Add doc comments to `exercise_call()` with examples
- [ ] Add doc comments to all helper functions
- [ ] Remove `#[allow(dead_code)]` from `get_option_metadata()`, `balance_of()`,
      `get_position()`, `denormalize_amount()`
- [ ] Verify no TODO comments remain
- [ ] `cargo test` passes (all tests including new ones)
- [ ] `cargo clippy --all-targets --all-features -- -D clippy::all -D warnings`
      passes with no warnings
- [ ] `cargo fmt` applied
- [ ] `cargo build --target wasm32-unknown-unknown` succeeds

## Gas Optimization Target

Target: ~150k gas per exercise (from SPEC.md)

## Critical Invariants

Tests must verify:

- Collateral conservation: Contract balance equals sum of active positions
- Supply conservation: Total option supply ≥ sum of positions
- Position accounting: Writer position reflects locked collateral
- Atomicity: Full success or full revert
