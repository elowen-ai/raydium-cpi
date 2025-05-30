use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    memo::Memo,
    metadata::Metadata,
    token::Token,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use raydium_cp_swap::program::RaydiumCpSwap;

#[derive(Accounts)]
pub struct LockCpLiquidity<'info> {
    /// CHECK: the authority of token vault that cp is locked
    #[account(
        seeds = [
            crate::LOCK_CP_AUTH_SEED.as_bytes(),
        ],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    /// Pay to create account lamports
    #[account(mut)]
    pub payer: Signer<'info>,

    /// who want to lock liquidity
    pub liquidity_owner: Signer<'info>,

    /// CHECK: locked liquidity allow who to collect fee
    pub fee_nft_owner: UncheckedAccount<'info>,

    /// Create a unique fee nft mint
    #[account(
        init,
        mint::decimals = 0,
        mint::authority = authority,
        payer = payer,
        mint::token_program = token_program,
    )]
    pub fee_nft_mint: Box<InterfaceAccount<'info, Mint>>,

     /// CHECK: Token account where fee nft will be minted to, init by locking program
     #[account(
        mut,
        // init,
        // associated_token::mint = fee_nft_mint,
        // associated_token::authority = fee_nft_owner,
        // token::token_program = token_program,
    )]
    pub fee_nft_account: UncheckedAccount<'info>,

    /// CHECK: Indicates which pool the locked liquidity belong to
    #[account()]
    pub pool_state: UncheckedAccount<'info>,

    /// Store the locked information of liquidity
    #[account(
        init,
        seeds = [
            LOCKED_LIQUIDITY_SEED.as_bytes(),
            fee_nft_mint.key().as_ref(),
        ],
        bump,
        payer = payer,
        space = LockedCpLiquidityState::LEN
    )]
    pub locked_liquidity: Box<Account<'info, LockedCpLiquidityState>>,

    /// The mint of liquidity token
    /// address = pool_state.lp_mint
    #[account(mut)]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    /// liquidity owner lp token account
    #[account(
        mut,
        token::mint = lp_mint,
        token::authority = liquidity_owner,
    )]
    pub liquidity_owner_lp: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Locked lp token deposit to
    #[account(
        init_if_needed,
        associated_token::mint = lp_mint,
        associated_token::authority = authority,
        payer = payer,
        token::token_program = token_program,
    )]
    pub locked_lp_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_0
    /// address = pool_state.token_0_vault
    #[account(mut)]
    pub token_0_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_1
    /// address = pool_state.token_1_vault
    #[account(mut)]
    pub token_1_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// To store metaplex metadata
    /// CHECK: Safety check performed inside function body
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// Sysvar for token mint and ATA creation
    pub rent: Sysvar<'info, Rent>,

    /// Program to create the new account
    pub system_program: Program<'info, System>,

    /// Program to create/transfer mint/token account
    pub token_program: Program<'info, Token>,

    /// Program to create an ATA for receiving fee NFT
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Program to create NFT metadata accunt
    /// CHECK: Metadata program address constraint applied
    pub metadata_program: Program<'info, Metadata>,
}

#[derive(Accounts)]
pub struct CollectCpFee<'info> {
    /// CHECK: the authority of token vault that cp is locked
    #[account(
        seeds = [
            crate::LOCK_CP_AUTH_SEED.as_bytes(),
        ],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    /// Fee nft owner who is allowed to receive fees
    pub fee_nft_owner: Signer<'info>,

    /// Fee token account
    #[account(
        token::mint = locked_liquidity.fee_nft_mint,
        token::authority = fee_nft_owner,
        constraint = fee_nft_account.amount == 1
    )]
    pub fee_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Store the locked the information of liquidity
    #[account(
        mut,
        constraint = locked_liquidity.fee_nft_mint == fee_nft_account.mint
    )]
    pub locked_liquidity: Account<'info, LockedCpLiquidityState>,

    /// cpmm program
    pub cpmm_program: Program<'info, RaydiumCpSwap>,

    /// CHECK: cp program vault and lp mint authority
    #[account(
        seeds = [
            raydium_cp_swap::AUTH_SEED.as_bytes(),
        ],
        bump,
        seeds::program = cpmm_program.key(),
    )]
    pub cp_authority: UncheckedAccount<'info>,

    /// CHECK: CPMM Pool state account
    #[account(
        mut,
        address = locked_liquidity.pool_id
    )]
    pub pool_state: UncheckedAccount<'info>,

    /// lp mint
    /// address = pool_state.lp_mint
    #[account(mut)]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The token account for receive token_0
    #[account(
        mut,
        token::mint = token_0_vault.mint,
    )]
    pub recipient_token_0_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The token account for receive token_1
    #[account(
        mut,
        token::mint = token_1_vault.mint,
    )]
    pub recipient_token_1_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_0
    /// address = pool_state.token_0_vault
    #[account(mut)]
    pub token_0_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_1
    /// address = pool_state.token_1_vault
    #[account(mut)]
    pub token_1_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The mint of token_0 vault
    #[account(
        address = token_0_vault.mint
    )]
    pub vault_0_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token_1 vault
    #[account(
        address = token_1_vault.mint
    )]
    pub vault_1_mint: Box<InterfaceAccount<'info, Mint>>,

    /// locked lp token account
    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = authority,
        token::token_program = token_program,
    )]
    pub locked_lp_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// token Program
    pub token_program: Program<'info, Token>,

    /// Token program 2022
    pub token_program_2022: Program<'info, Token2022>,

    /// memo program
    #[account()]
    pub memo_program: Program<'info, Memo>,
}
