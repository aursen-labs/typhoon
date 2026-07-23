use {
    bytemuck::{AnyBitPattern, NoUninit},
    solana_address::Address,
    solana_instruction_view::{cpi::Seed, seeds},
    typhoon_account_macro::*,
    typhoon_traits::{BytemuckStrategy, CheckOwner, DataStrategy, Discriminator},
};

pub const ID: Address = Address::new_from_array([
    218, 7, 92, 178, 255, 94, 198, 129, 118, 19, 222, 83, 11, 105, 42, 135, 53, 71, 119, 105, 218,
    71, 67, 12, 189, 129, 84, 51, 92, 74, 131, 39,
]);

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
pub struct TestState {
    pub foo: u64,
}

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
pub struct TestAnotherState {
    #[key]
    pub foo: Address,
}

pub fn main() {}
