mod test_erc20;

use alloy_primitives::{Address, U256};
use motsu::prelude::*;
use options::{Options, Token};
use test_erc20::TestERC20;

#[motsu::test]
fn write_call_option_happy_path(
    contract: Contract<Options>,
    underlying_token: Contract<TestERC20>,
) {
    let writer = Address::from([0xAA; 20]);
    let options_addr = contract.address();

    let mint_amount = U256::from(100_000_000);
    underlying_token.sender(writer).mint(writer, mint_amount);
    underlying_token
        .sender(writer)
        .approve(options_addr, mint_amount);

    let underlying = Token {
        address: underlying_token.address(),
        decimals: 8,
    };
    let quote = Token {
        address: Address::from([0x22; 20]),
        decimals: 6,
    };
    let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
    let expiry = 2_000_000_000u64;
    let quantity = U256::from(100_000_000);

    let token_id = contract
        .sender(writer)
        .write_call_option(strike, expiry, quantity, underlying, quote)
        .unwrap();

    assert_ne!(token_id, alloy_primitives::B256::ZERO);
}

#[motsu::test]
fn write_same_option_twice_returns_same_token_id(
    contract: Contract<Options>,
    underlying_token: Contract<TestERC20>,
) {
    let writer = Address::from([0xAA; 20]);
    let options_addr = contract.address();

    let mint_amount = U256::from(200_000_000);
    underlying_token.sender(writer).mint(writer, mint_amount);
    underlying_token
        .sender(writer)
        .approve(options_addr, mint_amount);

    let underlying = Token {
        address: underlying_token.address(),
        decimals: 8,
    };
    let quote = Token {
        address: Address::from([0x22; 20]),
        decimals: 6,
    };
    let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
    let expiry = 2_000_000_000u64;
    let quantity = U256::from(100_000_000);

    let token_id_1 = contract
        .sender(writer)
        .write_call_option(strike, expiry, quantity, underlying, quote)
        .unwrap();

    let token_id_2 = contract
        .sender(writer)
        .write_call_option(strike, expiry, quantity, underlying, quote)
        .unwrap();

    assert_eq!(token_id_1, token_id_2);
}

#[motsu::test]
fn different_options_return_different_token_ids(
    contract: Contract<Options>,
    underlying_token1: Contract<TestERC20>,
    underlying_token2: Contract<TestERC20>,
) {
    let writer = Address::from([0xAA; 20]);
    let options_addr = contract.address();

    let mint_amount = U256::from(100_000_000);
    underlying_token1.sender(writer).mint(writer, mint_amount);
    underlying_token1
        .sender(writer)
        .approve(options_addr, mint_amount);

    underlying_token2.sender(writer).mint(writer, mint_amount);
    underlying_token2
        .sender(writer)
        .approve(options_addr, mint_amount);

    let underlying1 = Token {
        address: underlying_token1.address(),
        decimals: 8,
    };
    let quote1 = Token {
        address: Address::from([0x22; 20]),
        decimals: 6,
    };
    let strike1 = U256::from(60_000) * U256::from(10).pow(U256::from(18));

    let underlying2 = Token {
        address: underlying_token2.address(),
        decimals: 18,
    };
    let quote2 = Token {
        address: Address::from([0x44; 20]),
        decimals: 6,
    };
    let strike2 = U256::from(3_000) * U256::from(10).pow(U256::from(18));

    let expiry = 2_000_000_000u64;
    let quantity = U256::from(100_000_000);

    let token_id_1 = contract
        .sender(writer)
        .write_call_option(strike1, expiry, quantity, underlying1, quote1)
        .unwrap();

    let token_id_2 = contract
        .sender(writer)
        .write_call_option(strike2, expiry, quantity, underlying2, quote2)
        .unwrap();

    assert_ne!(token_id_1, token_id_2);
}
