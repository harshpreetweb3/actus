// blue print for
mod events;
use crate::events::*;
mod proposal;
use scrypto::prelude::*;

#[blueprint]
#[events(PandaoEvent, DaoEvent, TokenWightedDeployment, DaoType, EventType)]
mod radixdao {

    use proposal::pandao_praposal::TokenWeightProposal;

    pub struct TokenWeigtedDao {
        current_praposal: Option<Global<TokenWeightProposal>>,

        dao_token: Vault,

        organization_name: String,

        shares: Vault,

        dao_token_address: ResourceAddress,

        owner_token_addresss: ResourceAddress,

        token_price: Decimal,

        buy_back_price: Decimal,
    }

    impl TokenWeigtedDao {
        pub fn initiate(
            //community name | community title
            organization_name: String,

            //there must be a function to refill the token supply
            token_supply: i32,

            //divisibility must be zero if there is no fractional ownership
            divisibility: u8,

            // define the price of 1 token for xrd
            token_price: Decimal,

            //price at which community would take it's token back
            token_buy_back_price: Decimal,

            //logo to represent community
            org_ico_url: String,

            //logo representing a token
            power_token_url: String,

            //elaborate community
            description: String,
        ) -> (Global<TokenWeigtedDao>, Bucket) {
            // reserve an address for the DAO component
            let (address_reservation, _) =
                Runtime::allocate_component_address(TokenWeigtedDao::blueprint_id());

            let owner_badge_description = format!("{}'s owner badge", &organization_name);

            // ! create a owner role, this role is only for changing the praposal and inserting a new praposal

            // this is not seen by me as of yet
            // ! Being a DAO, proposal can be created by any person

            // * owner badge creation
            // * Moreover this is fungible token (IT MUST BE NON_FUNGIBLE)

            // Owner Badge Creation: Creates a non-divisible owner badge with metadata containing
            // the organization's name and icon URL.
            // ! This badge likely represents administrative control over the DAO.

            // * THERE CANNOT BE ADMINISTRATIVE CONTROL

            let owner_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(0)
                .metadata(metadata!(
                    init{
                        "name"=>owner_badge_description,locked;
                        "icon_url" => Url::of(&org_ico_url), locked;
                    }
                ))
                .mint_initial_supply(1)
                .into();

            // create nft to be sold for voting purpose
            let dao_token_description = format!("{} voting share", &organization_name);

            let voting_power_tokens: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(divisibility)
                .metadata(metadata!(init{
                    "name"=>dao_token_description,locked ;
                    "organization name" => organization_name.as_str() , locked ;
                    "icon_url" => Url::of(&power_token_url), locked;
                }))
                .mint_initial_supply(token_supply)
                .into();

            let dao_token_address = voting_power_tokens.resource_address();

            let owner_token_addresss = owner_badge.resource_address();

            let component = Self {
                token_price: token_price.clone(),

                organization_name: organization_name.clone(),

                dao_token_address: dao_token_address.clone(),

                owner_token_addresss: owner_token_addresss.clone(),

                current_praposal: None,

                dao_token: Vault::with_bucket(voting_power_tokens),

                buy_back_price: token_buy_back_price.clone(),

                shares: Vault::new(XRD),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                owner_token_addresss.clone()
            ))))
            .with_address(address_reservation.clone())
            .globalize();

            let component_address = component.address();

            // create a metadata for event named TokenWeightedDeployment
            let event_metadata = TokenWightedDeployment {
                component_address,

                token_address: dao_token_address,

                owner_token_address: owner_token_addresss,

                community_name: organization_name,

                community_image: org_ico_url,

                token_price,

                token_buy_back_price,

                description,

                total_token: token_supply,

                token_image: power_token_url,
            };

            // emit event | event emission
            Runtime::emit_event(PandaoEvent {
                event_type: EventType::DEPLOYMENT,

                dao_type: DaoType::TokenWeight,

                component_address,

                meta_data: DaoEvent::TokenWeightedDEployment(event_metadata),
            });

            //Event emission in blockchain systems is primarily used for transparency,
            //enabling tracking of significant actions and changes in state,
            //and facilitating communication between smart contracts and external applications.

            // TODO: THERE WOULD BE INTRIGUING TO SEE WHERE THIS EMISSION IS BEING USED?

            (component, owner_badge)
        }

        // TODO: OBTAIN A COMMUNITY TOKEN

        pub fn obtain_community_token(
            &mut self,
            mut xrd: Bucket,
            token_amount: Decimal,
            // minter_address: Option<String>,
        ) -> (Bucket, Bucket) {
            //TODO: given amount >= amount needed to purchase community tokens as per required amount
            assert!((self.token_price * token_amount) <= xrd.amount());

            let collected_xrd = xrd.take(self.token_price * token_amount);

            let power_share = self.dao_token.take(token_amount);

            let amount_paid = self.token_price * token_amount;

            self.shares.put(collected_xrd);

            //emit event

            let event_metadata = TokenWeightBuyToken {
                amount: token_amount,

                resource_address: self.dao_token_address,

                amount_paid,

                current_component_share: self.shares.amount(), // current component's collected xrd
            };

            let component_address = Runtime::global_address();

            Runtime::emit_event(PandaoEvent {
                event_type: EventType::TOKEN_BOUGHT,

                dao_type: DaoType::TokenWeight,

                component_address,

                meta_data: DaoEvent::TokenWeightedTokenPurchase(event_metadata),
            });

            (xrd, power_share)
        }

        pub fn withdraw_power(&mut self, voting_power: Bucket) -> Bucket {

            // put the voting power back
            assert!(
                self.current_praposal.is_none(),
                "token can not be sold when there is an active praposal or incomplete proposal"
            );

            let power_amount = voting_power.amount();

            self.dao_token.put(voting_power);

            let event_metadata = TokenWeightBuyToken {

                amount: power_amount,

                resource_address: self.dao_token_address.clone(),

                amount_paid: power_amount * self.buy_back_price,

                current_component_share: self.shares.amount()

            };

            let component_address = Runtime::global_address();

            Runtime::emit_event(PandaoEvent {
                event_type: EventType::TOKEN_SELL,
                dao_type: DaoType::TokenWeight,
                component_address,
                meta_data: DaoEvent::TokenWeightedTokenPurchase(event_metadata),
            });

            self.shares.take(power_amount * self.buy_back_price)

        }

        pub fn create_praposal(
            &mut self,
            title: String,
            description: String,
            minimun_quorum: u8,
            start_time: scrypto::time::UtcDateTime,     
            end_time: scrypto::time::UtcDateTime,
        ) -> Global<crate::proposal::pandao_praposal::TokenWeightProposal> {

            use crate::proposal::pandao_praposal::TokenWeightProposal;

            assert!(
                self.current_praposal.is_none(),
                "there is already a praposal underway , can not create more"
            );

            let (global_proposal_component, _) = TokenWeightProposal::new(
                title.clone(),
                description.clone(),
                minimun_quorum,
                start_time,
                end_time,
                self.owner_token_addresss.clone(),
                self.dao_token_address.clone(),
            );

            // global_proposal_component.callme("string".into()) ;
            let start_time_ts: i64 = start_time.to_instant().seconds_since_unix_epoch;
            let end_time_ts: i64 = end_time.to_instant().seconds_since_unix_epoch;
            let praposal_metadata = PraposalMetadata {
                title: title,
                description: description,
                minimum_quorum: minimun_quorum.into(),
                end_time_ts,
                start_time_ts,
                owner_token_address: self.owner_token_addresss.clone(),
                component_address: global_proposal_component.address(),
            };
            let component_address = Runtime::global_address();

            Runtime::emit_event(PandaoEvent {
                event_type: EventType::PRAPOSAL,
                dao_type: DaoType::TokenWeight,
                meta_data: DaoEvent::PraposalDeployment(praposal_metadata),
                component_address,
            });

            // assign proposal to component
            self.current_praposal = Some(global_proposal_component);
            global_proposal_component
        }

        pub fn execute_proposal(&mut self) {
            if let Some(proposal) = self.current_praposal {
                let praposal_metadata = PraposalExecute {
                    praposal_address: proposal.address(),
                };
                let component_address = Runtime::global_address();

                Runtime::emit_event(PandaoEvent {
                    event_type: EventType::EXECUTE_PROPOSAL,
                    dao_type: DaoType::TokenWeight,
                    meta_data: DaoEvent::ProposalExecute(praposal_metadata),
                    component_address,
                });
                self.current_praposal = None;
            } else {
                assert!(false, "there is no current active proposal")
            }
        }

        
    }
}

//*initialize
// resim call-function package_sim1p4nk9h5kw2mcmwn5u2xcmlmwap8j6dzet7w7zztzz55p70rgqs4vag TokenWeigtedDao initiate "Panjab Investment DAO" 100 0 5 2 "https://pbs.twimg.com/profile_images/1643159245389713408/47gnTbms_200x200.jpg" "https://pbs.twimg.com/profile_images/1548373397289455616/OFhGnboY_400x400.jpg" "This is a DAO for managing community projects"
// resim call-function package_sim1p4nk9h5kw2mcmwn5u2xcmlmwap8j6dzet7w7zztzz55p70rgqs4vag TokenWeigtedDao initiate "Panjab Investment DAO" 100 0 5 2 "https://pbs.twimg.com/profile_images/1643159245389713408/47gnTbms_200x200.jpg" "https://pbs.twimg.com/profile_images/1548373397289455616/OFhGnboY_400x400.jpg" "This is a DAO for managing community projects" --manifest instantiate_pandao.rtm

// account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma
// component_sim1czwnyl3pfn955s45a2js64w8zjlptwz4y3w4wwwl944rk2l2ceapsc

//*obtain_token
// resim call-method component_sim1czwnyl3pfn955s45a2js64w8zjlptwz4y3w4wwwl944rk2l2ceapsc obtain_token resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:5 1
// resim call-method component_sim1cpwu4wc6rg0am8l9prnh2lzqkk6hue6stzqhdx48rzvek2mmm5vp0p obtain_token resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:5 1
    
// create_proposal
// resim call-method component_sim1czwnyl3pfn955s45a2js64w8zjlptwz4y3w4wwwl944rk2l2ceapsc create_praposal "Panda Fridays" "Introduce a fun Panda-themed event every Friday." 10 1694774400 1695052800
// resim call-method 02012345 create_praposal "Panda Fridays" "Introduce a fun Panda-themed event every Friday." 10 1694774400 1695052800
// resim call-method component_sim1czwnyl3pfn955s45a2js64w8zjlptwz4y3w4wwwl944rk2l2ceapsc create_praposal "Proposal Title" "Description" 10 {"year":2024,"month":9,"day_of_month":15,"hour":0,"minute":0} {"year":2024,"month":9,"day_of_month":20,"hour":23,"minute":59}

//*test-net
// txid_tdx_2_1uyf8cmuvgd0kredvzg2fh4mfff79zaj5t6trmnwkaet3n36ahkcq6dcwgq
// package_tdx_2_1phtjjxh563e37wtp008re5zlpau7v7y8xudpy9mmw4cp22k56frszt
// component_tdx_2_1czddjhay2jv0e03h78mapw2k3y8mnqmn47hxyz7svm4tm76wf8azmq
// account_tdx_2_1285pq36tg53usvdhvwjlu40plmzf6dj8uyhrqxp6j0kvpl2znqtt54
// resource_tdx_2_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxtfd2jc
//*owner badge
// resource_tdx_2_1tkam4tmrj2xl4hry7gsvpx0sq56xljudckmxxhr72tehqj4mq3rzna

//*community native token
//resource_tdx_2_1thp48upl275dm4ar0675we2ew83fn04k3cg7ca57swlzumctk4xvgc


//*manifest to call obtain_token*/
// CALL_METHOD
//             Address("account_tdx_2_1285pq36tg53usvdhvwjlu40plmzf6dj8uyhrqxp6j0kvpl2znqtt54")
//             "withdraw"
//             Address("resource_tdx_2_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxtfd2jc")
//             Decimal("5")
//         ;

// TAKE_FROM_WORKTOP
//             Address("resource_tdx_2_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxtfd2jc")
//             Decimal("5")
//             Bucket("bucket1")
//         ;

// CALL_METHOD
//         Address("component_tdx_2_1czddjhay2jv0e03h78mapw2k3y8mnqmn47hxyz7svm4tm76wf8azmq")
//         "obtain_token"
//         Bucket("bucket1")
//         Decimal("1")
//         ;

// CALL_METHOD
//             Address("account_tdx_2_1285pq36tg53usvdhvwjlu40plmzf6dj8uyhrqxp6j0kvpl2znqtt54")
//             "deposit_batch"
//             Expression("ENTIRE_WORKTOP")
//         ;

//*create_propoosal
// CALL_METHOD
// Address("component_tdx_2_1czddjhay2jv0e03h78mapw2k3y8mnqmn47hxyz7svm4tm76wf8azmq")
// "create_praposal"
// "should we purchase laptop or not"
// "abc"
// 6u8
// Tuple(
// 2024u32 ,
// 9u8 ,
// 9u8 ,
// 20u8 ,
// 22u8 ,
// 22u8)
// Tuple(
// 2024u32 ,
// 9u8 ,
// 12u8 ,
// 1u8 ,
// 1u8 ,
// 1u8)
// ;

//*cast_a_vote

// CALL_METHOD
//             Address("account_tdx_2_1285pq36tg53usvdhvwjlu40plmzf6dj8uyhrqxp6j0kvpl2znqtt54")
//             "withdraw"
//             Address("resource_tdx_2_1thp48upl275dm4ar0675we2ew83fn04k3cg7ca57swlzumctk4xvgc") // community token
//             Decimal("1")
//         ;

// TAKE_FROM_WORKTOP
//             Address("resource_tdx_2_1thp48upl275dm4ar0675we2ew83fn04k3cg7ca57swlzumctk4xvgc")
//             Decimal("1")
//             Bucket("bucket2")
//         ;

// CALL_METHOD
//         Address("component_tdx_2_1czddjhay2jv0e03h78mapw2k3y8mnqmn47hxyz7svm4tm76wf8azmq")
//         "vote"
//         Bucket("bucket2")
//         true
//         ;

// CALL_METHOD
//             Address("account_tdx_2_1285pq36tg53usvdhvwjlu40plmzf6dj8uyhrqxp6j0kvpl2znqtt54")
//             "deposit_batch"
//             Expression("ENTIRE_WORKTOP")
//         ;

// CALL_METHOD
//             Address("component_tdx_2_1czddjhay2jv0e03h78mapw2k3y8mnqmn47hxyz7svm4tm76wf8azmq")
//             "execute_proposal"
//         ;

//* vote is being casted multiple times 
//* execute proposal and try to check proposal creation by an account other that community creator
//* (I BELIEVE COMMUNITY CREATOR IS RESPONSIBLE FOR PROPOSAL CREATION) 

//* TEST-CASES:
//* CAN ANY COMMUNITY MEMBER CREATE A PROPOSAL ?    (OR ONLY COMMUNITY CREATOR WILL CREATE)
//  yes! any member can create a proposal 

//* CAN ANY COMMUNITY MEMBER EXECUTE THE PROPOSAL ? (OR ONLY PROPOSAL CREATOR WILL EXECUTE)
//* DO WE REALLY NEED TO HAVE A COMMUNITY TOKEN FOR PROPOSAL CREATION?*/ 

//*hustlepreet secondry account 
// account_tdx_2_128e6fmjkhjqx0n8h9562rrvstl883wq22pzea4ucnnx0762ptlch4s

//*missing functions
// become_a_dao_member



