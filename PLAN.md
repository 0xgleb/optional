# Implementation Plan: Call Option Writing (Issue #4)

This plan implements complete call option writing functionality using **vertical
slices** where each task delivers a complete, testable feature following
Type-Test Driven Development (TTDD).

## Task 1. Token ID Generation

Generate deterministic token IDs for option series.

**TTDD Step 1 - Types:**

- [x] Add `OptionType::to_u8()` helper method (Call=0, Put=1)
- [x] Error variants will be added during test writing if needed

**TTDD Step 2 - Tests:**

- [x] Test: Same parameters produce identical token ID
- [x] Test: Different strikes produce different token IDs
- [x] Test: Different expiries produce different token IDs
- [x] Test: Different option types (Call vs Put) produce different token IDs
- [x] Test: Different underlying addresses produce different token IDs
- [x] Test: Different quote addresses produce different token IDs
- [x] Property test: Token ID determinism - calling function N times with same
      inputs produces same output

**TTDD Step 3 - Implementation:**

- [x] Add
      `generate_token_id(underlying: Address, quote: Address, strike: U256, expiry: U256, option_type: OptionType) -> B256`
      function
- [x] Concatenate byte slices for encoding
- [x] Use `alloy_primitives::keccak256()` for hashing
- [x] Return `B256` hash

**Validation:**

- [x] `cargo test` passes
- [x] `cargo clippy` passes (expected dead_code warning until Task 9)
- [x] `cargo fmt --check` passes

**Design Decision**: Token IDs are globally unique per option series. All
writers sharing the same parameters get the same token ID, enabling fungibility
and secondary trading.

## Task 2. Decimal Normalization

Convert amounts between native token decimals and 18-decimal internal
representation.

**TTDD Step 1 - Types:**

- [x] Add error variants as discovered during test writing:
  - `InvalidDecimals` (decimals > 18)
  - `NormalizationOverflow` (scale factor or multiplication overflow)

**TTDD Step 2 - Tests:**

- [x] Test: Normalize 1_000_000 from 6 decimals (USDC) = 1_000_000 \* 10^12
- [x] Test: Normalize 100_000_000 from 8 decimals (WBTC) = 100_000_000 \* 10^10
- [x] Test: Normalize 1 ether from 18 decimals = 1 ether (no change)
- [x] Test: Normalize from 0 decimals = amount \* 10^18
- [x] Test: Normalize from 24 decimals fails with InvalidDecimals
- [x] Test: Denormalize round-trip preserves value (6, 8, 18 decimals)
- [x] Test: Normalize U256::MAX fails with NormalizationOverflow

**TTDD Step 3 - Implementation:**

- [x] Add
      `normalize_amount(amount: U256, from_decimals: u8) -> Result<U256, OptionsError>`
  - Validate `from_decimals <= 18`
  - Calculate `scale_factor = 10^(18 - from_decimals)` using `checked_pow`
  - Return `amount.checked_mul(scale_factor)`
- [x] Add
      `denormalize_amount(amount: U256, to_decimals: u8) -> Result<U256, OptionsError>`
  - Validate `to_decimals <= 18`
  - Calculate `scale_factor = 10^(18 - to_decimals)` using `checked_pow`
  - Return `amount / scale_factor` (division cannot underflow)

**Validation:**

- [x] `cargo test` passes (20 tests)
- [x] `cargo clippy` passes (expected dead_code warnings until Task 9)
- [x] `cargo fmt --check` passes

**Design Decision**: All internal calculations use 18 decimals. Conversion
happens only at ERC20 boundaries. Checked arithmetic prevents silent
overflow/underflow.

## Task 3. Mock ERC20 Tokens

Mock tokens for testing without external dependencies.

**TTDD Step 1 - Types:**

- [x] Create mock token types in tests module
- [x] Define `MockERC20` structure with BTreeMap
- [x] Define `FeeOnTransferERC20` structure with BTreeMap

**TTDD Step 2 - Tests:**

- [x] Test: MockERC20 mint increases balance
- [x] Test: MockERC20 transfer moves tokens
- [x] Test: MockERC20 transferFrom with approval works
- [x] Test: MockERC20 transferFrom without approval fails
- [x] Test: MockERC20 decimals() returns configured value
- [x] Test: FeeOnTransferERC20 deducts 1% fee on transfer
- [x] Test: FeeOnTransferERC20 balanceOf shows reduced amount after transfer

**TTDD Step 3 - Implementation:**

- [x] Implement `MockERC20` with:
  - `BTreeMap<Address, U256> balances`
  - `BTreeMap<Address, BTreeMap<Address, U256>> allowances`
  - `u8 decimals_value`
  - `mint(to: Address, amount: U256)` - test helper
  - `transfer(from: Address, to: Address, amount: U256) -> bool`
  - `transferFrom(spender, from: Address, to: Address, amount: U256) -> bool`
  - `approve(owner, spender: Address, amount: U256) -> bool`
  - `balance_of(account: Address) -> U256`
  - `decimals() -> u8` and `set_decimals(u8)`
- [x] Implement `FeeOnTransferERC20`:
  - Simplified version with only needed methods
  - `transfer` deducts 1% fee (amount \* 99 / 100)
- [x] Mark types with `#[cfg(test)]`

**Validation:**

- [x] `cargo test` passes (27 tests)
- [x] `cargo clippy` passes (expected dead_code warnings for Task 1-2 functions
      until Task 9)
- [x] `cargo fmt` passes

**Design Decision**: Mock tokens enable isolated testing. FeeOnTransferERC20
validates our detection logic. Mocks are simpler and faster than deploying real
contracts.

## Task 4. ERC-1155 Balance Tracking

Track option token balances and total supply.

**TTDD Step 1 - Types:**

- [x] Add to `Options` storage struct:
  - `StorageMap<B256, U256> balances` - (owner, tokenId) -> balance
  - `StorageMap<B256, U256> total_supply` - tokenId -> total supply
- [x] Add error variants as discovered:
  - `InsufficientBalance { available: U256, requested: U256 }`
  - `Overflow`

**TTDD Step 2 - Tests:**

- [x] Test: Minting to address increases balance
- [x] Test: Minting increases total supply
- [x] Test: Burning from address decreases balance
- [x] Test: Burning decreases total supply
- [x] Test: Burning more than balance fails with InsufficientBalance
- [x] Test: Minting U256::MAX then minting 1 fails with Overflow
- [x] Test: Multiple mints accumulate correctly
- [x] Property test: Mint then burn same amount returns balance to zero

**TTDD Step 3 - Implementation:**

- [x] Add `balance_key(owner: Address, token_id: B256) -> B256` helper
  - Use `keccak256(abi.encodePacked(owner, token_id))`
- [x] Add
      `_mint(to: Address, token_id: B256, quantity: U256) -> Result<(), OptionsError>`
  - Get current balance using `balance_key(to, token_id)`
  - Update balance with `checked_add`
  - Update total_supply with `checked_add`
- [x] Add
      `_burn(from: Address, token_id: B256, quantity: U256) -> Result<(), OptionsError>`
  - Get current balance using `balance_key(from, token_id)`
  - Verify `balance >= quantity`
  - Update balance with `checked_sub`
  - Update total_supply with `checked_sub`

**Validation:**

- [x] `cargo test` passes
- [x] `cargo clippy` passes (expected dead_code warnings until Task 9)
- [x] `cargo fmt --check` passes

**Design Decision**: Minimal ERC-1155 storage for option writing. Full
compliance (transfers, approvals, events) deferred to #11. Using `keccak256` for
composite keys enables efficient lookups.

## Task 5. Input Validation

Validate option parameters at contract boundary.

**TTDD Step 1 - Types:**

- [x] Add error variants as discovered:
  - `InvalidStrike`
  - `ExpiredOption { expiry: U256, current: U256 }`
  - `InvalidQuantity`
  - `SameToken`

**TTDD Step 2 - Tests:**

- [x] Test: Valid parameters pass validation
- [x] Test: Zero strike fails with InvalidStrike
- [x] Test: Past expiry fails with ExpiredOption
- [x] Test: Expiry == current timestamp fails with ExpiredOption
- [x] Test: Zero quantity fails with InvalidQuantity
- [x] Test: Same underlying and quote addresses fail with SameToken
- [x] Test: Minimum valid expiry (current + 1) passes

**TTDD Step 3 - Implementation:**

- [x] Add
      `validate_write_params(strike: U256, expiry: u64, quantity: U256, underlying: Token, quote: Token, current_timestamp: u64) -> Result<(), OptionsError>`
  - Validate `strike > 0`
  - Validate `expiry > current_timestamp`
  - Validate `quantity > 0`
  - Validate `underlying.address != quote.address`

**Validation:**

- [x] `cargo test` passes (42 tests)
- [x] `cargo clippy` passes (expected dead_code warnings until Task 9)
- [x] `cargo fmt --check` passes

**Design Decision**: Fail fast at boundaries. Minimal validation - only check
fundamental correctness (non-zero values, different tokens). Decimals validation
happens during normalization. Zero addresses are allowed (contract is fully
permissionless).

## Task 6. Fee-on-Transfer Detection

Detect and reject tokens that deduct fees during transfers.

**TTDD Step 1 - Types:**

- [ ] Define `IERC20` interface using `sol_interface!`:
  - `function balanceOf(address) external view returns (uint256)`
  - `function transferFrom(address, address, uint256) external returns (bool)`
- [ ] Add error variants:
  - `FeeOnTransferDetected { expected: U256, received: U256 }`
  - `TransferFailed`
  - `UnexpectedBalanceDecrease`

**TTDD Step 2 - Tests:**

- [ ] Test: Transfer from MockERC20 succeeds
- [ ] Test: Transfer from FeeOnTransferERC20 fails with FeeOnTransferDetected
- [ ] Test: Error contains correct expected and received amounts
- [ ] Test: Multiple safe transfers in sequence all succeed

**TTDD Step 3 - Implementation:**

- [ ] Add
      `safe_transfer_from(token: Address, from: Address, to: Address, amount: U256) -> Result<(), OptionsError>`
  - Create `IERC20::new(token)` interface instance
  - Get `balance_before = token.balance_of(to)?`
  - Call `token.transfer_from(from, to, amount)?` (propagate errors as
    TransferFailed)
  - Get `balance_after = token.balance_of(to)?`
  - Calculate `received = balance_after.checked_sub(balance_before)?`
  - Verify `received == amount`, else return FeeOnTransferDetected

**Validation:**

- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] `cargo fmt --check` passes

**Design Decision**: Fee-on-transfer tokens break collateral accounting
(catastrophic). We can detect and prevent this at protocol level.
Rebasing/blacklist tokens are user-assumed risks (documented).

## Task 7. Option Metadata Storage

Store and retrieve option parameters.

**TTDD Step 1 - Types:**

- [ ] Define `OptionMetadata` in `sol_storage!` block:
  - `address underlying`
  - `address quote`
  - `uint8 underlying_decimals`
  - `uint8 quote_decimals`
  - `uint256 strike` (18 decimals)
  - `uint256 expiry`
  - `uint8 option_type` (0=Call, 1=Put)
- [ ] Add to `Options` storage:
  - `StorageMap<B256, OptionMetadata> option_metadata`

**TTDD Step 2 - Tests:**

- [ ] Test: Store metadata and retrieve it by token ID
- [ ] Test: Metadata fields match input parameters
- [ ] Test: Same token ID retrieves same metadata
- [ ] Test: Different token IDs have independent metadata

**TTDD Step 3 - Implementation:**

- [ ] Add
      `store_option_metadata(token_id: B256, underlying: Token, quote: Token, strike: U256, expiry: U256, option_type: OptionType) -> Result<(), OptionsError>`
  - Create `OptionMetadata` instance
  - Store in `self.option_metadata` using `token_id` as key
- [ ] Add
      `get_option_metadata(token_id: B256) -> Result<OptionMetadata, OptionsError>`
  - Retrieve from `self.option_metadata`
  - Return error if not found

**Validation:**

- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] `cargo fmt --check` passes

**Design Decision**: Metadata stored once per token ID on first write. All
subsequent writes of same option series reuse metadata. Normalized strike price
stored for consistency.

## Task 8. Writer Position Tracking

Track collateral locked by each writer per option series.

**TTDD Step 1 - Types:**

- [ ] Define `Position` in `sol_storage!` block:
  - `uint256 quantity_written` (18 decimals)
  - `uint256 collateral_locked` (18 decimals)
- [ ] Add to `Options` storage:
  - `StorageMap<B256, Position> positions`

**TTDD Step 2 - Tests:**

- [ ] Test: Create new position stores quantity and collateral
- [ ] Test: Increase existing position accumulates correctly
- [ ] Test: Different writers same token ID have independent positions
- [ ] Test: Position key is deterministic for (writer, token_id)

**TTDD Step 3 - Implementation:**

- [ ] Add `position_key(writer: Address, token_id: B256) -> B256` helper
  - Use `keccak256(abi.encodePacked(writer, token_id))`
- [ ] Add
      `create_or_update_position(writer: Address, token_id: B256, quantity: U256, collateral: U256) -> Result<(), OptionsError>`
  - Get position using `position_key(writer, token_id)`
  - If exists: update with `checked_add`
  - If new: create new `Position`
  - Store back to `self.positions`

**Validation:**

- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] `cargo fmt --check` passes

**Design Decision**: Each writer has independent position per token ID. Position
keys use `keccak256(writer, tokenId)` for deterministic lookups. StorageMap
doesn't iterate, so events provide queryability.

## Task 9. Write Call Option Implementation

Complete call option writing with collateral transfer.

**TTDD Step 1 - Types:**

- [ ] Define events in `sol!` block:
  - `event OptionWritten(address indexed writer, bytes32 indexed tokenId, uint256 quantity, uint256 collateral)`

**TTDD Step 2 - Tests:**

_Success Cases:_

- [ ] Test: Write 1 WBTC call (8 decimals) succeeds
  - Verify token ID returned
  - Verify writer balance increased
  - Verify total supply increased
  - Verify position created
  - Verify metadata stored
  - Verify collateral transferred
  - Verify event emitted
- [ ] Test: Write same option twice increases position
- [ ] Test: Different writers same option create separate positions
- [ ] Test: Write different options creates different token IDs

_Failure Cases:_

- [ ] Test: Invalid parameters fail validation (reuse Task 5 tests)
- [ ] Test: Fee-on-transfer token fails (reuse Task 6 tests)
- [ ] Test: Insufficient ERC20 balance fails
- [ ] Test: No ERC20 approval fails

_Edge Cases:_

- [ ] Test: Minimum values (quantity=1, strike=1)
- [ ] Test: Large values (near U256::MAX, should handle or error gracefully)
- [ ] Test: Various decimals (0, 6, 8, 18)

**TTDD Step 3 - Implementation:**

- [ ] Replace stub in `write_call_option()`
  - **Step 1 - Checks:**
    - Call `validate_write_params(strike, expiry, quantity, underlying, quote)?`
  - **Step 2 - Effects:**
    - Generate `token_id = generate_token_id(...)`
    - Normalize
      `normalized_quantity = normalize_amount(quantity, underlying.decimals)?`
    - Calculate `collateral_native = quantity` (1:1 for calls in native
      decimals)
    - Calculate `collateral_normalized = normalized_quantity` (for storage)
    - Store metadata if first write: `store_option_metadata(...)?`
    - Update position:
      `create_or_update_position(msg::sender(), token_id, normalized_quantity, collateral_normalized)?`
    - Mint tokens: `_mint(msg::sender(), token_id, normalized_quantity)?`
  - **Step 3 - Interactions:**
    - Transfer collateral:
      `safe_transfer_from(underlying.address, msg::sender(), contract_address(), collateral_native)?`
    - Emit event:
      `OptionWritten { writer: msg::sender(), tokenId: token_id, quantity: normalized_quantity, collateral: collateral_normalized }`
  - Return `token_id`

**Validation:**

- [ ] `cargo test` passes all tests
- [ ] `cargo clippy` passes
- [ ] `cargo fmt --check` passes
- [ ] `cargo build --target wasm32-unknown-unknown --release` succeeds

**Design Decision**: Strict checks-effects-interactions pattern prevents
reentrancy. All state updates before external calls. Events provide off-chain
queryability. 1:1 collateral ensures covered calls.

## Task 10. Property-Based Testing

Verify invariants hold across random inputs.

**TTDD Step 1 - Types:**

- [ ] Add `proptest` dependency to `Cargo.toml`

**TTDD Step 2 & 3 - Tests & Implementation:**

- [ ] Property: Token ID determinism
  - Generate random option parameters
  - Call `generate_token_id` N times
  - Verify all results identical
- [ ] Property: Decimal round-trip
  - Generate random (amount, decimals) where decimals ∈ [0, 18]
  - Normalize then denormalize
  - Verify result equals original
- [ ] Property: No arithmetic panics
  - Generate random (amount, decimals, quantity, strike, expiry)
  - Call all functions
  - Verify either Ok() or Err(), never panic
- [ ] Property: Balance invariant
  - Generate random mint/burn sequences
  - Verify balance + total_supply always consistent
- [ ] Property: Position invariant
  - Generate random write sequences
  - Verify position.quantity_written == sum of writes
  - Verify position.collateral_locked == sum of collateral

**Validation:**

- [ ] `cargo test` passes all property tests
- [ ] Tests run reasonable number of iterations (100-1000)

**Design Decision**: Property-based tests find edge cases unit tests miss.
Critical for financial contracts where edge cases = fund loss.

## Task 11. Final Integration Testing & Quality Checks

End-to-end validation and quality gates.

**Integration Tests:**

- [ ] Test: Full happy path flow
  - Deploy MockERC20
  - Mint tokens to writer
  - Approve contract
  - Write call option
  - Verify all state (balance, position, metadata, supply)
- [ ] Test: Multiple writers multiple options
  - Multiple writers write different options
  - Verify token IDs unique
  - Verify positions independent
- [ ] Test: Position accumulation
  - Same writer writes same option 3 times
  - Verify position accumulates correctly
  - Verify total supply matches sum

**Quality Checks:**

- [ ] Run `cargo test` - all tests pass (unit + property + integration)
- [ ] Run `cargo fmt --check` - code formatted
- [ ] Run
      `cargo clippy --all-targets --all-features -- -D clippy::all -D warnings` -
      zero warnings
- [ ] Run `cargo build --target wasm32-unknown-unknown --release` - WASM builds
- [ ] Manual review:
  - No `unwrap()`, `expect()`, `panic!()` in production code
  - All arithmetic uses checked operations
  - All public functions have doc comments
  - Checks-effects-interactions pattern followed
  - All error variants have clear descriptions
- [ ] Review against AGENTS.md guidelines
- [ ] Delete PLAN.md before creating PR

---

## Summary

**Total Tasks**: 11 vertical slices **Each task delivers**: Complete, testable
feature **TTDD approach**: Types → Tests → Implementation for every task
**Validation**: Tests + clippy + fmt after each task

**Key improvements over original plan:**

1. ✅ Each task is independently testable
2. ✅ No circular dependencies
3. ✅ Mock tokens early (Task 3) enable testing
4. ✅ Storage added incrementally as needed
5. ✅ Quality checks after each task, not just at end
6. ✅ True vertical slices with working features

**Dependencies**: Issue #2 (completed) **Blocks**: Issue #5 (Put Option
Writing), Issue #6 (Exercise), Issue #11 (Full ERC-1155)
