use scrypto::prelude::*;

#[blueprint]
mod pandao_praposal {
    use std::path::Component;

    pub struct TokenWeightProposal {
        /// A simple string representing the current proposal.
        pub title: String,
    
        /// A detailed description of the proposal.
        pub description: String,
    
        /// The total weight of votes in favor of the proposal.
        pub voted_for: Decimal,
    
        /// The total weight of votes against the proposal.
        pub voted_against: Decimal,
    
        /// The minimum amount of quorum required for this proposal to pass.
        pub minimum_quorum: Decimal,
    
        /// The time when the proposal ends.
        pub end_time: UtcDateTime,
    
        /// The time when the proposal starts.
        pub start_time: UtcDateTime,
    
        /// The address of the owner token.
        pub owner_token_address: ResourceAddress,
    
        /// The address of the voter badge.
        pub voter_badge_address: ResourceAddress,
    
        // A mapping of addresses to their respective vote weights.
        // pub votes: HashMap<Address, Decimal>,

        pub address_issued_bonds_to_sell : Option<ComponentAddress>,
        pub target_xrd_amount: Option<Decimal>,

        pub vote_caster_addresses : HashSet<ComponentAddress>,

        
    }

    impl TokenWeightProposal  {
        
        pub fn new(
            title: String ,
            description : String , 
            minimun_quorum: u8,
            start_time: scrypto::time::UtcDateTime,
            end_time: scrypto::time::UtcDateTime,
            owner_badge_address: ResourceAddress,
            voter_badge_address: ResourceAddress,
            address_issued_bonds_to_sell : Option<ComponentAddress>,
            target_xrd_amount : Option<Decimal>
        ) -> (Global<TokenWeightProposal >, GlobalAddressReservation) {
            let (address_reservation, _) =
                Runtime::allocate_component_address(TokenWeightProposal ::blueprint_id());

            let proposal = TokenWeightProposal {
                title,
                description,
                voted_for:0.into(),
                voted_against:0.into(),
                minimum_quorum:minimun_quorum.into(),
                end_time,
                start_time,
                owner_token_address:owner_badge_address,
                voter_badge_address,
                address_issued_bonds_to_sell,
                target_xrd_amount,
                vote_caster_addresses : HashSet::new()
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(address_reservation.clone())
            .globalize();

            (proposal, address_reservation)
        }

        pub fn vote(&mut self, token: Bucket, against: bool) -> Bucket {

            assert_eq!(
                token.resource_address(),
                self.voter_badge_address,
                "wrong voting token supplied"
            );

            let amount = token.amount();

            if against {
                
                self.voted_against += amount;

                token

            } else {

                self.voted_for += amount;

                token

            }
        }

        pub fn get_address_issued_bonds(&self) -> ComponentAddress {

            if let Some(address_issued_bonds_to_sell) =self.address_issued_bonds_to_sell{
                address_issued_bonds_to_sell
            }else{
                panic!("address issued bonds to sell is None")
            }
        }

        pub fn get_target_xrd_amount(&self) -> Decimal {

            if let Some(target_xrd_amount) =self.target_xrd_amount{
                target_xrd_amount
            }else{
                panic!("target xrd amount is None")
            }
        }

        pub fn get_vote_caster_addresses(&self) -> HashSet<ComponentAddress> {
            self.vote_caster_addresses.clone()
        }

        pub fn set_vote_caster_address(&mut self, address : ComponentAddress){
            self.vote_caster_addresses.insert(address);
        }

    }
}
