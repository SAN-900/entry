use anchor_lang::prelude::*;

#[error_code]
pub enum EntryError {
    #[msg("Fee exceeds the configured maximum")]
    FeeTooHigh,
    #[msg("Event price must be greater than zero")]
    InvalidPrice,
    #[msg("Supply must be greater than zero")]
    InvalidSupply,
    #[msg("Resale cap must be at least 100%")]
    InvalidResaleCap,
    #[msg("Royalty cannot exceed the resale premium")]
    InvalidRoyalty,
    #[msg("Event is not active")]
    EventNotActive,
    #[msg("Event supply has been exhausted")]
    SoldOut,
    #[msg("Asking price exceeds the face-price resale cap")]
    ResaleCapExceeded,
    #[msg("Ticket is already listed")]
    AlreadyListed,
    #[msg("Listing does not belong to the signer")]
    NotListingSeller,
    #[msg("Ticket has already been checked in")]
    AlreadyCheckedIn,
    #[msg("Signer is not an authorized gate operator")]
    UnauthorizedGate,
    #[msg("Config initialization must be signed by the program upgrade authority")]
    UnauthorizedConfigInitializer,
    #[msg("Program data account does not belong to this program")]
    InvalidProgramData,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Ticket identifier is outside the event supply")]
    TicketIdOutOfRange,
    #[msg("Compressed asset identifier does not match the tree and leaf nonce")]
    InvalidAssetId,
    #[msg("Merkle proof contains too many accounts or writable/signing accounts")]
    InvalidMerkleProofAccounts,
    #[msg("Only the event organizer may register inventory tickets")]
    UnauthorizedOrganizer,
    #[msg("Signer is not the owner recorded for this ticket")]
    NotTicketOwner,
    #[msg("Ticket must be purchased before it can be listed for resale")]
    TicketNotPurchased,
    #[msg("Payment mint must be canonical six-decimal USDC on mainnet or devnet")]
    InvalidPaymentMint,
    #[msg("Bubblegum proof fields do not describe one consistent ticket leaf")]
    InvalidBubblegumProof,
    #[msg("Ticket leaf freeze flags do not describe the required frozen/unfrozen transition")]
    InvalidBubblegumFlags,
    #[msg("Wallet cannot buy its own ticket")]
    SelfPurchase,
    #[msg("Invalid event check-in bitmap account")]
    InvalidCheckInAccount,
    #[msg("Ticket has already been checked in")]
    TicketAlreadyCheckedIn,
    #[msg("Check-in batch exceeds the supported maximum")]
    CheckInBatchTooLarge,
    #[msg("Gatekeeper must sign this purchase")]
    UnauthorizedGatekeeper,
    #[msg("Tree must be private, delegated to the event PDA, and empty")]
    InvalidEventTree,
    #[msg("Event is inactive")]
    EventInactive,
    #[msg("Metadata field exceeds the supported size")]
    InvalidMetadata,
    #[msg("Ownership registry tree or proof is invalid")]
    InvalidRegistryProof,
    #[msg("Ownership registry tree must be distinct from the ticket tree")]
    InvalidRegistryTree,
}
