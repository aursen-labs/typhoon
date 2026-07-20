use {
    crate::HandlerContext,
    solana_account_view::AccountView,
    solana_address::Address,
    typhoon_errors::Error,
    typhoon_traits::{Accessor, BytemuckStrategy},
};

pub type ArgData<'a, T, S> = <S as Accessor<'a, T>>::Data;

pub struct Arg<'a, T, S = BytemuckStrategy>(pub ArgData<'a, T, S>)
where
    S: Accessor<'a, T>;

impl<'b, 'c, T, S> HandlerContext<'_, 'b, 'c> for Arg<'c, T, S>
where
    S: Accessor<'c, T>,
{
    #[inline(always)]
    fn from_entrypoint(
        _program_id: &Address,
        accounts: &'b mut [AccountView],
        instruction_data: &mut &'c [u8],
    ) -> Result<(Self, &'b mut [AccountView]), Error> {
        Ok((Self(S::read(instruction_data)?), accounts))
    }
}
