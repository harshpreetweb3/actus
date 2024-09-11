use scrypto::prelude::*;

#[allow(non_camel_case_types)]
#[derive(ScryptoSbor, ScryptoEvent)]
pub enum EventType {

    DEPLOYMENT,

    TOKEN_BOUGHT,

    TOKEN_SELL,

    PRAPOSAL,

    VOTE,

    EXECUTE_PROPOSAL,

    BOND_ISSUANCE,  // New event type for bond issuance
    BOND_PURCHASE,

}

#[derive(ScryptoSbor, ScryptoEvent)]
pub enum DaoType {

    TokenWeight

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
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct TokenWeightBuyToken {

    pub amount: Decimal,

    pub resource_address: ResourceAddress,

    pub amount_paid: Decimal,

    pub current_component_share: Decimal,

}

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
}
// ... (keep DaoType as is) ...

#[derive(ScryptoSbor, Debug)]
pub struct BondDetails {
    pub price: Decimal,
    pub maturity_date: u64,
    pub bond_name: String,
    pub bond_symbol: String,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct BondPurchase {
    pub bond_address: ResourceAddress,
    pub amount: Decimal,
    pub price_paid: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub enum DaoEvent {

    ProposalExecute(PraposalExecute),

    TokenWeightedDEployment(TokenWightedDeployment),

    TokenWeightedTokenPurchase(TokenWeightBuyToken),

    PraposalDeployment(PraposalMetadata),

    PraposalVote(ProposalVote),

    BondIssuance(BondDetails),
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PraposalExecute{
    pub praposal_address : ComponentAddress ,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ProposalVote{
    pub praposal_address : ComponentAddress ,
    pub voting_amount : Decimal ,
    pub againts:bool
}

// create an event for community_creation
#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PandaoEvent {

    pub event_type: EventType,

    pub dao_type: DaoType,

    pub component_address: ComponentAddress,
    
    pub meta_data: DaoEvent,
}

// create an event for community_creation
#[derive(ScryptoSbor, ScryptoEvent)]
pub struct BoughtToken {
    pub component_address: ComponentAddress,
    pub user_address: ResourceAddress,
    pub amount: Decimal,
}
