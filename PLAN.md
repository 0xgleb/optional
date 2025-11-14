# Implementation Plan: Issue #11 - ERC-1155 Full Compliance

## Overview

Implement complete ERC-1155 standard interface with custom transfer restrictions
for option tokens. Custom implementation required (not OpenZeppelin) because
transfer restrictions for signaled exercise quantities require hooks into
transfer logic.

**Dependencies:** Issue #10 (must be completed first)

**Integration:** Must work with Issue #8 (Transfer Restrictions for Signaled
Tokens)

## Architecture

### Storage Structure

Current storage foundation:

- `balances: mapping(bytes32 => uint256)` - balance tracking via
  `balance_key(owner, token_id)`
- `total_supply: mapping(bytes32 => uint256)` - total supply per token ID

Add:

- `operator_approvals: mapping(bytes32 => bool)` - operator approvals via
  `approval_key(owner, operator)`

### Implementation Patterns

1. **Checks-Effects-Interactions**: State updates before external calls
   (reentrancy protection)
2. **ERC-1155 Receiver Callbacks**: Safety mechanism prevents token lockup in
   incompatible contracts
3. **Transfer Restriction Hooks**: Integration point for Issue #8 signaled
   quantity blocking
4. **Batch Operations**: Gas-efficient multi-token transfers

### Transfer Restriction Integration (Issue #8)

If Issue #8 not yet implemented, use stub:

```rust
#[allow(unused_variables)]
fn _check_transfer_allowed(
    &self,
    from: Address,
    token_id: B256,
    value: U256,
) -> Result<(), OptionsError> {
    // TODO: Remove when Issue #8 implemented
    Ok(())
}
```

When Issue #8 implemented, this becomes:

```rust
fn _check_transfer_allowed(
    &self,
    from: Address,
    token_id: B256,
    value: U256,
) -> Result<(), OptionsError> {
    let signaled = self.get_signaled_quantity(from, token_id);
    let balance = self.balance_of(from, token_id);
    let transferable = balance.checked_sub(signaled)
        .ok_or(OptionsError::Underflow)?;

    if value > transferable {
        return Err(OptionsError::InsufficientTransferableBalance {
            available: transferable,
            requested: value,
        });
    }
    Ok(())
}
```

### ERC-1155 Receiver Callbacks

Standard behavior:

1. Check if `to.code.len() > 0` (is contract)
2. If contract: call receiver callback, verify magic return value
3. If EOA: skip callback (implicit acceptance)
4. If callback fails: revert transfer

Magic values (function selectors):

- `onERC1155Received`: `0xf23a6e61`
- `onERC1155BatchReceived`: `0xbc197c81`

## Task Breakdown

### Task 1. Operator Approval System

Implement approval mechanism allowing external contracts (CLOB) to transfer
option tokens on behalf of users.

**Implementation**:

- [x] Add `operator_approvals: mapping(bytes32 => bool)` to `Options` storage
      struct
- [x] Define `approval_key(owner: Address, operator: Address) -> B256` helper:
  - [x] `keccak256(owner || operator)` for composite key
- [x] Define `ApprovalForAll` event:
  - [x] `event ApprovalForAll(address indexed owner, address indexed operator, bool approved)`
- [x] Implement `set_approval_for_all(operator: Address, approved: bool)` public
      function:
  - [x] Validate `operator != msg_sender` (cannot approve self)
  - [x] Add `SelfApproval` error variant
  - [x] Store approval:
        `self.operator_approvals.insert(approval_key(msg_sender, operator), approved)`
  - [x] Emit `ApprovalForAll` event
- [x] Implement `is_approved_for_all(owner: Address, operator: Address) -> bool`
      public view:
  - [x] Query: `self.operator_approvals.get(approval_key(owner, operator))`
- [x] Implement `_is_authorized(owner: Address, operator: Address) -> bool`
      internal helper:
  - [x] Return `operator == owner || is_approved_for_all(owner, operator)`

**Tests**:

- [x] Test approval/revocation stores correctly
- [x] Test cannot approve self
- [x] Test `_is_authorized` returns true for owner and approved operators
- [x] Test `ApprovalForAll` event emission
- [x] Run `cargo test`, `cargo fmt`, `cargo clippy`

**Completion Criteria**: Approval system works, all tests pass, clippy clean.

---

### Task 2. Balance Query Interface

Provide standard ERC-1155 balance query functions.

**Implementation**:

- [x] Add `AccountsIdsMismatch` error variant to `OptionsError`
- [x] Implement `balance_of_public(account: Address, id: B256) -> U256` public
      view:
  - [x] Call existing internal `balance_of(account, id)`
  - [x] Rename this function to match ERC-1155 standard name `balanceOf` (Stylus
        allows camelCase in `#[public]`)
- [x] Implement
      `balance_of_batch(accounts: Vec<Address>, ids: Vec<B256>) -> Result<Vec<U256>, OptionsError>`:
  - [x] Validate `accounts.len() == ids.len()`, return `AccountsIdsMismatch` if
        not
  - [x] Iterate: collect `balance_of(accounts[i], ids[i])` into result vector
  - [x] Return result vector
- [x] Remove `#[allow(dead_code)]` from internal `balance_of` function

**Tests**:

- [x] Test `balanceOf` returns correct balance
- [x] Test `balance_of_batch` with multiple accounts/ids
- [x] Test `balance_of_batch` fails when lengths mismatch
- [x] Test empty batch query returns empty vector
- [x] Run `cargo test`, `cargo fmt`, `cargo clippy`

**Completion Criteria**: Balance queries work, all tests pass, clippy clean.

---

### Task 3. ERC-1155 Transfer Events

Add standard events for mint/burn operations.

**Implementation**:

- [x] Define `TransferSingle` event:
  - [x] `event TransferSingle(address indexed operator, address indexed from, address indexed to, uint256 id, uint256 value)`
- [x] Update `_mint` internal function to emit `TransferSingle`:
  - [x] `log(self.vm(), TransferSingle { operator: to, from: Address::ZERO, to, id, value: quantity })`
  - [x] Operator = recipient (standard for self-minting)
- [x] Update `_burn` internal function to emit `TransferSingle`:
  - [x] `log(self.vm(), TransferSingle { operator: from, from, to: Address::ZERO, id, value: quantity })`
- [x] Verify `write_call_option` now emits event via `_mint`

**Tests**:

- [x] Test minting emits `TransferSingle` with correct parameters
- [x] Test burning emits `TransferSingle` with correct parameters
- [x] Test `write_call_option` emits event (verified via \_mint)
- [x] Run `cargo test`, `cargo fmt`, `cargo clippy`

**Completion Criteria**: Mint/burn emit events, all tests pass, clippy clean.

---

### Task 4. Single Token Transfer

Implement core `safeTransferFrom` function with transfer restrictions.

**Implementation**:

- [ ] Add error variants: `UnauthorizedTransfer`, `TransferToZeroAddress`,
      `TransferExceedsBalance`
- [ ] Add transfer restriction stub (if Issue #8 not implemented):
  ```rust
  #[allow(unused_variables)]
  fn _check_transfer_allowed(&self, from: Address, id: B256, value: U256) -> Result<(), OptionsError> {
      Ok(()) // TODO: Replace when Issue #8 implemented
  }
  ```
- [ ] Implement
      `_transfer(from: Address, to: Address, id: B256, value: U256) -> Result<(), OptionsError>`:
  - [ ] Get `from` balance
  - [ ] Check `balance >= value`, return `TransferExceedsBalance` if not
  - [ ] Decrease `from` balance with checked_sub
  - [ ] Increase `to` balance with checked_add
  - [ ] Store updated balances
- [ ] Implement
      `safe_transfer_from(from: Address, to: Address, id: B256, value: U256, data: Vec<u8>)`:
  - [ ] Validate `to != Address::ZERO`
  - [ ] Get `msg_sender` as operator
  - [ ] Check authorization: `_is_authorized(from, operator)?`
  - [ ] **Checks**: Call `_check_transfer_allowed(from, id, value)?`
  - [ ] **Effects**: Call `_transfer(from, to, id, value)?`
  - [ ] **Interactions**: Skip receiver callback for now (Task 5)
  - [ ] Emit `TransferSingle` event

**Tests**:

- [ ] Test owner can transfer own tokens
- [ ] Test approved operator can transfer
- [ ] Test non-approved operator fails
- [ ] Test transfer to zero address fails
- [ ] Test transfer exceeding balance fails
- [ ] Test balances updated correctly
- [ ] Test `TransferSingle` event emitted
- [ ] Run `cargo test`, `cargo fmt`, `cargo clippy`

**Completion Criteria**: Single transfers work, all tests pass, clippy clean.

---

### Task 5. ERC-1155 Receiver Callbacks

Prevent tokens from being locked in incompatible contracts.

**Implementation**:

- [ ] Add `sol_interface!` for `IERC1155Receiver`:
  ```rust
  sol_interface! {
      interface IERC1155Receiver {
          function onERC1155Received(address operator, address from, uint256 id, uint256 value, bytes calldata data) external returns (bytes4);
      }
  }
  ```
- [ ] Add `UnsafeRecipient` error variant
- [ ] Define magic value constant:
  ```rust
  const IERC1155_RECEIVER_SINGLE_MAGIC: [u8; 4] = [0xf2, 0x3a, 0x6e, 0x61];
  ```
- [ ] Implement
      `_check_on_erc1155_received(operator: Address, from: Address, to: Address, id: B256, value: U256, data: Vec<u8>)`:
  - [ ] Check if `to` is contract (check code size via Stylus SDK)
  - [ ] If EOA: return Ok
  - [ ] If contract: call
        `IERC1155Receiver::new(to).onERC1155Received(Call::new_in(self), operator, from, id, value, data)`
  - [ ] Verify return value == `IERC1155_RECEIVER_SINGLE_MAGIC`
  - [ ] Return `UnsafeRecipient` if wrong magic or call fails
- [ ] Update `safe_transfer_from` to call `_check_on_erc1155_received` after
      `_transfer`

**Tests**:

- [ ] Test transfer to EOA succeeds (no callback)
- [ ] Test transfer to contract with correct callback succeeds
- [ ] Test transfer to contract with wrong magic fails
- [ ] Test transfer to contract without callback fails
- [ ] Create mock receiver contract for testing
- [ ] Run `cargo test`, `cargo fmt`, `cargo clippy`

**Completion Criteria**: Receiver callbacks work, all tests pass, clippy clean.

---

### Task 6. Batch Transfer Operations

Gas-efficient multi-token transfers.

**Implementation**:

- [ ] Add `IdsValuesMismatch` error variant
- [ ] Define `TransferBatch` event:
  - [ ] `event TransferBatch(address indexed operator, address indexed from, address indexed to, uint256[] ids, uint256[] values)`
- [ ] Add `sol_interface!` for batch receiver:
  ```rust
  sol_interface! {
      interface IERC1155Receiver {
          function onERC1155BatchReceived(address operator, address from, uint256[] calldata ids, uint256[] calldata values, bytes calldata data) external returns (bytes4);
      }
  }
  ```
- [ ] Define batch magic constant:
  ```rust
  const IERC1155_RECEIVER_BATCH_MAGIC: [u8; 4] = [0xbc, 0x19, 0x7c, 0x81];
  ```
- [ ] Implement `_check_on_erc1155_batch_received`:
  - [ ] Same pattern as single receiver check
  - [ ] Call `onERC1155BatchReceived` with id/value arrays
- [ ] Implement
      `safe_batch_transfer_from(from: Address, to: Address, ids: Vec<B256>, values: Vec<U256>, data: Vec<u8>)`:
  - [ ] Validate `ids.len() == values.len()`
  - [ ] Validate `to != Address::ZERO`
  - [ ] Get operator = msg_sender
  - [ ] Check authorization once
  - [ ] Iterate: for each (id, value):
    - [ ] Call `_check_transfer_allowed(from, id, value)?`
    - [ ] Call `_transfer(from, to, id, value)?`
  - [ ] Call `_check_on_erc1155_batch_received`
  - [ ] Emit single `TransferBatch` event

**Tests**:

- [ ] Test batch transfer multiple tokens
- [ ] Test fails if ids/values length mismatch
- [ ] Test authorized operator can batch transfer
- [ ] Test batch to contract calls batch receiver
- [ ] Test balances updated correctly for all tokens
- [ ] Test `TransferBatch` event emitted
- [ ] Run `cargo test`, `cargo fmt`, `cargo clippy`

**Completion Criteria**: Batch transfers work, all tests pass, clippy clean.

---

### Task 7. Property-Based Testing

Verify invariants hold under random inputs.

**Implementation**:

- [ ] Add proptest to dev dependencies if not present
- [ ] Implement `prop_approval_idempotent`: Setting approval twice = setting
      once
- [ ] Implement `prop_transfer_preserves_total_supply`: Transfers don't change
      total supply
- [ ] Implement `prop_batch_equivalent_to_single`: Batch = multiple single
      transfers
- [ ] Implement `prop_authorization_transitive`: Owner authorization implies can
      transfer
- [ ] Run property tests with `cargo test`

**Tests**:

- [ ] All property tests pass with 100+ random cases each
- [ ] No panics or unexpected errors
- [ ] Run `cargo test`, `cargo fmt`, `cargo clippy`

**Completion Criteria**: Property tests pass, invariants verified, clippy clean.

---

### Task 8. Documentation and Final Validation

Document implementation and verify completeness.

**Implementation**:

- [ ] Add doc comments to all public functions:
  - [ ] `set_approval_for_all`: "Grants or revokes permission for operator to
        transfer all tokens on behalf of caller"
  - [ ] `is_approved_for_all`: "Returns true if operator is approved to transfer
        owner's tokens"
  - [ ] `balance_of`: "Returns the amount of tokens owned by account for token
        id"
  - [ ] `balance_of_batch`: "Batched version of balanceOf"
  - [ ] `safe_transfer_from`: "Transfers amount tokens of id from from to to"
  - [ ] `safe_batch_transfer_from`: "Batched version of safeTransferFrom"
- [ ] Document Issue #8 integration point in `_check_transfer_allowed`
- [ ] Run full test suite: `cargo test`
- [ ] Run linters:
      `cargo fmt && cargo clippy --all-targets --all-features -- -D clippy::all -D warnings`
- [ ] Build WASM: `cargo build --target wasm32-unknown-unknown`
- [ ] Run `cargo stylus check` with 60s timeout (expected to pass)

**Completion Criteria**: All documentation complete, all checks pass, ready for
review.

---

## Success Criteria

1. ✅ Full ERC-1155 standard compliance (all functions present and working)
2. ✅ Operator approval system functional (CLOB can trade options)
3. ✅ Transfer restrictions integration point ready for Issue #8
4. ✅ Receiver callbacks prevent token lockup
5. ✅ All tests pass (unit + property tests)
6. ✅ Clippy clean (all warnings denied)
7. ✅ Stylus compatible (builds to WASM, passes check)
