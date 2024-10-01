use super::*;
use anchor_lang::system_program;
use anchor_spl::token;

pub fn token_transfer<
    'info,
    P: ToAccountInfo<'info>,
    A: ToAccountInfo<'info>,
    S: ToAccountInfo<'info>,
>(
    amount: u64,
    token_program: &P,
    from: &A,
    to: &A,
    authority: &S,
) -> Result<()> {
    if amount > 0 {
        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: authority.to_account_info(),
                },
            ),
            amount,
        )
    } else {
        Ok(())
    }
}

pub fn burn_token<
    'info,
    P: ToAccountInfo<'info>,
    A: ToAccountInfo<'info>,
    S: ToAccountInfo<'info>,
    D: ToAccountInfo<'info>,
>(
    amount: u64,
    token_program: &P,
    from: &A,
    mint: &D,
    authority: &S,
) -> Result<()> {
    token::burn(
        CpiContext::new(
            token_program.to_account_info(),
            token::Burn {
                from: from.to_account_info(),
                mint: mint.to_account_info(),
                authority: authority.to_account_info(),
            },
        ),
        amount,
    )?;
    Ok(())
}

pub fn token_transfer_signed<
    'info,
    P: ToAccountInfo<'info>,
    A: ToAccountInfo<'info>,
    L: ToAccountInfo<'info>,
>(
    amount: u64,
    token_program: &P,
    from: &A,
    to: &A,
    authority: &L,
    seeds: &[&[u8]],
) -> Result<()> {
    token::transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            token::Transfer {
                from: from.to_account_info(),
                to: to.to_account_info(),
                authority: authority.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;

    Ok(())
}

pub fn system_program_transfer<
    'info,
    S: ToAccountInfo<'info>,
    A: ToAccountInfo<'info>,
    L: ToAccountInfo<'info>,
>(
    amount: u64,
    system_program: &S,
    from: &A,
    to: &L,
) -> Result<()> {
    if amount > 0 {
        system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                },
            ),
            amount,
        )
    } else {
        Ok(())
    }
}

pub fn clock_now() -> (i64, u64) {
    let clock = Clock::get().unwrap();

    (clock.unix_timestamp, clock.slot)
}

//refactor the logic of finding and removing a locker id from the associated vec stored in the lp
// info acc if the locker count for a user info drops to 0, clean the user info acc as well
pub fn clean_locker_from_tracker_and_close<'info>(
    user_lp_tracker_acc: &mut Account<'info, UserLpInfoAccount>,
    user_info_acc: &mut Account<'info, UserInfoAccount>,
    refund_rent_to: AccountInfo<'info>,
    locker_id: u64,
) -> Result<()> {
    user_lp_tracker_acc.find_and_remove_locker_from_tracking(locker_id)?;

    if user_lp_tracker_acc.associated_locker_ids.is_empty() {
        user_lp_tracker_acc.close(refund_rent_to.clone())?;
    }

    //account for the lp locker being removed from the tracker
    user_info_acc.lp_locker_count -= 1;

    //if total numbers of lp lockers of a specific amm id for a user drops to 0, clean up the user
    // info account as well.
    if user_info_acc.lp_locker_count == 0 {
        user_info_acc.close(refund_rent_to)?;
    }

    Ok(())
}
