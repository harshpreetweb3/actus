use scrypto::prelude::*;
#[blueprint]
mod pandao_praposal {
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
        ) -> (Global<TokenWeightProposal >, GlobalAddressReservation) {
            let (address_reservation, _) =
                Runtime::allocate_component_address(TokenWeightProposal ::blueprint_id());

            let proposal = TokenWeightProposal {
                title:title,
                description:description,
                voted_for:0.into(),
                voted_against:0.into(),
                minimum_quorum:minimun_quorum.into(),
                end_time:end_time,start_time:start_time,
                owner_token_address:owner_badge_address,
                voter_badge_address:voter_badge_address,
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

    }
}
