use {
    proc_macro2::TokenStream,
    quote::{format_ident, quote},
    syn::{parse_quote, punctuated::Punctuated, Expr, Ident, Token},
    typhoon_syn::{
        constraints::{ConstraintAddress, ConstraintAssert},
        error,
        utils::{ContextExpr, SeedsExpr},
        InstructionAccount,
    },
};

pub enum AccountType {
    TokenAccount {
        is_ata: bool,
        mint: Option<Ident>,
        owner: Option<Expr>,
    },
    Mint {
        decimals: Option<Expr>,
        authority: Option<Expr>,
        freeze_authority: Box<Option<Expr>>,
    },
    Other {
        space: Option<Expr>,
        targets: Vec<(Ident, Option<Expr>)>,
    },
}

#[derive(Default)]
pub struct InitContext {
    pub is_init_if_needed: bool,
    pub payer: Option<Ident>,
}

#[derive(Default)]
pub struct PdaContext {
    pub keys: Option<SeedsExpr>,
    pub bump: Option<ContextExpr>,
    pub is_seeded: bool,
    pub program_id: Option<Expr>,
}

/// Controls how `get_pda` derives the PDA address and bump.
enum PdaMode {
    /// Derive both address and bump. Uses `create_program_address` when bump is known.
    DeriveAddress,
    /// Derive only the bump. Uses `create_program_address` when bump is known.
    DeriveBump,
    /// Always use `find_program_address`, deriving only the bump.
    FindBump,
}

struct AccountIdents {
    key: Ident,
    bump: Ident,
    state: Ident,
}

impl AccountIdents {
    fn new(name: &Ident) -> Self {
        Self {
            key: format_ident!("{}_key", name),
            bump: format_ident!("{}_bump", name),
            state: format_ident!("{}_state", name),
        }
    }
}

pub struct AccountGenerator<'a> {
    pub account: &'a InstructionAccount,
    pub account_ty: AccountType,
    pub init: Option<InitContext>,
    pub pda: Option<PdaContext>,
    pub init_state: bool,
    pub asserts: Vec<ConstraintAssert>,
    pub address_checks: Vec<ConstraintAddress>,
}

impl<'a> AccountGenerator<'a> {
    pub fn new(account: &'a InstructionAccount, account_ty: AccountType) -> Self {
        Self {
            account,
            account_ty,
            init: None,
            pda: Default::default(),
            init_state: false,
            asserts: Vec::new(),
            address_checks: Vec::new(),
        }
    }
}

/// Generates temporary `let` bindings for each seed expression to extend
/// the lifetime of any temporaries (e.g. `&args.amount.to_le_bytes()`).
fn seed_temp_bindings(
    name: &Ident,
    punctuated: &Punctuated<Expr, Token![,]>,
) -> (TokenStream, Vec<Ident>) {
    let mut bindings = TokenStream::new();
    let mut temps = Vec::new();
    for (i, expr) in punctuated.iter().enumerate() {
        let temp = format_ident!("__seed_{}_{}", name, i);
        bindings.extend(quote! { let #temp = #expr; });
        temps.push(temp);
    }
    (bindings, temps)
}

/// Generates a guard that returns an error when an address comparison fails.
fn gen_address_guard(lhs: TokenStream, rhs: TokenStream, err: TokenStream) -> TokenStream {
    quote! {
        if hint::unlikely(!address::address_eq(#lhs, #rhs)) {
            return Err(#err);
        }
    }
}

impl AccountGenerator<'_> {
    pub fn needs_programs(&self) -> Vec<String> {
        let mut programs = Vec::with_capacity(3);
        if self.init.is_some() {
            programs.push("System".to_string());
            match self.account_ty {
                AccountType::TokenAccount { is_ata, .. } => {
                    programs.push("TokenProgram".to_string());

                    if is_ata {
                        programs.push("AtaTokenProgram".to_string());
                    }
                }
                AccountType::Mint { .. } => programs.push("TokenProgram".to_string()),
                _ => (),
            }
        }
        programs
    }

    pub fn is_init_signer(&self) -> bool {
        match self.account_ty {
            AccountType::TokenAccount { is_ata: true, .. } => true,
            _ => self.pda.is_some() || self.account.meta.is_signer,
        }
    }

    fn needs_return_bump(&self) -> bool {
        self.pda.as_ref().is_some_and(|pda| {
            pda.bump.is_none() || self.init.as_ref().is_some_and(|i| i.is_init_if_needed)
        })
    }

    fn get_pda(&self, ctx: &PdaContext, mode: PdaMode) -> Result<TokenStream, syn::Error> {
        let idents = AccountIdents::new(&self.account.name);
        let program_id = ctx
            .program_id
            .as_ref()
            .map(|p| quote!(#p))
            .unwrap_or(quote!(program_id));

        let use_create = !matches!(mode, PdaMode::FindBump) && ctx.bump.is_some();
        let define_key = matches!(mode, PdaMode::DeriveAddress);

        if use_create {
            let Some(bump) = ctx.bump.as_ref() else {
                error!(&self.account.name, "No bump specified for the current PDA.");
            };
            let pda_bump = &idents.bump;
            let pda_key = &idents.key;

            let seeds_token = if ctx.is_seeded {
                let state = &idents.state;
                quote!(#state.seeds().seeds_with_bump(&[#pda_bump]))
            } else {
                let Some(ref seed_keys) = ctx.keys else {
                    error!(
                        &self.account.name,
                        "No seeds specified for the current PDA."
                    );
                };

                match seed_keys {
                    SeedsExpr::Punctuated(punctuated) => quote!([#punctuated, &[#pda_bump]]),
                    SeedsExpr::Single(expr) => quote!(#expr(&[#pda_bump])),
                }
            };

            let create_pda = if define_key {
                quote!(let #pda_key = Address::create_program_address(&#seeds_token, &#program_id)?;)
            } else {
                quote!(Address::create_program_address(&#seeds_token, &#program_id)?;)
            };
            Ok(quote! {
                let #pda_bump = #bump;
                #create_pda
            })
        } else {
            let Some(ref seed_keys) = ctx.keys else {
                error!(
                    &self.account.name,
                    "No seeds specified for the current PDA."
                );
            };

            let seeds_token = if ctx.is_seeded {
                let inner_ty = &self.account.inner_ty;
                quote!(#inner_ty::derive(#seed_keys).as_seeds())
            } else {
                match seed_keys {
                    SeedsExpr::Punctuated(punctuated) => quote!([#punctuated]),
                    SeedsExpr::Single(expr) => quote!(#expr),
                }
            };

            let pda_key = &idents.key;
            let pda_bump = &idents.bump;
            let key_token = if define_key {
                quote! {
                    let (#pda_key, #pda_bump) = Address::find_program_address(&#seeds_token, &#program_id);
                }
            } else {
                quote! {
                    let (_, #pda_bump) = Address::find_program_address(&#seeds_token, &#program_id);
                }
            };
            Ok(key_token)
        }
    }

    fn get_signer_init(&self, ctx: &PdaContext) -> Result<TokenStream, syn::Error> {
        let idents = AccountIdents::new(&self.account.name);
        let Some(ref punctuated_keys) = ctx.keys else {
            error!(&self.account.name, "The seeds cannot be empty.");
        };

        let seeds = if ctx.is_seeded {
            let account_ty = &self.account.inner_ty;
            let seed_bytes_var = format_ident!("__{}_seed_bytes", self.account.name);
            quote! {
                let #seed_bytes_var = #account_ty::derive(#punctuated_keys);
                let seeds = #seed_bytes_var.signer_seeds_with_bump(&bump);
                let signer = CpiSigner::from(&seeds);
            }
        } else {
            match punctuated_keys {
                SeedsExpr::Punctuated(punctuated) => {
                    let (bindings, temps) = seed_temp_bindings(&self.account.name, punctuated);
                    quote! {
                        #bindings
                        let seeds = seeds!(#(#temps),*, &bump);
                        let signer = CpiSigner::from(&seeds);
                    }
                }
                SeedsExpr::Single(expr) => {
                    // SAFETY: `buffer` elements `[0..expr_len]` are initialised by the
                    // `for` loop, and element `[expr_len]` is written right after. The
                    // resulting `from_raw_parts` slice covers exactly `expr_len + 1`
                    // fully-initialised `Seed` values.
                    quote! {
                        let expr = #expr;
                        let expr_len = expr.len();
                        let mut buffer = [bytes::UNINIT_SEED; MAX_SEEDS];
                        for (uninit_byte, &src_byte) in buffer[..expr_len].iter_mut().zip(&expr) {
                            uninit_byte.write(Seed::from(src_byte));
                        }
                        buffer[expr_len].write(Seed::from(&bump));

                        let signer = CpiSigner::from(unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const Seed, expr_len + 1) });
                    }
                }
            }
        };

        let pda_bump = &idents.bump;
        Ok(quote! {
            // TODO: avoid reusing seeds here and in verifications
            let bump = [#pda_bump];
            #seeds
        })
    }

    fn get_init_token(
        &self,
        ctx: &InitContext,
        signers: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let name = &self.account.name;
        if !self.account.meta.is_mutable || !self.is_init_signer() {
            error!(name, "The account needs to be mutable and signer");
        }
        let Some(ref payer) = ctx.payer else {
            error!(
                name,
                "A payer needs to be specified for `init` or init_if_needed` constraint."
            );
        };

        let init_token = match &self.account_ty {
            AccountType::TokenAccount {
                is_ata,
                mint,
                owner,
            } => {
                let Some(owner) = owner else {
                    error!(name, "An `owner` needs to be specified for the `init` or `init_if_needed` constraint.");
                };
                let Some(mint) = mint else {
                    error!(name, "A `mint` needs to be specified for the `init` or `init_if_needed` constraint.");
                };

                if *is_ata {
                    quote!(SplCreateToken::create_associated_token_account(#name, &#payer, &#mint, &#owner, &system_program, &token_program)?)
                } else {
                    quote!(SplCreateToken::create_token_account(#name, &rent, &#payer, &#mint, &#owner, #signers)?)
                }
            }
            AccountType::Mint {
                decimals,
                authority,
                freeze_authority,
            } => {
                let default_decimals = parse_quote!(9);
                let decimals = decimals.as_ref().unwrap_or(&default_decimals);
                let Some(authority) = authority else {
                    error!(name, "An `authority` needs to be specified for the `init` or `init_if_needed` constraint.");
                };
                let f_auth_token = if let Some(auth) = freeze_authority.as_ref() {
                    quote!(Some(#auth))
                } else {
                    quote!(None)
                };
                quote!(SplCreateMint::create_mint(#name, &rent, &#payer, &#authority, #decimals, #f_auth_token, #signers)?)
            }
            AccountType::Other { space, .. } => {
                let account_ty = &self.account.inner_ty;
                let default_space = parse_quote!(#account_ty::SPACE);
                let space = space.as_ref().unwrap_or(&default_space);
                quote!(CreateAccountCpi::create(#name, &rent, &#payer, &program_id, #space, #signers)?)
            }
        };

        Ok(init_token)
    }

    fn generate_init(
        &self,
        init_ctx: &InitContext,
        return_ty: &TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let name = &self.account.name;
        let account_ty = self.account.get_ty();

        let signers = if self.pda.is_some() {
            quote!(Some(&[signer]))
        } else {
            quote!(None)
        };
        let init_token = self.get_init_token(init_ctx, signers)?;

        let init_account_token = if let Some(ref pda_ctx) = self.pda {
            let mode = if init_ctx.is_init_if_needed {
                PdaMode::FindBump
            } else {
                PdaMode::DeriveBump
            };
            let pda_token = self.get_pda(pda_ctx, mode)?;
            let seeds_token = self.get_signer_init(pda_ctx)?;
            quote! {
                #pda_token
                #seeds_token
                let #name = { #init_token };
            }
        } else {
            quote! {
                let #name: #account_ty = {
                    #init_token
                };
            }
        };

        if init_ctx.is_init_if_needed {
            let account_token = self.account_token()?;
            Ok(quote! {
                let #return_ty = if !#name.owned_by(&Address::default()) {
                    #account_token
                    #return_ty
                }else {
                    #init_account_token
                    #return_ty
                };
            })
        } else {
            Ok(init_account_token)
        }
    }

    fn verify_pda_address(&self, idents: &AccountIdents) -> Result<TokenStream, syn::Error> {
        let Some(ref pda_ctx) = self.pda else {
            return Ok(TokenStream::new());
        };

        let name = &self.account.name;
        let name_str = name.to_string();
        let pda_key = &idents.key;

        let pda = self.get_pda(pda_ctx, PdaMode::DeriveAddress)?;
        let guard = gen_address_guard(
            quote!(#name.address()),
            quote!(&#pda_key),
            quote!(Error::new(ProgramError::InvalidSeeds).with_account(#name_str)),
        );

        Ok(quote! { #pda #guard })
    }

    fn verify_type_constraints(&self, idents: &AccountIdents) -> TokenStream {
        let name_str = self.account.name.to_string();
        let state = &idents.state;

        match self.account_ty {
            AccountType::TokenAccount {
                ref mint,
                ref owner,
                ..
            } => {
                let mut token = TokenStream::new();
                if let Some(mint) = mint {
                    token.extend(gen_address_guard(
                        quote!(#state.mint()),
                        quote!(#mint.address()),
                        quote!(ErrorCode::TokenConstraintViolated.into()),
                    ));
                }

                if let Some(owner) = owner {
                    token.extend(gen_address_guard(
                        quote!(#state.owner()),
                        quote!(#owner.address()),
                        quote!(ErrorCode::TokenConstraintViolated.into()),
                    ));
                }
                token
            }
            AccountType::Mint { .. } => TokenStream::new(),
            AccountType::Other { ref targets, .. } => {
                let basic_error: Expr = parse_quote!(ErrorCode::HasOneConstraint);
                targets
                    .iter()
                    .map(|(target, error)| {
                        let error = error.as_ref().unwrap_or(&basic_error);
                        gen_address_guard(
                            quote!(&#state.#target),
                            quote!(#target.address()),
                            quote!(Error::from(#error).with_account(#name_str)),
                        )
                    })
                    .collect()
            }
        }
    }

    fn verify_assertions(&self) -> TokenStream {
        self.asserts
            .iter()
            .map(|ConstraintAssert { assert, error }| {
                let basic_error: Expr = parse_quote!(ErrorCode::AssertConstraint);
                let error = error.as_ref().unwrap_or(&basic_error);
                quote! {
                    if hint::unlikely(!(#assert)) {
                        return Err(#error.into());
                    }
                }
            })
            .collect()
    }

    pub fn account_token(&self) -> Result<TokenStream, syn::Error> {
        let name = &self.account.name;
        let name_str = name.to_string();
        let account_ty = self.account.get_ty();
        let idents = AccountIdents::new(name);

        let mut token = if self.account.meta.is_mutable {
            quote!(let #name = <#account_ty>::try_from(#name).trace_account(#name_str)?;)
        } else {
            quote!(let #name = <#account_ty>::try_from(&*#name).trace_account(#name_str)?;)
        };

        if self.init_state {
            let state = &idents.state;
            token.extend(quote!(let #state = #name.data_unchecked()?;));
        }

        token.extend(self.verify_pda_address(&idents)?);
        token.extend(self.verify_type_constraints(&idents));
        token.extend(self.verify_assertions());

        Ok(token)
    }

    pub fn generate(self) -> Result<TokenStream, syn::Error> {
        let name = &self.account.name;
        let idents = AccountIdents::new(name);
        let pda_bump = &idents.bump;

        let return_ty = if self.needs_return_bump() {
            quote!((#name, #pda_bump))
        } else {
            quote!(#name)
        };

        let account_checks_token = if let Some(ref init_ctx) = self.init {
            self.generate_init(init_ctx, &return_ty)?
        } else {
            self.account_token()?
        };

        let mut token = TokenStream::new();

        if self.account.meta.is_optional {
            token.extend(quote! {
                let #return_ty = if #name.address() == program_id {
                    None
                } else {
                    #account_checks_token
                    Some(#return_ty)
                };
            });
        } else {
            token.extend(account_checks_token);
        };

        for ConstraintAddress { check, error } in &self.address_checks {
            let basic_error: Expr = parse_quote!(ErrorCode::AddressConstraint);
            let error = error.as_ref().unwrap_or(&basic_error);

            token.extend(gen_address_guard(
                quote!(#name.address()),
                quote!(#check),
                quote!(#error.into()),
            ));
        }

        Ok(token)
    }
}
