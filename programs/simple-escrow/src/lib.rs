use anchor_lang::prelude::*;

declare_id!("71axVYtoJyNVYVtJzVVcKEJLETtvenx2yJ89eMP1a2PK");

#[program]
pub mod simple_escrow {
    use super::*;

    pub fn create_escrow(ctx: Context<CreateEscrow>, escrow_id: u64, amount: u64) -> Result<()> {
        // Debug logs to track execution
        msg!("Allocated space: {}", ctx.accounts.escrow.to_account_info().data_len());
        msg!("Escrow size (memory): {}", std::mem::size_of::<Escrow>());
        msg!("Starting initialization");

        let escrow = &mut ctx.accounts.escrow;
        escrow.escrow_id = escrow_id;
        escrow.seller = ctx.accounts.seller.key();
        escrow.amount = amount;

        msg!("Escrow data: {:?}", escrow);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(escrow_id: u64)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        init,
        payer = seller,
        space = 8 + 8 + 32 + 8, // Discriminator (8) + escrow_id (8) + seller (32) + amount (8) = 56 bytes
        seeds = [b"escrow", escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Debug)]
pub struct Escrow {
    pub escrow_id: u64,
    pub seller: Pubkey,
    pub amount: u64,
}