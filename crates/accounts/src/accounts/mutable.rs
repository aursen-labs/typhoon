use {
    crate::{
        AccountData, ReadableAccount, Signer, SignerAccount, SignerCheck, ValidateView,
        WritableAccount,
    },
    core::marker::PhantomData,
    pinocchio::hint::unlikely,
    solana_account_view::AccountView,
    typhoon_errors::{Error, ErrorCode},
};

/// Writable wrapper around a [`solana_account_view::AccountView`].
///
/// `Mut<'a, T>` owns a mutable reborrow of an account slot. The inner `T` is
/// kept only as a phantom marker so that:
///
/// 1. `T::validate` runs the same checks a read-only `T` would have run.
/// 2. Trait selection (e.g. `AccountData::Data`, `SignerAccount`) mirrors the
///    behavior of the underlying read-only wrapper.
///
/// The actual storage is the `&'a mut AccountView`, which is required by
/// `solana-account-view` 2.0 for any mutating operation (`set_lamports`,
/// `assign`, `try_borrow_mut`, `close`, ...). Storing the inner `T` *and* a
/// `&mut` to the same slot would create aliasing references; the marker-only
/// approach sidesteps that.
pub struct Mut<'a, T> {
    pub(crate) view: &'a mut AccountView,
    _phantom: PhantomData<T>,
}

impl<'a, T> Mut<'a, T> {
    /// Build a `Mut` without running any validation. Used by initialization
    /// helpers (token, utility-traits) that have just created an account and
    /// know its state.
    #[doc(hidden)]
    #[inline(always)]
    pub fn from_raw_info(info: &'a mut AccountView) -> Self {
        Self {
            view: info,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> TryFrom<&'a mut AccountView> for Mut<'a, T>
where
    T: ValidateView,
{
    type Error = Error;

    #[inline(always)]
    fn try_from(view: &'a mut AccountView) -> Result<Self, Self::Error> {
        if unlikely(!view.is_writable()) {
            return Err(ErrorCode::AccountNotMutable.into());
        }
        T::validate(&*view)?;
        Ok(Self {
            view,
            _phantom: PhantomData,
        })
    }
}

impl<'a, T> From<Mut<'a, T>> for &'a AccountView {
    #[inline(always)]
    fn from(value: Mut<'a, T>) -> Self {
        value.view
    }
}

impl<T> AsRef<AccountView> for Mut<'_, T> {
    #[inline(always)]
    fn as_ref(&self) -> &AccountView {
        self.view
    }
}

impl<T> AsMut<AccountView> for Mut<'_, T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut AccountView {
        self.view
    }
}

impl<T> ReadableAccount for Mut<'_, T> {}
impl<T> WritableAccount for Mut<'_, T> {}

impl<T> AccountData for Mut<'_, T>
where
    T: AccountData,
{
    type Data = T::Data;
}

impl<T, C> SignerAccount for Mut<'_, Signer<'_, T, C>>
where
    T: ReadableAccount,
    C: SignerCheck,
{
}
