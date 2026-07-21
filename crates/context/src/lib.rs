#![no_std]

mod arg;
mod array;
mod entrypoint;
mod iterator;
mod program_id;
mod remaining_accounts;

use {
    crate::entrypoint::deserialize, bytemuck::NoUninit, core::mem::MaybeUninit, pastey::paste,
    solana_account_view::AccountView, solana_address::Address,
    solana_instruction_view::cpi::set_return_data, solana_program_error::ProgramError,
    typhoon_errors::Error,
};
pub use {arg::*, array::*, iterator::*, program_id::*, remaining_accounts::*};

/// Marker trait for context types. This trait is used only for identification purposes.
pub trait Context {}

pub trait HandlerContext<'a, 'b, 'c>: Sized {
    fn from_entrypoint(
        program_id: &'a Address,
        accounts: &'b mut [AccountView],
        instruction_data: &mut &'c [u8],
    ) -> Result<(Self, &'b mut [AccountView]), Error>;
}

pub trait Handler<'a, 'b, 'c, T> {
    type Output: NoUninit;

    fn call(
        self,
        program_id: &'a Address,
        accounts: &'b mut [AccountView],
        instruction_data: &mut &'c [u8],
    ) -> Result<Self::Output, Error>;
}

impl<F, O> Handler<'_, '_, '_, ()> for F
where
    F: FnOnce() -> Result<O, Error>,
    O: NoUninit,
{
    type Output = O;

    #[inline(always)]
    fn call(
        self,
        _program_id: &Address,
        _accounts: &mut [AccountView],
        _instruction_data: &mut &[u8],
    ) -> Result<Self::Output, Error> {
        (self)()
    }
}

macro_rules! impl_handler {
    ($( $t:ident ),+) => {
        impl<'a, 'b, 'c, $( $t, )* F, O> Handler<'a, 'b, 'c, ($( $t, )*)> for F
        where
            F: FnOnce($( $t ),*) -> Result<O, Error>,
            O: NoUninit,
            $(
                $t: HandlerContext<'a, 'b, 'c>,
            )*
        {
            type Output = O;

            #[inline(always)]
            fn call(
                self,
                program_id: &'a Address,
                accounts: &'b mut [AccountView],
                instruction_data: &mut &'c [u8],
            ) -> Result<Self::Output, Error> {
                paste! {
                    $(
                        let ([<$t:lower>], accounts) = $t::from_entrypoint(program_id, accounts, instruction_data)?;
                    )*
                    let _ = accounts;
                    (self)($( [<$t:lower>], )*)
                }
            }
        }
    };
}

impl_handler!(T1);
impl_handler!(T1, T2);
impl_handler!(T1, T2, T3);
impl_handler!(T1, T2, T3, T4);
impl_handler!(T1, T2, T3, T4, T5);
impl_handler!(T1, T2, T3, T4, T5, T6);
impl_handler!(T1, T2, T3, T4, T5, T6, T7);

#[inline(always)]
pub fn handle<'a, 'b, 'c, T, H>(
    program_id: &'a Address,
    accounts: &'b mut [AccountView],
    mut instruction_data: &'c [u8],
    handler: H,
) -> Result<(), Error>
where
    H: Handler<'a, 'b, 'c, T>,
{
    match handler.call(program_id, accounts, &mut instruction_data) {
        Ok(res) => {
            if core::mem::size_of::<H::Output>() > 0 {
                set_return_data(bytemuck::bytes_of(&res));
            }

            Ok(())
        }
        Err(err) => Err(err),
    }
}

/// Return code for a missing or unknown instruction discriminator.
///
/// Kept `cold` + `inline(never)` so the constant is materialized off the hot
/// dispatch path: LLVM otherwise preloads it into the return register before
/// the discriminator match, wasting an instruction on every success path.
#[cold]
#[inline(never)]
pub fn invalid_instruction_data() -> u64 {
    ProgramError::InvalidInstructionData.into()
}

#[macro_export]
macro_rules! basic_router {
    ($($dis:literal => $fn_ident: ident),+ $(,)?) => {
        |program_id: &Address, accounts: &mut [AccountView], instruction_data: &[u8]| -> u64 {
            let [discriminator, data @ ..] = instruction_data else {
                return invalid_instruction_data();
            };

            let result = match discriminator {
                $($dis => handle(program_id, accounts, data, $fn_ident),)*
                _ => return invalid_instruction_data(),
            };
            #[cfg(feature = "logging")]
            let result = result.inspect_err(|e| log_error::<LogError>(e));
            match result {
                Ok(()) => 0,
                Err(e) => e.into(),
            }
        }
    };
}

pub type EntryFn = fn(&Address, &mut [AccountView], &[u8]) -> u64;

#[macro_export]
macro_rules! entrypoint {
    ($router_fn: ident) => {
        #[doc = r" Program entrypoint."]
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8, data: *mut u8) -> u64 {
            process_program_input(input, data, $router_fn)
        }
    };
}

/// # Safety
///
/// This must only be called by the Solana runtime from the program's
/// `entrypoint`. `input` and `data` must be the raw pointers passed in by the
/// runtime, pointing to a valid serialized instruction context:
/// - `data` points to the instruction data, immediately preceded by a `u64`
///   length prefix and followed by the 32-byte program id.
/// - `input` points to the serialized account buffer in the runtime's layout.
///
/// The serialized input must contain no more than 255 accounts.
#[inline(always)]
pub unsafe fn process_program_input<F>(input: *mut u8, data: *mut u8, process_instruction: F) -> u64
where
    F: FnOnce(&Address, &mut [AccountView], &[u8]) -> u64,
{
    let instruction_data_len = unsafe { *(data.sub(size_of::<u64>()) as *const u64) as usize };
    let instruction_data = unsafe { core::slice::from_raw_parts(data, instruction_data_len) };
    let program_id: &Address = unsafe { &*(data.add(instruction_data_len) as *const Address) };
    let mut buffer = [const { MaybeUninit::uninit() }; 255];
    let count = unsafe { deserialize(input, &mut buffer) };
    let accounts = unsafe { core::slice::from_raw_parts_mut(buffer.as_mut_ptr() as _, count) };

    process_instruction(program_id, accounts, instruction_data)
}
