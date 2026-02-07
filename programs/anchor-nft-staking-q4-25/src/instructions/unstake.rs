use anchor_lang::prelude::*;
use mpl_core::{
    instructions::{RemovePluginV1CpiBuilder, UpdatePluginV1CpiBuilder},
    types::{FreezeDelegate, Plugin, PluginType},
};

use crate::{
    errors::StakeError, state::{StakeAccount, StakeConfig, UserAccount}
};

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: You need constraints or this check comment for anchor to build an IDL
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,
    /// CHECK: You need constraints or this check comment for anchor to build an IDL
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    #[account(
        mut,
        close = user,
        seeds = [b"stake", config.key().as_ref(), user.key().as_ref()],
        bump = stake_account.bump,
        constraint = stake_account.owner == user.key() @StakeError::NotOwner
    )]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(
        seeds = [b"config".as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, StakeConfig>,
    #[account(
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,
    
    //programs
    pub system_program: Program<'info, System>,
    /// CHECK: You need constraints or this check comment for anchor to build an IDL
    pub core_program: UncheckedAccount<'info>,
}

impl<'info> Unstake<'info> {
    pub fn unstake(&mut self) -> Result<()> {
        let time_elapsed = (Clock::get()?.unix_timestamp - self.stake_account.staked_at/86400) as u32;
        
        require!(
            time_elapsed > self.config.freeze_period,
            StakeError::FreezePeriodNotPassed);
        
        let points_earned = time_elapsed * (self.config.points_per_stake) as u32;
        
        self.user_account.points += points_earned;
        
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"stake",
            &self.config.key().to_bytes(),
            &self.user.key().to_bytes(),
            &[self.stake_account.bump]
        ]];
        
        UpdatePluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(None)
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke_signed(signer_seeds)?;
        
        RemovePluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(None)
            .system_program(&self.system_program.to_account_info())
            .plugin_type(PluginType::FreezeDelegate)
            .invoke()?;
        
        self.user_account.amount_staked -= 1;
        Ok(())
    }
}
