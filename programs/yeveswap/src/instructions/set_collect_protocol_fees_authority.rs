use anchor_lang::prelude::*;

use crate::state::YevepoolsConfig;

#[derive(Accounts)]
pub struct SetCollectProtocolFeesAuthority<'info> {
    #[account(mut)]
    pub yevepools_config: Account<'info, YevepoolsConfig>,

    #[account(address = yevepools_config.collect_protocol_fees_authority)]
    pub collect_protocol_fees_authority: Signer<'info>,

    /// CHECK: safe, the account that will be new authority can be arbitrary
    pub new_collect_protocol_fees_authority: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<SetCollectProtocolFeesAuthority>) -> Result<()> {
    Ok(ctx
        .accounts
        .yevepools_config
        .update_collect_protocol_fees_authority(
            ctx.accounts.new_collect_protocol_fees_authority.key(),
        ))
}
