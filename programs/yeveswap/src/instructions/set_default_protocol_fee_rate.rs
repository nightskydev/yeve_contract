use anchor_lang::prelude::*;

use crate::state::YevepoolsConfig;

#[derive(Accounts)]
pub struct SetDefaultProtocolFeeRate<'info> {
    #[account(mut)]
    pub yevepools_config: Account<'info, YevepoolsConfig>,

    #[account(address = yevepools_config.fee_authority)]
    pub fee_authority: Signer<'info>,
}

pub fn handler(
    ctx: Context<SetDefaultProtocolFeeRate>,
    default_protocol_fee_rate: u16,
) -> Result<()> {
    Ok(ctx
        .accounts
        .yevepools_config
        .update_default_protocol_fee_rate(default_protocol_fee_rate)?)
}
