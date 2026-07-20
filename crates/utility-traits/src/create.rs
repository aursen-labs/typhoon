use {
    pinocchio::{cpi, sysvars::rent::Rent, AccountView, Address},
    typhoon_accounts::{Account, Mut, Signer, SignerCheck, WritableAccount},
    typhoon_errors::Error,
    typhoon_traits::Discriminator,
    typhoon_utility::create_account_with_minimum_balance_signed,
};

/// CPI helper used by macro-generated `init` blocks.
///
/// The `Self` type is the raw mutable view of an account slot before it has
/// been initialised. `create` calls the system program to allocate space and
/// assign ownership, then stamps the discriminator and hands back a `Mut<T>`.
pub trait CreateAccountCpi<'a, T>
where
    Self: Sized + Into<&'a mut AccountView>,
{
    type D: Discriminator;

    #[inline(always)]
    fn create(
        self,
        rent: &Rent,
        payer: &impl WritableAccount,
        owner: &Address,
        space: usize,
        seeds: Option<&[cpi::Signer]>,
    ) -> Result<Mut<'a, T>, Error> {
        let info: &'a mut AccountView = self.into();
        create_account_with_minimum_balance_signed(
            info,
            space,
            owner,
            payer.as_ref(),
            rent,
            seeds.unwrap_or_default(),
        )?;

        // Stamp the discriminator at the start of the freshly allocated data.
        unsafe {
            core::ptr::copy_nonoverlapping(
                Self::D::DISCRIMINATOR.as_ptr(),
                info.data_mut_ptr(),
                Self::D::DISCRIMINATOR.len(),
            );
        }

        Ok(Mut::from_raw_info(info))
    }
}

impl<'a, T, C> CreateAccountCpi<'a, Signer<'a, Account<'a, T>, C>> for &'a mut AccountView
where
    T: Discriminator,
    C: SignerCheck,
{
    type D = T;
}

impl<'a, T> CreateAccountCpi<'a, Account<'a, T>> for &'a mut AccountView
where
    T: Discriminator,
{
    type D = T;
}
