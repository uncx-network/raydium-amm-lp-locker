use super::*;
use anchor_lang::prelude::{Account, UncheckedAccount};
use anchor_spl::token::TokenAccount;
use state::AmmInfo;

use super::math::{Calculator, U128, U256};

/// The Detailed calculation of pnl
/// 1. calc last_k witch dose not take pnl: last_k = calc_pnl_x * calc_pnl_y;
/// 2. calc current price: current_price = current_x / current_y;
/// 3. calc x after take pnl: x_after_take_pnl = sqrt(last_k * current_price);
/// 4. calc y after take pnl: y_after_take_pnl = x_after_take_pnl / current_price;
///                           y_after_take_pnl = x_after_take_pnl * current_y / current_x;
/// 5. calc pnl_x & pnl_y:  pnl_x = current_x - x_after_take_pnl;
///                         pnl_y = current_y - y_after_take_pnl;
pub fn calc_take_pnl(
    target: &TargetOrders,
    amm: &AmmInfo,
    total_pc_without_take_pnl: &mut u64,
    total_coin_without_take_pnl: &mut u64,
    x1: U256,
    y1: U256,
) -> anchor_lang::prelude::Result<(u128, u128)> {
    // calc pnl
    let mut delta_x: u128;
    let mut delta_y: u128;
    let calc_pc_amount = Calculator::restore_decimal(
        target.calc_pnl_x.into(),
        amm.pc_decimals,
        amm.sys_decimal_value,
    );
    let calc_coin_amount = Calculator::restore_decimal(
        target.calc_pnl_y.into(),
        amm.coin_decimals,
        amm.sys_decimal_value,
    );
    let pool_pc_amount = U128::from(*total_pc_without_take_pnl);
    let pool_coin_amount = U128::from(*total_coin_without_take_pnl);
    if pool_pc_amount.checked_mul(pool_coin_amount).unwrap()
        >= (calc_pc_amount).checked_mul(calc_coin_amount).unwrap()
    {
        // last k is
        // let last_k: u128 = (target.calc_pnl_x as u128).checked_mul(target.calc_pnl_y as u128).unwrap();
        // current k is
        // let current_k: u128 = (x1 as u128).checked_mul(y1 as u128).unwrap();
        // current p is
        // let current_p: u128 = (x1 as u128).checked_div(y1 as u128).unwrap();
        let x2_power =
            Calculator::calc_x_power(target.calc_pnl_x.into(), target.calc_pnl_y.into(), x1, y1);
        // let x2 = Calculator::sqrt(x2_power).unwrap();
        let x2 = x2_power.integer_sqrt();
        // msg!(arrform!(LOG_SIZE, "calc_take_pnl x2_power:{}, x2:{}", x2_power, x2).as_str());
        let y2 = x2.checked_mul(y1).unwrap().checked_div(x1).unwrap();
        // msg!(arrform!(LOG_SIZE, "calc_take_pnl y2:{}", y2).as_str());

        // transfer to token_coin_pnl and token_pc_pnl
        // (x1 -x2) * pnl / sys_decimal_value
        let diff_x = U128::from(x1.checked_sub(x2).unwrap().as_u128());
        let diff_y = U128::from(y1.checked_sub(y2).unwrap().as_u128());
        delta_x = diff_x
            .checked_mul(amm.fees.pnl_numerator.into())
            .unwrap()
            .checked_div(amm.fees.pnl_denominator.into())
            .unwrap()
            .as_u128();
        delta_y = diff_y
            .checked_mul(amm.fees.pnl_numerator.into())
            .unwrap()
            .checked_div(amm.fees.pnl_denominator.into())
            .unwrap()
            .as_u128();

        let diff_pc_pnl_amount =
            Calculator::restore_decimal(diff_x, amm.pc_decimals, amm.sys_decimal_value);
        let diff_coin_pnl_amount =
            Calculator::restore_decimal(diff_y, amm.coin_decimals, amm.sys_decimal_value);
        let pc_pnl_amount = diff_pc_pnl_amount
            .checked_mul(amm.fees.pnl_numerator.into())
            .unwrap()
            .checked_div(amm.fees.pnl_denominator.into())
            .unwrap()
            .as_u64();
        let coin_pnl_amount = diff_coin_pnl_amount
            .checked_mul(amm.fees.pnl_numerator.into())
            .unwrap()
            .checked_div(amm.fees.pnl_denominator.into())
            .unwrap()
            .as_u64();
        if pc_pnl_amount != 0 && coin_pnl_amount != 0 {
            // We can Skip step2 , we dont need to mutate amm info
            // step2: save total_pnl_pc & total_pnl_coin
            // amm.state_data.total_pnl_pc = amm
            //     .state_data
            //     .total_pnl_pc
            //     .checked_add(diff_pc_pnl_amount.as_u64())
            //     .unwrap();
            // amm.state_data.total_pnl_coin = amm
            //     .state_data
            //     .total_pnl_coin
            //     .checked_add(diff_coin_pnl_amount.as_u64())
            //     .unwrap();
            // amm.state_data.need_take_pnl_pc = amm
            //     .state_data
            //     .need_take_pnl_pc
            //     .checked_add(pc_pnl_amount)
            //     .unwrap();
            // amm.state_data.need_take_pnl_coin = amm
            //     .state_data
            //     .need_take_pnl_coin
            //     .checked_add(coin_pnl_amount)
            //     .unwrap();

            // step3: update total_coin and total_pc without pnl
            *total_pc_without_take_pnl = (*total_pc_without_take_pnl)
                .checked_sub(pc_pnl_amount)
                .unwrap();
            *total_coin_without_take_pnl = (*total_coin_without_take_pnl)
                .checked_sub(coin_pnl_amount)
                .unwrap();
        } else {
            delta_x = 0;
            delta_y = 0;
        }
    } else {
        // msg!(arrform!(
        //     LOG_SIZE,
        //     "calc_take_pnl error x:{}, y:{}, calc_pnl_x:{}, calc_pnl_y:{}",
        //     x1,
        //     y1,
        //     target.calc_pnl_x,
        //     target.calc_pnl_y
        // )
        // .as_str());
        return Err(RaydiumAmmError::CalcPnlError.into());
    }

    Ok((delta_x, delta_y))
}
pub fn calc_actual_reserves<'info>(
    amm: &AmmInfo,
    amm_target_orders_info: &UncheckedAccount<'info>,
    amm_pc_vault: &Account<'info, TokenAccount>,
    amm_coin_vault: &Account<'info, TokenAccount>,
    //return amm lp mint amount,amm true pc amount, amm true coin amount
) -> anchor_lang::prelude::Result<(u64, u64, u64)> {
    // Incase Orderbook is enabled we accept the inaccuracy, but it should be quite rare,

    let target_orders = TargetOrders::load_unchecked(amm_target_orders_info)?;
    let (mut total_pc_without_take_pnl, mut total_coin_without_take_pnl) =
        Calculator::calc_total_without_take_pnl_no_orderbook(
            amm_pc_vault.amount,
            amm_coin_vault.amount,
            amm,
        )?;

    let x1 = Calculator::normalize_decimal_v2(
        total_pc_without_take_pnl,
        amm.pc_decimals,
        amm.sys_decimal_value,
    );
    let y1 = Calculator::normalize_decimal_v2(
        total_coin_without_take_pnl,
        amm.coin_decimals,
        amm.sys_decimal_value,
    );
    // calc and update pnl
    // let (delta_x, delta_y) =
    if amm.status != AmmStatus::WithdrawOnly.into_u64() {
        calc_take_pnl(
            &target_orders,
            amm,
            &mut total_pc_without_take_pnl,
            &mut total_coin_without_take_pnl,
            x1.as_u128().into(),
            y1.as_u128().into(),
        )?;
    }
    //return Actual pc and coin reserves
    Ok((
        amm.lp_amount,
        total_pc_without_take_pnl,
        total_coin_without_take_pnl,
    ))
}
