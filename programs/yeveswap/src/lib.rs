use anchor_lang::prelude::*;

declare_id!("3Vjbq7mGrLvSBb9v4n4TCeyS7DPFkx9kbqQNUfULgzrE");

#[doc(hidden)]
pub mod constants;
#[doc(hidden)]
pub mod errors;
#[doc(hidden)]
pub mod instructions;
#[doc(hidden)]
pub mod manager;
#[doc(hidden)]
pub mod math;
pub mod state;
#[doc(hidden)]
pub mod util;

use crate::state::{OpenPositionBumps, YevepoolBumps};
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

    /////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////////////////////////////
    // initialize_tick_array function need to be added

    /// Initializes a fee_tier account usable by Yevepools in a YevepoolConfig space.
    ///
    /// ### Authority
    /// - "fee_authority" - Set authority in the YevepoolConfig
    ///
    /// ### Parameters
    /// - `tick_spacing` - The tick-spacing that this fee-tier suggests the default_fee_rate for.
    /// - `default_fee_rate` - The default fee rate that a pool will use if the pool uses this
    ///                        fee tier during initialization.
    ///
    /// #### Special Errors
    /// - `FeeRateMaxExceeded` - If the provided default_fee_rate exceeds MAX_FEE_RATE.
    pub fn initialize_fee_tier(
        ctx: Context<InitializeFeeTier>,
        tick_spacing: u16,
        default_fee_rate: u16,
    ) -> Result<()> {
        return instructions::initialize_fee_tier::handler(ctx, tick_spacing, default_fee_rate);
    }

    /// Initialize reward for a Yevepool. A pool can only support up to a set number of rewards.
    ///
    /// ### Authority
    /// - "reward_authority" - assigned authority by the reward_super_authority for the specified
    ///                        reward-index in this Yevepool
    ///
    /// ### Parameters
    /// - `reward_index` - The reward index that we'd like to initialize. (0 <= index <= NUM_REWARDS)
    ///
    /// #### Special Errors
    /// - `InvalidRewardIndex` - If the provided reward index doesn't match the lowest uninitialized
    ///                          index in this pool, or exceeds NUM_REWARDS, or
    ///                          all reward slots for this pool has been initialized.
    pub fn initialize_reward(ctx: Context<InitializeReward>, reward_index: u8) -> Result<()> {
        return instructions::initialize_reward::handler(ctx, reward_index);
    }

    /// ### Authority
    /// - "reward_authority" - assigned authority by the reward_super_authority for the specified
    ///                        reward-index in this Yevepool
    ///
    /// ### Parameters
    /// - `reward_index` - The reward index (0 <= index <= NUM_REWARDS) that we'd like to modify.
    /// - `emissions_per_second_x64` - The amount of rewards emitted in this pool.
    ///
    /// #### Special Errors
    /// - `RewardVaultAmountInsufficient` - The amount of rewards in the reward vault cannot emit
    ///                                     more than a day of desired emissions.
    /// - `InvalidTimestamp` - Provided timestamp is not in order with the previous timestamp.
    /// - `InvalidRewardIndex` - If the provided reward index doesn't match the lowest uninitialized
    ///                          index in this pool, or exceeds NUM_REWARDS, or
    ///                          all reward slots for this pool has been initialized.
    pub fn set_reward_emissions(
        ctx: Context<SetRewardEmissions>,
        reward_index: u8,
        emissions_per_second_x64: u128,
    ) -> Result<()> {
        return instructions::set_reward_emissions::handler(
            ctx,
            reward_index,
            emissions_per_second_x64,
        );
    }

    /// Open a position in a Yevepool. A unique token will be minted to represent the position
    /// in the users wallet. The position will start off with 0 liquidity.
    ///
    /// ### Parameters
    /// - `tick_lower_index` - The tick specifying the lower end of the position range.
    /// - `tick_upper_index` - The tick specifying the upper end of the position range.
    ///
    /// #### Special Errors
    /// - `InvalidTickIndex` - If a provided tick is out of bounds, out of order or not a multiple of
    ///                        the tick-spacing in this pool.
    pub fn open_position(
        ctx: Context<OpenPosition>,
        bumps: OpenPositionBumps,
        tick_lower_index: i32,
        tick_upper_index: i32,
    ) -> Result<()> {
        return instructions::open_position::handler(
            ctx,
            bumps,
            tick_lower_index,
            tick_upper_index,
        );
    }
}
