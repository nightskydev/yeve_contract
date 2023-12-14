use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

#[derive(Accounts)]
#[instruction(bumps: YevepoolBumps, tick_spacing: u16)]
pub struct InitializePool<'info> {
    pub yevepools_config: Box<Account<'info, YevepoolsConfig>>,

    pub token_mint_a: Account<'info, Mint>,
    pub token_mint_b: Account<'info, Mint>,

    #[account(mut)]
    pub funder: Signer<'info>,

    #[account(init,
      seeds = [
        b"yevepool".as_ref(),
        yevepools_config.key().as_ref(),
        token_mint_a.key().as_ref(),
        token_mint_b.key().as_ref(),
        tick_spacing.to_le_bytes().as_ref()
      ],
      bump,
      payer = funder,
      space = Yevepool::LEN)]
    pub yevepool: Box<Account<'info, Yevepool>>,

    #[account(init,
      payer = funder,
      token::mint = token_mint_a,
      token::authority = yevepool)]
    pub token_vault_a: Box<Account<'info, TokenAccount>>,

    #[account(init,
      payer = funder,
      token::mint = token_mint_b,
      token::authority = yevepool)]
    pub token_vault_b: Box<Account<'info, TokenAccount>>,

    #[account(has_one = yevepools_config)]
    pub fee_tier: Account<'info, FeeTier>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<InitializePool>,
    _bumps: YevepoolBumps,
    tick_spacing: u16,
    initial_sqrt_price: u128,
) -> Result<()> {
    let token_mint_a = ctx.accounts.token_mint_a.key();
    let token_mint_b = ctx.accounts.token_mint_b.key();

    let yevepool = &mut ctx.accounts.yevepool;
    let yevepools_config = &ctx.accounts.yevepools_config;

    let default_fee_rate = ctx.accounts.fee_tier.default_fee_rate;

    // ignore the bump passed and use one Anchor derived
    let bump = *ctx.bumps.get("yevepool").unwrap();

    Ok(yevepool.initialize(
        yevepools_config,
        bump,
        tick_spacing,
        initial_sqrt_price,
        default_fee_rate,
        token_mint_a,
        ctx.accounts.token_vault_a.key(),
        token_mint_b,
        ctx.accounts.token_vault_b.key(),
    )?)
}
