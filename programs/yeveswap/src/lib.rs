use anchor_lang::prelude::*;

declare_id!("3Vjbq7mGrLvSBb9v4n4TCeyS7DPFkx9kbqQNUfULgzrE");

#[doc(hidden)]
pub mod errors;
#[doc(hidden)]
pub mod instructions;
#[doc(hidden)]
pub mod math;
pub mod state;

use crate::state::{YevepoolBumps};
use instructions::*;

#[program]
pub mod yeveswap {
    use super::*;

    /// Initializes a YevepoolsConfig account that hosts info & authorities
    /// required to govern a set of yevepools.
    ///
    /// ### Parameters
    /// - `fee_authority` - Authority authorized to initialize fee-tiers and set customs fees.
    /// - `collect_protocol_fees_authority` - Authority authorized to collect protocol fees.
    /// - `reward_emissions_super_authority` - Authority authorized to set reward authorities in pools.
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        fee_authority: Pubkey,
        collect_protocol_fees_authority: Pubkey,
        reward_emissions_super_authority: Pubkey,
        default_protocol_fee_rate: u16,
    ) -> Result<()> {
        return instructions::initialize_config::handler(
            ctx,
            fee_authority,
            collect_protocol_fees_authority,
            reward_emissions_super_authority,
            default_protocol_fee_rate,
        );
    }

    /// Initializes a Yevepool account.
    /// Fee rate is set to the default values on the config and supplied fee_tier.
    ///
    /// ### Parameters
    /// - `bumps` - The bump value when deriving the PDA of the Yevepool address.
    /// - `tick_spacing` - The desired tick spacing for this pool.
    /// - `initial_sqrt_price` - The desired initial sqrt-price for this pool
    ///
    /// #### Special Errors
    /// `InvalidTokenMintOrder` - The order of mints have to be ordered by
    /// `SqrtPriceOutOfBounds` - provided initial_sqrt_price is not between 2^-64 to 2^64
    ///
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        bumps: YevepoolBumps,
        tick_spacing: u16,
        initial_sqrt_price: u128,
    ) -> Result<()> {
        return instructions::initialize_pool::handler(
            ctx,
            bumps,
            tick_spacing,
            initial_sqrt_price,
        );
    }
}