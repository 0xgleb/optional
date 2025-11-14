extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct TestERC20 {
        mapping(address => uint256) balances;
        mapping(address => mapping(address => uint256)) allowances;
    }
}

#[public]
impl TestERC20 {
    #[must_use]
    pub fn balance_of(&self, account: Address) -> U256 {
        self.balances.get(account)
    }

    pub fn transfer(&mut self, to: Address, amount: U256) -> bool {
        let from = self.vm().msg_sender();
        let sender_balance = self.balances.get(from);

        if sender_balance < amount {
            return false;
        }

        self.balances.insert(from, sender_balance - amount);
        let recipient_balance = self.balances.get(to);
        self.balances.insert(to, recipient_balance + amount);

        true
    }

    pub fn transfer_from(&mut self, from: Address, to: Address, amount: U256) -> bool {
        let spender = self.vm().msg_sender();
        let allowance = self.allowances.getter(from).get(spender);

        if allowance < amount {
            return false;
        }

        let sender_balance = self.balances.get(from);
        if sender_balance < amount {
            return false;
        }

        self.balances.insert(from, sender_balance - amount);
        let recipient_balance = self.balances.get(to);
        self.balances.insert(to, recipient_balance + amount);

        let mut allowance_setter = self.allowances.setter(from);
        allowance_setter.insert(spender, allowance - amount);

        true
    }

    pub fn approve(&mut self, spender: Address, amount: U256) {
        let owner = self.vm().msg_sender();
        let mut allowance_setter = self.allowances.setter(owner);
        allowance_setter.insert(spender, amount);
    }

    pub fn mint(&mut self, to: Address, amount: U256) {
        let current_balance = self.balances.get(to);
        self.balances.insert(to, current_balance + amount);
    }
}
