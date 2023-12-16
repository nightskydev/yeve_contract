use anchor_lang::prelude::*;

use crate::{
    manager::liquidity_manager::calculate_fee_and_reward_growths, state::*, util::to_timestamp_u64,
};

#[derive(Accounts)]
pub struct UpdateFeesAndRewards<'info> {
    #[account(mut)]
    pub yevepool: Account<'info, Yevepool>,

    #[account(mut, has_one = yevepool)]
    pub position: Account<'info, Position>,

    #[account(has_one = yevepool)]
    pub tick_array_lower: AccountLoader<'info, TickArray>,
    #[account(has_one = yevepool)]
    pub tick_array_upper: AccountLoader<'info, TickArray>,
}

pub fn handler(ctx: Context<UpdateFeesAndRewards>) -> Result<()> {
    let yevepool = &mut ctx.accounts.yevepool;
    let position = &mut ctx.accounts.position;
    let clock = Clock::get()?;
    let timestamp = to_timestamp_u64(clock.unix_timestamp)?;

    let (position_update, reward_infos) = calculate_fee_and_reward_growths(
        yevepool,
        position,
        &ctx.accounts.tick_array_lower,
        &ctx.accounts.tick_array_upper,
        timestamp,
    )?;

    yevepool.update_rewards(reward_infos, timestamp);
    position.update(&position_update);

    Ok(())
}
