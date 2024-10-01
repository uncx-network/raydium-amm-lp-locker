use super::*;

///Takes in a struct of InitialConfig which includes important configuration settings such as the
/// admin pubkey required for operations requiring a signer acconut whose address is the admin
/// address, dev wallet pubkey via which atas will be made and native fees be transferred to. Second
/// Parameter is of an inital list of blacklisted countries, initally only 10 blacklisted countries
/// are allowed to be added.

pub fn handle_initialize_config(
    ctx: Context<InitializeConfig>,
    initial_config: Config,
    initial_black_listed_countries: Option<[u8; 10]>,
) -> Result<()> {
    let config_acc = &mut ctx.accounts.config_account;

    //ensure locker unique id starts at 0
    require_eq!(initial_config.next_locker_unique_id, 0);

    // require_eq!(ctx.bumps.config_account,initial_config.)
    config_acc.config = initial_config;

    //store bump
    config_acc.bump = ctx.bumps.config_account;

    //run sanity checks
    config_acc.config.fee_config.basis_points_sanity_check()?;

    //add authority acccount of uncx locker
    config_acc.uncx_authority_pda_address = ctx.accounts.uncx_authority_acc.key();

    //store uncx bump for future invoke signed signing
    config_acc.uncx_authority_bump = ctx.bumps.uncx_authority_acc;

    //sanity checks
    crate::debug!(
        " before black list countries {:?}",
        config_acc.blacklisted_countries
    );

    config_acc.blacklisted_countries = {
        let mut temp_vec = Vec::with_capacity(256);
        if let Some(blacklisted_countries) = initial_black_listed_countries {
            temp_vec.extend_from_slice(&blacklisted_countries);
        };
        temp_vec
    };
    crate::debug!(
        "after black list countries {:?}",
        config_acc.blacklisted_countries
    );

    Ok(())
}
