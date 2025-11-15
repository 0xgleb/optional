#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

#[cfg(feature = "export-abi")]
pub fn print_abi_from_args() {
    stylus_sdk::export_abi!("vault", 1);
}

use alloc::{vec, vec::Vec};
use alloy_primitives::{Address, U256, U8};
use alloy_sol_types::sol;
use stylus_sdk::prelude::*;
use stylus_sdk::storage::{StorageAddress, StorageBool, StorageU256, StorageU8};

sol! {
    /// Deposit checkpoint for FIFO assignment tracking.
    #[derive(Copy)]
    struct DepositCheckpoint {
        address writer;
        uint256 amount;
        uint256 cumulative_total;
    }

    /// Emitted when a writer deposits collateral to the vault.
    event Deposit(
        address indexed writer,
        uint256 assets,
        uint256 shares,
        uint256 indexed checkpoint_index,
        uint256 cumulative_total
    );

    /// Emitted when options are exercised and vault assets withdrawn.
    event ExerciseWithdraw(
        address indexed recipient,
        uint256 assets,
        uint256 total_exercised
    );

    /// Emitted when a writer claims their entitlement after expiry.
    event Claim(
        address indexed writer,
        uint256 strike_payment,
        uint256 collateral_returned
    );

    /// Errors that can occur in the OptionVault contract.
    #[derive(Debug)]
    error Unimplemented();
    #[derive(Debug)]
    error NotExpired(uint256 expiry, uint256 current);
    #[derive(Debug)]
    error AlreadyExpired(uint256 expiry, uint256 current);
    #[derive(Debug)]
    error UnauthorizedCaller(address expected, address actual);
    #[derive(Debug)]
    error InsufficientBacking(uint256 shares, uint256 options_outstanding);
    #[derive(Debug)]
    error ZeroAmount();
}

sol_storage! {
    #[entrypoint]
    pub struct OptionVault {
        // Asset token address (underlying for calls, quote for puts)
        StorageAddress asset;

        // Hardcoded decimals offset for inflation attack protection
        // Value: 3 (provides 1000x security multiplier)
        StorageU8 decimals_offset;

        // Option series this vault backs
        StorageAddress options_contract;
        StorageU256 expiry;

        // Backing constraints
        StorageU256 options_outstanding;
        StorageBool expired;

        // FIFO deposit tracking for assignment (simplified for stub)
        StorageU256 checkpoint_count;
        StorageU256 total_exercised;

        // Total assets held by vault (for ERC-4626 compliance)
        StorageU256 total_assets;
    }
}

// Private helper methods
#[allow(dead_code)]
impl OptionVault {
    /// Returns the address of the underlying asset token.
    ///
    /// # Returns
    /// Address of the ERC20 asset token
    fn asset(&self) -> Address {
        self.asset.get()
    }

    /// Returns the total amount of underlying assets held by the vault.
    ///
    /// # Returns
    /// Total assets in the vault
    fn total_assets(&self) -> U256 {
        self.total_assets.get()
    }

    /// Returns the decimals offset used for inflation attack protection.
    ///
    /// # Returns
    /// Hardcoded value of 3 (1000x security multiplier)
    fn decimals_offset(&self) -> U8 {
        self.decimals_offset.get()
    }
}

#[public]
impl OptionVault {
    /// Initializes the vault with the asset token and hardcoded inflation protection.
    ///
    /// TODO: Replace with proper constructor when upgrading to stylus-sdk that supports it.
    ///
    /// # Arguments
    /// * `asset` - The ERC20 token used as collateral
    /// * `options_contract` - The OptionToken contract address
    /// * `expiry` - The option expiry timestamp
    ///
    /// # Security
    /// Hardcodes `decimals_offset=3` for ERC-4626 inflation attack protection.
    /// This provides a 1000x security multiplier without requiring pricing oracles.
    ///
    /// # Errors
    /// Currently returns no errors (stub implementation).
    ///
    /// # WARNING
    /// This is a temporary initialization pattern. In production, this MUST be replaced
    /// with a proper constructor or initialization guard to prevent re-initialization attacks.
    pub fn initialize(
        &mut self,
        asset: Address,
        options_contract: Address,
        expiry: U256,
    ) -> Result<(), VaultError> {
        // TODO: Add initialization guard to prevent calling this twice
        // Store asset
        self.asset.set(asset);

        // Hardcode decimals_offset=3 for uniform inflation protection
        // NOT a parameter - prevents bypass attacks
        self.decimals_offset.set(U8::from(3));

        // Store option series info
        self.options_contract.set(options_contract);
        self.expiry.set(expiry);

        // Initialize state
        self.options_outstanding.set(U256::ZERO);
        self.expired.set(false);
        self.checkpoint_count.set(U256::ZERO);
        self.total_exercised.set(U256::ZERO);
        self.total_assets.set(U256::ZERO);

        Ok(())
    }
    /// Deposits assets into the vault and mints shares to receiver.
    /// Creates a checkpoint for FIFO assignment tracking.
    ///
    /// # Arguments
    /// * `assets` - Amount of asset tokens to deposit
    /// * `receiver` - Address to receive vault shares
    ///
    /// # Returns
    /// Number of shares minted
    ///
    /// # Errors
    /// - `AlreadyExpired` if called after expiry
    /// - `ZeroAmount` if assets is zero
    pub fn deposit(&mut self, _assets: U256, _receiver: Address) -> Result<U256, VaultError> {
        Err(VaultError::Unimplemented(Unimplemented {}))
    }

    /// Withdraws assets from the vault during option exercise.
    /// Can only be called by the associated OptionToken contract.
    ///
    /// # Arguments
    /// * `assets` - Amount of assets to withdraw
    /// * `recipient` - Address to receive the assets
    ///
    /// # Returns
    /// Amount of assets withdrawn
    ///
    /// # Errors
    /// - `UnauthorizedCaller` if caller is not the options contract
    /// - `AlreadyExpired` if called after expiry
    pub fn exercise_withdraw(
        &mut self,
        _assets: U256,
        _recipient: Address,
    ) -> Result<U256, VaultError> {
        Err(VaultError::Unimplemented(Unimplemented {}))
    }

    /// Claims strike payments (if assigned) or collateral (if unassigned) after expiry.
    /// Uses FIFO assignment based on deposit order.
    ///
    /// # Returns
    /// Tuple of (strike_payment, collateral_returned)
    ///
    /// # Errors
    /// - `NotExpired` if called before expiry
    pub fn claim(&mut self) -> Result<(U256, U256), VaultError> {
        Err(VaultError::Unimplemented(Unimplemented {}))
    }

    /// Burns vault shares along with option tokens for early collateral redemption.
    /// Can only be called by the associated OptionToken contract.
    ///
    /// # Arguments
    /// * `shares` - Amount of shares to burn
    /// * `account` - Account that owns the shares and options
    ///
    /// # Returns
    /// Amount of collateral returned
    ///
    /// # Errors
    /// - `UnauthorizedCaller` if caller is not the options contract
    /// - `InsufficientBacking` if not enough backing exists
    pub fn burn_shares_with_options(
        &mut self,
        _shares: U256,
        _account: Address,
    ) -> Result<U256, VaultError> {
        Err(VaultError::Unimplemented(Unimplemented {}))
    }

    /// Marks the vault as expired. Can be called by anyone after expiry time.
    ///
    /// # Errors
    /// - `NotExpired` if current time is before expiry
    pub fn mark_expired(&mut self) -> Result<(), VaultError> {
        Err(VaultError::Unimplemented(Unimplemented {}))
    }

    // ========================================
    // View Functions
    // ========================================

    /// Returns the total number of checkpoints created.
    #[must_use]
    pub fn get_checkpoint_count(&self) -> U256 {
        self.checkpoint_count.get()
    }

    /// Returns the total amount of options exercised.
    #[must_use]
    pub fn get_total_exercised(&self) -> U256 {
        self.total_exercised.get()
    }

    /// Returns the total amount of options outstanding.
    #[must_use]
    pub fn get_options_outstanding(&self) -> U256 {
        self.options_outstanding.get()
    }

    /// Returns whether the vault has been marked as expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expired.get()
    }

    /// Returns the expiry timestamp for this vault.
    #[must_use]
    pub fn get_expiry(&self) -> U256 {
        self.expiry.get()
    }

    /// Returns the checkpoint at the given index.
    ///
    /// TODO: Implement when checkpoint storage is added.
    ///
    /// # Arguments
    /// * `_index` - Checkpoint index
    ///
    /// # Returns
    /// Checkpoint data (writer, amount, cumulative_total)
    #[must_use]
    pub const fn get_checkpoint(&self, _index: U256) -> (Address, U256, U256) {
        (Address::ZERO, U256::ZERO, U256::ZERO)
    }

    /// Returns the list of checkpoint indices for a writer.
    ///
    /// TODO: Implement when checkpoint storage is added.
    ///
    /// # Arguments
    /// * `_writer` - Writer address
    ///
    /// # Returns
    /// Array of checkpoint indices
    #[must_use]
    pub const fn get_writer_checkpoints(&self, _writer: Address) -> Vec<U256> {
        Vec::new()
    }
}

/// Custom error type combining vault errors.
#[derive(SolidityError, Debug)]
pub enum VaultError {
    Unimplemented(Unimplemented),
    NotExpired(NotExpired),
    AlreadyExpired(AlreadyExpired),
    UnauthorizedCaller(UnauthorizedCaller),
    InsufficientBacking(InsufficientBacking),
    ZeroAmount(ZeroAmount),
}

// TODO: Add tests once vault implementation is complete
