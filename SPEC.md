# SPEC.md

This specification outlines a fully on-chain Central Limit Order Book (CLOB) for options trading, built on Arbitrum using Stylus (Rust/WASM) for compute-intensive operations. The design prioritizes simplicity and reliability through physical settlement with 100% collateralization, eliminating the need for oracles, risk management systems, and liquidation mechanisms in the initial version.

## Overview

### PoC Scope

- Users can write (sell) and buy options as ERC-1155 tokens
- Options trade on a fully on-chain CLOB with price-time priority matching
- Settlement is physical (actual token delivery) with manual exercise
- All collateral is 100% locked in the underlying assets (no fractional reserve)
- ERC20 token pairs
- European options

### Future work

- Cash settlement (requires oracles and risk management)
- Automatic exercise at maturity (requires oracles)
- Advanced order types
- Native token support
- American options

### Key Architectural Decisions

- **Trustless by Design**: Physical settlement means no reliance on external price feeds
- **Simplicity First**: 100% collateralization eliminates complex risk management
- **Future Compatible**: Architecture supports adding cash settlement and oracles later
- **Gas Efficient**: All contracts in Rust/Stylus for maximum performance
- **Permissionless**: Any ERC20 token pair can have options created

## Background

### Definitions

**Call Option**: Right (not obligation) to BUY the underlying ERC20 token at strike price

- Holder: Pays premium, can exercise to buy underlying at strike price
- Writer: Receives premium, must deliver underlying token if holder exercises
- Collateral: Writer locks 1:1 underlying ERC20, e.g. 1 WBTC for 1 WBTC call (covered call)

**Put Option**: Right (not obligation) to SELL the underlying ERC20 token at strike price

- Holder: Pays premium, can exercise to sell underlying at strike price
- Writer: Receives premium, must accept underlying and pay strike if holder exercises
- Collateral: Writer locks strike amount in quote token, e.g. $123,000 USDC for 1 WBTC put at $123k strike (cash secured put)

---

**European Option**: An option that can only be exercised at maturity.

**American Option**: An option that can be exercised at or before maturity.

---

**Physical Settlement**: Actual token delivery on exercise

- Call exercise: Holder pays strike in quote token → receives underlying token
- Put exercise: Holder delivers underlying token → receives strike in quote token
- No oracle required (holder decides if exercise is profitable)

### User Flows

#### Flow 1: Writing (Selling) an Option

Actors: Option Writer
Steps:

1. Writer selects option parameters (underlying ERC20, quote ERC20, strike, expiry, type, quantity)
2. Contract calculates required collateral based on option type
3. Writer approves ERC20 token transfer to contract
4. Contract transfers collateral from writer
5. Contract mints ERC-1155 option tokens to writer
6. Writer can now sell these tokens via CLOB or elsewhere or hold them

Collateral:

- Calls: Underlying ERC20 tokens (1:1 ratio)
- Puts: Quote ERC20 tokens (strike × quantity)

Outcome: Option tokens (ERC-1155) minted, collateral (ERC20) locked

#### Flow 2: Trading Options

Actors: Maker, Taker

##### Adding Liquidity (Maker)

Steps:

1. Maker places a limit order
2. Maker's tokens locked:
  - Selling: ERC-1155 option tokens locked
  - Buying: Quote ERC20 locked (price × quantity)
3. Order added to orderbook at specified price level
4. Order waits for taker

Outcome: Limit order in orderbook

##### Taking Liquidity (Taker)

Steps:

1. Taker calls marketOrder(tokenId, quantity, side) (no price specified)
2. Matching engine fills against best available prices:
  - Buying: Matches ascending from best ask
  - Selling: Matches descending from best bid
3. If insufficient liquidity for full quantity -> REVERT
4. If sufficient liquidity:
  - ERC-1155 option tokens transfer: Seller → Buyer
  - Quote ERC20 premium transfer: Buyer → Seller (at makers' prices)
  - Maker orders filled/reduced (FIFO at each price)

Outcome: Taker receives full fill at makers' prices, or transaction reverts
