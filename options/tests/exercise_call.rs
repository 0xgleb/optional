mod test_erc20;

use alloy_primitives::{Address, B256, U256};
use motsu::prelude::*;
use options::{Options, Token};
use test_erc20::TestERC20;

#[motsu::test]
fn writer_exercises_own_options_successfully(
    contract: Contract<Options>,
    underlying_token: Contract<TestERC20>,
) {
    let writer = Address::from([0xAA; 20]);
    let options_addr = contract.address();

    let write_quantity = U256::from(100_000_000);
    underlying_token.sender(writer).mint(writer, write_quantity);
    underlying_token
        .sender(writer)
        .approve(options_addr, write_quantity);

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

    let token_id = contract
        .sender(writer)
        .write_call_option(strike, expiry, write_quantity, underlying, quote)
        .unwrap();

    let exercise_quantity = U256::from(50_000_000);
    let result = contract
        .sender(writer)
        .exercise_call(token_id, exercise_quantity);

    assert!(result.is_ok());
}

#[motsu::test]
fn option_tokens_burned_correctly(
    contract: Contract<Options>,
    underlying_token: Contract<TestERC20>,
) {
    let writer = Address::from([0xBB; 20]);
    let options_addr = contract.address();

    let write_quantity = U256::from(100_000_000);
    underlying_token.sender(writer).mint(writer, write_quantity);
    underlying_token
        .sender(writer)
        .approve(options_addr, write_quantity);

    let underlying = Token {
        address: underlying_token.address(),
        decimals: 8,
    };
    let quote = Token {
        address: Address::from([0x33; 20]),
        decimals: 6,
    };
    let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
    let expiry = 2_000_000_000u64;

    let token_id = contract
        .sender(writer)
        .write_call_option(strike, expiry, write_quantity, underlying, quote)
        .unwrap();

    let normalized_quantity = write_quantity * U256::from(10).pow(U256::from(10));
    let balance_before = contract.sender(writer).balance_of(writer, token_id);

    let exercise_quantity = U256::from(30_000_000) * U256::from(10).pow(U256::from(10));
    contract
        .sender(writer)
        .exercise_call(token_id, exercise_quantity)
        .unwrap();

    let balance_after = contract.sender(writer).balance_of(writer, token_id);

    assert_eq!(balance_before, normalized_quantity);
    assert_eq!(balance_after, normalized_quantity - exercise_quantity);
}

#[motsu::test]
fn position_reduced_correctly(contract: Contract<Options>, underlying_token: Contract<TestERC20>) {
    let writer = Address::from([0xCC; 20]);
    let options_addr = contract.address();

    let write_quantity = U256::from(100_000_000);
    underlying_token.sender(writer).mint(writer, write_quantity);
    underlying_token
        .sender(writer)
        .approve(options_addr, write_quantity);

    let underlying = Token {
        address: underlying_token.address(),
        decimals: 8,
    };
    let quote = Token {
        address: Address::from([0x44; 20]),
        decimals: 6,
    };
    let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
    let expiry = 2_000_000_000u64;

    let token_id = contract
        .sender(writer)
        .write_call_option(strike, expiry, write_quantity, underlying, quote)
        .unwrap();

    let normalized_quantity = write_quantity * U256::from(10).pow(U256::from(10));
    let (quantity_before, collateral_before) =
        contract.sender(writer).get_position(writer, token_id);

    let exercise_quantity = U256::from(40_000_000) * U256::from(10).pow(U256::from(10));
    contract
        .sender(writer)
        .exercise_call(token_id, exercise_quantity)
        .unwrap();

    let (quantity_after, collateral_after) = contract.sender(writer).get_position(writer, token_id);

    assert_eq!(quantity_before, normalized_quantity);
    assert_eq!(quantity_after, normalized_quantity - exercise_quantity);
    assert_eq!(collateral_before, normalized_quantity);
    assert_eq!(collateral_after, normalized_quantity - exercise_quantity);
}

#[motsu::test]
fn underlying_tokens_transferred(
    contract: Contract<Options>,
    underlying_token: Contract<TestERC20>,
) {
    let writer = Address::from([0xDD; 20]);
    let options_addr = contract.address();

    let write_quantity = U256::from(100_000_000);
    underlying_token.sender(writer).mint(writer, write_quantity);
    underlying_token
        .sender(writer)
        .approve(options_addr, write_quantity);

    let underlying = Token {
        address: underlying_token.address(),
        decimals: 8,
    };
    let quote = Token {
        address: Address::from([0x55; 20]),
        decimals: 6,
    };
    let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
    let expiry = 2_000_000_000u64;

    let token_id = contract
        .sender(writer)
        .write_call_option(strike, expiry, write_quantity, underlying, quote)
        .unwrap();

    let writer_balance_before = underlying_token.sender(writer).balance_of(writer);
    let contract_balance_before = underlying_token.sender(writer).balance_of(options_addr);

    let exercise_quantity_raw = U256::from(25_000_000);
    let exercise_quantity_normalized = exercise_quantity_raw * U256::from(10).pow(U256::from(10));
    contract
        .sender(writer)
        .exercise_call(token_id, exercise_quantity_normalized)
        .unwrap();

    let writer_balance_after = underlying_token.sender(writer).balance_of(writer);
    let contract_balance_after = underlying_token.sender(writer).balance_of(options_addr);

    assert_eq!(writer_balance_before, U256::ZERO);
    assert_eq!(writer_balance_after, exercise_quantity_raw);
    assert_eq!(contract_balance_before, write_quantity);
    assert_eq!(
        contract_balance_after,
        write_quantity - exercise_quantity_raw
    );
}

#[motsu::test]
fn exercise_full_position(contract: Contract<Options>, underlying_token: Contract<TestERC20>) {
    let writer = Address::from([0xEE; 20]);
    let options_addr = contract.address();

    let write_quantity = U256::from(50_000_000);
    underlying_token.sender(writer).mint(writer, write_quantity);
    underlying_token
        .sender(writer)
        .approve(options_addr, write_quantity);

    let underlying = Token {
        address: underlying_token.address(),
        decimals: 8,
    };
    let quote = Token {
        address: Address::from([0x66; 20]),
        decimals: 6,
    };
    let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
    let expiry = 2_000_000_000u64;

    let token_id = contract
        .sender(writer)
        .write_call_option(strike, expiry, write_quantity, underlying, quote)
        .unwrap();

    let normalized_quantity = write_quantity * U256::from(10).pow(U256::from(10));
    contract
        .sender(writer)
        .exercise_call(token_id, normalized_quantity)
        .unwrap();

    let balance_after = contract.sender(writer).balance_of(writer, token_id);
    let (quantity_after, collateral_after) = contract.sender(writer).get_position(writer, token_id);

    assert_eq!(balance_after, U256::ZERO);
    assert_eq!(quantity_after, U256::ZERO);
    assert_eq!(collateral_after, U256::ZERO);
}

#[motsu::test]
fn multiple_partial_exercises_deplete_balance(
    contract: Contract<Options>,
    underlying_token: Contract<TestERC20>,
) {
    let writer = Address::from([0x11; 20]);
    let options_addr = contract.address();

    let write_quantity = U256::from(100_000_000);
    underlying_token.sender(writer).mint(writer, write_quantity);
    underlying_token
        .sender(writer)
        .approve(options_addr, write_quantity);

    let underlying = Token {
        address: underlying_token.address(),
        decimals: 8,
    };
    let quote = Token {
        address: Address::from([0x77; 20]),
        decimals: 6,
    };
    let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
    let expiry = 2_000_000_000u64;

    let token_id = contract
        .sender(writer)
        .write_call_option(strike, expiry, write_quantity, underlying, quote)
        .unwrap();

    let normalized_total = write_quantity * U256::from(10).pow(U256::from(10));
    let exercise_1 = U256::from(25_000_000) * U256::from(10).pow(U256::from(10));
    let exercise_2 = U256::from(35_000_000) * U256::from(10).pow(U256::from(10));
    let exercise_3 = U256::from(40_000_000) * U256::from(10).pow(U256::from(10));

    contract
        .sender(writer)
        .exercise_call(token_id, exercise_1)
        .unwrap();
    let balance_after_1 = contract.sender(writer).balance_of(writer, token_id);
    assert_eq!(balance_after_1, normalized_total - exercise_1);

    contract
        .sender(writer)
        .exercise_call(token_id, exercise_2)
        .unwrap();
    let balance_after_2 = contract.sender(writer).balance_of(writer, token_id);
    assert_eq!(balance_after_2, normalized_total - exercise_1 - exercise_2);

    contract
        .sender(writer)
        .exercise_call(token_id, exercise_3)
        .unwrap();
    let balance_after_3 = contract.sender(writer).balance_of(writer, token_id);
    assert_eq!(balance_after_3, U256::ZERO);
}

#[motsu::test]
fn exercising_more_than_balance_fails(
    contract: Contract<Options>,
    underlying_token: Contract<TestERC20>,
) {
    let writer = Address::from([0x22; 20]);
    let options_addr = contract.address();

    let write_quantity = U256::from(100_000_000);
    underlying_token.sender(writer).mint(writer, write_quantity);
    underlying_token
        .sender(writer)
        .approve(options_addr, write_quantity);

    let underlying = Token {
        address: underlying_token.address(),
        decimals: 8,
    };
    let quote = Token {
        address: Address::from([0x88; 20]),
        decimals: 6,
    };
    let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
    let expiry = 2_000_000_000u64;

    let token_id = contract
        .sender(writer)
        .write_call_option(strike, expiry, write_quantity, underlying, quote)
        .unwrap();

    let normalized_quantity = write_quantity * U256::from(10).pow(U256::from(10));
    let excessive_quantity = normalized_quantity + U256::from(1);

    let result = contract
        .sender(writer)
        .exercise_call(token_id, excessive_quantity);

    assert!(matches!(result, Err(_)));
}

#[motsu::test]
fn write_and_exercise_near_expiry_succeeds(
    contract: Contract<Options>,
    underlying_token: Contract<TestERC20>,
) {
    let writer = Address::from([0x33; 20]);
    let options_addr = contract.address();

    let write_quantity = U256::from(100_000_000);
    underlying_token.sender(writer).mint(writer, write_quantity);
    underlying_token
        .sender(writer)
        .approve(options_addr, write_quantity);

    let underlying = Token {
        address: underlying_token.address(),
        decimals: 8,
    };
    let quote = Token {
        address: Address::from([0x99; 20]),
        decimals: 6,
    };
    let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
    let expiry = 2_000_000_000u64;

    let token_id = contract
        .sender(writer)
        .write_call_option(strike, expiry, write_quantity, underlying, quote)
        .unwrap();

    let normalized_quantity = write_quantity * U256::from(10).pow(U256::from(10));
    let result = contract
        .sender(writer)
        .exercise_call(token_id, normalized_quantity);

    assert!(result.is_ok());
}

#[motsu::test]
fn exercise_with_wrong_token_id_fails(
    contract: Contract<Options>,
    underlying_token: Contract<TestERC20>,
) {
    let writer = Address::from([0x55; 20]);
    let options_addr = contract.address();

    let write_quantity = U256::from(100_000_000);
    underlying_token.sender(writer).mint(writer, write_quantity);
    underlying_token
        .sender(writer)
        .approve(options_addr, write_quantity);

    let underlying = Token {
        address: underlying_token.address(),
        decimals: 8,
    };
    let quote = Token {
        address: Address::from([0xBB; 20]),
        decimals: 6,
    };
    let strike = U256::from(60_000) * U256::from(10).pow(U256::from(18));
    let expiry = 2_000_000_000u64;

    let _token_id = contract
        .sender(writer)
        .write_call_option(strike, expiry, write_quantity, underlying, quote)
        .unwrap();

    let wrong_token_id = B256::from([0xFF; 32]);
    let normalized_quantity = write_quantity * U256::from(10).pow(U256::from(10));

    let result = contract
        .sender(writer)
        .exercise_call(wrong_token_id, normalized_quantity);

    assert!(matches!(result, Err(_)));
}
