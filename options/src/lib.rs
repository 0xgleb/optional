#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::{vec, vec::Vec};
use alloy_primitives::{keccak256, Address, B256, U256};
use alloy_sol_types::sol;

use stylus_sdk::prelude::*;

sol! {
    /// Represents a token with its address and decimal precision.
    #[derive(Copy)]
    struct Token {
        address address;
        uint8 decimals;
    }
}

// Implement AbiType for Token to make it usable in #[public] functions
impl stylus_sdk::abi::AbiType for Token {
    type SolType = Self;
    const ABI: stylus_sdk::abi::ConstString = stylus_sdk::abi::ConstString::new("(address,uint8)");
}

/// Represents the type of option contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptionType {
    /// Call option: Right to BUY underlying at strike price.
    #[default]
    Call,
    /// Put option: Right to SELL underlying at strike price.
    Put,
}

impl OptionType {
    /// Converts option type to u8 for encoding.
    ///
    /// # Returns
    /// - `0` for Call
    /// - `1` for Put
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::Call => 0,
            Self::Put => 1,
        }
    }
}

sol! {
    /// Errors that can occur in the Options contract.
    #[derive(Debug)]
    error Unimplemented();
    #[derive(Debug)]
    error InvalidDecimals(uint8 decimals);
    #[derive(Debug)]
    error NormalizationOverflow();
    #[derive(Debug)]
    error InsufficientBalance(uint256 available, uint256 requested);
    #[derive(Debug)]
    error Overflow();
}

#[derive(SolidityError, Debug)]
pub enum OptionsError {
    /// Stub implementation placeholder - function not yet implemented.
    Unimplemented(Unimplemented),
    /// Token decimals exceed maximum of 18.
    InvalidDecimals(InvalidDecimals),
    /// Arithmetic overflow during normalization.
    NormalizationOverflow(NormalizationOverflow),
    /// Insufficient token balance for operation.
    InsufficientBalance(InsufficientBalance),
    /// Arithmetic overflow.
    Overflow(Overflow),
}

sol_storage! {
    #[entrypoint]
    pub struct Options {
        /// Mapping from balance_key(owner, token_id) to balance
        mapping(bytes32 => uint256) balances;
        /// Mapping from token_id to total supply
        mapping(bytes32 => uint256) total_supply;
    }
}

/// Generates a deterministic token ID for an option series.
///
/// Token ID is computed as `keccak256(underlying, quote, strike, expiry, option_type)`.
/// All writers of the same option parameters share the same token ID, enabling
/// fungibility and secondary market trading.
///
/// # Parameters
/// - `underlying`: Address of the underlying token
/// - `quote`: Address of the quote token
/// - `strike`: Strike price (18 decimals normalized)
/// - `expiry`: Expiration timestamp
/// - `option_type`: Call or Put
///
/// # Returns
/// Deterministic `B256` hash as token ID
pub(crate) fn generate_token_id(
    underlying: Address,
    quote: Address,
    strike: U256,
    expiry: U256,
    option_type: OptionType,
) -> B256 {
    let encoded = [
        underlying.as_slice(),
        quote.as_slice(),
        strike.to_be_bytes::<32>().as_slice(),
        expiry.to_be_bytes::<32>().as_slice(),
        &[option_type.to_u8()],
    ]
    .concat();

    keccak256(encoded)
}

/// Normalizes an amount from native token decimals to 18 decimals.
///
/// All internal calculations use 18-decimal precision. This function converts
/// amounts from their native decimal representation to the internal 18-decimal format.
///
/// # Parameters
/// - `amount`: Amount in native decimals
/// - `from_decimals`: Number of decimals in the native token (must be <= 18)
///
/// # Returns
/// Amount normalized to 18 decimals
///
/// # Errors
/// - `InvalidDecimals`: If `from_decimals > 18`
/// - `NormalizationOverflow`: If multiplication would overflow U256
pub(crate) fn normalize_amount(amount: U256, from_decimals: u8) -> Result<U256, OptionsError> {
    if from_decimals > 18 {
        return Err(OptionsError::InvalidDecimals(InvalidDecimals {
            decimals: from_decimals,
        }));
    }

    let scale_exp = 18 - from_decimals;
    let scale_factor = U256::from(10).checked_pow(U256::from(scale_exp)).ok_or(
        OptionsError::NormalizationOverflow(NormalizationOverflow {}),
    )?;

    amount
        .checked_mul(scale_factor)
        .ok_or(OptionsError::NormalizationOverflow(
            NormalizationOverflow {},
        ))
}

/// Denormalizes an amount from 18 decimals to native token decimals.
///
/// Converts amounts from the internal 18-decimal representation back to
/// native token decimals for ERC20 transfers.
///
/// # Parameters
/// - `amount`: Amount in 18 decimals
/// - `to_decimals`: Number of decimals in the target token (must be <= 18)
///
/// # Returns
/// Amount in native token decimals
///
/// # Errors
/// - `InvalidDecimals`: If `to_decimals > 18`
/// - `NormalizationOverflow`: If scale factor calculation would overflow
pub(crate) fn denormalize_amount(amount: U256, to_decimals: u8) -> Result<U256, OptionsError> {
    if to_decimals > 18 {
        return Err(OptionsError::InvalidDecimals(InvalidDecimals {
            decimals: to_decimals,
        }));
    }

    let scale_exp = 18 - to_decimals;
    let scale_factor = U256::from(10).checked_pow(U256::from(scale_exp)).ok_or(
        OptionsError::NormalizationOverflow(NormalizationOverflow {}),
    )?;

    Ok(amount / scale_factor)
}

#[public]
impl Options {
    /// Writes a call option by locking underlying tokens as collateral (1:1).
    ///
    /// Mints ERC-1155 tokens representing the call option and returns a deterministic token ID
    /// based on the option parameters (keccak256 hash).
    ///
    /// # Parameters
    /// - `strike`: Strike price (18 decimals normalized)
    /// - `expiry`: Expiration timestamp
    /// - `quantity`: Quantity of options to write
    /// - `underlying`: Underlying token (address and decimals)
    /// - `quote`: Quote token (address and decimals)
    ///
    /// # Errors
    /// Returns `OptionsError::Unimplemented` (stub implementation).
    pub fn write_call_option(
        &mut self,
        strike: U256,
        expiry: U256,
        quantity: U256,
        underlying: Token,
        quote: Token,
    ) -> Result<B256, OptionsError> {
        let _ = (strike, expiry, quantity, underlying, quote);
        Err(OptionsError::Unimplemented(Unimplemented {}))
    }

    /// Writes a put option by locking quote tokens as collateral (strike * quantity).
    ///
    /// Mints ERC-1155 tokens representing the put option and returns a deterministic token ID
    /// based on the option parameters (keccak256 hash).
    ///
    /// # Parameters
    /// - `strike`: Strike price (18 decimals normalized)
    /// - `expiry`: Expiration timestamp
    /// - `quantity`: Quantity of options to write
    /// - `underlying`: Underlying token (address and decimals)
    /// - `quote`: Quote token (address and decimals)
    ///
    /// # Errors
    /// Returns `OptionsError::Unimplemented` (stub implementation).
    pub fn write_put_option(
        &mut self,
        strike: U256,
        expiry: U256,
        quantity: U256,
        underlying: Token,
        quote: Token,
    ) -> Result<B256, OptionsError> {
        let _ = (strike, expiry, quantity, underlying, quote);
        Err(OptionsError::Unimplemented(Unimplemented {}))
    }

    /// Exercises a call option
    ///
    /// Immediate atomic settlement: holder pays strike (quote tokens) to writer,
    /// receives underlying tokens from collateral, burns option tokens.
    /// Can only be called before option expiry.
    ///
    /// # Parameters
    /// - `token_id`: The ERC-1155 token ID of the call option (keccak256 hash)
    /// - `quantity`: Quantity of options to exercise
    ///
    /// # Errors
    /// Returns `OptionsError::Unimplemented` (stub implementation).
    pub fn exercise_call(&mut self, token_id: B256, quantity: U256) -> Result<(), OptionsError> {
        let _ = (token_id, quantity);
        Err(OptionsError::Unimplemented(Unimplemented {}))
    }

    /// Exercises a put option
    ///
    /// Immediate atomic settlement: holder delivers underlying tokens to writer,
    /// receives strike (quote tokens) from collateral, burns option tokens.
    /// Can only be called before option expiry.
    ///
    /// # Parameters
    /// - `token_id`: The ERC-1155 token ID of the put option (keccak256 hash)
    /// - `quantity`: Quantity of options to exercise
    ///
    /// # Errors
    /// Returns `OptionsError::Unimplemented` (stub implementation).
    pub fn exercise_put(&mut self, token_id: B256, quantity: U256) -> Result<(), OptionsError> {
        let _ = (token_id, quantity);
        Err(OptionsError::Unimplemented(Unimplemented {}))
    }

    /// Withdraws collateral for expired unexercised options.
    ///
    /// Writers can reclaim their locked collateral after option expiry.
    /// Returns underlying tokens for calls, quote tokens for puts.
    /// Reduces or closes the writer's position. Only callable after expiry.
    ///
    /// # Parameters
    /// - `token_id`: The ERC-1155 token ID of the option (keccak256 hash)
    /// - `quantity`: Quantity of collateral to withdraw
    ///
    /// # Errors
    /// Returns `OptionsError::Unimplemented` (stub implementation).
    pub fn withdraw_expired_collateral(
        &mut self,
        token_id: B256,
        quantity: U256,
    ) -> Result<(), OptionsError> {
        let _ = (token_id, quantity);
        Err(OptionsError::Unimplemented(Unimplemented {}))
    }
}

/// Test-only helper methods (accessible through motsu deref)
impl Options {
    /// Test wrapper for _mint - accessible in motsu tests through deref
    #[cfg(test)]
    pub fn test_mint(
        &mut self,
        to: Address,
        token_id: B256,
        quantity: U256,
    ) -> Result<(), OptionsError> {
        self._mint(to, token_id, quantity)
    }

    /// Test wrapper for _burn - accessible in motsu tests through deref
    #[cfg(test)]
    pub fn test_burn(
        &mut self,
        from: Address,
        token_id: B256,
        quantity: U256,
    ) -> Result<(), OptionsError> {
        self._burn(from, token_id, quantity)
    }

    /// Test wrapper for balance_of - accessible in motsu tests through deref
    #[cfg(test)]
    pub fn test_balance_of(&self, owner: Address, token_id: B256) -> U256 {
        self.balance_of(owner, token_id)
    }

    /// Test wrapper for total_supply_of - accessible in motsu tests through deref
    #[cfg(test)]
    pub fn test_total_supply_of(&self, token_id: B256) -> U256 {
        self.total_supply_of(token_id)
    }
}

/// Internal helper functions for Options contract
impl Options {
    /// Generates a composite key for balance lookups.
    ///
    /// Combines owner address and token ID into a single key for storage mapping.
    ///
    /// # Parameters
    /// - `owner`: Token holder address
    /// - `token_id`: ERC-1155 token ID
    ///
    /// # Returns
    /// `keccak256(owner || token_id)` as composite key
    fn balance_key(owner: Address, token_id: B256) -> B256 {
        let encoded = [owner.as_slice(), token_id.as_slice()].concat();
        keccak256(encoded)
    }

    /// Mints option tokens to an address.
    ///
    /// Increases both the recipient's balance and the token's total supply.
    /// Uses checked arithmetic to prevent overflow.
    ///
    /// # Parameters
    /// - `to`: Recipient address
    /// - `token_id`: ERC-1155 token ID
    /// - `quantity`: Amount to mint
    ///
    /// # Errors
    /// - `OptionsError::Overflow` if balance or total supply would overflow
    pub(crate) fn _mint(
        &mut self,
        to: Address,
        token_id: B256,
        quantity: U256,
    ) -> Result<(), OptionsError> {
        let key = Self::balance_key(to, token_id);
        let current_balance = self.balances.get(key);
        let new_balance = current_balance
            .checked_add(quantity)
            .ok_or(OptionsError::Overflow(Overflow {}))?;
        self.balances.insert(key, new_balance);

        let current_supply = self.total_supply.get(token_id);
        let new_supply = current_supply
            .checked_add(quantity)
            .ok_or(OptionsError::Overflow(Overflow {}))?;
        self.total_supply.insert(token_id, new_supply);

        Ok(())
    }

    /// Burns option tokens from an address.
    ///
    /// Decreases both the holder's balance and the token's total supply.
    /// Uses checked arithmetic to prevent underflow.
    ///
    /// # Parameters
    /// - `from`: Token holder address
    /// - `token_id`: ERC-1155 token ID
    /// - `quantity`: Amount to burn
    ///
    /// # Errors
    /// - `OptionsError::InsufficientBalance` if balance < quantity
    pub(crate) fn _burn(
        &mut self,
        from: Address,
        token_id: B256,
        quantity: U256,
    ) -> Result<(), OptionsError> {
        let key = Self::balance_key(from, token_id);
        let current_balance = self.balances.get(key);

        if current_balance < quantity {
            return Err(OptionsError::InsufficientBalance(InsufficientBalance {
                available: current_balance,
                requested: quantity,
            }));
        }

        let new_balance = current_balance
            .checked_sub(quantity)
            .ok_or(OptionsError::Overflow(Overflow {}))?;
        self.balances.insert(key, new_balance);

        let current_supply = self.total_supply.get(token_id);
        let new_supply = current_supply
            .checked_sub(quantity)
            .ok_or(OptionsError::Overflow(Overflow {}))?;
        self.total_supply.insert(token_id, new_supply);

        Ok(())
    }

    /// Returns the balance of an account for a specific token.
    ///
    /// # Parameters
    /// - `owner`: Token holder address
    /// - `token_id`: ERC-1155 token ID
    ///
    /// # Returns
    /// Token balance (0 if no balance exists)
    pub(crate) fn balance_of(&self, owner: Address, token_id: B256) -> U256 {
        let key = Self::balance_key(owner, token_id);
        self.balances.get(key)
    }

    /// Returns the total supply of a token.
    ///
    /// # Parameters
    /// - `token_id`: ERC-1155 token ID
    ///
    /// # Returns
    /// Total supply (0 if no tokens minted)
    pub(crate) fn total_supply_of(&self, token_id: B256) -> U256 {
        self.total_supply.get(token_id)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Address;
    use motsu::prelude::*;

    use super::*;

    use alloc::collections::BTreeMap;

    #[derive(Default)]
    pub struct MockERC20 {
        balances: BTreeMap<Address, U256>,
        allowances: BTreeMap<Address, BTreeMap<Address, U256>>,
        decimals_value: u8,
    }

    #[derive(Default)]
    pub struct FeeOnTransferERC20 {
        balances: BTreeMap<Address, U256>,
    }

    impl MockERC20 {
        pub fn mint(&mut self, to: Address, amount: U256) {
            let current_balance = self.balances.get(&to).copied().unwrap_or(U256::ZERO);
            self.balances.insert(to, current_balance + amount);
        }

        pub fn transfer(&mut self, from: Address, to: Address, amount: U256) -> bool {
            let sender_balance = self.balances.get(&from).copied().unwrap_or(U256::ZERO);

            if sender_balance < amount {
                return false;
            }

            self.balances.insert(from, sender_balance - amount);
            let recipient_balance = self.balances.get(&to).copied().unwrap_or(U256::ZERO);
            self.balances.insert(to, recipient_balance + amount);
            true
        }

        pub fn transfer_from(
            &mut self,
            spender: Address,
            from: Address,
            to: Address,
            amount: U256,
        ) -> bool {
            let allowance = self
                .allowances
                .get(&from)
                .and_then(|m| m.get(&spender))
                .copied()
                .unwrap_or(U256::ZERO);
            let from_balance = self.balances.get(&from).copied().unwrap_or(U256::ZERO);

            if allowance < amount || from_balance < amount {
                return false;
            }

            self.allowances
                .entry(from)
                .or_default()
                .insert(spender, allowance - amount);
            self.balances.insert(from, from_balance - amount);
            let to_balance = self.balances.get(&to).copied().unwrap_or(U256::ZERO);
            self.balances.insert(to, to_balance + amount);
            true
        }

        pub fn approve(&mut self, owner: Address, spender: Address, amount: U256) -> bool {
            self.allowances
                .entry(owner)
                .or_default()
                .insert(spender, amount);
            true
        }

        pub fn balance_of(&self, account: Address) -> U256 {
            self.balances.get(&account).copied().unwrap_or(U256::ZERO)
        }

        pub const fn decimals(&self) -> u8 {
            self.decimals_value
        }

        pub fn set_decimals(&mut self, decimals: u8) {
            self.decimals_value = decimals;
        }
    }

    impl FeeOnTransferERC20 {
        pub fn mint(&mut self, to: Address, amount: U256) {
            let current_balance = self.balances.get(&to).copied().unwrap_or(U256::ZERO);
            self.balances.insert(to, current_balance + amount);
        }

        pub fn transfer(&mut self, from: Address, to: Address, amount: U256) -> bool {
            let sender_balance = self.balances.get(&from).copied().unwrap_or(U256::ZERO);

            if sender_balance < amount {
                return false;
            }

            let fee = amount / U256::from(100);
            let amount_after_fee = amount - fee;

            self.balances.insert(from, sender_balance - amount);
            let recipient_balance = self.balances.get(&to).copied().unwrap_or(U256::ZERO);
            self.balances
                .insert(to, recipient_balance + amount_after_fee);
            true
        }

        pub fn balance_of(&self, account: Address) -> U256 {
            self.balances.get(&account).copied().unwrap_or(U256::ZERO)
        }
    }

    // Mock ERC20 Tests
    #[test]
    fn test_mock_erc20_mint_increases_balance() {
        let mut token = MockERC20::default();
        let alice = Address::from([1u8; 20]);
        let amount = U256::from(1000);

        token.mint(alice, amount);

        assert_eq!(token.balance_of(alice), amount);
    }

    #[test]
    fn test_mock_erc20_transfer_moves_tokens() {
        let mut token = MockERC20::default();
        let alice = Address::from([1u8; 20]);
        let bob = Address::from([2u8; 20]);
        let amount = U256::from(1000);

        token.mint(alice, amount);

        let transfer_amount = U256::from(600);
        let success = token.transfer(alice, bob, transfer_amount);

        assert!(success);
        assert_eq!(token.balance_of(alice), U256::from(400));
        assert_eq!(token.balance_of(bob), transfer_amount);
    }

    #[test]
    fn test_mock_erc20_transfer_from_with_approval() {
        let mut token = MockERC20::default();
        let alice = Address::from([1u8; 20]);
        let bob = Address::from([2u8; 20]);
        let charlie = Address::from([3u8; 20]);
        let amount = U256::from(1000);

        token.mint(alice, amount);

        let approval_amount = U256::from(600);
        token.approve(alice, bob, approval_amount);

        let transfer_amount = U256::from(400);
        let success = token.transfer_from(bob, alice, charlie, transfer_amount);

        assert!(success);
        assert_eq!(token.balance_of(alice), U256::from(600));
        assert_eq!(token.balance_of(charlie), transfer_amount);
    }

    #[test]
    fn test_mock_erc20_transfer_from_without_approval_fails() {
        let mut token = MockERC20::default();
        let alice = Address::from([1u8; 20]);
        let bob = Address::from([2u8; 20]);
        let charlie = Address::from([3u8; 20]);
        let amount = U256::from(1000);

        token.mint(alice, amount);

        let transfer_amount = U256::from(400);
        let success = token.transfer_from(bob, alice, charlie, transfer_amount);

        assert!(!success);
    }

    #[test]
    fn test_mock_erc20_decimals_returns_configured_value() {
        let mut token = MockERC20::default();
        let decimals = 6u8;

        token.set_decimals(decimals);

        assert_eq!(token.decimals(), decimals);
    }

    #[test]
    fn test_fee_on_transfer_erc20_deducts_fee() {
        let mut token = FeeOnTransferERC20::default();
        let alice = Address::from([1u8; 20]);
        let bob = Address::from([2u8; 20]);
        let amount = U256::from(1000);

        token.mint(alice, amount);

        let transfer_amount = U256::from(1000);
        token.transfer(alice, bob, transfer_amount);

        let expected_received = U256::from(990); // 99% of 1000
        assert_eq!(token.balance_of(bob), expected_received);
        assert_eq!(token.balance_of(alice), U256::ZERO);
    }

    #[test]
    fn test_fee_on_transfer_balance_after_transfer() {
        let mut token = FeeOnTransferERC20::default();
        let alice = Address::from([1u8; 20]);
        let bob = Address::from([2u8; 20]);
        let amount = U256::from(2000);

        token.mint(alice, amount);

        let transfer_amount = U256::from(1000);
        token.transfer(alice, bob, transfer_amount);

        let expected_bob_balance = U256::from(990); // 99% of 1000
        let expected_alice_balance = U256::from(1000); // 2000 - 1000

        assert_eq!(token.balance_of(bob), expected_bob_balance);
        assert_eq!(token.balance_of(alice), expected_alice_balance);
    }

    // Decimal Normalization Tests
    #[test]
    fn test_normalize_amount_usdc_6_decimals() {
        let amount = U256::from(1_000_000); // 1 USDC
        let result = normalize_amount(amount, 6);
        assert_eq!(result.unwrap(), U256::from(1_000_000_000_000_000_000u128)); // 10^18
    }

    #[test]
    fn test_normalize_amount_wbtc_8_decimals() {
        let amount = U256::from(100_000_000); // 1 WBTC
        let result = normalize_amount(amount, 8);
        assert_eq!(result.unwrap(), U256::from(1_000_000_000_000_000_000u128)); // 10^18
    }

    #[test]
    fn test_normalize_amount_18_decimals_no_change() {
        let amount = U256::from(1_000_000_000_000_000_000u128); // 1 ether
        let result = normalize_amount(amount, 18);
        assert_eq!(result.unwrap(), U256::from(1_000_000_000_000_000_000u128));
    }

    #[test]
    fn test_normalize_amount_0_decimals() {
        let amount = U256::from(1);
        let result = normalize_amount(amount, 0);
        assert_eq!(result.unwrap(), U256::from(1_000_000_000_000_000_000u128));
    }

    #[test]
    fn test_normalize_amount_invalid_decimals_24() {
        let amount = U256::from(1000);
        let result = normalize_amount(amount, 24);
        assert!(matches!(result, Err(OptionsError::InvalidDecimals(_))));
    }

    #[test]
    fn test_normalize_amount_overflow() {
        let result = normalize_amount(U256::MAX, 0);
        assert!(matches!(
            result,
            Err(OptionsError::NormalizationOverflow(_))
        ));
    }

    #[test]
    fn test_denormalize_amount_round_trip_6_decimals() {
        let original = U256::from(1_000_000); // 1 USDC
        let normalized = normalize_amount(original, 6).unwrap();
        let denormalized = denormalize_amount(normalized, 6).unwrap();
        assert_eq!(denormalized, original);
    }

    #[test]
    fn test_denormalize_amount_round_trip_8_decimals() {
        let original = U256::from(100_000_000); // 1 WBTC
        let normalized = normalize_amount(original, 8).unwrap();
        let denormalized = denormalize_amount(normalized, 8).unwrap();
        assert_eq!(denormalized, original);
    }

    #[test]
    fn test_denormalize_amount_round_trip_18_decimals() {
        let original = U256::from(1_000_000_000_000_000_000u128);
        let normalized = normalize_amount(original, 18).unwrap();
        let denormalized = denormalize_amount(normalized, 18).unwrap();
        assert_eq!(denormalized, original);
    }

    // ERC-1155 Balance Tracking Tests
    #[motsu::test]
    fn test_mint_increases_balance(contract: Contract<Options>, alice: Address) {
        let token_id = B256::from([0x42; 32]);
        let quantity = U256::from(100);

        contract
            .sender(alice)
            ._mint(alice, token_id, quantity)
            .unwrap();

        let balance = contract.sender(alice).balance_of(alice, token_id);
        assert_eq!(balance, quantity);
    }

    #[motsu::test]
    fn test_mint_increases_total_supply(contract: Contract<Options>, alice: Address) {
        let token_id = B256::from([0x42; 32]);
        let quantity = U256::from(100);

        contract
            .sender(alice)
            ._mint(alice, token_id, quantity)
            .unwrap();

        let total_supply = contract.sender(alice).total_supply_of(token_id);
        assert_eq!(total_supply, quantity);
    }

    #[motsu::test]
    fn test_burn_decreases_balance(contract: Contract<Options>, alice: Address) {
        let token_id = B256::from([0x42; 32]);
        let mint_quantity = U256::from(100);
        let burn_quantity = U256::from(40);

        contract
            .sender(alice)
            ._mint(alice, token_id, mint_quantity)
            .unwrap();
        contract
            .sender(alice)
            ._burn(alice, token_id, burn_quantity)
            .unwrap();

        let balance = contract.sender(alice).balance_of(alice, token_id);
        assert_eq!(balance, U256::from(60));
    }

    #[motsu::test]
    fn test_burn_decreases_total_supply(contract: Contract<Options>, alice: Address) {
        let token_id = B256::from([0x42; 32]);
        let mint_quantity = U256::from(100);
        let burn_quantity = U256::from(40);

        contract
            .sender(alice)
            ._mint(alice, token_id, mint_quantity)
            .unwrap();
        contract
            .sender(alice)
            ._burn(alice, token_id, burn_quantity)
            .unwrap();

        let total_supply = contract.sender(alice).total_supply_of(token_id);
        assert_eq!(total_supply, U256::from(60));
    }

    #[motsu::test]
    fn test_burn_insufficient_balance_fails(contract: Contract<Options>, alice: Address) {
        let token_id = B256::from([0x42; 32]);
        let mint_quantity = U256::from(100);
        let burn_quantity = U256::from(150);

        contract
            .sender(alice)
            ._mint(alice, token_id, mint_quantity)
            .unwrap();

        let result = contract.sender(alice)._burn(alice, token_id, burn_quantity);
        assert!(matches!(result, Err(OptionsError::InsufficientBalance(_))));
    }

    #[motsu::test]
    fn test_mint_overflow_fails(contract: Contract<Options>, alice: Address) {
        let token_id = B256::from([0x42; 32]);

        contract
            .sender(alice)
            ._mint(alice, token_id, U256::MAX)
            .unwrap();

        let result = contract.sender(alice)._mint(alice, token_id, U256::from(1));
        assert!(matches!(result, Err(OptionsError::Overflow(_))));
    }

    #[motsu::test]
    fn test_multiple_mints_accumulate(contract: Contract<Options>, alice: Address) {
        let token_id = B256::from([0x42; 32]);

        contract
            .sender(alice)
            ._mint(alice, token_id, U256::from(50))
            .unwrap();
        contract
            .sender(alice)
            ._mint(alice, token_id, U256::from(30))
            .unwrap();
        contract
            .sender(alice)
            ._mint(alice, token_id, U256::from(20))
            .unwrap();

        let balance = contract.sender(alice).balance_of(alice, token_id);
        assert_eq!(balance, U256::from(100));
    }

    #[motsu::test]
    fn test_mint_then_burn_same_amount_returns_zero(contract: Contract<Options>, alice: Address) {
        let token_id = B256::from([0x42; 32]);
        let quantity = U256::from(100);

        contract
            .sender(alice)
            ._mint(alice, token_id, quantity)
            .unwrap();
        contract
            .sender(alice)
            ._burn(alice, token_id, quantity)
            .unwrap();

        let balance = contract.sender(alice).balance_of(alice, token_id);
        assert_eq!(balance, U256::ZERO);

        let total_supply = contract.sender(alice).total_supply_of(token_id);
        assert_eq!(total_supply, U256::ZERO);
    }

    // Token ID Generation Tests
    #[test]
    fn test_generate_token_id_same_parameters_identical() {
        let underlying = Address::from([0x11; 20]);
        let quote = Address::from([0x22; 20]);
        let strike = U256::from(100_000);
        let expiry = U256::from(1_700_000_000);
        let option_type = OptionType::Call;

        let token_id_1 = generate_token_id(underlying, quote, strike, expiry, option_type);
        let token_id_2 = generate_token_id(underlying, quote, strike, expiry, option_type);

        assert_eq!(token_id_1, token_id_2);
    }

    #[test]
    fn test_generate_token_id_different_strikes() {
        let underlying = Address::from([0x11; 20]);
        let quote = Address::from([0x22; 20]);
        let expiry = U256::from(1_700_000_000);
        let option_type = OptionType::Call;

        let token_id_1 =
            generate_token_id(underlying, quote, U256::from(100_000), expiry, option_type);
        let token_id_2 =
            generate_token_id(underlying, quote, U256::from(200_000), expiry, option_type);

        assert_ne!(token_id_1, token_id_2);
    }

    #[test]
    fn test_generate_token_id_different_expiries() {
        let underlying = Address::from([0x11; 20]);
        let quote = Address::from([0x22; 20]);
        let strike = U256::from(100_000);
        let option_type = OptionType::Call;

        let token_id_1 = generate_token_id(
            underlying,
            quote,
            strike,
            U256::from(1_700_000_000),
            option_type,
        );
        let token_id_2 = generate_token_id(
            underlying,
            quote,
            strike,
            U256::from(1_800_000_000),
            option_type,
        );

        assert_ne!(token_id_1, token_id_2);
    }

    #[test]
    fn test_generate_token_id_different_option_types() {
        let underlying = Address::from([0x11; 20]);
        let quote = Address::from([0x22; 20]);
        let strike = U256::from(100_000);
        let expiry = U256::from(1_700_000_000);

        let token_id_call = generate_token_id(underlying, quote, strike, expiry, OptionType::Call);
        let token_id_put = generate_token_id(underlying, quote, strike, expiry, OptionType::Put);

        assert_ne!(token_id_call, token_id_put);
    }

    #[test]
    fn test_generate_token_id_different_underlying() {
        let quote = Address::from([0x22; 20]);
        let strike = U256::from(100_000);
        let expiry = U256::from(1_700_000_000);
        let option_type = OptionType::Call;

        let token_id_1 = generate_token_id(
            Address::from([0x11; 20]),
            quote,
            strike,
            expiry,
            option_type,
        );
        let token_id_2 = generate_token_id(
            Address::from([0x33; 20]),
            quote,
            strike,
            expiry,
            option_type,
        );

        assert_ne!(token_id_1, token_id_2);
    }

    #[test]
    fn test_generate_token_id_different_quote() {
        let underlying = Address::from([0x11; 20]);
        let strike = U256::from(100_000);
        let expiry = U256::from(1_700_000_000);
        let option_type = OptionType::Call;

        let token_id_1 = generate_token_id(
            underlying,
            Address::from([0x22; 20]),
            strike,
            expiry,
            option_type,
        );
        let token_id_2 = generate_token_id(
            underlying,
            Address::from([0x33; 20]),
            strike,
            expiry,
            option_type,
        );

        assert_ne!(token_id_1, token_id_2);
    }

    #[motsu::test]
    fn test_write_call_option_returns_unimplemented(contract: Contract<Options>, alice: Address) {
        let underlying = Token {
            address: Address::ZERO,
            decimals: 18,
        };
        let quote = Token {
            address: Address::ZERO,
            decimals: 6,
        };

        let result = contract.sender(alice).write_call_option(
            U256::from(1000),
            U256::from(1_234_567_890),
            U256::from(100),
            underlying,
            quote,
        );

        assert!(matches!(result, Err(OptionsError::Unimplemented(_))));
    }

    #[motsu::test]
    fn test_write_put_option_returns_unimplemented(contract: Contract<Options>, alice: Address) {
        let underlying = Token {
            address: Address::ZERO,
            decimals: 18,
        };
        let quote = Token {
            address: Address::ZERO,
            decimals: 6,
        };

        let result = contract.sender(alice).write_put_option(
            U256::from(1000),
            U256::from(1_234_567_890),
            U256::from(100),
            underlying,
            quote,
        );

        assert!(matches!(result, Err(OptionsError::Unimplemented(_))));
    }

    #[motsu::test]
    fn test_exercise_call_unimplemented(contract: Contract<Options>, alice: Address) {
        let result = contract
            .sender(alice)
            .exercise_call(B256::ZERO, U256::from(10));
        assert!(matches!(result, Err(OptionsError::Unimplemented(_))));
    }

    #[motsu::test]
    fn test_exercise_put_unimplemented(contract: Contract<Options>, alice: Address) {
        let result = contract
            .sender(alice)
            .exercise_put(B256::ZERO, U256::from(10));
        assert!(matches!(result, Err(OptionsError::Unimplemented(_))));
    }

    #[motsu::test]
    fn test_withdraw_expired_collateral_unimplemented(contract: Contract<Options>, alice: Address) {
        let result = contract
            .sender(alice)
            .withdraw_expired_collateral(B256::ZERO, U256::from(10));
        assert!(matches!(result, Err(OptionsError::Unimplemented(_))));
    }
}
