# SPEC.md

This specification outlines a fully on-chain Central Limit Order Book (CLOB) for
options trading, built on Arbitrum using Stylus (Rust/WASM) for
compute-intensive operations. The design prioritizes simplicity and reliability
through physical settlement with 100% collateralization, eliminating the need
for oracles, risk management systems, and liquidation mechanisms in the initial
version.

## Overview

### PoC Scope

- Users can write (sell) and buy options as ERC-1155 tokens
- Options trade on a fully on-chain CLOB with price-time priority matching
- Settlement is physical (actual token delivery) with manual exercise
- All collateral is 100% locked in the underlying assets (no fractional reserve)
- ERC20 token pairs
- American options (exercise any time before expiry)

### Future work

- Cash settlement (requires oracles and risk management)
- Automatic exercise at maturity (requires oracles)
- Advanced order types
- Native token support

### Key Architectural Decisions

- **Trustless by Design**: Physical settlement means no reliance on external
  price feeds
- **Simplicity First**: 100% collateralization eliminates complex risk
  management
- **Future Compatible**: Architecture supports adding cash settlement and
  oracles later
- **Gas Efficient**: All contracts in Rust/Stylus for maximum performance
- **Permissionless**: Any ERC20 token pair can have options created

### Definitions

**Call Option**: Right (not obligation) to BUY the underlying ERC20 token at
strike price

- Holder: Pays premium, can exercise to buy underlying at strike price
- Writer: Receives premium, must deliver underlying token if holder exercises
- Collateral: Writer locks 1:1 underlying ERC20, e.g. 1 WBTC for 1 WBTC call
  (covered call)

**Put Option**: Right (not obligation) to SELL the underlying ERC20 token at
strike price

- Holder: Pays premium, can exercise to sell underlying at strike price
- Writer: Receives premium, must accept underlying and pay strike if holder
  exercises
- Collateral: Writer locks strike amount in quote token, e.g. $123,000 USDC for
  1 WBTC put at $123k strike (cash secured put)

---

**American Option**: An option that can be exercised at any time before or at
expiry. This PoC implements American-style exercise, allowing holders to
exercise immediately whenever profitable. The 100% collateralization model means
writers are always fully protected regardless of when exercise occurs.

---

**Physical Settlement**: Actual token delivery on exercise

- Call exercise: Holder pays strike in quote token -> receives underlying token
- Put exercise: Holder delivers underlying token -> receives strike in quote
  token
- No oracle required (holder decides if exercise is profitable)

## User Flows

#### Flow 1: Writing (Selling) an Option

Actors: Option Writer Steps:

1. Writer selects option parameters (underlying ERC20, quote ERC20, strike,
   expiry, type, quantity)
2. Contract calculates required collateral based on option type
3. Writer approves ERC20 token transfer to contract
4. Contract transfers collateral from writer
5. Contract mints ERC-1155 option tokens to writer
6. Writer can now sell these tokens via CLOB or elsewhere or hold them

Collateral:

- Calls: Underlying ERC20 tokens (1:1 ratio)
- Puts: Quote ERC20 tokens (strike \* quantity)

Outcome: Option tokens (ERC-1155) minted, collateral (ERC20) locked

#### Flow 2: Trading Options

Actors: Maker, Taker

##### Adding Liquidity (Maker)

Steps:

1. Maker places a limit order
2. Maker's tokens locked:

- Selling: ERC-1155 option tokens locked
- Buying: Quote ERC20 locked (price \* quantity)

3. Order added to orderbook at specified price level
4. Order waits for taker

Outcome: Limit order in orderbook

##### Taking Liquidity (Taker)

Steps:

1. Taker places a market order
2. Matching engine fills against best available prices:

- Buying: Matches ascending from best ask
- Selling: Matches descending from best bid

3. If insufficient liquidity for full quantity -> REVERT
4. If sufficient liquidity:

- ERC-1155 option tokens transfer: Seller -> Buyer
- Quote ERC20 premium transfer: Buyer -> Seller (at makers' prices)
- Maker orders filled/reduced (FIFO at each price)

Outcome: Taker receives full fill at makers' prices, or transaction reverts

#### Flow 3: Cancelling Orders

Actors: Maker

Steps:

1. Maker requests to cancel their order
2. Contract verifies order ownership
3. Order removed from orderbook
4. Locked tokens returned to maker:

- Sell orders: ERC-1155 option tokens unlocked
- Buy orders: Quote ERC20 unlocked

Outcome: Order deleted, locked tokens returned

#### Flow 4: Exercise (American-Style)

Actors: Option Holder

Steps:

1. Holder decides to exercise option tokens (any time before expiry)
2. Holder approves ERC20 token transfer to contract:
   - For calls: Quote tokens (strike payment)
   - For puts: Underlying tokens
3. Holder calls `exercise_call(tokenId, quantity)` or
   `exercise_put(tokenId, quantity)`
4. Contract executes immediately:
   - Transfers holder's payment to writer
   - Transfers writer's collateral to holder
   - Burns holder's ERC-1155 option tokens
   - Releases writer's position

**Constraints and Edge Cases:**

- **Partial exercise:** Holder can exercise any quantity <= their balance

  - Example: Own 10 options, exercise 7.5, keep 2.5 active

- **Timing:** Can exercise any time before expiry

  - Before expiry: Full exercise available
  - At/after expiry: Exercise disabled, options expire worthless

- **Requirements:**

  - Holder must have sufficient option token balance
  - Holder must have approved sufficient payment tokens
  - Holder must have sufficient payment token balance
  - Transaction must occur before expiry timestamp

- **No cancellation:** Exercise is immediate and irreversible

  - Tokens exchanged atomically in single transaction
  - No intermediate state

- **Multiple writers:** If option has multiple writers:
  - Exercise proportionally reduces all writers' positions (FIFO or pro-rata)
  - Payment distributed to writers accordingly
  - Implementation detail TBD

Outcome: Immediate settlement, tokens exchanged, option tokens burned

#### Flow 5: Collateral Withdrawal After Expiry

Actors: Option Writer

After expiry, any remaining unexercised options expire worthless. Writers can
reclaim their locked collateral for these expired positions.

Steps:

1. Time passes beyond expiry timestamp
2. Writer calls `withdraw_expired_collateral(tokenId)` or
   `withdraw_expired_collateral(tokenId, quantity)`
3. Contract verifies:
   - Current time > expiry timestamp
   - Writer has a position with locked collateral for this token
4. Contract transfers collateral back to writer:
   - For calls: Returns underlying tokens
   - For puts: Returns quote tokens (strike amount)
5. Contract reduces/closes writer's position

**Constraints:**

- **Only after expiry:** Cannot withdraw while options are still active
- **Partial withdrawal:** Writer can withdraw collateral for any quantity <=
  their unexercised position
- **Permissionless:** Anyone can call on behalf of writer (collateral goes to
  original writer)
- **No time limit:** Collateral remains available indefinitely after expiry

**Why needed:**

- With American options, holders exercise immediately when profitable
- Any options not exercised before expiry are out-of-the-money
- Writers deserve to reclaim this "unexercised" collateral
- No automatic processing - writers claim when convenient (low gas environment)

Outcome: Writer reclaims collateral for expired unexercised options

### Option Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Written
    Written --> Trading
    Trading --> Exercised: holder exercises (any time before expiry)
    Trading --> Expired: time passes expiry without exercise
    Expired --> CollateralWithdrawn: writer withdraws collateral
    Exercised --> [*]: tokens exchanged, writer position closed
    CollateralWithdrawn --> [*]: collateral returned to writer
```

### Contract execution flows

#### Write Call Option (No CLOB)

```mermaid
sequenceDiagram
    participant Writer
    participant UnderlyingERC20
    participant OptionsToken

    Writer->>UnderlyingERC20: approve(OptionsToken, collateral)
    Writer->>OptionsToken: writeOption(underlying, quote, strike, expiry, CALL, quantity)
    OptionsToken->>UnderlyingERC20: transferFrom(Writer, OptionsToken, collateral)
    OptionsToken->>OptionsToken: mint ERC-1155 tokens to Writer
    OptionsToken->>OptionsToken: record position (Writer, collateral_locked)
```

#### Write Put Option (No CLOB)

```mermaid
sequenceDiagram
    participant Writer
    participant QuoteERC20
    participant OptionsToken

    Writer->>QuoteERC20: approve(OptionsToken, strike_collateral)
    Writer->>OptionsToken: writeOption(underlying, quote, strike, expiry, PUT, quantity)
    OptionsToken->>QuoteERC20: transferFrom(Writer, OptionsToken, strike_collateral)
    OptionsToken->>OptionsToken: mint ERC-1155 tokens to Writer
    OptionsToken->>OptionsToken: record position (Writer, collateral_locked)
```

#### Trade Options

```mermaid
sequenceDiagram
    participant Seller
    participant OptionsToken
    participant CLOB
    participant QuoteERC20
    participant Buyer

    Seller->>OptionsToken: setApprovalForAll(CLOB, true)
    Seller->>CLOB: placeOrder(tokenId, price, quantity, SELL)
    CLOB->>CLOB: Add order to book

    Buyer->>QuoteERC20: approve(CLOB, premium)
    Buyer->>CLOB: marketOrder(tokenId, quantity, BUY)
    CLOB->>OptionsToken: safeTransferFrom(Seller, Buyer, tokenId, quantity)
    CLOB->>QuoteERC20: transferFrom(Buyer, Seller, premium)
```

#### Call Exercise

```mermaid
sequenceDiagram
    participant Holder
    participant QuoteERC20
    participant OptionsToken
    participant UnderlyingERC20
    participant Writer

    Note over Holder,OptionsToken: Any time before expiry
    Holder->>QuoteERC20: approve(OptionsToken, strike_payment)
    Holder->>OptionsToken: exercise_call(tokenId, quantity)

    Note over OptionsToken: Immediate atomic settlement
    OptionsToken->>QuoteERC20: transferFrom(Holder, Writer, strike_payment)
    OptionsToken->>UnderlyingERC20: transfer(Holder, underlying_from_collateral)
    OptionsToken->>OptionsToken: burn Holder's ERC-1155 tokens
    OptionsToken->>OptionsToken: reduce/close Writer's position
```

#### Put Exercise

```mermaid
sequenceDiagram
    participant Holder
    participant UnderlyingERC20
    participant OptionsToken
    participant QuoteERC20
    participant Writer

    Note over Holder,OptionsToken: Any time before expiry
    Holder->>UnderlyingERC20: approve(OptionsToken, underlying_amount)
    Holder->>OptionsToken: exercise_put(tokenId, quantity)

    Note over OptionsToken: Immediate atomic settlement
    OptionsToken->>UnderlyingERC20: transferFrom(Holder, Writer, underlying)
    OptionsToken->>QuoteERC20: transfer(Holder, strike_from_collateral)
    OptionsToken->>OptionsToken: burn Holder's ERC-1155 tokens
    OptionsToken->>OptionsToken: reduce/close Writer's position
```

#### Collateral Withdrawal After Expiry

```mermaid
sequenceDiagram
    participant Writer
    participant OptionsToken
    participant CollateralERC20

    Note over OptionsToken: After expiry (option not exercised)
    Writer->>OptionsToken: withdraw_expired_collateral(tokenId, quantity)
    OptionsToken->>OptionsToken: Verify time > expiry, writer has position
    OptionsToken->>CollateralERC20: transfer(Writer, collateral)
    OptionsToken->>OptionsToken: reduce/close Writer's position

    Note over Writer: Collateral returned, option expired worthless
```

## Architecture

All contracts in Rust/WASM using Arbitrum Stylus SDK.

### Separate Contracts Design

#### OptionsToken Contract

- ERC-1155 token implementation (OpenZeppelin Stylus)
- Collateral custody for ALL options (all ERC20 tokens held here)
- Option minting/burning
- Exercise intent signaling
- Settlement execution at expiry
- Standalone functionality - users never need CLOB to use options

#### Central Limit Order Book (CLOB)

- Orderbook storage (`StorageMap`-based, see storage limitations below)
- Order matching engine (price-time priority)
- Trades existing ERC-1155 option tokens only
- Requires ERC-1155 approval from users
- Just one trading venue among many possible

#### Why Separate

- Options tokens fully composable (tradeable on AMMs, other DEXs, OTC)
- Users can write and exercise options without CLOB
- CLOB is optional trading venue, not core primitive
- Modular: upgrade CLOB without affecting options
- Clear security boundaries

### Stylus Contract Maintenance

**CRITICAL: Yearly Reactivation Requirement**

Stylus smart contracts must be reactivated every 365 days or after any
Stylus/ArbOS upgrade to remain callable. This applies to both OptionsToken and
CLOB contracts.

Reactivation process:

- Can be performed by anyone using `cargo-stylus` or the ArbWasm precompile
- Necessary because WASM is lowered to native machine code during activation
- Contracts become non-callable if not reactivated (collateral remains safe but
  locked)
- Recommend automated monitoring and reactivation infrastructure

### OptionsToken Contract

Responsibilities:

- Mint ERC-1155 tokens when options written
- Hold all collateral (underlying and quote ERC20s)
- Track writer positions and locked collateral
- Execute immediate American exercise (any time before expiry)
- Allow writers to withdraw collateral for expired unexercised options
- Burn tokens on exercise

Storage Structure (draft, TBC)

```rust
sol_storage! {
    #[entrypoint]
    pub struct OptionsToken {
        // ERC-1155 state (from OpenZeppelin)
        mapping(address => mapping(uint256 => uint256)) balances;
        mapping(address => mapping(address => bool)) operator_approvals;

        // Writer positions: (writer, tokenId) -> Position
        mapping(bytes32 => Position) positions;

        // Option metadata: tokenId -> OptionMetadata
        mapping(uint256 => OptionMetadata) option_metadata;

        // Available collateral: (user, token) -> amount
        mapping(bytes32 => uint256) collateral_balances;

        // Total supply per token ID
        mapping(uint256 => uint256) total_supply;
    }

    pub struct Position {
        address writer;
        uint256 quantity_written;
        uint256 collateral_locked;
        address collateral_token;
    }

    pub struct OptionMetadata {
        address underlying;
        address quote;
        uint256 strike;
        uint256 expiry;
        uint8 option_type; // 0 = Call, 1 = Put
    }
}
```

Token ID is `keccak256` hash of

- Address of the underlying ERC20 token
- Address of the quote ERC20 token
- Strike price (normalized 18 decimals)
- Expiration timestamp
- Option kind (call/put)

#### Token Decimals Normalization

ERC20 tokens have varying decimal places, e.g.

- Standard: 18 decimals (ETH, most tokens)
- Stablecoins: 6 decimals (USDC, USDT)
- Wrapped BTC: 8 decimals (WBTC)

**Normalization Strategy:**

All amounts are normalized to 18 decimals for internal calculations and token ID
generation.

For a token with $d$ decimals, normalize amount $a$ to 18 decimals:

$$\text{normalized amount} = a \times 10^{(18 - d)}$$

**Example: 1 WBTC (8 decimals) call at 60,000 USDC (6 decimals) strike**

Normalized underlying amount:
$$1 \times 10^{8} \times 10^{(18-8)} = 1 \times 10^{18}$$

Normalized strike price:
$$60000 \times 10^{6} \times 10^{(18-6)} = 60000 \times 10^{18}$$

**Collateral Requirements (in native decimals):**

- Call options: Lock $1 \times 10^{8}$ WBTC (1:1 underlying)
- Put options: Lock $60000 \times 10^{6}$ USDC (strike amount in quote token)

**Key Properties:**

- Token ID uniqueness: Same parameters always produce same token ID
- Decimal handling: Caller passes decimals as parameters (ERC20 `decimals()` is
  optional and unreliable)
- Precision: All math uses 18-decimal precision, convert to native decimals only
  for ERC20 transfers

#### Unsafe Token Handling

Certain ERC20 token types are incompatible with the options protocol and must be
handled carefully:

**Fee-on-Transfer Tokens:**

- Tokens that deduct fees during `transfer()` or `transferFrom()` (e.g., some
  deflationary tokens)
- **Problem:** Contract expects to receive amount $X$ but actually receives
  $X - \text{fee}$
- **Impact:** Collateral shortfall, can't settle all exercises
- **Protection:** Check balance before/after transfer, revert if mismatch
  detected (enforceable at contract level)

**Rebasing Tokens:**

- Tokens where balances change automatically (e.g., stETH, aTokens)
- **Problem:** Collateral amount changes over time, accounting breaks
- **Impact:** Either excess collateral benefits random party, or shortfall
  prevents settlement
- **Protection:** None at protocol level - permissionless system can't prevent
  use. Document risks clearly, advise against using rebasing tokens, but
  ultimately user's choice

**Tokens with Blacklists:**

- Tokens like USDC can blacklist addresses (e.g., OFAC sanctions)
- **Problem:** If writer gets blacklisted, can't return collateral or receive
  strike payment
- **Impact:** Funds locked permanently
- **Protection:** None - accept as known risk of using such tokens

**Arithmetic Overflow/Underflow:**

- Extreme decimal values or amounts could cause overflow in normalization math
- **Problem:** Normalizing very large amounts or tokens with many decimals:
  $a \times 10^{(18-d)}$ might exceed `uint256`
- **Protection:** Use checked arithmetic (Rust's `checked_mul`, `checked_pow`) -
  reverts automatically on overflow/underflow

**PoC Approach:** Fully permissionless - any ERC20 pair can be used, any decimal
count supported. Contract protects against fee-on-transfer (detectable) and
arithmetic overflow (checked math). For rebasing tokens and blacklist tokens,
users assume full risk. Buyer beware.

Storage Access Pattern:

- Individual position lookup: O(1) via StorageMap key
- Lazy loading: Only requested slots loaded via SLOAD
- SDK automatic caching: Multiple reads within transaction nearly free after
  first access

#### Storage Limitations & Design Tradeoffs

**StorageMap Constraints:**

Stylus StorageMap (equivalent to Solidity mapping) has critical limitations:

1. **No iteration:** Cannot enumerate keys or values

   - Cannot list all active options
   - Cannot find all positions for a user
   - Must track metadata separately

2. **No deletion:** Maps cannot be truly erased

   - Can only zero out values
   - Storage slots remain allocated
   - Impacts long-term storage costs

3. **No size queries:** Cannot get count of entries
   - Must maintain separate counter
   - Adds gas overhead for updates

**Implications for Options Contract:**

```rust
// CANNOT do this:
for position in positions.iter() { ... } // No iter() method

// MUST do this instead:
// 1. Track position IDs separately
mapping(address => uint256[]) user_position_ids;

// 2. Query specific positions
let position = positions.get(position_id);
```

**Design Decisions:**

- Store position lookups by deterministic key: `keccak256(writer, tokenId)`
- Use events for off-chain indexing (subgraph) to build queryable state
- Accept that on-chain enumeration is impossible without extra tracking

### CLOB Contract

Responsibilities:

- Maintain orderbooks per option series (ERC-1155 token ID)
- Match orders (price-time priority, FIFO)
- Transfer ERC-1155 tokens between traders (via approved transfers)
- Transfer quote ERC20 premium payments
- Cancel orders
- Query orderbook state

Storage Structure (draft, TBC)

```rust
sol_storage! {
    #[entrypoint]
    pub struct CLOB {
        // Orders at price: tokenId -> price -> order list
        mapping(uint256 => mapping(uint256 => Order[])) bid_orders;
        mapping(uint256 => mapping(uint256 => Order[])) ask_orders;

        // Best prices (must maintain manually)
        mapping(uint256 => uint256) best_bid;
        mapping(uint256 => uint256) best_ask;

        // Order lookup: orderId -> Order
        mapping(uint256 => Order) orders;

        // User's orders: user -> orderId[]
        mapping(address => uint256[]) user_orders;

        // Active price levels for scanning: tokenId -> price[]
        mapping(uint256 => uint256[]) active_bid_prices;
        mapping(uint256 => uint256[]) active_ask_prices;

        uint256 next_order_id;
    }

    pub struct Order {
        uint256 order_id;
        address maker;
        uint256 token_id;
        uint256 price;
        uint256 quantity;
        uint256 filled;
        uint8 side; // 0 = Buy, 1 = Sell
        uint256 timestamp;
    }
}
```

### CLOB Trading Safeguards

#### Front-Running Protection

**Price-time priority provides inherent protection:**

- Orders at same price level execute FIFO by timestamp
- Sequencer can't prioritize specific orders at same price level
- Prevents same-block order jumping

**Market order behavior:**

- PoC: Market orders revert if insufficient liquidity (all-or-nothing)
- No slippage protection parameters in PoC (full fill or revert)

**Limit order protection:**

- Makers set exact price, never worse execution
- Orders only fill at maker's price or better
- No slippage for makers

## Gas Optimization Targets (Arbitrum)

Estimated costs at 0.1 gwei gas price, $0.05 per transaction average:

- Write option: approx. 150k gas (approx. $0.0075)
- Place limit order: approx. 100k gas (approx. $0.005)
- Cancel order: approx. 50k gas (approx. $0.0025)
- Market order (5 fills): approx. 250k gas (approx. $0.0125)
- Exercise (immediate settlement): approx. 150k gas (approx. $0.0075)
- Withdraw expired collateral: approx. 80k gas (approx. $0.004)

Target: Keep all operations under 300k gas to stay economically viable even at
higher gas prices.

## Security Considerations

### Testing Strategy

**Unit Testing with Motsu:**

- Pure Rust testing framework for Stylus contracts
- Mock VM affordances without running actual blockchain
- Test individual functions in isolation
- Fast feedback loop during development

**Property-Based Testing with Proptest:**

- Generate random test cases to find edge cases
- Invariant checking across random inputs
- Critical for financial contracts
- Example: verify collateral always covers maximum possible payout across random
  inputs

**Integration Testing:**

- Test OptionsToken <-> CLOB interactions
- Test ERC-1155 transfers and approvals
- Test settlement flows end-to-end
- Run locally or in CI (no testnet dependency)

### Attack Vectors & Mitigations

**Reentrancy:**

- Stylus contracts follow checks-effects-interactions pattern
- Update state before external calls (ERC20 transfers)
- Consider reentrancy guards on critical functions

**Front-Running:**

- CLOB uses price-time priority (FIFO), inherently fair
- Arbitrum's sequencer provides some ordering guarantees
- Market orders vulnerable to sandwiching (add slippage limits before prod)

**Integer Overflow/Underflow:**

- Rust panics on overflow in debug mode
- Use `checked_add()`, `checked_mul()` in production
- Verify all math operations in critical paths

**Collateral Theft:**

- No admin withdrawal functions
- Collateral only released through:
  1. Settlement to holder (on exercise)
  2. Return to writer (on expiry without exercise)

**Time Manipulation:**

- Expiry uses `block.timestamp` (Arbitrum block time)
- Miners have approx. 15 second influence on timestamp
- Not exploitable for 1+ hour expiries
- Consider using `block.number` for stricter timing (at cost of UX)

### Known Limitations & Risks

**Collateral Lock Risk:**

- 100% collateralization means capital inefficient vs cash-settled options
- Writers' collateral locked until expiry (or until exercised)
- No early exit for writers (except buying back options on market)
- American exercise helps: holders exercise early when ITM, releasing writer
  collateral sooner

**Post-Expiry Collateral Withdrawal:**

- Writers must manually call `withdraw_expired_collateral()` after expiry
- No automatic return of collateral for unexercised options
- Collateral remains safe indefinitely, but requires writer action to reclaim
- Gas cost minimal (~$0.004 on Arbitrum), so writers incentivized to claim

## Future Work

### Automatic Exercise & Cash Settlement

Features:

- Automatic exercise of ITM options at expiry (no manual action required)
- Cash settlement option (receive profit in quote token instead of physical
  delivery)
- Better capital efficiency (quote token collateral for calls when cash-settled)

Requirements: Oracle integration for determining ITM status and settlement
prices

### Flash-Loan-Compatible Exercise (Oracle-Free Alternative)

Alternative to oracle-based cash settlement using arbitrage mechanics for price
discovery.

**Mechanism:** Arbitrageurs use flash loans to atomically: borrow strike payment
-> exercise option -> sell underlying on DEX -> repay loan -> keep profit.
Transaction only succeeds if truly ITM; market liquidity reveals true price
without oracles.

**Key Properties:**

- Eliminates oracle manipulation risk ($400M+ losses in 2022)
- Maximally decentralized (only trusts blockchain + proven AMMs)
- Best for liquid assets with deep DEX markets (ETH, WBTC, major tokens)
- Not suitable for long-tail assets without DEX liquidity

**Examples:** Primitive Finance (first implementation), Panoptic (perpetual
options on Uniswap v3, 5-10x capital efficiency)

**Trade-offs:** Zero oracle risk vs limited to liquid pairs; superior capital
efficiency vs MEV vulnerability; permissionless vs requires sophisticated
arbitrageur network

Requirements: Flash loan integration (Aave/Balancer), deep DEX liquidity, MEV
protections

### Advanced Order Types

Features:

- Partial fills for market orders (fill-or-kill, immediate-or-cancel with
  minimum fill quantity)
- Market orders with slippage protection (max price per unit limits)
- Stop-loss and take-profit orders
- Spread orders (multi-leg strategies, e.g. vertical spreads / iron condors)
- Iceberg orders (hidden quantity)

Requirements: Enhanced orderbook logic, potentially off-chain sequencer for
complex conditional orders

### Capital Efficiency Improvements

Features:

- Partial collateralization for spread positions (recognize offsetting risk)
- Portfolio margining (single collateral pool across positions)
- Cross-account margin to reduce total capital requirements

Requirements: Sophisticated risk calculation, liquidation system, insurance fund

### Trading Improvements

Features:

- Minimum order sizes to prevent spam/dust orders (requires per-quote-token
  configuration)
- Price tick sizes for orderbook efficiency
- Maximum orders per user to prevent spam
- Maximum price levels and orders per level to bound gas costs
- RFQ system for large block trades
- Better price discovery mechanisms
- Gasless order submission

Requirements: Per-token parameter configuration, governance for setting limits,
off-chain infrastructure

### UX Enhancements

Features:

- Exercise reminders and notifications
- Token safety/popularity indicators
- Historical analytics and charts
- Standard expiry date suggestions

Requirements: Subgraph indexing, frontend improvements

## References & Resources

### Stylus Documentation

- [Stylus Gentle Introduction](https://docs.arbitrum.io/stylus/stylus-gentle-introduction)
- [Rust SDK Reference](https://docs.rs/stylus-sdk/latest/stylus_sdk/)
- [Stylus by Example](https://stylus-by-example.org)
- [OpenZeppelin Stylus Contracts](https://github.com/OpenZeppelin/rust-contracts-stylus)

### Stylus Storage Research

- Arbitrum Stylus Storage Patterns (project artifacts): Critical analysis of
  StorageMap limitations
- SwissBorg CLOB Benchmark: Red-Black tree showing 25% overhead vs Solidity
- Renegade Architecture: Off-chain orderbook with on-chain ZK settlement
- Superposition: AMM-first approach, future CLOB plans

### DeFi Options Research

- Opyn v1: American options, physical settlement, separate ERC20 per series
- Opyn Gamma (v2): European options, **cash settlement**, ERC20 tokens,
  oracle-based
- Premia Finance: **ERC-1155 options**, covered call/put with locked collateral,
  composable across DEXs
- Lyra V1: ERC-1155 implementation, AMM-based pricing
- Premia V3: Per-market isolation
- Hegic: Peer-to-pool liquidity (contrasting approach)

Note: Premia Finance is the most relevant reference for our ERC-1155 + physical
settlement design. Opyn Gamma moved to cash settlement to enable spreads and
margin efficiencies.

### Orderbook Design

- Serum DEX: Slab allocator for orderbook storage on Solana
- dYdX v3: Off-chain orderbook with on-chain settlement (proven model)
- Vertex Protocol: Hybrid CLOB architecture

### Data Structures

- [Rust Collections](https://doc.rust-lang.org/std/collections/)
- [BTreeMap (in-memory)](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html)
- [VecDeque (in-memory)](https://doc.rust-lang.org/std/collections/struct.VecDeque.html)
- Stylus Storage Types: StorageMap, StorageVec in SDK documentation

### Testing & Security

- Motsu: Pure Rust testing framework for Stylus
- Proptest: Property-based testing in Rust
- OpenZeppelin Stylus Audit Report: Security patterns for Rust smart contracts

### Standards

- OCC Options Symbology: Standard ticker format
- ERC-1155: Multi-token standard specification
