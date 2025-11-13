use alloc::collections::BTreeMap;
use alloy_primitives::{Address, U256};

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

        if allowance < amount {
            return false;
        }

        let sender_balance = self.balances.get(&from).copied().unwrap_or(U256::ZERO);

        if sender_balance < amount {
            return false;
        }

        self.balances.insert(from, sender_balance - amount);
        let recipient_balance = self.balances.get(&to).copied().unwrap_or(U256::ZERO);
        self.balances.insert(to, recipient_balance + amount);

        self.allowances
            .entry(from)
            .or_default()
            .insert(spender, allowance - amount);

        true
    }

    pub fn approve(&mut self, owner: Address, spender: Address, amount: U256) {
        self.allowances
            .entry(owner)
            .or_default()
            .insert(spender, amount);
    }

    #[must_use]
    pub fn balance_of(&self, account: Address) -> U256 {
        self.balances.get(&account).copied().unwrap_or(U256::ZERO)
    }

    #[must_use]
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

    #[must_use]
    pub fn balance_of(&self, account: Address) -> U256 {
        self.balances.get(&account).copied().unwrap_or(U256::ZERO)
    }
}
