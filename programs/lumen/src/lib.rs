use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, CloseAccount, Mint, MintTo, Token, TokenAccount, Transfer};


declare_id!("123nCJRmafYDvdYBhisUcFHfoqanF6hxt1tHFNZyRD8");

const DECIMALS: u8 = 6;

#[program]
pub mod lumen {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>, 
        program_name: String,
        bumps: PoolBumps,
    ) -> ProgramResult{
        let base_config = &mut ctx.accounts.base_config;
        base_config.program_name = program_name.clone();

        let ido_account = &mut ctx.accounts.ido_account;
        let name_bytes = program_name.as_bytes();
        let mut name_data = [b' '; 10];
        name_data[..name_bytes.len()].copy_from_slice(name_bytes);
        
        ido_account.program_name = name_data;
        ido_account.bumps = bumps;

        ido_account.ido_authority = ctx.accounts.user.key();
        ido_account.usdc_mint = ctx.accounts.usdc_token.key();
        ido_account.redeemable_mint = ctx.accounts.redeemable_mint.key();
        ido_account.pool_usdc = ctx.accounts.pool_usdc.key();
        // ido_account.bumps = PoolBumps{
        //     ido_account: *ctx.bumps.get("ido_account").unwrap(),
        //     redeemable_mint: *ctx.bumps.get("redeemable_mint").unwrap(),
        //     pool_usdc: *ctx.bumps.get("pool_usdc").unwrap(),

        // };  
        
        Ok(())
    }

    
    pub fn init_user_redeemable( ctx: Context<InitUserRedeemable>) -> ProgramResult{
        Ok(())
    }

    pub fn exchange_usdc_for_redeemable(
        ctx: Context<ExchangeUsdcForRedeemable>,
        amount: u64
    ) -> ProgramResult {
        msg!("EXCHANGE USDC FOR REDEEMABLE");
        let cpi_accounts = Transfer{
            from: ctx.accounts.user_usdc.to_account_info(),
            to: ctx.accounts.pool_usdc.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // let ido_name = ctx.accounts.ido_account.program_name.as_ref();
        // let seeds = &[
        //     ido_name,
        //     &[ctx.accounts.ido_account.bumps.ido_account],
        // ];
        // let signer = &[&seeds[..]];
        // let cpi_accounts = MintTo{
        //     mint: ctx.accounts.redeemable_mint.to_account_info(),
        //     to: ctx.accounts.user_redeemable.to_account_info(),
        //     authority: ctx.accounts.ido_account.to_account_info()
        // };
        // let cpi_program = ctx.accounts.token_program.to_account_info();
        // let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        // token::mint_to(cpi_ctx, amount)?;
        Ok(())
    }

}

#[derive(Accounts)]
pub struct ExchangeUsdcForRedeemable<'info>{
    #[account(mut)]
    pub user_authority: Signer<'info>,
    #[account(
        mut,
        constraint = user_usdc.owner == user_authority.key(),
        constraint = user_usdc.mint == usdc_mint.key()
    )]
    pub user_usdc: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [
            user_authority.key().as_ref(),
            ido_account.program_name.as_ref(),
            b"user_redeemable"
        ],
        bump
    )]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    #[account(
        seeds = [
            ido_account.program_name.as_ref()
        ],
        bump = ido_account.bumps.ido_account,
        has_one = usdc_mint
    )]
    pub ido_account: Box<Account<'info, IdoAccount>>,
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            ido_account.program_name.as_ref(),
            b"redeemable_mint"
        ],
        bump = ido_account.bumps.redeemable_mint
    )]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [ido_account.program_name.as_ref(), b"pool_usdc"],
        bump = ido_account.bumps.pool_usdc
    )]
    pub pool_usdc:Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct InitUserRedeemable<'info>{
    #[account(mut)]
    user_authority: Signer<'info>,
    #[account(
        init,
        token::mint = redeemable_mint,
        token::authority = user_authority,
        seeds = [
            user_authority.key().as_ref(),
            ido_account.program_name.as_ref(),
            b"user_redeemable"
        ],
        bump,
        payer = user_authority
    )]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,
    #[account(seeds = [ido_account.program_name.as_ref()],
    bump = ido_account.bumps.ido_account)]
    pub ido_account: Box<Account<'info, IdoAccount>>,
    #[account(seeds = [ido_account.program_name.as_ref(), b"redeemable_mint"],
    bump = ido_account.bumps.redeemable_mint)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(program_name: String, bumps: PoolBumps)]
pub struct Initialize<'info>{
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        seeds = [program_name.as_bytes()],
        bump,
        payer = user
    )]
    pub ido_account: Box<Account<'info, IdoAccount>>,
    #[account(init, payer=user, space = 16 + 16)]
    pub base_config: Account<'info, BaseConfig>,
    #[account(constraint = usdc_token.decimals == DECIMALS)]
    pub usdc_token: Box<Account<'info, Mint>>,
    #[account(
        init,
        token::mint = usdc_token,
        token::authority = user,
        seeds = [program_name.as_bytes(), b"pool_usdc"],
        bump,
        payer=user

    )]
    pub pool_usdc: Box<Account<'info,TokenAccount>>,
    #[account(
        init,
        mint::decimals = DECIMALS,
        mint::authority = user,
        seeds = [program_name.as_bytes(), b"redeemable_mint".as_ref()],
        bump,
        payer = user
    )]
    pub redeemable_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[account]
#[derive(Default)]
pub struct IdoAccount {
    pub program_name: [u8; 10], // Setting an arbitrary max of ten characters in the ido name.
    pub bumps: PoolBumps,
    pub ido_authority: Pubkey,

    pub usdc_mint: Pubkey,
    pub redeemable_mint: Pubkey,
    pub pool_usdc: Pubkey,

}

#[account]
pub struct BaseConfig{
    pub program_name: String
}


#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct PoolBumps {
    pub ido_account: u8,
    pub redeemable_mint: u8,
    pub pool_usdc: u8,
}