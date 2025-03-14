use anchor_lang::prelude::*;

declare_id!("4PonUp1nPEzDPnRMPjTqufLT3f37QuBJGk1CVnsTXx7x");

#[program]
pub mod simple_escrow {
    use super::*;

    pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        escrow_id: u64,
        trade_id: u64,
        amount: u64,
        sequential: bool,
        sequential_escrow_address: Option<Pubkey>,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        escrow.escrow_id = escrow_id;
        escrow.trade_id = trade_id;
        escrow.seller = ctx.accounts.seller.key();
        escrow.buyer = ctx.accounts.buyer.key();
        escrow.amount = amount;
        escrow.sequential = sequential;
        escrow.sequential_escrow_address = sequential_escrow_address;
        escrow.dispute_evidence_hash_buyer = None;
        escrow.dispute_evidence_hash_seller = None;
        escrow.state = EscrowState::Pending;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(escrow_id: u64, trade_id: u64, amount: u64, sequential: bool, sequential_escrow_address: Option<Pubkey>)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    /// CHECK: Buyer account is just used for escrow parameters
    pub buyer: AccountInfo<'info>,
    #[account(
        init,
        payer = seller,
        space = 165, // Exact serialized size
        seeds = [b"escrow", escrow_id.to_le_bytes().as_ref(), trade_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Escrow {
    pub escrow_id: u64,                        // 8 bytes
    pub trade_id: u64,                         // 8 bytes
    pub seller: Pubkey,                        // 32 bytes
    pub buyer: Pubkey,                         // 32 bytes
    pub amount: u64,                           // 8 bytes
    pub sequential: bool,                      // 1 byte
    pub sequential_escrow_address: Option<Pubkey>, // 1 + 32 = 33 bytes (Some)
    pub dispute_evidence_hash_buyer: Option<[u8; 32]>,   // 1 + 32 = 33 bytes (Some)
    pub dispute_evidence_hash_seller: Option<[u8; 32]>,  // 1 + 32 = 33 bytes (Some)
    pub state: EscrowState,                    // 1 byte
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum EscrowState {
    Pending,
    Completed,
    Cancelled,
}