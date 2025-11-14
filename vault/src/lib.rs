#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, B256, U256};
use alloy_sol_types::sol;
use openzeppelin_stylus::token::erc20::extensions::Erc20Metadata;
use openzeppelin_stylus::token::erc20::Erc20;
use stylus_sdk::prelude::*;
use stylus_sdk::storage::{StorageAddress, StorageMap, StorageU256, StorageU8, StorageVec};

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
        // ERC-4626 base implementation
        #[borrow]
        Erc20 erc20;

        #[borrow]
        Erc20Metadata metadata;

        // Asset token address (underlying for calls, quote for puts)
        StorageAddress asset;

        // Hardcoded decimals offset for inflation attack protection
        // Value: 3 (provides 1000x security multiplier)
        StorageU8 decimals_offset;

        // Option series this vault backs
        StorageAddress options_contract;
        B256 token_id;
        StorageU256 expiry;

        // Backing constraints
        StorageU256 options_outstanding;
        bool expired;

        // FIFO deposit tracking for assignment
        StorageMap<U256, DepositCheckpoint> checkpoints;
        StorageMap<Address, StorageVec<U256>> writer_checkpoints;
        StorageU256 checkpoint_count;
        StorageU256 total_exercised;

        // Total assets held by vault (for ERC-4626 compliance)
        StorageU256 total_assets;
    }
}

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
    fn decimals_offset(&self) -> u8 {
        self.decimals_offset.get()
    }
}

#[public]
impl OptionVault {
    /// Initializes the vault with the asset token and hardcoded inflation protection.
    ///
    /// # Arguments
    /// * `asset` - The ERC20 token used as collateral
    /// * `options_contract` - The OptionToken contract address
    /// * `token_id` - The option series token ID
    /// * `expiry` - The option expiry timestamp
    ///
    /// # Security
    /// Hardcodes `decimals_offset=3` for ERC-4626 inflation attack protection.
    /// This provides a 1000x security multiplier without requiring pricing oracles.
    pub fn initialize(
        &mut self,
        asset: Address,
        options_contract: Address,
        token_id: B256,
        expiry: U256,
    ) -> Result<(), Vec<u8>> {
        // Store asset
        self.asset.set(asset);

        // Hardcode decimals_offset=3 for uniform inflation protection
        // NOT a parameter - prevents bypass attacks
        self.decimals_offset.set(3);

        // Store option series info
        self.options_contract.set(options_contract);
        self.token_id = token_id;
        self.expiry.set(expiry);

        // Initialize state
        self.options_outstanding.set(U256::ZERO);
        self.expired = false;
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
    pub fn deposit(&mut self, assets: U256, receiver: Address) -> Result<U256, Vec<u8>> {
        Err(VaultError::Unimplemented(Unimplemented {}).into())
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
    pub fn exercise_withdraw(&mut self, assets: U256, recipient: Address) -> Result<U256, Vec<u8>> {
        Err(VaultError::Unimplemented(Unimplemented {}).into())
    }

    /// Claims strike payments (if assigned) or collateral (if unassigned) after expiry.
    /// Uses FIFO assignment based on deposit order.
    ///
    /// # Returns
    /// Tuple of (strike_payment, collateral_returned)
    ///
    /// # Errors
    /// - `NotExpired` if called before expiry
    pub fn claim(&mut self) -> Result<(U256, U256), Vec<u8>> {
        Err(VaultError::Unimplemented(Unimplemented {}).into())
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
        shares: U256,
        account: Address,
    ) -> Result<U256, Vec<u8>> {
        Err(VaultError::Unimplemented(Unimplemented {}).into())
    }

    /// Marks the vault as expired. Can be called by anyone after expiry time.
    ///
    /// # Errors
    /// - `NotExpired` if current time is before expiry
    pub fn mark_expired(&mut self) -> Result<(), Vec<u8>> {
        Err(VaultError::Unimplemented(Unimplemented {}).into())
    }

    // ========================================
    // View Functions
    // ========================================

    /// Returns the total number of checkpoints created.
    pub fn get_checkpoint_count(&self) -> U256 {
        self.checkpoint_count.get()
    }

    /// Returns the total amount of options exercised.
    pub fn get_total_exercised(&self) -> U256 {
        self.total_exercised.get()
    }

    /// Returns the total amount of options outstanding.
    pub fn get_options_outstanding(&self) -> U256 {
        self.options_outstanding.get()
    }

    /// Returns whether the vault has been marked as expired.
    pub fn is_expired(&self) -> bool {
        self.expired
    }

    /// Returns the expiry timestamp for this vault.
    pub fn get_expiry(&self) -> U256 {
        self.expiry.get()
    }

    /// Returns the checkpoint at the given index.
    ///
    /// # Arguments
    /// * `index` - Checkpoint index
    ///
    /// # Returns
    /// Checkpoint data (writer, amount, cumulative_total)
    pub fn get_checkpoint(&self, index: U256) -> DepositCheckpoint {
        self.checkpoints.get(index)
    }

    /// Returns the list of checkpoint indices for a writer.
    ///
    /// # Arguments
    /// * `writer` - Writer address
    ///
    /// # Returns
    /// Array of checkpoint indices
    pub fn get_writer_checkpoints(&self, writer: Address) -> Vec<U256> {
        let checkpoints = self.writer_checkpoints.get(writer);
        let len = checkpoints.len();
        let mut result = Vec::new();

        for i in 0..len {
            result.push(checkpoints.get(i).unwrap_or(U256::ZERO));
        }

        result
    }
}

/// Custom error type combining vault errors.
#[derive(Debug)]
pub enum VaultError {
    Unimplemented(Unimplemented),
    NotExpired(NotExpired),
    AlreadyExpired(AlreadyExpired),
    UnauthorizedCaller(UnauthorizedCaller),
    InsufficientBacking(InsufficientBacking),
    ZeroAmount(ZeroAmount),
}

impl From<VaultError> for Vec<u8> {
    fn from(err: VaultError) -> Vec<u8> {
        match err {
            VaultError::Unimplemented(e) => e.encode(),
            VaultError::NotExpired(e) => e.encode(),
            VaultError::AlreadyExpired(e) => e.encode(),
            VaultError::UnauthorizedCaller(e) => e.encode(),
            VaultError::InsufficientBacking(e) => e.encode(),
            VaultError::ZeroAmount(e) => e.encode(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motsu::prelude::*;

    #[motsu::test]
    fn initialize_sets_decimals_offset_to_three(contract: OptionVault) {
        let asset = Address::ZERO;
        let options_contract = Address::ZERO;
        let token_id = B256::ZERO;
        let expiry = U256::from(1000);

        let result = contract.initialize(asset, options_contract, token_id, expiry);
        assert!(result.is_ok());
        assert_eq!(contract.decimals_offset(), 3);
    }

    #[motsu::test]
    fn deposit_returns_unimplemented(contract: OptionVault) {
        let result = contract.deposit(U256::from(100), Address::ZERO);
        assert!(result.is_err());
    }

    #[motsu::test]
    fn exercise_withdraw_returns_unimplemented(contract: OptionVault) {
        let result = contract.exercise_withdraw(U256::from(100), Address::ZERO);
        assert!(result.is_err());
    }

    #[motsu::test]
    fn claim_returns_unimplemented(contract: OptionVault) {
        let result = contract.claim();
        assert!(result.is_err());
    }

    #[motsu::test]
    fn burn_shares_with_options_returns_unimplemented(contract: OptionVault) {
        let result = contract.burn_shares_with_options(U256::from(100), Address::ZERO);
        assert!(result.is_err());
    }

    #[motsu::test]
    fn mark_expired_returns_unimplemented(contract: OptionVault) {
        let result = contract.mark_expired();
        assert!(result.is_err());
    }
}
