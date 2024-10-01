use super::*;
///Lp Locker Conifguration account, EVM Fee Data, Nonce and admin is combined into one account
/// In solana it is an anti-pattern to store single values in different accounts as there is a
/// limitation on tx size and each account takes 32 bytes reducing composability with other
/// instructions in a (legacy)transaction Note : referral can be anyone who holds the referral token
/// with about min referral token balance Account 1: seeds = [b"config_account"]
#[cfg_attr(feature = "client", derive(Debug))]
#[account]
#[derive(InitSpace)]

pub struct ConfigurationAccount {
    pub config: Config,
    pub bump: u8,
    pub(crate) uncx_authority_pda_address: Pubkey,
    pub(crate) uncx_authority_bump: u8,
    #[max_len(256)]
    pub blacklisted_countries: Vec<u8>,
}

///All of the methods are 'adminOnly'

impl ConfigurationAccount {
    pub(crate) fn is_country_allowed(&self, country_code: u8) -> bool {
        !self.blacklisted_countries.contains(&country_code)
    }
    pub(crate) fn add_country_to_blacklist(
        &mut self,
        country_code_to_add: u8,
    ) -> anchor_lang::prelude::Result<()> {
        let None = self
            .blacklisted_countries
            .iter()
            .position(|x| *x == country_code_to_add)
        else {
            return anchor_lang::err!(UncxLpError::CountryCodeAlreadyExists);
        };

        self.blacklisted_countries.push(country_code_to_add);
        Ok(())
    }
    pub fn remove_country_from_blacklist(
        &mut self,
        country_code_to_remove: u8,
    ) -> anchor_lang::prelude::Result<()> {
        let Some(idx) = self
            .blacklisted_countries
            .iter()
            .position(|x| *x == country_code_to_remove)
        else {
            return anchor_lang::err!(UncxLpError::CountryCodeNotPresent);
        };

        self.blacklisted_countries.swap_remove(idx);
        Ok(())
    }

    pub fn get_next_locker_id(&self) -> u64 {
        self.config.next_locker_unique_id
    }

    pub fn set_developer_address(&mut self, new_fee_wallet_address: Pubkey) {
        self.config.dev_addr = new_fee_wallet_address;
    }

    pub fn set_new_admin(&mut self, new_admin_address: Pubkey) {
        self.config.admin_key = new_admin_address;
    }

    pub fn set_secondary_fee_token(&mut self, new_secondary_fee_token: Option<Pubkey>) {
        self.config.secondary_token_address = new_secondary_fee_token;
    }

    pub fn set_referral_token_address(&mut self, new_referral_token_address: Option<Pubkey>) {
        self.config.referral_token_address = new_referral_token_address;
    }

    pub fn set_fees_config(&mut self, new_fees_data: FeesConfig) -> Result<()> {
        FeesConfig::basis_points_sanity_check(&new_fees_data)?;

        self.config.fee_config = new_fees_data;

        Ok(())
    }

    pub fn set_referral_token_and_min_hold_balance(
        &mut self,
        new_referral_token_address: Option<Pubkey>,
        new_min_hold_balance: u64,
    ) {
        self.config.referral_token_address = new_referral_token_address;

        self.config.min_referral_balance = new_min_hold_balance;
    }
}

//sanity checks
impl FeesConfig {
    pub fn basis_points_sanity_check(&self) -> Result<()> {
        //percentage is between 1-10000
        //min = 0.01% = 1
        //0.1% = 10
        //1% = 100
        //10% = 1000
        Self::basis_points_validate(self.liquidity_fee_bps)?;

        Self::basis_points_validate(self.secondary_token_discount_bps)?;

        Self::basis_points_validate(self.referral_share_bps)?;

        Self::basis_points_validate(self.referral_discount_bps)?;

        Ok(())
    }

    fn basis_points_validate(percentage_val: u16) -> Result<()> {
        require!(
            percentage_val <= crate::constants::MAX_BASIS_POINTS,
            UncxLpError::InvalidPercentage
        );

        Ok(())
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, InitSpace, Clone, Default)]
#[cfg_attr(feature = "client", derive(Copy, Debug))]

pub struct Config {
    pub fee_config: FeesConfig,
    /// balance the referrer must hold to qualify as a referrer aka referralHold
    pub min_referral_balance: u64,

    /// token the refferer must hold to qualify as a referrer
    pub referral_token_address: Option<Pubkey>,
    ///secondary token which we provide a discount on if the user
    ///agrees to burning it
    pub secondary_token_address: Option<Pubkey>,
    ///Stores the admin public key
    pub admin_key: Pubkey,
    //wallet address fees is sent to
    pub dev_addr: Pubkey,
    /// Sores tthe next locker id
    pub next_locker_unique_id: u64,
}

#[derive(AnchorDeserialize, AnchorSerialize, InitSpace, Clone, Default)]
#[cfg_attr(feature = "client", derive(Copy, Debug, PartialEq, Eq))]

pub struct FeesConfig {
    pub native_fee: u64,
    ///fee charged via alternative means, if paid via a secondary token
    pub secondary_token_fee: u64,
    /// discount on liquidity fee for burning secondaryToken
    pub secondary_token_discount_bps: u16,
    /// fee on raydiumv2 style liquidity tokens
    pub liquidity_fee_bps: u16,
    /// *use bps, 10000 = 100%
    ///  fee for referrals
    pub referral_share_bps: u16,
    /// discount on flatrate fees for using a valid referral address
    pub referral_discount_bps: u16,
}

///Whitelist account, derived via ["uncx-whitelist","user address"],Existence of a signer
///with whose address we will derive a whitelist account and check if its initialized.
#[account]
#[derive(InitSpace)]

pub struct Whitelisted {
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]

pub struct Migrator {
    pub bump: u8,
}
