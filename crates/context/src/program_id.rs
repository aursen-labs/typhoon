use {crate::HandlerContext, solana_account_view::AccountView, solana_address::Address};

pub struct ProgramIdArg<'a>(pub &'a Address);

impl<'a, 'b> HandlerContext<'a, 'b, '_> for ProgramIdArg<'a> {
    #[inline(always)]
    fn from_entrypoint(
        program_id: &'a Address,
        accounts: &'b mut [AccountView],
        _instruction_data: &mut &[u8],
    ) -> Result<(Self, &'b mut [AccountView]), typhoon_errors::Error> {
        Ok((ProgramIdArg(program_id), accounts))
    }
}
