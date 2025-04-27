use crate::error::ErrorCode;
use crate::states::*;
use crate::{libraries::tick_math, util};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
// use solana_program::{program::invoke_signed, system_instruction};
#[derive(Accounts)]
pub struct CreatePool<'info> {
    /// Address paying to create the pool. Can be anyone
    //Eg: 95DU3vnFgNCNjADq9k4KYewMXoNviYZ76NNjZwqRhCLR
    #[account(mut)]
    pub pool_creator: Signer<'info>,

    //Amm Config: E64NGkDLLCdQ2yFNPcavaKptrEgmiQaNykUuLC1Qgwyp
    /// Which config the pool belongs to.
    pub amm_config: Box<Account<'info, AmmConfig>>,

    /// Initialize an account to store the pool state
    // Eg: 8dqmW9E4x56udR2hRR4qtPa54UMpWJ7m32zjmfcj72AP
    #[account(
        init,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            token_mint_0.key().as_ref(),
            token_mint_1.key().as_ref(),
        ],
        bump,
        payer = pool_creator,
        space = PoolState::LEN
    )]
    pub pool_state: AccountLoader<'info, PoolState>,

    //Eg: 91xDatdjG4C7Pkc4FmW1cKayb7HSVQEswtupr8xppump
    /// Token_0 mint, the key must be smaller then token_1 mint.
    /*
    In the Raydium CLMM contract, the check mint_a < mint_b during pool initialization is likely to ensure that the token mint addresses are in a consistent order, which can be crucial for calculations and operations within the contract. This consistent ordering helps maintain the integrity of the pool's state and avoids potential issues with how token addresses are referenced in the contract's logic. The "shorter" aspect likely refers to the fact that the mint address of token A is typically smaller than the mint address of token B in terms of lexicographical ordering.
    */
    //here we have an interface account because the mint can be from both token program or token-2022 program
    ///has_one = target
    ///mint::token_program = token_program_0

    #[account(
        constraint = token_mint_0.key() < token_mint_1.key(),
        mint::token_program = token_program_0
    )]
    pub token_mint_0: Box<InterfaceAccount<'info, Mint>>,

    //Eg: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
    /// Token_1 mint
    #[account(
        mint::token_program = token_program_1
    )]
    pub token_mint_1: Box<InterfaceAccount<'info, Mint>>,

    //Eg: Fcy6GZQqhHcXRV2WSMQxKMGfmkV5aUaHnSFWkHF7dcr5
    /// Token_0 vault for the pool
    #[account(
        init,
        //seeds are derived like mint of the token, pool_state pub key, pool_vault_seed
        seeds =[
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            token_mint_0.key().as_ref(),
        ],
        bump,
        ///payer for this account is the signer
        payer = pool_creator,
        ///here we specify the mint of this vault
        token::mint = token_mint_0,
        ///authority is given to pool_state(pda), so whenver the token needs to be transfered from this vault it would be signed by the pool_state i.e. program owned account
        token::authority = pool_state,
        //the token program to which the token belonfs to 
        token::token_program = token_program_0,
    )]
    pub token_vault_0: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Token_1 vault for the pool
    // Eg: 5MgB9o5jbkggp7iqj76dEySCcoyyYFuKF22iQHBa9FTi
    /*
     token::mint = token_mint_1
     this tells anchor when you create the pda, then token mint address should be token_mint_1

             token::authority = pool_state,

                     token::token_program = token_program_1,


    */
    #[account(
        init,
        seeds =[
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            token_mint_1.key().as_ref(),
        ],
        bump,
        payer = pool_creator,
        token::mint = token_mint_1,
        token::authority = pool_state,
        token::token_program = token_program_1,
    )]
    pub token_vault_1: Box<InterfaceAccount<'info, TokenAccount>>,

    //Eg: 4R7frgunatG7EWZQTR1XUJubu9kpw1s9fKFBLwt6iDVA
    /// Initialize an account to store oracle observations
    #[account(
        init,
        seeds = [
            OBSERVATION_SEED.as_bytes(),
            pool_state.key().as_ref(),
        ],
        bump,
        payer = pool_creator,
        space = ObservationState::LEN
    )]
    pub observation_state: AccountLoader<'info, ObservationState>,

    //Eg: 29xtQPYayLHbNx8wmMjewGWZ42MVRXS5ZfiSCNfetNPZ
    /// Initialize an account to store if a tick array is initialized.
    #[account(
        init,
        seeds = [
            POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
            pool_state.key().as_ref(),
        ],
        bump,
        payer = pool_creator,
        space = TickArrayBitmapExtension::LEN
    )]
    pub tick_array_bitmap: AccountLoader<'info, TickArrayBitmapExtension>,

    /// Spl token program or token program 2022
    // Eg: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
    pub token_program_0: Interface<'info, TokenInterface>,

    /*
    What is TokenInterface?
TokenInterface is a trait/interface that lets Anchor work with either:

🟢 SPL Token program (Tokenkeg...)

🟠 Token2022 program (TokenzQd...)

=> Interface is used to reference a program account
So your program supports either one, and the client can pass in whichever token program it's using.
    */
    //Eg: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
    /// Spl token program or token program 2022
    pub token_program_1: Interface<'info, TokenInterface>,

    //Eg: 11111111111111111111111111111111
    /// To create a new program account
    ///We pass the system_program because it facilates the creation of a new program account
    pub system_program: Program<'info, System>,

    //Eg:SysvarRent111111111111111111111111111111111
    /// Sysvar for program account
    // it is used to facilate the on-chain account creation
    pub rent: Sysvar<'info, Rent>,
    // remaining account
    // #[account(
    //     seeds = [
    //     SUPPORT_MINT_SEED.as_bytes(),
    //     token_mint_0.key().as_ref(),
    // ],
    //     bump
    // )]
    // pub support_mint0_associated: Account<'info, SupportMintAssociated>,

    // #[account(
    //     seeds = [
    //     SUPPORT_MINT_SEED.as_bytes(),
    //     token_mint_1.key().as_ref(),
    // ],
    //     bump
    // )]
    // pub support_mint1_associated: Account<'info, SupportMintAssociated>,
}

pub fn create_pool(ctx: Context<CreatePool>, sqrt_price_x64: u128, open_time: u64) -> Result<()> {
    let mint0_associated_is_initialized = util::support_mint_associated_is_initialized(
        &ctx.remaining_accounts,
        &ctx.accounts.token_mint_0,
    )?;
    let mint1_associated_is_initialized = util::support_mint_associated_is_initialized(
        &ctx.remaining_accounts,
        &ctx.accounts.token_mint_1,
    )?;
    if !(util::is_supported_mint(&ctx.accounts.token_mint_0, mint0_associated_is_initialized)
        .unwrap()
        && util::is_supported_mint(&ctx.accounts.token_mint_1, mint1_associated_is_initialized)
            .unwrap())
    {
        return err!(ErrorCode::NotSupportMint);
    }
    let block_timestamp = solana_program::clock::Clock::get()?.unix_timestamp as u64;
    /// this checks that the block_timestamp should be greater than open_time
    require_gt!(block_timestamp, open_time);
    let pool_id = ctx.accounts.pool_state.key();
    //Here we load the pool_state as THE MUT REF, it should be called once when account is being initialised
    let mut pool_state = ctx.accounts.pool_state.load_init()?;

    let tick = tick_math::get_tick_at_sqrt_price(sqrt_price_x64)?;
    ///cfg -> conditional compilation attribute
    #[cfg(feature = "enable-log")]
    msg!(
        "create pool, init_price: {}, init_tick:{}",
        sqrt_price_x64,
        tick
    );
    // init observation
    ctx.accounts
        .observation_state
        .load_init()?
        .initialize(pool_id)?;
    ///we can get bump for a specific pda by writing context.bumps.pda_name
    let bump = ctx.bumps.pool_state;
    /*
    So, calling .initialize(...) writes actual data into the account:
    Sets values like tick_current, token_mint_0, token_vault_0, etc.
    Saves them into the PDA’s memory so other instructions can read them later
    */
    pool_state.initialize(
        bump,
        sqrt_price_x64,
        0,
        tick,
        ctx.accounts.pool_creator.key(),
        ctx.accounts.token_vault_0.key(),
        ctx.accounts.token_vault_1.key(),
        ctx.accounts.amm_config.as_ref(),
        ctx.accounts.token_mint_0.as_ref(),
        ctx.accounts.token_mint_1.as_ref(),
        ctx.accounts.observation_state.key(),
    )?;
    /*
    Here we add the pool_id to the tick_array_bitmap
    */
    ctx.accounts
        .tick_array_bitmap
        .load_init()?
        .initialize(pool_id);

    //the emit! this allows us to write data in program logs
    emit!(PoolCreatedEvent {
        token_mint_0: ctx.accounts.token_mint_0.key(),
        token_mint_1: ctx.accounts.token_mint_1.key(),
        tick_spacing: ctx.accounts.amm_config.tick_spacing,
        pool_state: ctx.accounts.pool_state.key(),
        sqrt_price_x64,
        tick,
        token_vault_0: ctx.accounts.token_vault_0.key(),
        token_vault_1: ctx.accounts.token_vault_1.key(),
    });
    Ok(())
}
