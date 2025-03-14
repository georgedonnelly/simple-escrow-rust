use anchor_lang::prelude::*;

declare_id!("4PonUp1nPEzDPnRMPjTqufLT3f37QuBJGk1CVnsTXx7x");

mod constants {
    // Maximum amount allowed (100 USDC)
    pub const MAX_AMOUNT: u64 = 100_000_000; // 6 decimals for USDC

    // Fee percentage (1%)
    pub const FEE_BASIS_POINTS: u64 = 100; // 1% = 100 basis points

    // Dispute bond percentage (5%)
    pub const DISPUTE_BOND_BASIS_POINTS: u64 = 500; // 5% = 500 basis points

    // Deadlines
    pub const DEPOSIT_DEADLINE_MINUTES: i64 = 15; // 15 minutes from order initiation
    pub const FIAT_DEADLINE_MINUTES: i64 = 30;    // 30 minutes after funding
    pub const DISPUTE_RESPONSE_DEADLINE_HOURS: i64 = 72; // 72 hours to respond to dispute
    pub const ARBITRATION_DEADLINE_HOURS: i64 = 168;     // 7 days for arbitrator to make decision

    // Other constants
    pub const SECONDS_PER_MINUTE: i64 = 60;
    pub const SECONDS_PER_HOUR: i64 = 3600;


    // Hardcoded arbitrator Pubkey (base58: GGrXhNVxUZXaA2uMopsa5q23aPmoNvQF14uxqo8qENUr)
    pub const ARBITRATOR_BYTES: [u8; 32] = 
    [0xe2, 0xef, 0x04, 0xd8, 0x35, 0x5b, 0x03, 0xd1, 0xdb, 0x14, 0x87, 0x9e, 0x38, 0x84, 0x4d, 0x64, 0x74, 0xc7, 0x8b, 0xe4, 0xbe, 0x4e, 0x31, 0xb4, 0xae, 0xfe, 0x13, 0xc8, 0x2f, 0xdb, 0xdb, 0x2b];
}

// Custom error codes
#[error_code]
pub enum EscrowError {
    #[msg("Invalid amount: Zero or negative")]
    InvalidAmount,
    #[msg("Amount exceeds maximum (100 USDC)")]
    ExceedsMaximum,
    #[msg("Unauthorized caller")]
    Unauthorized,
    #[msg("Deposit deadline expired")]
    DepositDeadlineExpired,
    #[msg("Fiat payment deadline expired")]
    FiatDeadlineExpired,
    #[msg("Invalid state transition")]
    InvalidState,
    #[msg("Missing sequential escrow address")]
    MissingSequentialAddress,
    #[msg("Already in terminal state")]
    TerminalState,
    #[msg("Fee calculation error")]
    FeeCalculationError,
    #[msg("Insufficient funds to cover principal and fee")]
    InsufficientFunds,
    #[msg("Dispute bond amount incorrect")]
    IncorrectBondAmount,
    #[msg("Dispute response deadline expired")]
    ResponseDeadlineExpired,
    #[msg("Evidence hash missing or invalid")]
    InvalidEvidenceHash,
    #[msg("Duplicate evidence submission")]
    DuplicateEvidence,
    #[msg("Arbitration deadline expired")]
    ArbitrationDeadlineExpired,
    #[msg("Missing dispute bond")]
    MissingDisputeBond,
    #[msg("Invalid resolution explanation")]
    InvalidResolutionExplanation,
    #[msg("Required bump seed not found")]
    BumpNotFound,
}

#[program]
pub mod simple_escrow {
    use super::*;
    use constants::*;

    pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        escrow_id: u64,
        trade_id: u64,
        amount: u64,
        sequential: bool,
        sequential_escrow_address: Option<Pubkey>,
    ) -> Result<()> {

        // debugging borsch
        msg!("Escrow size (memory): {}", std::mem::size_of::<Escrow>());
        msg!("Allocated space: {}", ctx.accounts.escrow.to_account_info().data_len()); // Should be 337
        msg!("Starting initialization");

        //--
        // Validate amount
        require!(amount > 0, EscrowError::InvalidAmount);
        require!(amount <= MAX_AMOUNT, EscrowError::ExceedsMaximum);

        // Validate sequential address if sequential is true
        if sequential {
            require!(sequential_escrow_address.is_some(), EscrowError::MissingSequentialAddress);
        }

        // Calculate fee (1% of principal)
        let fee = amount
            .checked_mul(FEE_BASIS_POINTS)
            .ok_or(EscrowError::FeeCalculationError)?
            .checked_div(10000)
            .ok_or(EscrowError::FeeCalculationError)?;

        // Calculate deposit deadline (current time + 15 minutes)
        let current_time = Clock::get()?.unix_timestamp;
        let deposit_deadline = current_time
            .checked_add(DEPOSIT_DEADLINE_MINUTES * SECONDS_PER_MINUTE)
            .ok_or(EscrowError::FeeCalculationError)?;

        // Store seller/buyer/arbitrator info
        let seller_key = ctx.accounts.seller.key();
        let buyer_key = ctx.accounts.buyer.key();
        //--

        let arbitrator = Pubkey::new_from_array(ARBITRATOR_BYTES);

        let escrow = &mut ctx.accounts.escrow;
        escrow.escrow_id = escrow_id;
        escrow.trade_id = trade_id;
        escrow.seller = seller_key;
        escrow.buyer = buyer_key;
        escrow.amount = amount;
        escrow.sequential = sequential;
        escrow.sequential_escrow_address = sequential_escrow_address;
        escrow.dispute_evidence_hash_buyer = None;
        escrow.dispute_evidence_hash_seller = None;
        escrow.state = EscrowState::Created;
        // added
        escrow.deposit_deadline = deposit_deadline; //
        escrow.dispute_initiated_time = None;
        escrow.fee = fee;
        escrow.fiat_deadline = 0;
        escrow.fiat_paid = false;
        escrow.counter = 0;
        // x5
        escrow.arbitrator = arbitrator;
        escrow.dispute_initiator = None;
        escrow.dispute_resolution_hash = None;

        // Log the escrow data after initialization
        // msg!("Escrow data: {:?}", escrow);

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
        space = 8 + std::mem::size_of::<Escrow>(),
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
    // added
    pub deposit_deadline: i64,
    pub dispute_initiated_time: Option<i64>,
    pub fee: u64,
    pub fiat_deadline: i64,
    pub fiat_paid: bool,
    pub counter: u64,
    // x5
    pub arbitrator: Pubkey,
    pub dispute_initiator: Option<Pubkey>,
    pub dispute_resolution_hash: Option<[u8; 32]>
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub enum EscrowState {
    Created,
    Funded,
    Released,
    Cancelled,
    Disputed,
    Resolved
}