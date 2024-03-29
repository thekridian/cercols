use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_spl::{
    token::{Mint, TokenAccount, Token}, 
    metadata::{MetadataAccount, Metadata, MasterEditionAccount, mpl_token_metadata::instructions::TransferV1CpiBuilder, TokenRecordAccount},
    associated_token::AssociatedToken,
};

declare_id!("FjZVUx8ergLAEaB1Mucqizhk9XtQTZpnAffS5FBT2h4c");

#[program]
pub mod cercols {
    use super::*;

    pub fn init_pool(ctx: Context<InitPool>, swap_fee_lamports: u64) -> Result<()> {
        // Initialize a pool
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.user.key();
        pool.collection_mint = ctx.accounts.collection_mint.key();
        pool.swap_fee_lamports = swap_fee_lamports;
        pool.size = 0;
        pool.pool_bump = ctx.bumps.pool;
        pool.nft_auth_bump = ctx.bumps.nft_authority;
        
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>) -> Result<()> {
        // Deposit collection items in the pool
        let pool = &mut ctx.accounts.pool;
        pool.size = pool.size.checked_add(1).unwrap();

        let mut transfer_cpi = TransferV1CpiBuilder::new(&ctx.accounts.metadata_program);

        let nft_token = &ctx.accounts.nft_token.to_account_info();
        let user = &ctx.accounts.user.to_account_info();
        let nft_custody = &ctx.accounts.nft_custody.to_account_info();
        let nft_mint = &ctx.accounts.nft_mint.to_account_info();
        let nft_metadata = &ctx.accounts.nft_metadata.to_account_info();
        let nft_edition = &ctx.accounts.nft_edition.to_account_info();
        let source_token_record = &ctx.accounts.source_token_record.to_account_info();
        let destination_token_record = &ctx.accounts.destination_token_record.to_account_info();
        // let auth_rules_program = &ctx.accounts.auth_rules_program.to_account_info();
        // let auth_rules = &ctx.accounts.auth_rules.to_account_info();
        
        transfer_cpi
        .token(nft_token)
        .token_owner(user)
        .destination_token(nft_custody)
        .destination_owner(&ctx.accounts.nft_authority)
        .mint(nft_mint)
        .metadata(nft_metadata)
        .edition(Some(nft_edition))
        .token_record(Some(source_token_record))
        .destination_token_record(Some(destination_token_record))
        .authority(user)
        .payer(user)
        .system_program(&ctx.accounts.system_program)
        .sysvar_instructions(&ctx.accounts.sysvar_instructions)
        .spl_token_program(&ctx.accounts.token_program)
        .spl_ata_program(&ctx.accounts.associated_token_program)
        // .authorization_rules_program(None)
        // .authorization_rules(None)
        .amount(1);

        transfer_cpi.invoke()?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.size = pool.size.checked_sub(1).unwrap();

        let mut transfer_cpi = TransferV1CpiBuilder::new(&ctx.accounts.metadata_program);

        let nft_token = &ctx.accounts.nft_token.to_account_info();
        let user = &ctx.accounts.user.to_account_info();
        let nft_custody = &ctx.accounts.nft_custody.to_account_info();
        let nft_mint = &ctx.accounts.nft_mint.to_account_info();
        let nft_metadata = &ctx.accounts.nft_metadata.to_account_info();
        let nft_edition = &ctx.accounts.nft_edition.to_account_info();
        let source_token_record = &ctx.accounts.source_token_record.to_account_info();
        let destination_token_record = &ctx.accounts.destination_token_record.to_account_info();
        // let auth_rules_program = &ctx.accounts.auth_rules_program.to_account_info();
        // let auth_rules = &ctx.accounts.auth_rules.to_account_info();
        
        transfer_cpi
        .token(nft_custody)
        .token_owner(&ctx.accounts.nft_authority)
        .destination_token(nft_token)
        .destination_owner(user)
        .mint(nft_mint)
        .metadata(nft_metadata)
        .edition(Some(nft_edition))
        .token_record(Some(source_token_record))
        .destination_token_record(Some(destination_token_record))
        .authority(&ctx.accounts.nft_authority)
        .payer(user)
        .system_program(&ctx.accounts.system_program)
        .sysvar_instructions(&ctx.accounts.sysvar_instructions)
        .spl_token_program(&ctx.accounts.token_program)
        .spl_ata_program(&ctx.accounts.associated_token_program)
        // .authorization_rules_program(None)
        // .authorization_rules(None)
        .amount(1);

        const PREFIX_SEED: &'static [u8] = b"nft_authority";
        let signer = [PREFIX_SEED, &pool.key().to_bytes(), &[pool.nft_auth_bump]];

        transfer_cpi.invoke_signed(&[&signer])?;

        // close the ata
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            anchor_spl::token::CloseAccount {
                account: nft_custody.to_account_info(),
                destination: user.to_account_info(),
                authority: ctx.accounts.nft_authority.to_account_info()
            },
            &[&signer]
        ))?;

        Ok(())
    }

    pub fn swap(ctx: Context<Swap>) -> Result<()> {
        // transfer the token the user is swapping in
        let mut transfer_cpi = TransferV1CpiBuilder::new(&ctx.accounts.metadata_program);

        let nft_token = &ctx.accounts.nft_token.to_account_info();
        let user = &ctx.accounts.user.to_account_info();
        let nft_custody = &ctx.accounts.nft_custody.to_account_info();
        let nft_mint = &ctx.accounts.nft_mint.to_account_info();
        let nft_metadata = &ctx.accounts.nft_metadata.to_account_info();
        let nft_edition = &ctx.accounts.nft_edition.to_account_info();
        let source_token_record = &ctx.accounts.source_token_record.to_account_info();
        let destination_token_record = &ctx.accounts.destination_token_record.to_account_info();
        // let auth_rules_program = &ctx.accounts.auth_rules_program.to_account_info();
        // let auth_rules = &ctx.accounts.auth_rules.to_account_info();
        
        // transfer the incoming token from the user
        transfer_cpi
        .token(nft_token)
        .token_owner(user)
        .destination_token(nft_custody)
        .destination_owner(&ctx.accounts.nft_authority)
        .mint(nft_mint)
        .metadata(nft_metadata)
        .edition(Some(nft_edition))
        .token_record(Some(source_token_record))
        .destination_token_record(Some(destination_token_record))
        .authority(user)
        .payer(user)
        .system_program(&ctx.accounts.system_program)
        .sysvar_instructions(&ctx.accounts.sysvar_instructions)
        .spl_token_program(&ctx.accounts.token_program)
        .spl_ata_program(&ctx.accounts.associated_token_program)
        // .authorization_rules_program(None)
        // .authorization_rules(None)
        .amount(1);

        transfer_cpi.invoke()?;

        // choose a pseudorandom token from the pool to swap back to them
        // pick a number between 1 and pool.size
        // use that to choose the right NFT mint
        // from NFT mint, derive token, metadata, edition, etc accounts
        // transfer the token to the user
        // close the ata of the outgoing token
        Ok(())
    }

    // pub fn close_pool(ctx: Context<ClosePool>) -> Result<()> {
    //     Ok(())
    // }
}

#[account]
pub struct PoolState { // 8
    /// The creator of the pool (32)
    authority: Pubkey, // 32
    /// The verified collection address of the NFT (32)
    collection_mint: Pubkey, // 32
    /// The fee in lamports for swapping in the pool
    swap_fee_lamports: u64, // 8
    /// The number of items currently in the pool
    size: u32, // 4
    /// The bump of pool details PDA (1)
    pool_bump: u8, // 1
    /// The bump of NFT authority PDA (1)
    nft_auth_bump: u8 // 1
    // TOTAL: 86
}

#[derive(Accounts)]
#[instruction(swap_fee_lamports: u64)]
pub struct InitPool<'info> {
    #[account(
        init, 
        payer = user, 
        seeds = [b"cercols_pool", collection_mint.key().as_ref(), user.key().as_ref()], 
        bump, 
        space = 86
    )]
    pub pool: Account<'info, PoolState>,

    #[account(mint::decimals = 0)]
    pub collection_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: This account is not read or written
    #[account(
        seeds = [b"nft_authority", pool.key().as_ref()],
        bump
    )]
    pub nft_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut, 
        seeds = [
            b"cercols_pool", 
            pool.collection_mint.as_ref(), 
            pool.authority.as_ref()
        ], 
        bump = pool.pool_bump
    )]
    pub pool: Account<'info, PoolState>,

    #[account(mint::decimals = 0, constraint = nft_mint.supply == 1)]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut, 
        associated_token::mint = nft_mint, 
        associated_token::authority = user, 
        constraint = nft_token.amount == 1
    )]
    pub nft_token: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"metadata", Metadata::id().as_ref(), nft_mint.key().as_ref()],
        seeds::program = Metadata::id(),
        bump,
        constraint = nft_metadata.collection.as_ref().unwrap().verified,
        constraint = nft_metadata.collection.as_ref().unwrap().key == pool.collection_mint
    )]
    pub nft_metadata: Box<Account<'info, MetadataAccount>>,

    #[account(
        seeds = [b"metadata",
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"edition"
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub nft_edition: Box<Account<'info, MasterEditionAccount>>,

    /// CHECK: This account is not read or written
    #[account(seeds = [b"nft_authority", pool.key().as_ref()], bump = pool.nft_auth_bump)]
    pub nft_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = user, 
        associated_token::mint = nft_mint, 
        associated_token::authority = nft_authority
    )]
    pub nft_custody: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"metadata", 
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"token_record",
            nft_token.key().as_ref(),
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub source_token_record: Account<'info, TokenRecordAccount>,
    
    /// CHECK: account constraints checked in account trait
    #[account(
        mut,
        seeds = [b"metadata", 
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"token_record",
            nft_custody.key().as_ref(),
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub destination_token_record: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,

    // /// CHECK: account constraints checked in account trait
    // #[account(address = mpl_token_auth_rules::id())]
    // pub auth_rules_program: UncheckedAccount<'info>,

    // /// CHECK: account constraints checked in account trait
    // #[account(owner = mpl_token_auth_rules::id())]
    // pub auth_rules: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut, 
        seeds = [
            b"cercols_pool", 
            pool.collection_mint.as_ref(), 
            pool.authority.as_ref()
        ], 
        bump = pool.pool_bump
    )]
    pub pool: Account<'info, PoolState>,

    #[account(mint::decimals = 0, constraint = nft_mint.supply == 1)]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut, 
        associated_token::mint = nft_mint, 
        associated_token::authority = user, 
        constraint = nft_token.amount == 0
    )]
    pub nft_token: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"metadata", Metadata::id().as_ref(), nft_mint.key().as_ref()],
        seeds::program = Metadata::id(),
        bump,
        constraint = nft_metadata.collection.as_ref().unwrap().verified,
        constraint = nft_metadata.collection.as_ref().unwrap().key == pool.collection_mint
    )]
    pub nft_metadata: Box<Account<'info, MetadataAccount>>,

    #[account(
        seeds = [b"metadata",
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"edition"
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub nft_edition: Box<Account<'info, MasterEditionAccount>>,

    /// CHECK: This account is not read or written
    #[account(mut, seeds = [b"nft_authority", pool.key().as_ref()], bump = pool.nft_auth_bump)]
    pub nft_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = nft_mint, 
        associated_token::authority = nft_authority
    )]
    pub nft_custody: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"metadata", 
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"token_record",
            nft_custody.key().as_ref(),
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub source_token_record: Account<'info, TokenRecordAccount>,

    /// CHECK: account constraints checked in account trait
    #[account(
        mut,
        seeds = [b"metadata", 
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"token_record",
            nft_token.key().as_ref(),
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub destination_token_record: UncheckedAccount<'info>,

    #[account(mut, constraint = user.key() == pool.authority)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(
        seeds = [
            b"cercols_pool", 
            pool.collection_mint.as_ref(), 
            pool.authority.as_ref()
        ], 
        bump = pool.pool_bump
    )]
    pub pool: Account<'info, PoolState>,

    #[account(mint::decimals = 0, constraint = nft_mint.supply == 1)]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut, 
        associated_token::mint = nft_mint, 
        associated_token::authority = user, 
        constraint = nft_token.amount == 0
    )]
    pub nft_token: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"metadata", Metadata::id().as_ref(), nft_mint.key().as_ref()],
        seeds::program = Metadata::id(),
        bump,
        constraint = nft_metadata.collection.as_ref().unwrap().verified,
        constraint = nft_metadata.collection.as_ref().unwrap().key == pool.collection_mint
    )]
    pub nft_metadata: Box<Account<'info, MetadataAccount>>,

    #[account(
        seeds = [b"metadata",
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"edition"
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub nft_edition: Box<Account<'info, MasterEditionAccount>>,

    /// CHECK: This account is not read or written
    #[account(mut, seeds = [b"nft_authority", pool.key().as_ref()], bump = pool.nft_auth_bump)]
    pub nft_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = nft_mint, 
        associated_token::authority = nft_authority
    )]
    pub nft_custody: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"metadata", 
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"token_record",
            nft_custody.key().as_ref(),
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub source_token_record: Account<'info, TokenRecordAccount>,

    /// CHECK: account constraints checked in account trait
    #[account(
        mut,
        seeds = [b"metadata", 
            Metadata::id().as_ref(),
            nft_mint.key().as_ref(),
            b"token_record",
            nft_token.key().as_ref(),
        ],
        seeds::program = Metadata::id(),
        bump
    )]
    pub destination_token_record: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct ClosePool {}
