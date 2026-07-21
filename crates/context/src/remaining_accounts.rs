use {
    crate::HandlerContext, solana_account_view::AccountView, solana_address::Address,
    typhoon_errors::Error,
};

pub struct Remaining<'a>(pub &'a mut [AccountView]);

impl<'b> HandlerContext<'_, 'b, '_> for Remaining<'b> {
    #[inline(always)]
    fn from_entrypoint(
        _program_id: &Address,
        accounts: &'b mut [AccountView],
        _instruction_data: &mut &[u8],
    ) -> Result<(Self, &'b mut [AccountView]), Error> {
        Ok((Remaining(accounts), &mut []))
    }
}
