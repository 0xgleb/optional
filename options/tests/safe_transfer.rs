mod test_erc20;

use alloy_primitives::{Address, U256};
use motsu::prelude::*;
use options::Options;
use test_erc20::TestERC20;

#[motsu::test]
fn safe_transfer_with_normal_erc20(contract: Contract<Options>, token: Contract<TestERC20>) {
    let recipient = Address::from([0xEE; 20]);
    let amount = U256::from(1000);

    token
        .sender(contract.address())
        .mint(contract.address(), amount);

    let result = contract
        .sender(recipient)
        .safe_transfer(token.address(), recipient, amount);

    assert!(result.is_ok());
}

#[motsu::test]
fn safe_transfer_zero_amount(contract: Contract<Options>, token: Contract<TestERC20>) {
    let recipient = Address::from([0xF0; 20]);

    let result = contract
        .sender(recipient)
        .safe_transfer(token.address(), recipient, U256::ZERO);

    assert!(result.is_ok());
}
