#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]
extern crate alloc;

use alloc::{format, vec, vec::Vec};

use stylus_sdk::prelude::*;

/// Represents the type of option contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionType {
    /// Call option: Right to BUY underlying at strike price.
    Call,
    /// Put option: Right to SELL underlying at strike price.
    Put,
}

/// Errors that can occur in the OptionsToken contract.
#[derive(Debug)]
pub enum OptionsError {
    /// Stub implementation placeholder - function not yet implemented.
    Unimplemented,
    // Additional error variants will be added as needed during implementation
}

impl From<OptionsError> for Vec<u8> {
    fn from(err: OptionsError) -> Self {
        format!("{err:?}").into_bytes()
    }
}

sol_storage! {
    #[entrypoint]
    pub struct OptionsToken {
        // Storage will be added in Task 2
    }
}

#[public]
impl OptionsToken {
    // Methods will be added in later tasks
}

#[cfg(test)]
mod tests {
    // Tests will be added when implementing actual logic
    // For Issue #2 (stubs), tests verify functions return Err(OptionsError::Unimplemented)
}
