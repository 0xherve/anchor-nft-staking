use anchor_lang::prelude::*;
use mpl_core::{
    instructions::AddPluginV1CpiBuilder,
    types::{FreezeDelegate, Plugin, PluginAuthority}
};

use crate::{
    errors::StakeError,
    state::{StakeAccount, StakeConfig, UserAccount},
};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: You need constraints for anchor to build an IDL
    #[account(
        mut,
        constraint = asset.owner == &mpl_core::ID)]
    pub asset: UncheckedAccount<'info>,
    /// CHECK: You need constraints for anchor to build an IDL
    #[account(
        mut,
        constraint = collection.owner == &mpl_core::ID )]
    pub collection: UncheckedAccount<'info>,
    #[account(
        init,
        payer = user,
        seeds = [b"stake", config.key().as_ref(), user.key().as_ref()],
        bump,
        space = StakeAccount::DISCRIMINATOR.len() + StakeAccount::INIT_SPACE,
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
    ///CHECK: MPL Core program account; only used for CPI 
    pub core_program: UncheckedAccount<'info>,
}

impl<'info> Stake<'info> {
    pub fn stake(&mut self, bumps: &StakeBumps) -> Result<()> {
        require!(self.user_account.amount_staked < self.config.max_stake, StakeError::MaxStakeReached);

        AddPluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(None)
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
            .init_authority(PluginAuthority::Address { address: self.stake_account.key()})
            .invoke()?;

        self.stake_account.set_inner(StakeAccount { 
            owner: self.user.key(), 
            mint: self.asset.key(), 
            staked_at: Clock::get()?.unix_timestamp, 
            bump: bumps.stake_account });
        
        Ok(())
    }
}

// was very informing to follow building this with Berg in the video