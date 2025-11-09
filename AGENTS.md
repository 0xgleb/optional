# AGENTS.md

This file provides guidance to AI agents working with code in this repository.

**CRITICAL: This codebase contains immutable permissionless smart contracts.
Code quality, security, and correctness standards are EXTREMELY strict. Deployed
contracts CANNOT be upgraded. Bugs can lead to permanent loss of user funds.**

Relevant docs:

- README.md
- SPEC.md

## Plan & Review

### Before starting work

- Write a comprehensive step-by-step plan to PLAN.md with each task having a
  corresponding section and a list of subtasks as checkboxes inside of it
- The task sections should follow the format `## Task N. <TASK NAME>`
- The plan should be a detailed implementation plan and the reasoning behind the
  design decisions
- Do not include timelines in the plan as they tend to be inaccurate
- Remain focused on the task at hand, do not include unrelated improvements or
  premature optimizations
- Once you write the plan, ask me to review it. Do not continue until I approve
  the plan.

### While implementing

- **CRITICAL: Complete tasks one at a time and wait for review**
  - When asked to complete a task from a plan, complete ONLY that task
  - Do NOT proceed to the next task until the user reviews and approves your
    changes
  - The user manually reviews all git diffs, so changes must be minimal and
    focused
  - **Any diff not required to complete the task is a guideline violation** - no
    drive-by improvements, refactorings, or style changes unless explicitly
    included in the scope of the task or requested by the user
  - Exception: If the user explicitly asks you to "complete the whole plan", you
    may work through multiple tasks
  - By default, always work one task at a time
- **CRITICAL: Tasks must be ordered correctly in plans**
  - When creating implementation plans, ensure tasks are in the correct order
  - Earlier tasks MUST NOT depend on code from later tasks
  - All checks (tests, clippy, fmt) SHOULD pass at the end of each task whenever
    possible
  - Focused git diffs and passing checks make reviewing much easier
- **CRITICAL: Keep PLAN.md in sync with implementation decisions**
  - If you change approach during implementation, immediately update PLAN.md to
    reflect the new approach
  - Plans are living documents during development - update them when you
    discover better solutions
  - Implementation and plan must always match - out-of-sync plans are worse than
    no plan
- Update PLAN.md every time you complete a task by marking checkboxes as `[x]`
- Keep PLAN.md concise - just tick off checkboxes, do not add "Changes Made"
  sections or verbose changelogs
- The code diffs themselves should be self-explanatory and easy to review

### Before creating a PR

- **CRITICAL**: Delete PLAN.md before submitting changes for review
- PLAN.md is a transient development file that should ONLY exist on development
  branches
- PLAN.md should NEVER appear in pull requests or be merged to main/master
- The plan is for development tracking only - final documentation goes in commit
  messages, docstrings, and permanent markdown documents
- **CRITICAL**: Update all documentation to reflect your changes
  - **SPEC.md**: Review and update if your changes affect:
    - User flows or contract execution flows
    - Storage structures or data layouts
    - Architecture decisions
    - Security considerations
    - Token ID calculation or normalization
    - Collateral requirements
    - Settlement mechanisms
  - **README.md**: Review and update if your changes affect:
    - Project overview or capabilities
    - Development setup or commands
    - Deployment instructions
    - Architecture overview
  - **AGENTS.md**: Update if you introduce new patterns, practices, or
    conventions that other developers should follow
  - Out-of-date documentation has negative value - it confuses more than it
    clarifies

## Project Overview

This is a fully on-chain Central Limit Order Book (CLOB) for options trading,
built on Arbitrum using Stylus (Rust/WASM). The system prioritizes simplicity
and reliability through physical settlement with 100% collateralization,
eliminating the need for oracles, risk management systems, and liquidation
mechanisms.

**Key Features:**

- **OptionsToken Contract**: ERC-1155 option tokens with physical settlement
- **CLOB Contract**: On-chain orderbook with price-time priority matching
- **100% Collateralization**: No fractional reserve, no liquidations needed
- **Oracle-Free**: Physical settlement means holders decide when to exercise
- **Permissionless**: Any ERC20 token pair can have options created

**CRITICAL - Immutability:** These are immutable smart contracts deployed on
Arbitrum. Once deployed, they CANNOT be upgraded. Every bug is permanent. Every
security flaw is exploitable forever. Code quality and correctness are
non-negotiable.

See SPEC.md for complete system architecture, user flows, and technical details.

## Key Development Commands

### Stylus Development

**CRITICAL: Always use Stylus-specific tooling for smart contract development**

- `cargo stylus check` - Check contract validity and compatibility with Stylus
  - **NOTE**: This command can be buggy and may hang or return errors. The user
    will typically run this themselves. If running via automation, use a timeout
    (e.g., 60 seconds).
- `cargo stylus deploy` - Deploy contract to Arbitrum (requires funded wallet)
- `cargo stylus activate` - Activate deployed contract (required every 365 days)
- `cargo stylus export-abi` - Export contract ABI for frontend integration
- `cargo stylus replay-tx <TX_HASH>` - Replay transaction for debugging

### Building & Testing

- `cargo build --target wasm32-unknown-unknown` - Build WASM contract
- `cargo test` - Run all tests (unit + property-based)
- `cargo test --lib` - Run library tests only
- `cargo test -q <test_name>` - Run specific test quietly

### Development Tools

- `cargo fmt` - Format code
- `cargo clippy --all-targets --all-features -- -D clippy::all -D warnings` -
  Run Clippy with all warnings denied
- `cargo check` - Fast type checking without building WASM

### Nix Development Environment

- `nix develop` - Enter development shell with all dependencies (Rust,
  cargo-stylus, solc, etc.)

## Development Workflow Notes

- When running `git diff`, make sure to add `--no-pager` to avoid opening it in
  the interactive view, e.g. `git --no-pager diff`
- Always build for `wasm32-unknown-unknown` target when testing contract
  compatibility
- Use `cargo stylus check` frequently to catch Stylus-specific issues early

## Architecture Overview

### Two-Contract Design

The system consists of two separate Stylus contracts:

#### OptionsToken Contract

- **ERC-1155 implementation**: Multi-token standard for option series
- **Collateral custody**: Holds ALL user collateral (underlying and quote
  tokens)
- **Option lifecycle management**: Minting, exercise signaling, settlement,
  burning
- **Standalone functionality**: Users can write and exercise options without
  CLOB
- **Token ID generation**: Deterministic hash of option parameters (underlying,
  quote, strike, expiry, type)

#### CLOB Contract

- **Orderbook storage**: Price-time priority matching engine
- **Trading venue**: Facilitates secondary market for option tokens
- **Modular**: Optional component, options work independently
- **ERC-1155 integration**: Requires approval to trade option tokens

**Why Separate:**

- Options are fully composable (tradeable on AMMs, other DEXs, OTC)
- CLOB is just one possible trading venue
- Clear security boundaries
- Can deploy improved CLOB versions without affecting options

### Critical Storage Patterns

**Stylus StorageMap Limitations:**

Stylus `StorageMap` is equivalent to Solidity `mapping` with critical
constraints:

1. **No iteration**: Cannot enumerate keys or values
2. **No deletion**: Can only zero out values, slots remain allocated
3. **No size queries**: Cannot get count of entries

**Implications for Contract Design:**

```rust
// WRONG: IMPOSSIBLE - StorageMap doesn't support iteration
for position in positions.iter() { ... }

// CORRECT: REQUIRED - Use deterministic keys and off-chain indexing
let position_key = keccak256(abi.encode(writer, token_id));
let position = positions.get(position_key);

// Rely on events + subgraphs for queryability
emit PositionCreated { writer, token_id, collateral };
```

**Design Decisions:**

- Use deterministic keys: `keccak256(writer, tokenId)` for positions
- Emit comprehensive events for off-chain indexing
- Accept that on-chain enumeration is impossible
- Build subgraphs to provide queryable views of contract state

### Token Decimals Normalization

ERC20 tokens have varying decimals (USDC = 6, WBTC = 8, standard = 18). All
amounts are normalized to 18 decimals internally:

$$\text{normalized\_amount} = \text{amount} \times 10^{(18 - \text{decimals})}$$

**Example: 1 WBTC (8 decimals) call at 60,000 USDC (6 decimals) strike**

- Normalized underlying: $1 \times 10^{8} \times 10^{10} = 10^{18}$
- Normalized strike: $60000 \times 10^{6} \times 10^{12} = 60000 \times 10^{18}$

**Critical Implementation Rules:**

- ALWAYS call `decimals()` dynamically - NEVER hardcode
- Use 18-decimal precision for all internal math
- Convert to native decimals ONLY for ERC20 transfers
- Use checked arithmetic for all normalization operations

### Stylus Contract Reactivation

**CRITICAL: Yearly Reactivation Required**

Stylus contracts MUST be reactivated every 365 days or after any Stylus/ArbOS
upgrade to remain callable:

- Can be performed by anyone using `cargo-stylus` or ArbWasm precompile
- Necessary because WASM is lowered to native machine code during activation
- Contracts become NON-CALLABLE if not reactivated
- **Collateral remains safe but locked** if reactivation lapses
- Recommend automated monitoring and reactivation infrastructure

**Deployment Strategy:**

- Set up automated monitoring for contract expiry dates
- Implement reactivation service with redundancy
- Test reactivation process thoroughly before mainnet deployment

## Code Quality & Best Practices

### CRITICAL: Immutable Smart Contract Standards

**These contracts are IMMUTABLE. Bugs are PERMANENT. Code quality is
NON-NEGOTIABLE.**

- **Zero Tolerance for Panics**: ANY panic is catastrophic. Use `Result`
  everywhere.
- **Zero Tolerance for Unchecked Math**: ALL arithmetic must use checked
  operations
- **Zero Tolerance for Assumptions**: Validate EVERYTHING. Trust NOTHING.
- **Zero Tolerance for Shortcuts**: No TODOs, no placeholders, no "good enough"

### CRITICAL: Zero Tolerance for Panics

**In smart contracts, panics are CATASTROPHIC. They can lock user funds
forever.**

- **FORBIDDEN**: `unwrap()`, `expect()`, `panic!()`, `unreachable!()`,
  `unimplemented!()`
- **FORBIDDEN**: Index operations that can panic (`vec[i]`), use
  `.get(i).ok_or(Error)?`
- **FORBIDDEN**: Division without checking for zero
- **FORBIDDEN**: Unchecked arithmetic operations
- **REQUIRED**: Use `?` operator for error propagation
- **REQUIRED**: All fallible operations must return `Result` with descriptive
  errors
- **REQUIRED**: Use `checked_add`, `checked_sub`, `checked_mul`, `checked_div`
  for ALL arithmetic
- **Exception**: `unwrap()` and `expect()` are ONLY allowed in test code
  (`#[cfg(test)]`)

**Examples:**

```rust
// WRONG: CATASTROPHIC - Will panic and lock funds
fn calculate_collateral(strike: U256, quantity: U256) -> U256 {
    strike * quantity  // Can overflow!
}

// CORRECT: CORRECT - Returns error on overflow
fn calculate_collateral(strike: U256, quantity: U256) -> Result<U256, Error> {
    strike.checked_mul(quantity).ok_or(Error::Overflow)
}

// WRONG: CATASTROPHIC - Will panic on division by zero
fn calculate_price(total: U256, quantity: U256) -> U256 {
    total / quantity
}

// CORRECT: CORRECT - Checks for zero before dividing
fn calculate_price(total: U256, quantity: U256) -> Result<U256, Error> {
    if quantity.is_zero() {
        return Err(Error::ZeroQuantity);
    }
    total.checked_div(quantity).ok_or(Error::Overflow)
}

// WRONG: CATASTROPHIC - Will panic if index out of bounds
let order = orders[order_id];

// CORRECT: CORRECT - Returns error if not found
let order = orders.get(order_id).ok_or(Error::OrderNotFound)?;
```

### CRITICAL: Checked Arithmetic Everywhere

**Smart contracts must use checked arithmetic for ALL operations.
Overflows/underflows can lead to catastrophic losses.**

```rust
// WRONG: FORBIDDEN - Silent overflow
let total = a + b;
let result = total * c;

// CORRECT: REQUIRED - Explicit overflow checking
let total = a.checked_add(b).ok_or(Error::Overflow)?;
let result = total.checked_mul(c).ok_or(Error::Overflow)?;

// CORRECT: REQUIRED - Check underflow on subtraction
let remaining = total.checked_sub(filled).ok_or(Error::Underflow)?;

// CORRECT: REQUIRED - Check division by zero
if divisor.is_zero() {
    return Err(Error::DivisionByZero);
}
let quotient = dividend.checked_div(divisor).ok_or(Error::Overflow)?;
```

**No exceptions. Every single arithmetic operation must be checked.**

### CRITICAL: Financial Data Integrity

**This handles real user funds. Silent data corruption = catastrophic losses.**

**NEVER:**

- Defensive value capping that hides overflow/underflow
- Fallback to default values on conversion failure
- Silent truncation of precision
- Using `unwrap_or(default)` on financial calculations
- Conversion functions that "gracefully degrade"

**ALWAYS:**

- Use explicit error handling with proper error propagation
- Use checked arithmetic for ALL financial calculations
- Validate ALL inputs before processing
- Preserve precision in calculations
- Fail fast with clear errors rather than continue with corrupted data

**Examples:**

```rust
// WRONG: CATASTROPHIC - Silent data corruption
fn normalize_amount(amount: u64, from_decimals: u8) -> U256 {
    let scale_factor = 10u64.pow((18 - from_decimals) as u32);
    U256::from(amount * scale_factor)  // Can overflow!
}

// CORRECT: CORRECT - Explicit overflow checking
fn normalize_amount(amount: u64, from_decimals: u8) -> Result<U256, Error> {
    if from_decimals > 18 {
        return Err(Error::InvalidDecimals);
    }

    let scale_exp = 18 - from_decimals;
    let scale_factor = 10u64.checked_pow(scale_exp as u32)
        .ok_or(Error::ScaleFactorOverflow)?;

    let scaled = amount.checked_mul(scale_factor)
        .ok_or(Error::AmountOverflow)?;

    Ok(U256::from(scaled))
}

// WRONG: CATASTROPHIC - Hides conversion errors
fn u256_to_u64_safe(value: U256) -> u64 {
    value.to::<u64>().unwrap_or(u64::MAX)  // WRONG!
}

// CORRECT: CORRECT - Explicit conversion error
fn u256_to_u64(value: U256) -> Result<u64, Error> {
    value.to::<u64>().ok_or(Error::ValueTooLarge {
        value,
        max: U256::from(u64::MAX),
    })
}
```

### CRITICAL: Input Validation

**Validate ALL inputs at contract boundaries. Assume all external input is
adversarial.**

```rust
// CORRECT: REQUIRED - Validate before processing
pub fn write_option(
    &mut self,
    underlying: Address,
    quote: Address,
    strike: U256,
    expiry: u64,
    option_type: OptionType,
    quantity: U256,
) -> Result<U256, Error> {
    // Validate addresses
    if underlying == Address::ZERO || quote == Address::ZERO {
        return Err(Error::InvalidAddress);
    }

    // Validate strike
    if strike.is_zero() {
        return Err(Error::InvalidStrike);
    }

    // Validate expiry
    let current_time = block_timestamp();
    if expiry <= current_time {
        return Err(Error::ExpiredOption);
    }

    // Validate quantity
    if quantity.is_zero() {
        return Err(Error::InvalidQuantity);
    }

    // Now safe to process...
    self.mint_option_token(/* validated params */)
}
```

### CRITICAL: Reentrancy Protection

**External calls can reenter. ALWAYS follow checks-effects-interactions
pattern.**

```rust
// WRONG: DANGEROUS - State updated after external call
pub fn exercise_option(&mut self, token_id: U256, quantity: U256) -> Result<(), Error> {
    let option = self.get_option(token_id)?;

    // External call BEFORE state update
    option.underlying_token.transfer(msg::sender(), quantity)?;

    // State updated after external call - WRONG!
    self.burn_option_token(msg::sender(), token_id, quantity)?;

    Ok(())
}

// CORRECT: CORRECT - State updated before external call
pub fn exercise_option(&mut self, token_id: U256, quantity: U256) -> Result<(), Error> {
    let option = self.get_option(token_id)?;

    // 1. Checks - validate conditions
    self.validate_exercise(&option, quantity)?;

    // 2. Effects - update state BEFORE external calls
    self.burn_option_token(msg::sender(), token_id, quantity)?;

    // 3. Interactions - external calls LAST
    option.underlying_token.transfer(msg::sender(), quantity)?;

    Ok(())
}
```

### CRITICAL: Gas Optimization

**Gas costs determine economic viability on Arbitrum. Optimize critical paths.**

**Target gas costs (0.1 gwei, ~$0.05/tx):**

- Write option: ~~150k gas (~~$0.0075)
- Place limit order: ~~100k gas (~~$0.005)
- Market order (5 fills): ~~250k gas (~~$0.0125)
- Exercise signal: ~~30k gas (~~$0.0015)

**Optimization techniques:**

- Use `StorageMap` instead of `Vec` for lookups (O(1) vs O(n))
- Minimize `SSTORE` operations (most expensive)
- Batch operations where possible
- Use events for data that doesn't need on-chain storage
- Pack storage structs efficiently (Stylus does this automatically, but verify)

### CRITICAL: Unsafe Token Handling

**Certain token types are incompatible with the protocol:**

**Fee-on-Transfer Tokens:**

- Deduct fees during `transfer()`/`transferFrom()`
- **Impact**: Collateral shortfall, settlement failures
- **Protection**: Check balance before/after transfer, revert if mismatch

```rust
// CORRECT: REQUIRED - Detect fee-on-transfer tokens
fn safe_transfer_from(
    token: &ERC20,
    from: Address,
    to: Address,
    amount: U256,
) -> Result<(), Error> {
    let balance_before = token.balance_of(to)?;
    token.transfer_from(from, to, amount)?;
    let balance_after = token.balance_of(to)?;

    let received = balance_after.checked_sub(balance_before)
        .ok_or(Error::UnexpectedBalanceDecrease)?;

    if received != amount {
        return Err(Error::FeeOnTransferDetected);
    }

    Ok(())
}
```

**Rebasing Tokens:**

- Balances change automatically over time (stETH, aTokens)
- **Impact**: Accounting breaks, collateral mismatches
- **Protection**: None at protocol level - document risks, user assumes risk

**Tokens with Blacklists:**

- Can blacklist addresses (USDC, USDT)
- **Impact**: Funds locked if writer/holder blacklisted
- **Protection**: None - known risk

**Approach:** Fully permissionless. Protect against fee-on-transfer (detectable)
and arithmetic overflow (checked math). Users assume risk for rebasing/blacklist
tokens.

### Type Modeling and Enums

**Use Rust's type system to make invalid states unrepresentable:**

```rust
// CORRECT: GOOD - Each state is explicit
pub enum OptionType {
    Call,
    Put,
}

pub enum OrderSide {
    Buy,
    Sell,
}

// CORRECT: GOOD - Each state has only the data it needs
pub enum OrderStatus {
    Active { price: U256, quantity: U256, filled: U256 },
    Filled { price: U256, quantity: U256, filled_at: u64 },
    Cancelled { cancelled_at: u64 },
}

// WRONG: BAD - Multiple fields that can contradict each other
pub struct Order {
    price: U256,
    quantity: U256,
    filled: U256,
    is_active: bool,
    is_filled: bool,
    is_cancelled: bool,
    filled_at: Option<u64>,
    cancelled_at: Option<u64>,
}
```

### Error Handling

**Use descriptive error types with context:**

```rust
#[derive(Debug)]
pub enum OptionsError {
    InvalidStrike { strike: U256 },
    ExpiredOption { expiry: u64, current: u64 },
    InsufficientCollateral { required: U256, provided: U256 },
    Overflow { operation: &'static str },
    OrderNotFound { order_id: U256 },
}

impl From<OptionsError> for Vec<u8> {
    fn from(err: OptionsError) -> Vec<u8> {
        // Convert to revert data for Stylus
        format!("{:?}", err).into_bytes()
    }
}
```

### Comments and Documentation

**Smart contracts require MORE documentation than typical code:**

#### DO comment:

- **Contract invariants**: Conditions that must ALWAYS hold
- **Mathematical formulas**: Especially for financial calculations
- **Complex business logic**: Non-obvious option settlement rules
- **External dependencies**: ERC20 transfer behaviors, gas assumptions
- **Security considerations**: Reentrancy risks, overflow scenarios
- **Storage layout**: How data is organized and why

#### DON'T comment:

- Obvious code (e.g., "transfer tokens" before `token.transfer()`)
- Function signatures (use doc comments `///` instead)
- Redundant explanations of what code clearly does

**Examples:**

```rust
/// Calculates collateral required for writing a put option.
///
/// # Invariant
/// Collateral = strike * quantity (in quote token decimals)
///
/// # Panics
/// Never panics - uses checked arithmetic
///
/// # Errors
/// Returns `Error::Overflow` if strike * quantity > U256::MAX
pub fn calculate_put_collateral(
    strike: U256,
    quantity: U256,
) -> Result<U256, Error> {
    // Strike is denominated in quote token (e.g., USDC)
    // Quantity is in underlying token units (normalized to 18 decimals)
    // Result must be converted to quote token decimals for transfer
    strike.checked_mul(quantity).ok_or(Error::Overflow)
}

// CORRECT: GOOD - Explains non-obvious logic
// We lock funds when signaling exercise to prevent writer griefing.
// If holder could signal without locking funds, they could block
// writer's collateral indefinitely without committing capital.
self.lock_exercise_funds(holder, token_id, strike_payment)?;

// CORRECT: GOOD - Documents security consideration
// CRITICAL: Burn tokens BEFORE external call to prevent reentrancy.
// If we burned after, malicious contract could reenter and burn twice.
self.burn(msg::sender(), token_id, quantity)?;
token.transfer(msg::sender(), amount)?;

// WRONG: BAD - Obvious from code
// Transfer tokens
token.transfer(recipient, amount)?;
```

### Module Organization

Organize smart contract code logically:

```rust
// Public external functions first (contract interface)
#[external]
impl OptionsToken {
    pub fn write_option(...) -> Result<U256, Error> { }
    pub fn signal_exercise(...) -> Result<(), Error> { }
}

// Internal helper functions below
impl OptionsToken {
    fn calculate_collateral(...) -> Result<U256, Error> { }
    fn validate_option_params(...) -> Result<(), Error> { }
}

// Storage structs and types at top of file
sol_storage! {
    pub struct OptionsToken { ... }
    pub struct Position { ... }
}

// Enums and constants
pub enum OptionType { Call, Put }
pub const MAX_EXPIRY_DURATION: u64 = 365 * 86400; // 1 year
```

### Import Organization

**CRITICAL: Follow consistent two-group import pattern:**

```rust
// Group 1: External crates (std, alloy, stylus-sdk, etc.)
use alloy_primitives::{Address, U256};
use stylus_sdk::prelude::*;
use stylus_sdk::storage::{StorageMap, StorageVec};

// Empty line separator

// Group 2: Internal modules (crate::, super::)
use crate::erc20::IERC20;
use crate::errors::OptionsError;
```

**FORBIDDEN:**

- Three or more import groups
- Empty lines between imports within a group
- Function-level imports

### Testing Strategy

**Smart contracts require exhaustive testing:**

#### Test Quality Guidelines

**CRITICAL: Tests must validate OUR code logic, not language primitives.**

**FORBIDDEN:**

```rust
// WRONG: Testing language primitives
#[test]
fn test_enum_equality() {
    assert_eq!(OptionType::Call, OptionType::Call);  // Tests Rust's ==, not our logic
}

#[test]
fn test_error_conversion() {
    let err = OptionsError::Invalid;
    let bytes: Vec<u8> = err.into();
    assert!(!bytes.is_empty());  // Tests format! and into_bytes(), not our logic
}
```

**CORRECT:**

```rust
// CORRECT: Testing our code logic
#[test]
fn test_write_option_stub_returns_unimplemented() {
    let result = contract.write_call_option(...);
    assert!(matches!(result, Err(OptionsError::Unimplemented)));  // Tests OUR stub
}

#[test]
fn test_calculate_collateral_overflow() {
    let result = contract.calculate_collateral(U256::MAX, U256::from(2));
    assert!(matches!(result, Err(OptionsError::Overflow)));  // Tests OUR overflow handling
}
```

**Rule**: If removing your code wouldn't break the test, the test is useless.

#### Incremental Development

**CRITICAL: Add types/errors/functionality incrementally as needed, not
upfront.**

**WRONG:**

```rust
// Defining 20 error variants when we only need 1
pub enum OptionsError {
    Unimplemented,
    InvalidAddress,
    InvalidStrike,
    // ... 17 more variants we don't use yet
}
```

**CORRECT:**

```rust
// Start with only what's needed
pub enum OptionsError {
    Unimplemented,
    // Additional error variants will be added as needed during implementation
}

// Add errors when implementing the logic that needs them
```

**Rule**: Don't write code for future requirements. Add it when you need it.

### CRITICAL: Type-Test Driven Development (TTDD)

**For smart contracts, use TTDD methodology: Types → Tests → Implementation →
Refine.**

This is an adaptation of Test-Driven Development (TDD) that leverages Rust's
type system to make invalid states unrepresentable BEFORE writing tests or
implementation.

**TTDD Iteration (one per task):**

1. **Types**: Define/refine types, enums, structs, error variants needed for
   THIS task
2. **Tests**: Write tests that specify expected behavior for THIS task
3. **Implementation**: Write minimal code to make tests pass for THIS task
4. **Next Iteration**: Start the next task with step 1 (types), NOT step 3

**Critical Rules:**

- **Complete each step sequentially**: Types → Tests → Implementation, in that
  order
- **One iteration per task**: If planned correctly, each task = one TTDD
  iteration
- **Next iteration starts with types**: Don't jump straight to implementation on
  task 2
- **Refine types between iterations**: Update types first when starting a new
  task

**Example TTDD Flow (Task 1: "Validate option parameters"):**

```rust
// STEP 1: Types (what domain types do we need?)
pub enum OptionType { Call, Put }

pub struct OptionParams {
    underlying: Address,
    quote: Address,
    strike: U256,
    expiry: u64,
    option_type: OptionType,
}

pub enum OptionsError {
    // Variants added during tests/implementation as needed
}

// STEP 2: Tests (what behavior do we need? what can go wrong?)
#[cfg(test)]
mod tests {
    #[motsu::test]
    fn test_validates_zero_strike(contract: OptionsToken) {
        let result = contract.validate_params(params_with_zero_strike());
        assert!(matches!(result, Err(OptionsError::InvalidStrike)));
        //                              ^--- Need InvalidStrike variant - add it now
    }

    #[motsu::test]
    fn test_validates_zero_address(contract: OptionsToken) {
        let result = contract.validate_params(params_with_zero_address());
        assert!(matches!(result, Err(OptionsError::InvalidAddress)));
        //                              ^--- Need InvalidAddress variant - add it now
    }
}

// After writing tests, OptionsError now looks like:
pub enum OptionsError {
    InvalidStrike,
    InvalidAddress,
}

// STEP 3: Implementation (make tests pass)
impl OptionsToken {
    fn validate_params(&self, params: &OptionParams) -> Result<(), OptionsError> {
        if params.strike.is_zero() {
            return Err(OptionsError::InvalidStrike);
        }
        if params.underlying == Address::ZERO || params.quote == Address::ZERO {
            return Err(OptionsError::InvalidAddress);
        }
        Ok(())
    }
}
```

**Example Next Iteration (Task 2: "Calculate collateral requirements"):**

```rust
// STEP 1: Types (what NEW domain types do we need?)
// Maybe add a CollateralAmount newtype for type safety?
pub struct CollateralAmount(U256);

// OptionsError already exists, don't add variants yet

// STEP 2: Tests (what NEW behavior/failures do we need to test?)
#[cfg(test)]
mod tests {
    #[motsu::test]
    fn test_call_collateral_equals_quantity(contract: OptionsToken) {
        let collateral = contract.calculate_call_collateral(U256::from(100));
        assert_eq!(collateral.unwrap(), U256::from(100));
    }

    #[motsu::test]
    fn test_collateral_overflow_returns_error(contract: OptionsToken) {
        let result = contract.calculate_put_collateral(U256::MAX, U256::from(2));
        assert!(matches!(result, Err(OptionsError::Overflow)));
        //                              ^--- Need Overflow variant - add it now
    }
}

// After writing tests, OptionsError now looks like:
pub enum OptionsError {
    InvalidStrike,
    InvalidAddress,
    Overflow,  // NEW: added when writing overflow test
}

// STEP 3: Implementation (make tests pass)
impl OptionsToken {
    fn calculate_call_collateral(&self, quantity: U256) -> Result<U256, OptionsError> {
        Ok(quantity)  // Minimal: 1:1 collateral
    }

    fn calculate_put_collateral(&self, strike: U256, quantity: U256) -> Result<U256, OptionsError> {
        strike.checked_mul(quantity).ok_or(OptionsError::Overflow)
        //                                  ^--- Compiler might also tell us we need Overflow variant here
    }
}
```

**Why This Sequence Matters:**

- **Types first**: Prevents writing tests/code with wrong abstractions
- **Tests second**: Specifies behavior before implementation bias sets in
- **Implementation third**: Guided by both type constraints and test
  requirements
- **Restart with types**: Each task may need new types - don't skip this step!

**Anti-Pattern (DON'T DO THIS):**

```rust
// WRONG: Jumping straight to implementation on Task 2
// without considering if new types/errors are needed

// Task 1: validation (has types + tests + implementation)
// Task 2: collateral -- SKIP STRAIGHT TO IMPLEMENTATION -- WRONG!
impl OptionsToken {
    fn calculate_collateral(...) -> U256 {  // WRONG: no error handling!
        strike * quantity  // WRONG: unchecked math!
    }
}
```

**Rule**: Each task begins with "What types/errors do I need?" NOT "What code do
I write?"

#### Unit Tests with Motsu

```rust
#[cfg(test)]
mod tests {
    use motsu::prelude::*;

    #[motsu::test]
    fn test_calculate_call_collateral(contract: OptionsToken) {
        let quantity = U256::from(100);
        let result = contract.calculate_call_collateral(quantity);

        assert_eq!(result.unwrap(), U256::from(100));
    }

    #[motsu::test]
    fn test_overflow_protection(contract: OptionsToken) {
        let result = contract.calculate_collateral(U256::MAX, U256::from(2));

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Overflow));
    }
}
```

#### Property-Based Tests with Proptest

```rust
#[cfg(test)]
mod proptests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn collateral_never_overflows(
            strike in 0u64..1_000_000_000,
            quantity in 0u64..1_000_000_000,
        ) {
            let result = calculate_collateral(
                U256::from(strike),
                U256::from(quantity),
            );

            // Either succeeds or returns Overflow error
            match result {
                Ok(collateral) => assert!(collateral >= U256::from(strike)),
                Err(Error::Overflow) => (),
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }

        #[test]
        fn exercise_signal_locks_funds_idempotent(
            token_id in 0u64..1000u64,
            quantity in 1u64..1_000_000u64,
        ) {
            let mut contract = OptionsToken::default();

            // Signal exercise twice with same params
            let result1 = contract.signal_exercise(U256::from(token_id), U256::from(quantity));
            let result2 = contract.signal_exercise(U256::from(token_id), U256::from(quantity));

            // First succeeds or fails, second always fails (idempotency)
            if result1.is_ok() {
                assert!(result2.is_err());
            }
        }
    }
}
```

#### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    // Test complete flows: write option -> trade -> exercise -> settle

    #[test]
    fn test_full_call_option_lifecycle() {
        // Setup contracts
        let mut options_token = deploy_options_token();
        let mut clob = deploy_clob();

        // Write call option
        let token_id = options_token.write_option(/* params */)?;

        // Trade on CLOB
        clob.place_order(token_id, price, quantity)?;
        clob.market_order(token_id, quantity)?;

        // Exercise at expiry
        options_token.signal_exercise(token_id, quantity)?;
        advance_time_to_expiry();
        options_token.finalize_expiry(token_id)?;

        // Verify settlement
        assert_eq!(holder_balance_after - holder_balance_before, expected_payout);
    }
}
```

### Test Coverage Requirements

**CRITICAL: Smart contracts require near-100% test coverage:**

- **Every function**: Unit test with valid inputs
- **Every error path**: Test that errors trigger correctly
- **Every edge case**: Boundary values, overflows, underflows
- **Every invariant**: Property tests to verify invariants hold
- **Every external integration**: Mock ERC20 interactions
- **Full user flows**: End-to-end integration tests

**If a line of code isn't tested, assume it's broken.**

## Workflow Best Practices

- **Always run tests, clippy, and formatters before handing over a piece of
  work**
  - Run `cargo test` first, as changing tests can break clippy
  - Run
    `cargo clippy --all-targets --all-features -- -D clippy::all -D warnings`
    next
  - Run `cargo stylus check` to verify Stylus compatibility
  - Always run `cargo fmt` last to ensure clean code formatting

### CRITICAL: Lint Policy

**NEVER add `#[allow(...)]` attributes or disable any lints without explicit
user permission.**

**Required approach for lint issues:**

1. **Refactor the code** to address the root cause
2. **Break down large functions** into smaller, focused functions
3. **Improve code structure** to meet clippy's standards
4. **Use proper error handling** instead of suppressing warnings

**FORBIDDEN:**

```rust
// WRONG: NEVER DO THIS
#[allow(clippy::too_many_arguments)]
fn process_order(...) { }

#[allow(clippy::arithmetic_side_effects)]
let total = a + b + c;  // Use checked_add instead!
```

**If you encounter a lint issue:**

1. Understand WHY clippy is flagging the code
2. Refactor the code to address the underlying problem
3. If you believe a lint is incorrect, ask for permission before suppressing it

**Exception for third-party macro-generated code:**

```rust
// CORRECT: ACCEPTABLE - For external macro-generated code
sol_interface! {
    #[allow(clippy::too_many_arguments)]
    interface IERC20 {
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
    }
}
```

## Security Considerations

### Pre-Deployment Checklist

Before deploying to mainnet, verify:

- [ ] ALL arithmetic uses checked operations
- [ ] NO panics possible in any code path
- [ ] ALL inputs validated at contract boundaries
- [ ] Checks-effects-interactions pattern followed everywhere
- [ ] Fee-on-transfer token detection implemented
- [ ] Near-100% test coverage achieved
- [ ] Property-based tests verify critical invariants
- [ ] Full integration tests cover all user flows
- [ ] Gas costs measured and acceptable
- [ ] Stylus reactivation monitoring planned
- [ ] Multiple security reviews completed
- [ ] Formal verification considered for critical functions

### Known Attack Vectors

**Reentrancy:**

- Always update state before external calls
- Never trust external contract behavior
- Consider reentrancy guards on sensitive functions

**Integer Overflow/Underflow:**

- Use checked arithmetic EVERYWHERE
- Rust doesn't panic on overflow in release mode - MUST use checked methods

**Griefing:**

- Exercise signaling locks funds to prevent griefing writers
- Consider minimum order sizes to prevent spam attacks

**Front-Running:**

- Price-time priority provides FIFO ordering at same price level
- Market orders vulnerable to sandwiching - document this clearly
- Consider slippage protection for future versions

**Gas Manipulation:**

- Limit iterations in loops (e.g., max orders per price level)
- Bounded orderbook depth to prevent DoS

### Audit Recommendations

**Before mainnet deployment:**

1. **Internal review**: Multiple team members review every line
2. **External audit**: Professional smart contract auditor (Trail of Bits,
   OpenZeppelin, Consensys Diligence)
3. **Formal verification**: Consider for critical functions (collateral
   calculation, settlement)
4. **Bug bounty**: Offer rewards for vulnerabilities before mainnet
5. **Testnet deployment**: Extensive testing on Arbitrum Sepolia
6. **Limited mainnet**: Deploy with caps/limits initially, remove after
   confidence grows

## Additional Resources

### Stylus Documentation

- [Stylus Gentle Introduction](https://docs.arbitrum.io/stylus/stylus-gentle-introduction)
- [Rust SDK Reference](https://docs.rs/stylus-sdk/latest/stylus_sdk/)
- [Stylus by Example](https://stylus-by-example.org)
- [OpenZeppelin Stylus Contracts](https://github.com/OpenZeppelin/rust-contracts-stylus)

### Testing Resources

- [Motsu: Pure Rust testing framework for Stylus](https://github.com/OpenZeppelin/rust-contracts-stylus/tree/main/lib/motsu)
- [Proptest: Property-based testing in Rust](https://github.com/proptest-rs/proptest)

### Smart Contract Security

- [Consensys Smart Contract Best Practices](https://consensys.github.io/smart-contract-best-practices/)
- [Trail of Bits Building Secure Contracts](https://github.com/crytic/building-secure-contracts)
- [OpenZeppelin Stylus Audit Report](https://blog.openzeppelin.com/openzeppelin-contracts-for-stylus-audit)

---

**Remember: These contracts are IMMUTABLE. Every decision is PERMANENT. Code
quality and security are absolutely NON-NEGOTIABLE. When in doubt, be MORE
strict, not less.**
