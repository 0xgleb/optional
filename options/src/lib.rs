#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::{vec, vec::Vec};
use alloy_primitives::{keccak256, Address, B256, U256, U8};
use alloy_sol_types::sol;

// Note: Using deprecated Call until sol_interface! macro is updated to use new trait paths
#[allow(deprecated)]
use stylus_sdk::call::Call;
use stylus_sdk::prelude::*;

#[cfg(test)]
mod mock_erc20;

sol! {
    /// Represents a token with its address and decimal precision.
    #[derive(Copy)]
    struct Token {
        address address;
        uint8 decimals;
    }

    /// Metadata for an option series (non-storage version for returning data).
    #[derive(Copy)]
    struct OptionMetadataView {
        address underlying;
        address quote;
        uint8 underlying_decimals;
        uint8 quote_decimals;
        uint256 strike;
        uint256 expiry;
        uint8 option_type;
    }

    /// Emitted when an option is written.
    event OptionWritten(
        address indexed writer,
        bytes32 indexed tokenId,
        uint256 quantity,
        uint256 collateral
    );

    /// Emitted when approval for all is set.
    event ApprovalForAll(
        address indexed owner,
        address indexed operator,
        bool approved
    );
}

// Implement AbiType for Token to make it usable in #[public] functions
impl stylus_sdk::abi::AbiType for Token {
    type SolType = Self;
    const ABI: stylus_sdk::abi::ConstString = stylus_sdk::abi::ConstString::new("(address,uint8)");
}

sol_interface! {
    /// ERC20 interface for interacting with external token contracts.
    interface IERC20 {
        function balanceOf(address account) external view returns (uint256);
        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }
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
    #[derive(Debug)]
    error InvalidStrike();
    #[derive(Debug)]
    error ExpiredOption(uint256 expiry, uint256 current);
    #[derive(Debug)]
    error InvalidQuantity();
    #[derive(Debug)]
    error SameToken();
    #[derive(Debug)]
    error FeeOnTransferDetected(uint256 expected, uint256 received);
    #[derive(Debug)]
    error TransferFailed();
    #[derive(Debug)]
    error UnexpectedBalanceDecrease();
    #[derive(Debug)]
    error SelfApproval();
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
    /// Strike price must be greater than zero.
    InvalidStrike(InvalidStrike),
    /// Option expiry must be in the future.
    ExpiredOption(ExpiredOption),
    /// Quantity must be greater than zero.
    InvalidQuantity(InvalidQuantity),
    /// Underlying and quote tokens must be different.
    SameToken(SameToken),
    /// Fee-on-transfer token detected.
    FeeOnTransferDetected(FeeOnTransferDetected),
    /// ERC20 transfer failed.
    TransferFailed(TransferFailed),
    /// Balance decreased unexpectedly.
    UnexpectedBalanceDecrease(UnexpectedBalanceDecrease),
    /// Cannot approve self as operator.
    SelfApproval(SelfApproval),
}

sol_storage! {
    /// Metadata for an option series.
    pub struct OptionMetadata {
        /// Underlying token address
        address underlying;
        /// Quote token address
        address quote;
        /// Underlying token decimals
        uint8 underlying_decimals;
        /// Quote token decimals
        uint8 quote_decimals;
        /// Strike price (18 decimals normalized)
        uint256 strike;
        /// Expiration timestamp
        uint256 expiry;
        /// Option type (0=Call, 1=Put)
        uint8 option_type;
    }

    /// Writer position for an option series.
    pub struct Position {
        /// Quantity of options written (18 decimals normalized)
        uint256 quantity_written;
        /// Collateral locked (18 decimals normalized)
        uint256 collateral_locked;
    }

    #[entrypoint]
    pub struct Options {
        /// Mapping from balance_key(owner, token_id) to balance
        mapping(bytes32 => uint256) balances;
        /// Mapping from token_id to total supply
        mapping(bytes32 => uint256) total_supply;
        /// Mapping from token_id to option metadata
        mapping(bytes32 => OptionMetadata) option_metadata;
        /// Mapping from position_key(writer, token_id) to position
        mapping(bytes32 => Position) positions;
        /// Mapping from approval_key(owner, operator) to approval status
        mapping(bytes32 => bool) operator_approvals;
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
/// - `expiry`: Expiration timestamp (Unix seconds)
/// - `option_type`: Call or Put
///
/// # Returns
/// Deterministic `B256` hash as token ID
#[must_use]
pub(crate) fn generate_token_id(
    underlying: Address,
    quote: Address,
    strike: U256,
    expiry: u64,
    option_type: OptionType,
) -> B256 {
    let encoded = [
        underlying.as_slice(),
        quote.as_slice(),
        strike.to_be_bytes::<32>().as_slice(),
        &expiry.to_be_bytes(),
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
#[allow(dead_code)] // TODO: Remove when used in Issue #6 (Exercise)
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

/// Validates parameters for writing an option.
///
/// Performs comprehensive validation of all option parameters at the contract boundary.
/// All external input is treated as untrusted.
///
/// # Parameters
/// - `strike`: Strike price (must be > 0)
/// - `expiry`: Expiration timestamp (must be > current_timestamp)
/// - `quantity`: Quantity of options (must be > 0)
/// - `underlying`: Underlying token
/// - `quote`: Quote token
/// - `current_timestamp`: Current block timestamp
///
/// # Errors
/// - `InvalidStrike`: Strike price is zero
/// - `ExpiredOption`: Expiry is not in the future
/// - `InvalidQuantity`: Quantity is zero
/// - `SameToken`: Underlying and quote addresses are identical
pub(crate) fn validate_write_params(
    strike: U256,
    expiry: u64,
    quantity: U256,
    underlying: Token,
    quote: Token,
    current_timestamp: u64,
) -> Result<(), OptionsError> {
    // Validate strike > 0
    if strike.is_zero() {
        return Err(OptionsError::InvalidStrike(InvalidStrike {}));
    }

    // Validate expiry > current_timestamp
    if expiry <= current_timestamp {
        return Err(OptionsError::ExpiredOption(ExpiredOption {
            expiry: U256::from(expiry),
            current: U256::from(current_timestamp),
        }));
    }

    // Validate quantity > 0
    if quantity.is_zero() {
        return Err(OptionsError::InvalidQuantity(InvalidQuantity {}));
    }

    // Validate underlying != quote
    if underlying.address == quote.address {
        return Err(OptionsError::SameToken(SameToken {}));
    }

    Ok(())
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
    /// - `expiry`: Expiration timestamp (Unix seconds)
    /// - `quantity`: Quantity of options to write (in underlying token's native decimals)
    /// - `underlying`: Underlying token (address and decimals)
    /// - `quote`: Quote token (address and decimals)
    ///
    /// # Returns
    /// Token ID (B256) representing this option series
    ///
    /// # Errors
    /// - `InvalidStrike`: Strike price is zero
    /// - `ExpiredOption`: Expiry is not in the future
    /// - `InvalidQuantity`: Quantity is zero
    /// - `SameToken`: Underlying and quote addresses are identical
    /// - `InvalidDecimals`: Token decimals exceed 18
    /// - `NormalizationOverflow`: Amount normalization would overflow
    /// - `Overflow`: Position or balance accumulation would overflow
    /// - `FeeOnTransferDetected`: Underlying token deducts fees during transfer
    /// - `TransferFailed`: ERC20 transfer failed
    pub fn write_call_option(
        &mut self,
        strike: U256,
        expiry: u64,
        quantity: U256,
        underlying: Token,
        quote: Token,
    ) -> Result<B256, OptionsError> {
        let (current_timestamp, sender, contract_addr) = {
            let vm = self.vm();
            (vm.block_timestamp(), vm.msg_sender(), vm.contract_address())
        };

        validate_write_params(
            strike,
            expiry,
            quantity,
            underlying,
            quote,
            current_timestamp,
        )?;

        let token_id = generate_token_id(
            underlying.address,
            quote.address,
            strike,
            expiry,
            OptionType::Call,
        );

        let normalized_quantity = normalize_amount(quantity, underlying.decimals)?;

        self.store_option_metadata(
            token_id,
            underlying,
            quote,
            strike,
            expiry,
            OptionType::Call,
        );

        self.create_or_update_position(sender, token_id, normalized_quantity, normalized_quantity)?;

        self._mint(sender, token_id, normalized_quantity)?;

        // External call after all state updates (reentrancy protection)
        self.safe_transfer_from(underlying.address, sender, contract_addr, quantity)?;

        log(
            self.vm(),
            OptionWritten {
                writer: sender,
                tokenId: token_id,
                quantity: normalized_quantity,
                collateral: normalized_quantity,
            },
        );

        Ok(token_id)
    }

    /// Writes a put option by locking quote tokens as collateral (strike * quantity).
    ///
    /// Mints ERC-1155 tokens representing the put option and returns a deterministic token ID
    /// based on the option parameters (keccak256 hash).
    ///
    /// # Parameters
    /// - `strike`: Strike price (18 decimals normalized)
    /// - `expiry`: Expiration timestamp (Unix seconds)
    /// - `quantity`: Quantity of options to write
    /// - `underlying`: Underlying token (address and decimals)
    /// - `quote`: Quote token (address and decimals)
    ///
    /// # Errors
    /// Returns `OptionsError::Unimplemented` (stub implementation).
    pub fn write_put_option(
        &mut self,
        strike: U256,
        expiry: u64,
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

    /// Grants or revokes permission for operator to transfer all tokens on behalf of caller.
    ///
    /// Enables external contracts (like CLOB) to transfer option tokens on behalf of users.
    /// Required for secondary market trading.
    ///
    /// # Parameters
    /// - `operator`: Address to grant/revoke approval
    /// - `approved`: True to grant approval, false to revoke
    ///
    /// # Errors
    /// - `SelfApproval`: Cannot approve self as operator
    pub fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), OptionsError> {
        let sender = self.vm().msg_sender();

        if operator == sender {
            return Err(OptionsError::SelfApproval(SelfApproval {}));
        }

        let key = Self::approval_key(sender, operator);
        self.operator_approvals.insert(key, approved);

        log(
            self.vm(),
            ApprovalForAll {
                owner: sender,
                operator,
                approved,
            },
        );

        Ok(())
    }

    /// Returns true if operator is approved to transfer owner's tokens.
    ///
    /// # Parameters
    /// - `owner`: Token owner address
    /// - `operator`: Operator address to check
    ///
    /// # Returns
    /// True if operator is approved, false otherwise
    #[must_use]
    pub fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        let key = Self::approval_key(owner, operator);
        self.operator_approvals.get(key)
    }
}

/// Test-only helper methods (accessible through motsu deref)
impl Options {
    /// Test wrapper for _mint - accessible in motsu tests through deref
    ///
    /// # Errors
    /// Returns `OptionsError::Overflow` if balance or total supply would overflow
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
    ///
    /// # Errors
    /// Returns `OptionsError::InsufficientBalance` if balance is less than quantity
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
    #[must_use]
    pub fn test_balance_of(&self, owner: Address, token_id: B256) -> U256 {
        self.balance_of(owner, token_id)
    }

    /// Test wrapper for total_supply_of - accessible in motsu tests through deref
    #[cfg(test)]
    #[must_use]
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

    /// Generates a composite key for operator approval lookups.
    ///
    /// # Parameters
    /// - `owner`: Token owner address
    /// - `operator`: Operator address
    ///
    /// # Returns
    /// `keccak256(owner || operator)` as composite key
    fn approval_key(owner: Address, operator: Address) -> B256 {
        let encoded = [owner.as_slice(), operator.as_slice()].concat();
        keccak256(encoded)
    }

    /// Checks if operator is authorized to act on behalf of owner.
    ///
    /// Authorization is granted if operator is the owner themselves, or if
    /// operator has been approved via `set_approval_for_all`.
    ///
    /// # Parameters
    /// - `owner`: Token owner address
    /// - `operator`: Operator address to check
    ///
    /// # Returns
    /// True if operator is owner or approved, false otherwise
    #[must_use]
    pub(crate) fn _is_authorized(&self, owner: Address, operator: Address) -> bool {
        operator == owner || self.is_approved_for_all(owner, operator)
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
    #[allow(dead_code)] // TODO: Remove when used in Issue #11 (Full ERC-1155)
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
    #[allow(dead_code)] // TODO: Remove when used in Issue #11 (Full ERC-1155)
    pub(crate) fn total_supply_of(&self, token_id: B256) -> U256 {
        self.total_supply.get(token_id)
    }

    /// Safely transfers ERC20 tokens with fee-on-transfer detection.
    ///
    /// Checks the recipient's balance before and after transfer to ensure the full
    /// amount was received. This prevents fee-on-transfer tokens from breaking
    /// collateral accounting.
    ///
    /// # Parameters
    /// - `token`: ERC20 token contract address
    /// - `from`: Address to transfer from (requires prior approval)
    /// - `to`: Recipient address
    /// - `amount`: Amount to transfer
    ///
    /// # Errors
    /// - `TransferFailed`: ERC20 transferFrom call failed
    /// - `FeeOnTransferDetected`: Received amount doesn't match requested amount
    /// - `UnexpectedBalanceDecrease`: Balance decreased instead of increased
    #[allow(deprecated)]
    pub(crate) fn safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        amount: U256,
    ) -> Result<(), OptionsError> {
        let erc20 = IERC20::new(token);

        let balance_before = erc20
            .balance_of(Call::new_in(self), to)
            .map_err(|_| OptionsError::TransferFailed(TransferFailed {}))?;

        let success = erc20
            .transfer_from(Call::new_in(self), from, to, amount)
            .map_err(|_| OptionsError::TransferFailed(TransferFailed {}))?;

        if !success {
            return Err(OptionsError::TransferFailed(TransferFailed {}));
        }

        let balance_after = erc20
            .balance_of(Call::new_in(self), to)
            .map_err(|_| OptionsError::TransferFailed(TransferFailed {}))?;

        let received = balance_after.checked_sub(balance_before).ok_or(
            OptionsError::UnexpectedBalanceDecrease(UnexpectedBalanceDecrease {}),
        )?;

        if received != amount {
            return Err(OptionsError::FeeOnTransferDetected(FeeOnTransferDetected {
                expected: amount,
                received,
            }));
        }

        Ok(())
    }

    /// Stores option metadata for a token ID.
    ///
    /// Metadata is stored once per option series on first write. Subsequent writes
    /// of the same option parameters reuse the existing metadata.
    ///
    /// # Parameters
    /// - `token_id`: ERC-1155 token ID (deterministic hash of option parameters)
    /// - `underlying`: Underlying token (address and decimals)
    /// - `quote`: Quote token (address and decimals)
    /// - `strike`: Strike price (18 decimals normalized)
    /// - `expiry`: Expiration timestamp
    /// - `option_type`: Call or Put
    pub(crate) fn store_option_metadata(
        &mut self,
        token_id: B256,
        underlying: Token,
        quote: Token,
        strike: U256,
        expiry: u64,
        option_type: OptionType,
    ) {
        let mut metadata = self.option_metadata.setter(token_id);
        metadata.underlying.set(underlying.address);
        metadata.quote.set(quote.address);
        metadata
            .underlying_decimals
            .set(U8::from(underlying.decimals));
        metadata.quote_decimals.set(U8::from(quote.decimals));
        metadata.strike.set(strike);
        metadata.expiry.set(U256::from(expiry));
        metadata.option_type.set(U8::from(option_type.to_u8()));
    }

    /// Retrieves option metadata for a token ID.
    ///
    /// # Parameters
    /// - `token_id`: ERC-1155 token ID
    ///
    /// # Returns
    /// Option metadata struct with all option parameters
    #[allow(dead_code)] // TODO: Remove when used in Issue #6 (Exercise)
    pub(crate) fn get_option_metadata(&self, token_id: B256) -> OptionMetadataView {
        let metadata = self.option_metadata.get(token_id);
        OptionMetadataView {
            underlying: metadata.underlying.get(),
            quote: metadata.quote.get(),
            underlying_decimals: metadata.underlying_decimals.get().to::<u8>(),
            quote_decimals: metadata.quote_decimals.get().to::<u8>(),
            strike: metadata.strike.get(),
            expiry: metadata.expiry.get(),
            option_type: metadata.option_type.get().to::<u8>(),
        }
    }

    /// Generates a composite key for position lookups.
    ///
    /// Position key = keccak256(writer, token_id)
    ///
    /// Each writer has independent positions per option series.
    fn position_key(writer: Address, token_id: B256) -> B256 {
        keccak256([writer.as_slice(), token_id.as_slice()].concat())
    }

    /// Creates or updates a writer's position for an option series.
    ///
    /// If position exists, accumulates quantity and collateral using checked arithmetic.
    /// If position is new, creates it with provided values.
    ///
    /// # Parameters
    /// - `writer`: Writer address
    /// - `token_id`: ERC-1155 token ID
    /// - `quantity`: Quantity to add (18 decimals normalized)
    /// - `collateral`: Collateral to add (18 decimals normalized)
    ///
    /// # Errors
    /// Returns `OptionsError::Overflow` if accumulation would overflow
    pub(crate) fn create_or_update_position(
        &mut self,
        writer: Address,
        token_id: B256,
        quantity: U256,
        collateral: U256,
    ) -> Result<(), OptionsError> {
        let key = Self::position_key(writer, token_id);
        let mut position = self.positions.setter(key);

        let current_quantity = position.quantity_written.get();
        let current_collateral = position.collateral_locked.get();

        let new_quantity = current_quantity
            .checked_add(quantity)
            .ok_or(OptionsError::Overflow(Overflow {}))?;
        let new_collateral = current_collateral
            .checked_add(collateral)
            .ok_or(OptionsError::Overflow(Overflow {}))?;

        position.quantity_written.set(new_quantity);
        position.collateral_locked.set(new_collateral);

        Ok(())
    }

    /// Retrieves a writer's position for an option series.
    ///
    /// # Parameters
    /// - `writer`: Writer address
    /// - `token_id`: ERC-1155 token ID
    ///
    /// # Returns
    /// Tuple of (quantity_written, collateral_locked)
    #[allow(dead_code)] // TODO: Remove when used in Issue #7 (Withdraw Collateral)
    pub(crate) fn get_position(&self, writer: Address, token_id: B256) -> (U256, U256) {
        let key = Self::position_key(writer, token_id);
        let position = self.positions.get(key);
        (
            position.quantity_written.get(),
            position.collateral_locked.get(),
        )
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Address;
    use motsu::prelude::*;

    use super::*;
    use crate::mock_erc20::{FeeOnTransferERC20, MockERC20};

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

    #[test]
    fn test_valid_parameters_pass_validation() {
        let strike = U256::from(50_000);
        let expiry = 1_700_000_000u64;
        let quantity = U256::from(100);
        let underlying = Token {
            address: Address::from([0x11; 20]),
            decimals: 18,
        };
        let quote = Token {
            address: Address::from([0x22; 20]),
            decimals: 6,
        };
        let current_timestamp = 1_600_000_000u64;

        validate_write_params(
            strike,
            expiry,
            quantity,
            underlying,
            quote,
            current_timestamp,
        )
        .unwrap();
    }

    #[test]
    fn test_zero_strike_fails() {
        let strike = U256::ZERO;
        let expiry = 1_700_000_000u64;
        let quantity = U256::from(100);
        let underlying = Token {
            address: Address::from([0x11; 20]),
            decimals: 18,
        };
        let quote = Token {
            address: Address::from([0x22; 20]),
            decimals: 6,
        };
        let current_timestamp = 1_600_000_000u64;

        let result = validate_write_params(
            strike,
            expiry,
            quantity,
            underlying,
            quote,
            current_timestamp,
        );
        assert!(matches!(result, Err(OptionsError::InvalidStrike(_))));
    }

    #[test]
    fn test_past_expiry_fails() {
        let strike = U256::from(50_000);
        let expiry = 1_500_000_000u64; // Past timestamp
        let quantity = U256::from(100);
        let underlying = Token {
            address: Address::from([0x11; 20]),
            decimals: 18,
        };
        let quote = Token {
            address: Address::from([0x22; 20]),
            decimals: 6,
        };
        let current_timestamp = 1_600_000_000u64;

        let result = validate_write_params(
            strike,
            expiry,
            quantity,
            underlying,
            quote,
            current_timestamp,
        );
        assert!(matches!(result, Err(OptionsError::ExpiredOption(_))));
    }

    #[test]
    fn test_expiry_equals_current_timestamp_fails() {
        let strike = U256::from(50_000);
        let expiry = 1_600_000_000u64;
        let quantity = U256::from(100);
        let underlying = Token {
            address: Address::from([0x11; 20]),
            decimals: 18,
        };
        let quote = Token {
            address: Address::from([0x22; 20]),
            decimals: 6,
        };
        let current_timestamp = 1_600_000_000u64; // Same as expiry

        let result = validate_write_params(
            strike,
            expiry,
            quantity,
            underlying,
            quote,
            current_timestamp,
        );
        assert!(matches!(result, Err(OptionsError::ExpiredOption(_))));
    }

    #[test]
    fn test_zero_quantity_fails() {
        let strike = U256::from(50_000);
        let expiry = 1_700_000_000u64;
        let quantity = U256::ZERO;
        let underlying = Token {
            address: Address::from([0x11; 20]),
            decimals: 18,
        };
        let quote = Token {
            address: Address::from([0x22; 20]),
            decimals: 6,
        };
        let current_timestamp = 1_600_000_000u64;

        let result = validate_write_params(
            strike,
            expiry,
            quantity,
            underlying,
            quote,
            current_timestamp,
        );
        assert!(matches!(result, Err(OptionsError::InvalidQuantity(_))));
    }

    #[test]
    fn test_same_underlying_and_quote_fails() {
        let strike = U256::from(50_000);
        let expiry = 1_700_000_000u64;
        let quantity = U256::from(100);
        let same_address = Address::from([0x11; 20]);
        let underlying = Token {
            address: same_address,
            decimals: 18,
        };
        let quote = Token {
            address: same_address,
            decimals: 6,
        };
        let current_timestamp = 1_600_000_000u64;

        let result = validate_write_params(
            strike,
            expiry,
            quantity,
            underlying,
            quote,
            current_timestamp,
        );
        assert!(matches!(result, Err(OptionsError::SameToken(_))));
    }

    #[test]
    fn test_minimum_valid_expiry_passes() {
        let strike = U256::from(50_000);
        let current_timestamp = 1_600_000_000u64;
        let expiry = current_timestamp + 1; // Minimum valid expiry
        let quantity = U256::from(100);
        let underlying = Token {
            address: Address::from([0x11; 20]),
            decimals: 18,
        };
        let quote = Token {
            address: Address::from([0x22; 20]),
            decimals: 6,
        };

        validate_write_params(
            strike,
            expiry,
            quantity,
            underlying,
            quote,
            current_timestamp,
        )
        .unwrap();
    }

    // Fee-on-Transfer Detection Tests
    #[test]
    fn test_transfer_from_mock_erc20_succeeds() {
        let mut token = MockERC20::default();
        let from = Address::from([0x01; 20]);
        let to = Address::from([0x02; 20]);
        let amount = U256::from(1000);

        token.mint(from, U256::from(10000));
        token.approve(from, from, U256::from(10000));

        let balance_before = token.balance_of(to);
        let success = token.transfer_from(from, from, to, amount);
        let balance_after = token.balance_of(to);

        assert!(success);
        let received = balance_after.checked_sub(balance_before).unwrap();
        assert_eq!(received, amount);
    }

    #[test]
    fn test_transfer_from_fee_on_transfer_fails_detection() {
        let mut token = FeeOnTransferERC20::default();
        let from = Address::from([0x01; 20]);
        let to = Address::from([0x02; 20]);
        let amount = U256::from(1000);

        token.mint(from, U256::from(10000));

        let balance_before = token.balance_of(to);
        token.transfer(from, to, amount);
        let balance_after = token.balance_of(to);

        let received = balance_after.checked_sub(balance_before).unwrap();
        // FeeOnTransferERC20 deducts 1% fee, so received should be less than amount
        assert!(received < amount);
        assert_eq!(received, amount - (amount / U256::from(100)));
    }

    #[test]
    fn test_fee_on_transfer_error_contains_correct_amounts() {
        let expected = U256::from(1000);
        let received = U256::from(990); // 1% fee deducted

        let error =
            OptionsError::FeeOnTransferDetected(FeeOnTransferDetected { expected, received });

        match error {
            OptionsError::FeeOnTransferDetected(e) => {
                assert_eq!(e.expected, expected);
                assert_eq!(e.received, received);
            }
            _ => panic!("Expected FeeOnTransferDetected error"),
        }
    }

    #[test]
    fn test_multiple_safe_transfers_all_succeed() {
        let mut token = MockERC20::default();
        let from = Address::from([0x01; 20]);
        let to1 = Address::from([0x02; 20]);
        let to2 = Address::from([0x03; 20]);
        let to3 = Address::from([0x04; 20]);
        let amount = U256::from(100);

        token.mint(from, U256::from(10000));
        token.approve(from, from, U256::from(10000));

        let balance_before = token.balance_of(to1);
        let success = token.transfer_from(from, from, to1, amount);
        let balance_after = token.balance_of(to1);
        assert!(success);
        assert_eq!(balance_after.checked_sub(balance_before).unwrap(), amount);

        let balance_before = token.balance_of(to2);
        let success = token.transfer_from(from, from, to2, amount);
        let balance_after = token.balance_of(to2);
        assert!(success);
        assert_eq!(balance_after.checked_sub(balance_before).unwrap(), amount);

        let balance_before = token.balance_of(to3);
        let success = token.transfer_from(from, from, to3, amount);
        let balance_after = token.balance_of(to3);
        assert!(success);
        assert_eq!(balance_after.checked_sub(balance_before).unwrap(), amount);
    }

    // Option Metadata Storage Tests
    #[motsu::test]
    fn test_store_and_retrieve_metadata(contract: Contract<Options>) {
        let token_id = B256::from([0x42; 32]);
        let underlying = Token {
            address: Address::from([0x11; 20]),
            decimals: 8,
        };
        let quote = Token {
            address: Address::from([0x22; 20]),
            decimals: 6,
        };
        let strike = U256::from(50_000);
        let expiry = 1_700_000_000u64;
        let option_type = OptionType::Call;

        contract.sender(Address::ZERO).store_option_metadata(
            token_id,
            underlying,
            quote,
            strike,
            expiry,
            option_type,
        );

        let metadata = contract.sender(Address::ZERO).get_option_metadata(token_id);

        assert_eq!(metadata.underlying, underlying.address);
        assert_eq!(metadata.quote, quote.address);
        assert_eq!(metadata.underlying_decimals, underlying.decimals);
        assert_eq!(metadata.quote_decimals, quote.decimals);
        assert_eq!(metadata.strike, strike);
        assert_eq!(metadata.expiry, U256::from(expiry));
        assert_eq!(metadata.option_type, option_type.to_u8());
    }

    #[motsu::test]
    fn test_metadata_fields_match_input_parameters(contract: Contract<Options>) {
        let token_id = B256::from([0x99; 32]);
        let underlying = Token {
            address: Address::from([0xAA; 20]),
            decimals: 18,
        };
        let quote = Token {
            address: Address::from([0xBB; 20]),
            decimals: 6,
        };
        let strike = U256::from(100_000);
        let expiry = 1_800_000_000u64;
        let option_type = OptionType::Put;

        contract.sender(Address::ZERO).store_option_metadata(
            token_id,
            underlying,
            quote,
            strike,
            expiry,
            option_type,
        );

        let metadata = contract.sender(Address::ZERO).get_option_metadata(token_id);

        assert_eq!(metadata.underlying, underlying.address);
        assert_eq!(metadata.quote, quote.address);
        assert_eq!(metadata.underlying_decimals, 18);
        assert_eq!(metadata.quote_decimals, 6);
        assert_eq!(metadata.strike, U256::from(100_000));
        assert_eq!(metadata.expiry, U256::from(1_800_000_000u64));
        assert_eq!(metadata.option_type, 1); // Put = 1
    }

    #[motsu::test]
    fn test_same_token_id_retrieves_same_metadata(contract: Contract<Options>) {
        let token_id = B256::from([0x77; 32]);
        let underlying = Token {
            address: Address::from([0x33; 20]),
            decimals: 8,
        };
        let quote = Token {
            address: Address::from([0x44; 20]),
            decimals: 6,
        };
        let strike = U256::from(60_000);
        let expiry = 1_750_000_000u64;
        let option_type = OptionType::Call;

        contract.sender(Address::ZERO).store_option_metadata(
            token_id,
            underlying,
            quote,
            strike,
            expiry,
            option_type,
        );

        let metadata1 = contract.sender(Address::ZERO).get_option_metadata(token_id);

        let metadata2 = contract.sender(Address::ZERO).get_option_metadata(token_id);

        assert_eq!(metadata1.underlying, metadata2.underlying);
        assert_eq!(metadata1.quote, metadata2.quote);
        assert_eq!(metadata1.strike, metadata2.strike);
        assert_eq!(metadata1.expiry, metadata2.expiry);
        assert_eq!(metadata1.option_type, metadata2.option_type);
    }

    #[motsu::test]
    fn test_different_token_ids_have_independent_metadata(contract: Contract<Options>) {
        let token_id_1 = B256::from([0x11; 32]);
        let token_id_2 = B256::from([0x22; 32]);

        let underlying_1 = Token {
            address: Address::from([0xAA; 20]),
            decimals: 8,
        };
        let quote_1 = Token {
            address: Address::from([0xBB; 20]),
            decimals: 6,
        };
        let strike_1 = U256::from(50_000);
        let expiry_1 = 1_700_000_000u64;

        let underlying_2 = Token {
            address: Address::from([0xCC; 20]),
            decimals: 18,
        };
        let quote_2 = Token {
            address: Address::from([0xDD; 20]),
            decimals: 6,
        };
        let strike_2 = U256::from(100_000);
        let expiry_2 = 1_800_000_000u64;

        contract.sender(Address::ZERO).store_option_metadata(
            token_id_1,
            underlying_1,
            quote_1,
            strike_1,
            expiry_1,
            OptionType::Call,
        );

        contract.sender(Address::ZERO).store_option_metadata(
            token_id_2,
            underlying_2,
            quote_2,
            strike_2,
            expiry_2,
            OptionType::Put,
        );

        let metadata_1 = contract
            .sender(Address::ZERO)
            .get_option_metadata(token_id_1);

        let metadata_2 = contract
            .sender(Address::ZERO)
            .get_option_metadata(token_id_2);

        // Verify metadata_1
        assert_eq!(metadata_1.underlying, underlying_1.address);
        assert_eq!(metadata_1.strike, strike_1);
        assert_eq!(metadata_1.option_type, 0); // Call

        // Verify metadata_2
        assert_eq!(metadata_2.underlying, underlying_2.address);
        assert_eq!(metadata_2.strike, strike_2);
        assert_eq!(metadata_2.option_type, 1); // Put

        // Verify they're different
        assert_ne!(metadata_1.underlying, metadata_2.underlying);
        assert_ne!(metadata_1.strike, metadata_2.strike);
    }

    // Writer Position Tracking Tests
    #[motsu::test]
    fn test_create_new_position_stores_quantity_and_collateral(contract: Contract<Options>) {
        let writer = Address::from([0xAA; 20]);
        let token_id = B256::from([0x01; 32]);
        let quantity = U256::from(100);
        let collateral = U256::from(200);

        contract
            .sender(writer)
            .create_or_update_position(writer, token_id, quantity, collateral)
            .unwrap();

        let (stored_quantity, stored_collateral) =
            contract.sender(writer).get_position(writer, token_id);

        assert_eq!(stored_quantity, quantity);
        assert_eq!(stored_collateral, collateral);
    }

    #[motsu::test]
    fn test_increase_existing_position_accumulates_correctly(contract: Contract<Options>) {
        let writer = Address::from([0xBB; 20]);
        let token_id = B256::from([0x02; 32]);
        let initial_quantity = U256::from(50);
        let initial_collateral = U256::from(100);
        let additional_quantity = U256::from(30);
        let additional_collateral = U256::from(60);

        contract
            .sender(writer)
            .create_or_update_position(writer, token_id, initial_quantity, initial_collateral)
            .unwrap();

        contract
            .sender(writer)
            .create_or_update_position(writer, token_id, additional_quantity, additional_collateral)
            .unwrap();

        let (final_quantity, final_collateral) =
            contract.sender(writer).get_position(writer, token_id);

        assert_eq!(final_quantity, U256::from(80));
        assert_eq!(final_collateral, U256::from(160));
    }

    #[motsu::test]
    fn test_different_writers_same_token_id_have_independent_positions(
        contract: Contract<Options>,
    ) {
        let writer1 = Address::from([0xCC; 20]);
        let writer2 = Address::from([0xDD; 20]);
        let token_id = B256::from([0x03; 32]);
        let quantity1 = U256::from(100);
        let collateral1 = U256::from(200);
        let quantity2 = U256::from(150);
        let collateral2 = U256::from(300);

        contract
            .sender(writer1)
            .create_or_update_position(writer1, token_id, quantity1, collateral1)
            .unwrap();

        contract
            .sender(writer2)
            .create_or_update_position(writer2, token_id, quantity2, collateral2)
            .unwrap();

        let (stored_quantity1, stored_collateral1) =
            contract.sender(writer1).get_position(writer1, token_id);
        let (stored_quantity2, stored_collateral2) =
            contract.sender(writer2).get_position(writer2, token_id);

        assert_eq!(stored_quantity1, quantity1);
        assert_eq!(stored_collateral1, collateral1);
        assert_eq!(stored_quantity2, quantity2);
        assert_eq!(stored_collateral2, collateral2);
    }

    #[test]
    fn test_position_key_is_deterministic() {
        let writer = Address::from([0xEE; 20]);
        let token_id = B256::from([0x04; 32]);

        let key1 = Options::position_key(writer, token_id);
        let key2 = Options::position_key(writer, token_id);

        assert_eq!(key1, key2);
    }

    // Token ID Generation Tests
    #[test]
    fn test_generate_token_id_same_parameters_identical() {
        let underlying = Address::from([0x11; 20]);
        let quote = Address::from([0x22; 20]);
        let strike = U256::from(100_000);
        let expiry = 1_700_000_000u64;
        let option_type = OptionType::Call;

        let token_id_1 = generate_token_id(underlying, quote, strike, expiry, option_type);
        let token_id_2 = generate_token_id(underlying, quote, strike, expiry, option_type);

        assert_eq!(token_id_1, token_id_2);
    }

    #[test]
    fn test_generate_token_id_different_strikes() {
        let underlying = Address::from([0x11; 20]);
        let quote = Address::from([0x22; 20]);
        let expiry = 1_700_000_000u64;
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

        let token_id_1 =
            generate_token_id(underlying, quote, strike, 1_700_000_000u64, option_type);
        let token_id_2 =
            generate_token_id(underlying, quote, strike, 1_800_000_000u64, option_type);

        assert_ne!(token_id_1, token_id_2);
    }

    #[test]
    fn test_generate_token_id_different_option_types() {
        let underlying = Address::from([0x11; 20]);
        let quote = Address::from([0x22; 20]);
        let strike = U256::from(100_000);
        let expiry = 1_700_000_000u64;

        let token_id_call = generate_token_id(underlying, quote, strike, expiry, OptionType::Call);
        let token_id_put = generate_token_id(underlying, quote, strike, expiry, OptionType::Put);

        assert_ne!(token_id_call, token_id_put);
    }

    #[test]
    fn test_generate_token_id_different_underlying() {
        let quote = Address::from([0x22; 20]);
        let strike = U256::from(100_000);
        let expiry = 1_700_000_000u64;
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
        let expiry = 1_700_000_000u64;
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
    fn test_write_call_option_zero_strike_fails(contract: Contract<Options>, alice: Address) {
        let underlying = Token {
            address: Address::from([0x11; 20]),
            decimals: 8,
        };
        let quote = Token {
            address: Address::from([0x22; 20]),
            decimals: 6,
        };
        let strike = U256::ZERO;
        let expiry = 2_000_000_000u64;
        let quantity = U256::from(100_000_000);

        let result = contract
            .sender(alice)
            .write_call_option(strike, expiry, quantity, underlying, quote);

        assert!(matches!(result, Err(OptionsError::InvalidStrike(_))));
    }

    #[motsu::test]
    fn test_write_call_option_expired_option_fails(contract: Contract<Options>, alice: Address) {
        let underlying = Token {
            address: Address::from([0x11; 20]),
            decimals: 8,
        };
        let quote = Token {
            address: Address::from([0x22; 20]),
            decimals: 6,
        };
        let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
        let expiry = 1_000_000_000u64;
        let quantity = U256::from(100_000_000);

        let result = contract
            .sender(alice)
            .write_call_option(strike, expiry, quantity, underlying, quote);

        assert!(matches!(result, Err(OptionsError::ExpiredOption(_))));
    }

    #[motsu::test]
    fn test_write_call_option_zero_quantity_fails(contract: Contract<Options>, alice: Address) {
        let underlying = Token {
            address: Address::from([0x11; 20]),
            decimals: 8,
        };
        let quote = Token {
            address: Address::from([0x22; 20]),
            decimals: 6,
        };
        let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
        let expiry = 2_000_000_000u64;
        let quantity = U256::ZERO;

        let result = contract
            .sender(alice)
            .write_call_option(strike, expiry, quantity, underlying, quote);

        assert!(matches!(result, Err(OptionsError::InvalidQuantity(_))));
    }

    #[motsu::test]
    fn test_write_call_option_same_token_fails(contract: Contract<Options>, alice: Address) {
        let same_address = Address::from([0x11; 20]);
        let underlying = Token {
            address: same_address,
            decimals: 8,
        };
        let quote = Token {
            address: same_address,
            decimals: 6,
        };
        let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
        let expiry = 2_000_000_000u64;
        let quantity = U256::from(100_000_000);

        let result = contract
            .sender(alice)
            .write_call_option(strike, expiry, quantity, underlying, quote);

        assert!(matches!(result, Err(OptionsError::SameToken(_))));
    }

    #[motsu::test]
    fn test_write_call_option_invalid_decimals_fails(contract: Contract<Options>, alice: Address) {
        let underlying = Token {
            address: Address::from([0x11; 20]),
            decimals: 24,
        };
        let quote = Token {
            address: Address::from([0x22; 20]),
            decimals: 6,
        };
        let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
        let expiry = 2_000_000_000u64;
        let quantity = U256::from(100_000_000);

        let result = contract
            .sender(alice)
            .write_call_option(strike, expiry, quantity, underlying, quote);

        assert!(matches!(result, Err(OptionsError::InvalidDecimals(_))));
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
            1_234_567_890u64,
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

    #[motsu::test]
    fn test_set_approval_for_all_stores_approval(contract: Contract<Options>, alice: Address) {
        let operator = Address::from([0x99; 20]);

        contract
            .sender(alice)
            .set_approval_for_all(operator, true)
            .unwrap();

        let is_approved = contract.sender(alice).is_approved_for_all(alice, operator);

        assert!(is_approved);
    }

    #[motsu::test]
    fn test_set_approval_for_all_revokes_approval(contract: Contract<Options>, alice: Address) {
        let operator = Address::from([0x99; 20]);

        contract
            .sender(alice)
            .set_approval_for_all(operator, true)
            .unwrap();

        contract
            .sender(alice)
            .set_approval_for_all(operator, false)
            .unwrap();

        let is_approved = contract.sender(alice).is_approved_for_all(alice, operator);

        assert!(!is_approved);
    }

    #[motsu::test]
    fn test_cannot_approve_self(contract: Contract<Options>, alice: Address) {
        let result = contract.sender(alice).set_approval_for_all(alice, true);

        assert!(matches!(result, Err(OptionsError::SelfApproval(_))));
    }

    #[motsu::test]
    fn test_is_authorized_returns_true_for_owner(contract: Contract<Options>, alice: Address) {
        let is_authorized = contract.sender(alice)._is_authorized(alice, alice);

        assert!(is_authorized);
    }

    #[motsu::test]
    fn test_is_authorized_returns_true_for_approved_operator(
        contract: Contract<Options>,
        alice: Address,
    ) {
        let operator = Address::from([0x88; 20]);

        contract
            .sender(alice)
            .set_approval_for_all(operator, true)
            .unwrap();

        let is_authorized = contract.sender(alice)._is_authorized(alice, operator);

        assert!(is_authorized);
    }

    #[motsu::test]
    fn test_is_authorized_returns_false_for_non_approved_operator(
        contract: Contract<Options>,
        alice: Address,
    ) {
        let operator = Address::from([0x77; 20]);

        let is_authorized = contract.sender(alice)._is_authorized(alice, operator);

        assert!(!is_authorized);
    }

    #[motsu::test]
    fn test_default_approval_is_false(contract: Contract<Options>, alice: Address) {
        let operator = Address::from([0x66; 20]);

        let is_approved = contract.sender(alice).is_approved_for_all(alice, operator);

        assert!(!is_approved);
    }
}

#[cfg(test)]
mod proptests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn prop_token_id_determinism(
            underlying in any::<Address>(),
            quote in any::<Address>(),
            strike in any::<U256>(),
            expiry in any::<u64>(),
            is_call in any::<bool>(),
        ) {
            let option_type = if is_call { OptionType::Call } else { OptionType::Put };

            let token_id_1 = generate_token_id(underlying, quote, strike, expiry, option_type);
            let token_id_2 = generate_token_id(underlying, quote, strike, expiry, option_type);
            let token_id_3 = generate_token_id(underlying, quote, strike, expiry, option_type);

            prop_assert_eq!(token_id_1, token_id_2);
            prop_assert_eq!(token_id_2, token_id_3);
        }

        #[test]
        fn prop_decimal_round_trip(
            amount in 1u64..=1_000_000_000_000u64,
            decimals in 0u8..=18u8,
        ) {
            let amount_u256 = U256::from(amount);

            let normalized = normalize_amount(amount_u256, decimals);
            prop_assert!(normalized.is_ok());

            let normalized_value = normalized.unwrap();
            let denormalized = denormalize_amount(normalized_value, decimals);
            prop_assert!(denormalized.is_ok());

            prop_assert_eq!(denormalized.unwrap(), amount_u256);
        }

        #[test]
        fn prop_normalize_never_panics(
            amount in any::<u64>(),
            decimals in any::<u8>(),
        ) {
            let amount_u256 = U256::from(amount);
            let result = normalize_amount(amount_u256, decimals);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn prop_denormalize_never_panics(
            amount_low in any::<u64>(),
            decimals in any::<u8>(),
        ) {
            let amount = U256::from(amount_low);
            let result = denormalize_amount(amount, decimals);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn prop_validate_params_never_panics(
            strike in any::<U256>(),
            expiry in any::<u64>(),
            quantity in any::<U256>(),
            underlying_address in any::<Address>(),
            quote_address in any::<Address>(),
            underlying_decimals in any::<u8>(),
            quote_decimals in any::<u8>(),
            current_time in any::<u64>(),
        ) {
            let underlying = Token {
                address: underlying_address,
                decimals: underlying_decimals,
            };
            let quote = Token {
                address: quote_address,
                decimals: quote_decimals,
            };

            let result = validate_write_params(strike, expiry, quantity, underlying, quote, current_time);
            prop_assert!(result.is_ok() || result.is_err());
        }

        #[test]
        fn prop_position_key_determinism(
            writer in any::<Address>(),
            token_id in any::<B256>(),
        ) {
            let key1 = Options::position_key(writer, token_id);
            let key2 = Options::position_key(writer, token_id);
            let key3 = Options::position_key(writer, token_id);

            prop_assert_eq!(key1, key2);
            prop_assert_eq!(key2, key3);
        }

        #[test]
        fn prop_different_writers_different_keys(
            writer1 in any::<Address>(),
            writer2 in any::<Address>(),
            token_id in any::<B256>(),
        ) {
            prop_assume!(writer1 != writer2);

            let key1 = Options::position_key(writer1, token_id);
            let key2 = Options::position_key(writer2, token_id);
            prop_assert_ne!(key1, key2);
        }

        #[test]
        fn prop_different_token_ids_different_keys(
            writer in any::<Address>(),
            token_id1 in any::<B256>(),
            token_id2 in any::<B256>(),
        ) {
            prop_assume!(token_id1 != token_id2);

            let key1 = Options::position_key(writer, token_id1);
            let key2 = Options::position_key(writer, token_id2);
            prop_assert_ne!(key1, key2);
        }
    }
}
