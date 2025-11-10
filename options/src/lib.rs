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
}

#[derive(SolidityError, Debug)]
pub enum OptionsError {
    /// Stub implementation placeholder - function not yet implemented.
    Unimplemented(Unimplemented),
    /// Token decimals exceed maximum of 18.
    InvalidDecimals(InvalidDecimals),
    /// Arithmetic overflow during normalization.
    NormalizationOverflow(NormalizationOverflow),
}

sol_storage! {
    #[entrypoint]
    pub struct Options {
        bool placeholder;
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

#[cfg(test)]
mod tests {
    use alloy_primitives::Address;
    use motsu::prelude::*;

    use super::*;

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
