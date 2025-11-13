#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::{vec, vec::Vec};
use alloy_primitives::{B256, U256};
use alloy_sol_types::sol;
use stylus_sdk::prelude::*;

/// Represents the side of an order in the orderbook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OrderSide {
    /// Buy order: maker wants to buy option tokens with quote tokens.
    Buy = 0,
    /// Sell order: maker wants to sell option tokens for quote tokens.
    Sell = 1,
}

impl OrderSide {
    /// Converts a u8 to OrderSide.
    ///
    /// # Errors
    /// Returns `CLOBError::Unimplemented` for now (will add proper error variant later).
    const fn from_u8(value: u8) -> Result<Self, CLOBError> {
        match value {
            0 => Ok(Self::Buy),
            1 => Ok(Self::Sell),
            _ => Err(CLOBError::Unimplemented(Unimplemented {})),
        }
    }
}

sol! {
    /// Errors that can occur in the CLOB contract.
    #[derive(Debug)]
    error Unimplemented();
}

#[derive(SolidityError, Debug)]
pub enum CLOBError {
    /// Stub implementation placeholder - function not yet implemented.
    Unimplemented(Unimplemented),
}

sol_storage! {
    #[entrypoint]
    pub struct CLOB {
        bool placeholder;
    }
}

#[public]
impl CLOB {
    /// Places a limit order in the orderbook.
    ///
    /// Locks tokens from the maker:
    /// - For sell orders: Locks ERC-1155 option tokens (requires approval)
    /// - For buy orders: Locks quote ERC20 tokens (price * quantity)
    ///
    /// Orders are added to the orderbook at the specified price level and wait for takers.
    /// Uses price-time priority: orders at the same price execute FIFO.
    ///
    /// # Parameters
    /// - `token_id`: The ERC-1155 token ID of the option (keccak256 hash)
    /// - `price`: Price per option token in quote token units (18 decimals normalized)
    /// - `quantity`: Quantity of option tokens to buy/sell
    /// - `side`: Order side (0 = Buy, 1 = Sell)
    ///
    /// # Returns
    /// Order ID that can be used to cancel the order later.
    ///
    /// # Errors
    /// Returns `CLOBError::Unimplemented` (stub implementation).
    pub fn place_order(
        &mut self,
        token_id: B256,
        price: U256,
        quantity: U256,
        side: u8,
    ) -> Result<U256, CLOBError> {
        let _side = OrderSide::from_u8(side)?;
        let _ = (token_id, price, quantity);
        Err(CLOBError::Unimplemented(Unimplemented {}))
    }

    /// Cancels an existing limit order.
    ///
    /// Removes the order from the orderbook and returns locked tokens to the maker:
    /// - For sell orders: Returns ERC-1155 option tokens
    /// - For buy orders: Returns quote ERC20 tokens
    ///
    /// Only the order maker can cancel their own orders.
    ///
    /// # Parameters
    /// - `order_id`: The order ID returned from `place_order`
    ///
    /// # Errors
    /// Returns `CLOBError::Unimplemented` (stub implementation).
    pub fn cancel_order(&mut self, order_id: U256) -> Result<(), CLOBError> {
        let _ = order_id;
        Err(CLOBError::Unimplemented(Unimplemented {}))
    }

    /// Executes a market order against the orderbook.
    ///
    /// Matches against best available prices with price-time priority:
    /// - Buy orders: Match ascending from best ask (lowest sell price)
    /// - Sell orders: Match descending from best bid (highest buy price)
    ///
    /// All-or-nothing semantics: reverts if insufficient liquidity for full quantity.
    /// No partial fills in the PoC - either the entire order executes or transaction reverts.
    ///
    /// On successful execution:
    /// - ERC-1155 option tokens transfer from seller to buyer
    /// - Quote ERC20 premium transfers from buyer to seller (at maker prices)
    /// - Maker orders are filled/reduced in FIFO order at each price level
    ///
    /// # Parameters
    /// - `token_id`: The ERC-1155 token ID of the option (keccak256 hash)
    /// - `quantity`: Quantity of option tokens to buy/sell
    /// - `side`: Order side (0 = Buy to take liquidity from asks, 1 = Sell to take from bids)
    ///
    /// # Errors
    /// Returns `CLOBError::Unimplemented` (stub implementation).
    pub fn market_order(
        &mut self,
        token_id: B256,
        quantity: U256,
        side: u8,
    ) -> Result<(), CLOBError> {
        let _side = OrderSide::from_u8(side)?;
        let _ = (token_id, quantity);
        Err(CLOBError::Unimplemented(Unimplemented {}))
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Address;
    use motsu::prelude::*;

    use super::*;

    #[motsu::test]
    fn test_place_order_buy_returns_unimplemented(contract: Contract<CLOB>, alice: Address) {
        let result = contract.sender(alice).place_order(
            B256::ZERO,
            U256::from(1000),
            U256::from(100),
            0, // OrderSide::Buy
        );

        assert!(matches!(result, Err(CLOBError::Unimplemented(_))));
    }

    #[motsu::test]
    fn test_place_order_sell_returns_unimplemented(contract: Contract<CLOB>, alice: Address) {
        let result = contract.sender(alice).place_order(
            B256::ZERO,
            U256::from(1000),
            U256::from(100),
            1, // OrderSide::Sell
        );

        assert!(matches!(result, Err(CLOBError::Unimplemented(_))));
    }

    #[motsu::test]
    fn test_cancel_order_returns_unimplemented(contract: Contract<CLOB>, alice: Address) {
        let result = contract.sender(alice).cancel_order(U256::from(1));

        assert!(matches!(result, Err(CLOBError::Unimplemented(_))));
    }

    #[motsu::test]
    fn test_market_order_buy_returns_unimplemented(contract: Contract<CLOB>, alice: Address) {
        let result = contract
            .sender(alice)
            .market_order(B256::ZERO, U256::from(100), 0); // OrderSide::Buy

        assert!(matches!(result, Err(CLOBError::Unimplemented(_))));
    }

    #[motsu::test]
    fn test_market_order_sell_returns_unimplemented(contract: Contract<CLOB>, alice: Address) {
        let result = contract
            .sender(alice)
            .market_order(B256::ZERO, U256::from(100), 1); // OrderSide::Sell

        assert!(matches!(result, Err(CLOBError::Unimplemented(_))));
    }
}

#[cfg(test)]
mod proptests {
    use alloy_primitives::Address;
    use motsu::prelude::*;
    use proptest::prelude::*;

    use super::*;

    // Property-based tests for CLOB stub behavior
    //
    // These tests verify that all public functions return Unimplemented errors
    // for arbitrary inputs. Once implementation is added, these tests will be
    // updated to verify the actual orderbook invariants.

    proptest! {
        /// Property: place_order returns Unimplemented for all inputs
        #[test]
        fn prop_place_order_returns_unimplemented(
            price in 1u64..1_000_000u64,
            quantity in 1u64..1_000_000u64,
            side in 0u8..2u8,
        ) {
            let contract = Contract::<CLOB>::default();
            let alice = Address::repeat_byte(0x01);

            let result = contract.sender(alice).place_order(
                B256::ZERO,
                U256::from(price),
                U256::from(quantity),
                side,
            );

            prop_assert!(matches!(result, Err(CLOBError::Unimplemented(_))));
        }

        /// Property: cancel_order returns Unimplemented for all order IDs
        #[test]
        fn prop_cancel_order_returns_unimplemented(
            order_id in 0u64..1_000_000u64,
        ) {
            let contract = Contract::<CLOB>::default();
            let alice = Address::repeat_byte(0x01);

            let result = contract.sender(alice).cancel_order(U256::from(order_id));

            prop_assert!(matches!(result, Err(CLOBError::Unimplemented(_))));
        }

        /// Property: market_order returns Unimplemented for all inputs
        #[test]
        fn prop_market_order_returns_unimplemented(
            quantity in 1u64..1_000_000u64,
            side in 0u8..2u8,
        ) {
            let contract = Contract::<CLOB>::default();
            let alice = Address::repeat_byte(0x01);

            let result = contract.sender(alice).market_order(
                B256::ZERO,
                U256::from(quantity),
                side,
            );

            prop_assert!(matches!(result, Err(CLOBError::Unimplemented(_))));
        }
    }
}
