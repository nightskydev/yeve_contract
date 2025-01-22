use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program::program::invoke_signed;
use mpl_token_metadata::instruction::create_metadata_accounts_v3;
use mpl_token_metadata::state::{Collection, Creator, Metadata};
use spl_token::instruction::{mint_to, set_authority, AuthorityType};

declare_id!("5kZULJfKLcHw9UbnTpPxmzWfnZk5ZRgsFDUiggjqdKzY");

pub mod nft_update_auth {
    use super::*;
    declare_id!("3axbTs2z5GBy6usVbNVoqEgZMng3vZvMnAoX29BFfwhr");
}

use nft_update_auth::ID as NFT_UPDATE_AUTH;
const ADMIN_SEED: &[u8] = b"admin";
const USER_SEED: &[u8] = b"user";

#[program]
mod balance_nft {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        presale_start: i64,
        nft_name: String,
        nft_symbol: String,
        nft_uri: String,
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.total_minted = 0;
        state.presale_start = presale_start;
        state.nft_name = nft_name;
        state.nft_symbol = nft_symbol;
        state.nft_uri = nft_uri;
        state.admin = ctx.accounts.signer.key();
        Ok(())
    }

    pub fn add_to_whitelist(
        ctx: Context<AddToWhitelist>,
    ) -> Result<()> {
        let user_info = &mut ctx.accounts.user_info;
        user_info.is_whitelisted = true;
        Ok(())
    }

    pub fn update_presale_info(
        ctx: Context<UpdatePresaleInfo>,
        presale_start: i64,
        nft_name: String,
        nft_symbol: String,
        nft_uri: String,
        total_minted: u64
    ) -> Result<()> {
        require!(
            ctx.accounts.signer.key() == ctx.accounts.state.admin,
            MintError::Unauthorized
        );
        let state = &mut ctx.accounts.state;
        state.presale_start = presale_start;
        state.nft_name = nft_name;
        state.nft_symbol = nft_symbol;
        state.nft_uri = nft_uri;
        state.total_minted = total_minted;
        state.collection = ctx.accounts.collection.key();
        state.genesis_collection = ctx.accounts.genesis_collection.key();
        Ok(())
    }

    pub fn mint(ctx: Context<MintNft>) -> Result<()> {
        let (_state_authority, state_authority_bump) =
        Pubkey::find_program_address(&[ADMIN_SEED], ctx.program_id);
        let _ = mint_nft(
            &ctx.accounts.state,
            &ctx.accounts.token_mint,
            &ctx.accounts.nft_token_account,
            &ctx.accounts.token_program,
            state_authority_bump,
        );
        
        let state = &mut ctx.accounts.state;
        require!(state.total_minted < 20000, MintError::MaxSupplyReached);

        let current_time = Clock::get()?.unix_timestamp;
        let is_whitelisted = ctx.accounts.user_info.is_whitelisted;
        require!(
            (current_time > state.presale_start) && is_whitelisted || (current_time > state.presale_start + 3600), 
            MintError::Unauthorized
        );

        let metadata_mint_auth_account = &state;

        let creators = vec![
            Creator {
                address: _state_authority.key(),
                verified: false,
                share: 100,
            },
        ];

        invoke_signed(
            &create_metadata_accounts_v3(
                ctx.accounts.metadata_program.key(),
                ctx.accounts.nft_metadata_account.key(),
                ctx.accounts.token_mint.key(),
                metadata_mint_auth_account.key(),
                ctx.accounts.signer.key(),
                ctx.accounts.metadata_update_auth.key(),
                state.nft_name.to_string(),
                state.nft_symbol.to_string(),
                state.nft_uri.to_string(),
                Some(creators),
                0,
                false,
                true,
                Some(Collection{verified: false, key: state.collection}),
                None,
                None,
            ),
            &[
                ctx.accounts.nft_metadata_account.to_account_info(),
                ctx.accounts.token_mint.to_account_info(),
                metadata_mint_auth_account.to_account_info(),
                ctx.accounts.metadata_update_auth.to_account_info(),
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.metadata_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
            &[&[&ADMIN_SEED[..], [state_authority_bump].as_ref()]],
        )?;
        let _ = remove_nft_mint_authority(
            &state,
            &ctx.accounts.token_mint,
            &ctx.accounts.token_program,
            state_authority_bump,
        );

        let round_price = get_round_price(current_time, state.presale_start, state.total_minted)?;
        let mut final_price = round_price;

        let mut genesis_nft_metadata_account = &ctx.accounts.genesis_nft_metadata_account;
        let metadata: Metadata = Metadata::deserialize(&mut genesis_nft_metadata_account.data.try_borrow_mut().unwrap().as_ref())?;
        
        if let Some(collection) = metadata.collection {
            if collection.key == state.genesis_collection {
                final_price = (final_price as f64 * 0.9) as u64;
            }
        }

        let transfer_instruction = system_instruction::transfer(
            &ctx.accounts.signer.key(),
            &state.to_account_info().key(),
            final_price,
        );
        anchor_lang::solana_program::program::invoke(
            &transfer_instruction,
            &[
                ctx.accounts.signer.to_account_info(),
                state.to_account_info(),
            ],
        )?;

            state.total_minted += 1;

        Ok(())
    }
}

fn mint_nft<'info>(
    state: &Account<'info, State>,
    token_mint: &Account<'info, Mint>,
    nft_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    bump: u8,
) -> Result<()> {
    invoke_signed(
        &mint_to(
            token_program.key,
            token_mint.to_account_info().key,
            nft_token_account.to_account_info().key,
            state.to_account_info().key,
            &[state.to_account_info().key],
            1,
        )?,
        &[
            token_mint.to_account_info(),
            nft_token_account.to_account_info(),
            state.to_account_info(),
            token_program.to_account_info(),
        ],
        &[&[&ADMIN_SEED[..], [bump].as_ref()]],
    )?;
    Ok(())
}

fn remove_nft_mint_authority<'info>(
    state: &Account<'info, State>,
    token_mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
    bump: u8,
) -> Result<()> {
    invoke_signed(
        &set_authority(
            token_program.key,
            token_mint.to_account_info().key,
            Option::None,
            AuthorityType::MintTokens,
            state.to_account_info().key,
            &[state.to_account_info().key],
        )?,
        &[
            token_mint.to_account_info(),
            state.to_account_info(),
            token_program.to_account_info(),
        ],
        &[&[&ADMIN_SEED[..], [bump].as_ref()]],
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init, 
        payer = signer, 
        space = 8 + 1 + 8 + 8 + 8 + 32 + 32 + 128 + 128,
        seeds=[ADMIN_SEED],
        bump
    )]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddToWhitelist<'info> {
    #[account(
        init_if_needed, 
        payer = signer, 
        space = 8 + 1,
        seeds=[USER_SEED, signer.key().as_ref()],
        bump
    )]
    pub user_info: Account<'info, UserInfo>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePresaleInfo<'info> {
    #[account(
        mut,
        seeds=[ADMIN_SEED],
        bump
    )]
    pub state: Account<'info, State>,
    /// CHECK: collection address
    pub collection: UncheckedAccount<'info>,
     /// CHECK: genesis collection address
    pub genesis_collection: UncheckedAccount<'info>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    #[account(
        init_if_needed, 
        payer = signer, 
        space = 8 + 1,
        seeds=[USER_SEED, signer.key().as_ref()],
        bump
    )]
    pub user_info: Account<'info, UserInfo>,
    #[account(
        init,
        payer = signer,
        mint::authority = state,
        mint::decimals = 0,
    )]
    pub token_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = token_mint,
        associated_token::authority = signer,
    )]
    pub nft_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: checked via account constraints
    #[account(address = mpl_token_metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,
    /// CHECK: checked via the Metadata CPI call
    /// https://github.com/metaplex-foundation/metaplex-program-library/blob/master/token-metadata/program/src/utils.rs#L873
    #[account(mut)]
    pub nft_metadata_account: UncheckedAccount<'info>,
    /// CHECK: checked via account constraints
    #[account(address = NFT_UPDATE_AUTH)]
    pub metadata_update_auth: UncheckedAccount<'info>,

    /// CHECK: used to check genesis nft collection
    #[account(mut)]
    pub genesis_nft_metadata_account: AccountInfo<'info>, // used to check genesis nft ownership

    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[account]
pub struct State {
    pub total_minted: u64,
    pub presale_start: i64,
    pub admin: Pubkey,
    pub collection: Pubkey,
    pub genesis_collection: Pubkey,
    pub nft_name: String,
    pub nft_symbol: String,
    pub nft_uri: String,
}

#[account]
pub struct UserInfo {
    pub is_whitelisted: bool,
}

#[error_code]
pub enum MintError {
    #[msg("Max supply reached")]
    MaxSupplyReached,
    #[msg("Unauthorized")]
    Unauthorized,
}

fn get_round_price(current_time: i64, presale_start: i64, total_minted: u64) -> Result<u64> {
    let _elapsed_time = current_time - presale_start;

    match total_minted {
        0..=4999 => Ok(1_000_000_0),
        5000..=9999 => Ok(1_500_000_0),
        10000..=14999 => Ok(2_000_000_0),
        15000..=19999 => Ok(2_500_000_0),
        _ => Err(MintError::MaxSupplyReached.into()),
    }
}
