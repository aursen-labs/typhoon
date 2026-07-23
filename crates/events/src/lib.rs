#![no_std]

/// Types that can be emitted to the transaction logs.
///
/// Implemented automatically by the `#[event]` attribute macro for enums whose
/// variants are the events. Emit one by calling `.emit()` on a value:
///
/// ```ignore
/// MyEvent::Transferred { amount: 42 }.emit();
/// ```
pub trait Emit {
    /// Serialize `self` and write it to the transaction logs.
    fn emit(&self);
}

/// Write `data` to the transaction logs as a single `sol_log_data` field.
///
/// `#[event]` builds the payload as the one-byte variant discriminator (the
/// index of the variant) followed by the packed variant fields, producing a
/// `Program data: <base64>` log line that indexers can decode against the
/// program IDL.
#[inline(always)]
pub fn emit_bytes(data: &[u8]) {
    let fields: [&[u8]; 1] = [data];

    #[cfg(target_os = "solana")]
    unsafe {
        pinocchio::syscalls::sol_log_data(fields.as_ptr() as *const u8, fields.len() as u64);
    }

    #[cfg(not(target_os = "solana"))]
    {
        core::hint::black_box(fields);
    }
}
