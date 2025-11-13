#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

#[cfg(not(any(test, feature = "export-abi")))]
#[no_mangle]
pub const extern "C" fn main() {}

#[cfg(feature = "export-abi")]
fn main() {
    options::print_from_args();
}
