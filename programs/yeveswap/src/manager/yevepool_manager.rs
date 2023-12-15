use crate::errors::ErrorCode;
use crate::math::{add_liquidity_delta, checked_mul_div};
use crate::state::*;

// Calculates the next global reward growth variables based on the given timestamp.
// The provided timestamp must be greater than or equal to the last updated timestamp.
pub fn next_yevepool_reward_infos(
    yevepool: &Yevepool,
    next_timestamp: u64,
) -> Result<[YevepoolRewardInfo; NUM_REWARDS], ErrorCode> {
    let curr_timestamp = yevepool.reward_last_updated_timestamp;
    if next_timestamp < curr_timestamp {
        return Err(ErrorCode::InvalidTimestamp.into());
    }

    // No-op if no liquidity or no change in timestamp
    if yevepool.liquidity == 0 || next_timestamp == curr_timestamp {
        return Ok(yevepool.reward_infos);
    }

    // Calculate new global reward growth
    let mut next_reward_infos = yevepool.reward_infos;
    let time_delta = u128::from(next_timestamp - curr_timestamp);
    for i in 0..NUM_REWARDS {
        if !next_reward_infos[i].initialized() {
            continue;
        }
        let reward_info = &mut next_reward_infos[i];

        // Calculate the new reward growth delta.
        // If the calculation overflows, set the delta value to zero.
        // This will halt reward distributions for this reward.
        let reward_growth_delta = checked_mul_div(
            time_delta,
            reward_info.emissions_per_second_x64,
            yevepool.liquidity,
        )
        .unwrap_or(0);

        // Add the reward growth delta to the global reward growth.
        let curr_growth_global = reward_info.growth_global_x64;
        reward_info.growth_global_x64 = curr_growth_global.wrapping_add(reward_growth_delta);
    }

    Ok(next_reward_infos)
}

// Calculates the next global liquidity for a yevepool depending on its position relative
// to the lower and upper tick indexes and the liquidity_delta.
pub fn next_yevepool_liquidity(
    yevepool: &Yevepool,
    tick_upper_index: i32,
    tick_lower_index: i32,
    liquidity_delta: i128,
) -> Result<u128, ErrorCode> {
    if yevepool.tick_current_index < tick_upper_index
        && yevepool.tick_current_index >= tick_lower_index
    {
        add_liquidity_delta(yevepool.liquidity, liquidity_delta)
    } else {
        Ok(yevepool.liquidity)
    }
}

#[cfg(test)]
mod yevepool_manager_tests {

    use anchor_lang::prelude::Pubkey;

    use crate::manager::yevepool_manager::next_yevepool_reward_infos;
    use crate::math::Q64_RESOLUTION;
    use crate::state::yevepool::YevepoolRewardInfo;
    use crate::state::yevepool::NUM_REWARDS;
    use crate::state::yevepool_builder::YevepoolBuilder;
    use crate::state::Yevepool;

    // Initializes a yevepool for testing with all the rewards initialized
    fn init_test_yevepool(liquidity: u128, reward_last_updated_timestamp: u64) -> Yevepool {
        YevepoolBuilder::new()
            .liquidity(liquidity)
            .reward_last_updated_timestamp(reward_last_updated_timestamp) // Jan 1 2021 EST
            .reward_infos([
                YevepoolRewardInfo {
                    mint: Pubkey::new_unique(),
                    emissions_per_second_x64: 10 << Q64_RESOLUTION,
                    growth_global_x64: 100 << Q64_RESOLUTION,
                    ..Default::default()
                },
                YevepoolRewardInfo {
                    mint: Pubkey::new_unique(),
                    emissions_per_second_x64: 0b11 << (Q64_RESOLUTION - 1), // 1.5
                    growth_global_x64: 200 << Q64_RESOLUTION,
                    ..Default::default()
                },
                YevepoolRewardInfo {
                    mint: Pubkey::new_unique(),
                    emissions_per_second_x64: 1 << (Q64_RESOLUTION - 1), // 0.5
                    growth_global_x64: 300 << Q64_RESOLUTION,
                    ..Default::default()
                },
            ])
            .build()
    }

    #[test]
    fn test_next_yevepool_reward_infos_zero_liquidity_no_op() {
        let yevepool = init_test_yevepool(0, 1577854800);

        let result = next_yevepool_reward_infos(&yevepool, 1577855800);
        assert_eq!(
            YevepoolRewardInfo::to_reward_growths(&result.unwrap()),
            [
                100 << Q64_RESOLUTION,
                200 << Q64_RESOLUTION,
                300 << Q64_RESOLUTION
            ]
        );
    }

    #[test]
    fn test_next_yevepool_reward_infos_same_timestamp_no_op() {
        let yevepool = init_test_yevepool(100, 1577854800);

        let result = next_yevepool_reward_infos(&yevepool, 1577854800);
        assert_eq!(
            YevepoolRewardInfo::to_reward_growths(&result.unwrap()),
            [
                100 << Q64_RESOLUTION,
                200 << Q64_RESOLUTION,
                300 << Q64_RESOLUTION
            ]
        );
    }

    #[test]
    #[should_panic(expected = "InvalidTimestamp")]
    fn test_next_yevepool_reward_infos_invalid_timestamp() {
        let yevepool = &YevepoolBuilder::new()
            .liquidity(100)
            .reward_last_updated_timestamp(1577854800) // Jan 1 2020 EST
            .build();

        // New timestamp is earlier than the last updated timestamp
        next_yevepool_reward_infos(yevepool, 1577768400).unwrap(); // Dec 31 2019 EST
    }

    #[test]
    fn test_next_yevepool_reward_infos_no_initialized_rewards() {
        let yevepool = &YevepoolBuilder::new()
            .liquidity(100)
            .reward_last_updated_timestamp(1577854800) // Jan 1 2021 EST
            .build();

        let new_timestamp = 1577854800 + 300;
        let result = next_yevepool_reward_infos(yevepool, new_timestamp).unwrap();
        assert_eq!(YevepoolRewardInfo::to_reward_growths(&result), [0, 0, 0]);
    }

    #[test]
    fn test_next_yevepool_reward_infos_some_initialized_rewards() {
        let yevepool = &YevepoolBuilder::new()
            .liquidity(100)
            .reward_last_updated_timestamp(1577854800) // Jan 1 2021 EST
            .reward_info(
                0,
                YevepoolRewardInfo {
                    mint: Pubkey::new_unique(),
                    emissions_per_second_x64: 1 << Q64_RESOLUTION,
                    ..Default::default()
                },
            )
            .build();

        let new_timestamp = 1577854800 + 300;
        let result = next_yevepool_reward_infos(yevepool, new_timestamp).unwrap();
        assert_eq!(result[0].growth_global_x64, 3 << Q64_RESOLUTION);
        for i in 1..NUM_REWARDS {
            assert_eq!(yevepool.reward_infos[i].growth_global_x64, 0);
        }
    }

    #[test]
    fn test_next_yevepool_reward_infos_delta_zero_on_overflow() {
        let yevepool = &YevepoolBuilder::new()
            .liquidity(100)
            .reward_last_updated_timestamp(0)
            .reward_info(
                0,
                YevepoolRewardInfo {
                    mint: Pubkey::new_unique(),
                    emissions_per_second_x64: u128::MAX,
                    growth_global_x64: 100,
                    ..Default::default()
                },
            )
            .build();

        let new_timestamp = i64::MAX as u64;
        let result = next_yevepool_reward_infos(yevepool, new_timestamp).unwrap();
        assert_eq!(result[0].growth_global_x64, 100);
    }

    #[test]
    fn test_next_yevepool_reward_infos_all_initialized_rewards() {
        let yevepool = init_test_yevepool(100, 1577854800);

        let new_timestamp = 1577854800 + 300;
        let result = next_yevepool_reward_infos(&yevepool, new_timestamp).unwrap();
        assert_eq!(result[0].growth_global_x64, 130 << Q64_RESOLUTION);
        assert_eq!(
            result[1].growth_global_x64,
            0b110011001 << (Q64_RESOLUTION - 1) // 204.5
        );
        assert_eq!(
            result[2].growth_global_x64,
            0b1001011011 << (Q64_RESOLUTION - 1) // 301.5
        );
    }
}
