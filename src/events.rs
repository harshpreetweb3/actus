use scrypto::prelude::*;

#[allow(non_camel_case_types)]
#[derive(ScryptoSbor, ScryptoEvent)]
pub enum EventType {

    PROPOSAL_TO_MINT_MORE_TOKENS,

    PROPOSAL_TO_CHANGE_TOKEN_PRICE,
    
    DEPLOYMENT,

    TOKEN_BOUGHT,

    TOKEN_SELL,

    PRAPOSAL,

    PROPOSAL_TO_PURCHASE_BOND,

    VOTE,

    EXECUTE_PROPOSAL,

    TREASURY_CONTRIBUTION,

    ZERO_COUPON_BOND_CREATION,

    QUORUM_NOT_MET_AND_FAILED,

    PRICE_CHANGE_QUORUM_NOT_MET_AND_FAILED,

    QUORUM_MET_AND_SUCCESS,

    PRICE_CHANGE_QUORUM_MET_AND_SUCCESS,

    CHECK_BOND_ISSUER_BALANCE,

    LIQUIDATED_COLLATERAL,

    COLLATERAL_LIQUIDATION_FAILED,

    TAKE_OUT_INVESTED_XRDs,

    PUT_IN_MONEY_PLUS_INTEREST,


    PUT_IN_LESS_MONEY_PLUS_INTEREST,

    CLAIM_INVESTED_XRDs_PLUS_INTEREST,

    FAILED_CLAIM_INVESTED_XRDs_PLUS_INTEREST,

    FORCE_TRANSFER_OF_FUNDS,

    COLLATERAL_GOT_BACK,

    FAILED_IN_GETTING_BACK_COLLATERAL,

    EXECUTIVE_BADGE_MINTED,

    EXECUTIVE_APPOINTED,

    WITHDRAWAL_REQUESTED_SUCCESSFULLY,

    WITHDRAWAL_REQUEST_FAILED,

    WITHDRAWAL_REQUEST_APPROVED,

    WITHDRAWAL_REQUEST_DENIED,

    FUNDS_WITHDRAWN,

    FUNDS_NOT_WITHDRAWN,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub enum DaoType {
    Investment,
    Insurance
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct TokenWightedDeployment {

    pub component_address: ComponentAddress,

    pub token_address: ResourceAddress,

    pub owner_token_address: ResourceAddress,

    pub community_name: String,

    pub community_image: String,

    pub token_price: Decimal,

    pub token_buy_back_price: Decimal,

    pub description: String,

    pub total_token: i32,

    pub token_image: String,

    pub tags : Vec<String>,

    pub purpose : String,

    pub proposal_creation_right : ProposalCreationRight,

    pub token_name : String,

    // pub executive_token_address: ResourceAddress
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct TokenWeightBuyToken {

    pub amount: Decimal,

    pub resource_address: ResourceAddress,

    pub amount_paid: Decimal,

    pub current_component_share: Decimal,

}

// #[derive(ScryptoSbor, ScryptoEvent)]
// pub struct TokenWeightBuyToken {

//     pub amount: Decimal,

//     pub resource_address: ResourceAddress,

//     pub amount_paid: Decimal,

//     pub current_component_share: Decimal,

// }

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PraposalMetadata {
    // a simple string representing current praposal
    pub title:String,
    pub description:String,
    // represent the minimum amount of quorm requires for this praposal to pass
    pub minimum_quorum: Decimal,
    pub end_time_ts: i64,
    pub start_time_ts: i64,
    pub owner_token_address: ResourceAddress,
    pub component_address: ComponentAddress, // votes:HashMap<Address,Decimal>
    pub address_issued_bonds_to_sell : Option<ComponentAddress>,
    pub target_xrd_amount : Option<Decimal>,
    pub proposal_creator_address : Option<ComponentAddress>,
    pub amount_of_tokens_should_be_minted : Option<usize>,
    pub proposal_id : usize,
    pub governance_token_or_owner_token_address : ResourceAddress,
    pub token_type : VotingType,
    pub desired_token_price : Option<Decimal>,
    pub desired_token_buy_back_price : Option<Decimal>
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub enum DaoEvent {

    ProposalExecute(PraposalExecute),

    TokenWeightedDEployment(TokenWightedDeployment),

    TokenWeightedTokenPurchase(TokenWeightBuyToken),

    PraposalDeployment(PraposalMetadata),

    PraposalVote(ProposalVote),

    TreasuryContribution(TreasuryContribution), 

    ZeroCouponBondCreation(ZeroCouponBondCreation),

    ProposalQuorumNotMet(ProposalQuorumNotMet), // New event type

    PriceChangeProposalQuorumNotMet(PriceChangeProposalQuorumNotMet),

    PriceChangeProposalQuorumMet(PriceChangeProposalQuorumMet),

    ProposalQuorumMet(ProposalQuorumMet),

    CheckBondIssuerBalance(CheckBondIssuerBalanceEvent),

    LiquidatedCollateral(LiquidatedCollateralEvent),

    CollateralLiquidationFailed(CollateralLiquidationFailedEvent),

    TakeOutInvestedXRDs(TakeOutInvestedXRDsEvent),

    PutInMoneyPlusInterest(PutInMoneyPlusInterestEvent),

    ClaimInvestedXRDsPlusInterest(ClaimInvestedXRDsPlusInterestEvent),

    ClaimInvestedXRDsPlusInterestError(ClaimInvestedXRDsPlusInterestErrorEvent),

    ForceTransferFunds(ForceTransferFunds),

    GetBackTheCollateral(GetBackTheCollateralEvent),

    // ProposalCreationRightEveryone,

    // ProposalCreationRightTokenHolderThreshold(Decimal),

    // ProposalCreationRightAdmin

    ExecutiveBadgeMinted(ExecutiveBadgeMinted),

    ExecutiveAppointed(ExecutiveAppointed),

    WithdrawalRequested(WithdrawalRequested),

    WithdrawalRequestFailed(WithdrawalRequested),

    WithdrawalRequestApproved(WithdrawalRequestApproved),

    WithdrawalRequestDenied(WithdrawalRequestDenied),

    FundsWithdrawn(FundsWithdrawn),

    FundsNotWithdrawn(FundsNotWithdrawn)

}

// #[derive(ScryptoSbor, ScryptoEvent)]
// pub enum ProposalRightEvent {

//     ProposalCreationRightEveryone,

//     ProposalCreationRightTokenHolderThreshold(Decimal),

//     ProposalCreationRightAdmin

// }

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PraposalExecute{
    pub praposal_address : ComponentAddress ,
    pub proposal_id : usize
    // pub purchased_bond_address : Option<ResourceAddress>,
    // pub purchased_amount : Decimal
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ProposalVote{
    pub praposal_address : ComponentAddress,
    pub voting_amount : Decimal,
    pub againts: bool,
    pub voter_address : ComponentAddress,
    pub proposal_id : usize
}

// create an event for community_creation
#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PandaoEvent {

    pub event_type: EventType,

    pub dao_type: DaoType,

    pub component_address: ComponentAddress,
    
    pub meta_data: DaoEvent
}
// create an event for community_creation
#[derive(ScryptoSbor, ScryptoEvent)]
pub struct BoughtToken {
    pub component_address: ComponentAddress,
    pub user_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(ScryptoSbor, Debug)]
pub struct TreasuryContribution {
    pub contributor: ComponentAddress,
    pub amount: Decimal,
    pub timestamp: u64,
}

#[allow(non_camel_case_types)]
#[derive(ScryptoSbor, Clone, Debug, PartialEq, Eq)]
pub enum ProposalCreationRight {
    EVERYONE,
    TOKEN_HOLDER_THRESHOLD(Decimal),
    ADMIN,
}

#[allow(non_camel_case_types)]
#[derive(ScryptoSbor, Clone, Debug, PartialEq, Eq)]
pub enum VotingType {
    ResourceHold,
    Equality,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ZeroCouponBondCreation {
    pub component_address: ComponentAddress,
    pub contract_type: String,
    pub contract_role: String,
    pub contract_identifier: String,
    pub nominal_interest_rate: Decimal,
    pub currency: String,
    pub initial_exchange_date: u64,
    pub maturity_date: u64,
    pub notional_principal: Decimal,
    pub discount: u64,
    pub bond_position: String,
    pub price: u64,
    pub number_of_bonds: Decimal,
    pub creator_address: ComponentAddress,
    pub collateral_resource_address: ResourceAddress,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ProposalQuorumNotMet {
    pub proposal_id: usize,
    pub minimum_quorum: usize,
    pub number_of_voters: usize,
    pub bond_creator_address : ComponentAddress,
    pub contract_identity : String,
    pub proposal_type : EventType
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PriceChangeProposalQuorumNotMet {
    pub proposal_id: usize,
    pub minimum_quorum: usize,
    pub number_of_voters: usize,
    pub desired_price : Decimal,
    pub desired_token_buy_back_price : Decimal,
    pub proposal_type : EventType
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ProposalQuorumMet {
    pub proposal_id: usize,
    pub minimum_quorum: usize,
    pub number_of_voters: usize,
    pub bond_creator_address : ComponentAddress,
    pub contract_identity : String,
    pub proposal_type : EventType
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PriceChangeProposalQuorumMet {
    pub proposal_id: usize,
    pub minimum_quorum: usize,
    pub number_of_voters: usize,
    pub desired_token_price : Decimal,
    pub desired_token_buy_back_price : Decimal,
    pub proposal_type : EventType
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct CheckBondIssuerBalanceEvent {
    pub bond_creator_address: ComponentAddress,
    pub balance: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ForceTransferFunds {
    pub bond_creator_address: ComponentAddress,
    pub required_amount: Decimal,
    pub bond_component_balance : Decimal,
    pub transferred_amount_to_community_vault : Decimal
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct LiquidatedCollateralEvent {
    pub bond_creator_address: ComponentAddress,
    pub liquidated_amount: Decimal,
    pub collateral_resource_address : ResourceAddress
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct CollateralLiquidationFailedEvent {
    pub bond_creator_address: ComponentAddress,
    // pub liquidated_amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct GetBackTheCollateralEvent {
    pub bond_creator_address: ComponentAddress,

    pub is_given_money_claimed_by_community : bool,
    // pub liquidated_amount: Decimal,
    pub resource_address_of_collateral : ResourceAddress,

    pub message : String
}

// #[derive(ScryptoSbor, ScryptoEvent)]
// pub struct GettingCollateralBackEvent {
//     pub bond_creator_address: ComponentAddress,
//     pub message: String,
//     pub resource_address_of_collateral : ResourceAddress
// }

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct TakeOutInvestedXRDsEvent {
    pub bond_creator_address: ComponentAddress,
    pub taken_out_amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PutInMoneyPlusInterestEvent {
    pub bond_creator_address: ComponentAddress,
    pub amount_getting_deposited : Decimal,
    pub amount_required_by_the_community : Decimal,
    pub amount_taken_by_the_community : Decimal,
    pub extra_amount_given_back_to_the_sender : Decimal,
    pub more_xrd_amount_required_by_the_community : Decimal,
    pub collateral_given_back : bool
}


#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ClaimInvestedXRDsPlusInterestEvent {
    pub bond_creator_address: ComponentAddress,
    pub claimed_amount: Decimal,
    pub amount_required_by_the_community : Decimal,
    pub collateral_liquidated : bool
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ClaimInvestedXRDsPlusInterestErrorEvent {
    pub bond_creator_address: ComponentAddress,
    pub required_amount_by_the_community: Decimal,
    pub balance_of_bond_issuer : Decimal,
    pub collateral_liquidated : bool,
    pub collateral_resource_address : ResourceAddress,
    pub liquidated_amount : Decimal
}


#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ExecutiveBadgeMinted {
    pub name: String,
    pub number: u64,
    pub resource_address : ResourceAddress,
    pub local_id : NonFungibleLocalId,
    pub global_id : NonFungibleGlobalId
}


#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ExecutiveAppointed {
    pub account_address: ComponentAddress,
    pub resource_address : ResourceAddress,
    pub local_id : NonFungibleLocalId
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct WithdrawalRequested {
    pub requester_address: ComponentAddress,
    pub requested_amount: Decimal,
    pub max_withdrawal_amount : Decimal,
    pub withdrawal_request_occured : bool,
    pub requester_id : Option<u64>
}

// #[derive(ScryptoSbor, ScryptoEvent)]
// pub struct WithdrawalRequestFailed {
//     pub requester_address: ComponentAddress,
//     pub amount: Decimal,
//     pub max_withdrawal_amount : 
// }

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct WithdrawalRequestApproved {
    pub approver_address: ComponentAddress,
    pub user_address: ComponentAddress,
    pub is_approved : bool,
    pub request_id : u64
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct WithdrawalRequestDenied {
    pub disapprover_address: ComponentAddress,
    pub user_address: ComponentAddress,
    pub is_approved : bool,
    pub request_id : u64
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct FundsWithdrawn {
    pub user_address: ComponentAddress,
    pub requested_amount: Decimal,
    pub request_id : u64
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct FundsNotWithdrawn {
    pub user_address: ComponentAddress,
    pub requested_amount: Decimal,
    pub request_id : u64
}

