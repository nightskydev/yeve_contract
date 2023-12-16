use anchor_lang::prelude::*;

use crate::state::Yevepool;

#[derive(Accounts)]
#[instruction(reward_index: u8)]
pub struct SetRewardAuthority<'info> {
    #[account(mut)]
    pub yevepool: Account<'info, Yevepool>,

    #[account(address = yevepool.reward_infos[reward_index as usize].authority)]
    pub reward_authority: Signer<'info>,

    /// CHECK: safe, the account that will be new authority can be arbitrary
    pub new_reward_authority: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<SetRewardAuthority>, reward_index: u8) -> Result<()> {
    Ok(ctx.accounts.yevepool.update_reward_authority(
        reward_index as usize,
        ctx.accounts.new_reward_authority.key(),
    )?)
}
