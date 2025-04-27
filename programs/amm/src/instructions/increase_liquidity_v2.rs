use super::increase_liquidity::increase_liquidity;
use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, Token2022, TokenAccount};

/*
POOL_STRUCT INCLUDES:
        id,
        bump,
        sqrt_price_x64,
        0,
        tick,
        pool_creator: ctx.accounts.pool_creator.key(),
        token_vault_0:ctx.accounts.token_vault_0.key(),
        token_vault_1: ctx.accounts.token_vault_1.key(),
        amm_config: ctx.accounts.amm_config.as_ref(),
        token_mint_0: ctx.accounts.token_mint_0.as_ref(),
        token_mint_1: ctx.accounts.token_mint_1.as_ref(),
        observation_state: ctx.accounts.observation_state.key(),
    )?;
*/
#[derive(Accounts)]
pub struct IncreaseLiquidityV2<'info> {
    /// Pays to mint the position
    pub nft_owner: Signer<'info>,

    /// The token account for nft
    /*
    nft_account.mint == personal_position.nft_mint
    the token amount should be 1 i.e. unique
    checks here we set the token authority to mft_owner
    here         token::authority = nft_owner,
we are able to token::authority because we have passed the token programs whcih helps in this 
    */
    #[account(
        constraint = nft_account.mint == personal_position.nft_mint,
        constraint = nft_account.amount == 1,
        token::authority = nft_owner,
    )]
    pub nft_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,

        ///        constraint = protocol_position.pool_id == pool_state.key(), here the use of passing the id to the pda used
    #[account(
        mut,
        seeds = [
            POSITION_SEED.as_bytes(),
            pool_state.key().as_ref(),
            &personal_position.tick_lower_index.to_be_bytes(),
            &personal_position.tick_upper_index.to_be_bytes(),
        ],
        bump,
        constraint = protocol_position.pool_id == pool_state.key(),
    )]
    pub protocol_position: Box<Account<'info, ProtocolPositionState>>,

    /// Increase liquidity for this position
    /*
    personal_position pool id should match wih pool_state.key()
    */
    #[account(mut, constraint = personal_position.pool_id == pool_state.key())]
    pub personal_position: Box<Account<'info, PersonalPositionState>>,

    /// Stores init state for the lower tick
    //here we add a contraints that tick array lower pool id should match with pool_state.key()
    #[account(mut, constraint = tick_array_lower.load()?.pool_id == pool_state.key())]
    pub tick_array_lower: AccountLoader<'info, TickArrayState>,

    /// Stores init state for the upper tick
    //here we add a contraints that tick array upper pool id should match with pool_state.key()
    #[account(mut, constraint = tick_array_upper.load()?.pool_id == pool_state.key())]
    pub tick_array_upper: AccountLoader<'info, TickArrayState>,

    /// The payer's token account for token_0
    //here we verify that the user ata's mint should match with token vault where the tokens are to be transferred
    #[account(
        mut,
        token::mint = token_vault_0.mint
    )]
    pub token_account_0: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The token account spending token_1 to mint the position
    #[account(
        mut,
        token::mint = token_vault_1.mint
    )]
    pub token_account_1: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_0
    //the vault account key should match with pool_state's token vault
    #[account(
        mut,
        constraint = token_vault_0.key() == pool_state.load()?.token_vault_0
    )]
    pub token_vault_0: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_1
    #[account(
        mut,
        constraint = token_vault_1.key() == pool_state.load()?.token_vault_1
    )]
    pub token_vault_1: Box<InterfaceAccount<'info, TokenAccount>>,
/*
here instead of initialising two token program we can use Interface<'info, T>
*/
    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,

    /// Token program 2022
    pub token_program_2022: Program<'info, Token2022>,
        /*
        ALTERNATE: use interface program 
        */

    /// The mint of token vault 0
    //here we check the address of vault mint matches with token_vault_mint
    #[account(
            address = token_vault_0.mint
    )]
    pub vault_0_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token vault 1 
    //  here we check the address of vault mint matches with token_vault_mint
    #[account(
            address = token_vault_1.mint
    )]
    pub vault_1_mint: Box<InterfaceAccount<'info, Mint>>,
    // remaining account
    // #[account(
    //     seeds = [
    //         POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
    //         pool_state.key().as_ref(),
    //     ],
    //     bump
    // )]
    // pub tick_array_bitmap: AccountLoader<'info, TickArrayBitmapExtension>,
}

pub fn increase_liquidity_v2<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, IncreaseLiquidityV2<'info>>,
    liquidity: u128,
    amount_0_max: u64,
    amount_1_max: u64,
    base_flag: Option<bool>,
) -> Result<()> {
    increase_liquidity(
        &ctx.accounts.nft_owner,
        &ctx.accounts.pool_state,
        &mut ctx.accounts.protocol_position,
        &mut ctx.accounts.personal_position,
        &ctx.accounts.tick_array_lower,
        &ctx.accounts.tick_array_upper,
        &ctx.accounts.token_account_0.to_account_info(),
        &ctx.accounts.token_account_1.to_account_info(),
        &ctx.accounts.token_vault_0.to_account_info(),
        &ctx.accounts.token_vault_1.to_account_info(),
        &ctx.accounts.token_program,
        Some(&ctx.accounts.token_program_2022),
        Some(ctx.accounts.vault_0_mint.clone()),
        Some(ctx.accounts.vault_1_mint.clone()),
        &ctx.remaining_accounts,
        liquidity,
        amount_0_max,
        amount_1_max,
        base_flag,
    )
}
