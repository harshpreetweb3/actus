// blue print for
mod events;
use crate::events::*;
mod proposal;
use scrypto::prelude::*;
mod zerocouponbond;

#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct ExecutiveBadge {
    executive_number: u64,
    executive_name: String,
}

#[derive(ScryptoSbor, Clone)]
pub struct ApprovalDetails {
    approval_giver_addresses: Vec<ComponentAddress>,
    denial_giver_addresses: Vec<ComponentAddress>,
    approvals: u32,
    denials: u32,
}

#[derive(ScryptoSbor, Clone)] // Enum to represent approval responses
pub enum ApprovalResponse {
    Approve,
    Deny,
}

#[blueprint]
#[events(PandaoEvent, DaoEvent, TokenWightedDeployment, DaoType, EventType)]
mod radixdao {

    

    use super::*;
    use crate::zerocouponbond::BondDetails;
    use proposal::pandao_praposal::TokenWeightProposal;
    use scrypto::address;
    use std::collections::HashMap;
    use zerocouponbond::zerocouponbond::ZeroCouponBond;


    enable_method_auth! {
        roles {
            executive => updatable_by: [OWNER];
        },

        methods {
            // initiate => restrict_to: [OWNER];
            mint_executive_badge => restrict_to: [OWNER];
            make_an_executive => restrict_to: [OWNER];
            transfer_xrds_to_withdrawal_seeker => PUBLIC;
            approve_or_deny_withdrawal_request => restrict_to: [executive, OWNER];
            // withdraw_money => PUBLIC;
            obtain_community_token => PUBLIC;
            request_withdrawal => PUBLIC;
            create_praposal => PUBLIC;
            vote => PUBLIC;
            execute_proposal => PUBLIC;
            create_zero_coupon_bond => PUBLIC;
            purchase_bond => PUBLIC;
            sell_bond => PUBLIC;
            check_bond_maturity => PUBLIC;
            get_bond_details => PUBLIC;
            send_money_to_dao_treasury => PUBLIC;
            withdraw_power => PUBLIC;
            // get_usd_price => PUBLIC;
            // get_proposal_id => PUBLIC;
            create_proposal_to_mint_more_dao_tokens => PUBLIC;
            mint_more_tokens => PUBLIC;
            set_price => PUBLIC;
            execute_proposal_to_mint_more_tokens => PUBLIC;
            get_back_the_collateral => PUBLIC;
            liquidate_collateral => PUBLIC;
            claim_the_invested_XRDs_plus_interest => PUBLIC;
            take_out_the_invested_XRDs_by_the_community => PUBLIC;
            put_in_money_plus_interest_for_the_community_to_redeem => PUBLIC;
            check_the_balance_of_bond_issuer => PUBLIC;
            transfer_xrds_to_community_vault => PUBLIC;
            create_proposal_to_change_token_price => PUBLIC;
            execute_proposal_to_change_token_price => PUBLIC;
            get_all_contributors => PUBLIC;
            get_all_proposals => PUBLIC;
            get_bond_creator_addresses => PUBLIC;
            get_bond_creators => PUBLIC;
            get_bond_creator_and_details => PUBLIC;
            get_investment_details => PUBLIC;
            get_created_proposals => PUBLIC;
            get_proposal_using_proposal_id => PUBLIC;
            get_executives => PUBLIC;
        }

    }


    pub struct TokenWeigtedDao {
        // current_praposal: Option<Global<TokenWeightProposal>>,
        current_praposals: HashMap<ComponentAddress, HashMap<usize, Global<TokenWeightProposal>>>,

        dao_token_resource_manager: ResourceManager,

        dao_token: Vault,

        organization_name: String,

        shares: Vault,

        bonds: HashMap<ResourceAddress, Vault>,

        dao_token_address: ResourceAddress,

        owner_token_addresss: ResourceAddress,

        token_price: Decimal,

        buy_back_price: Decimal,

        // Add ZeroCouponBond component
        zero_coupon_bond: HashMap<ComponentAddress, Vec<Global<ZeroCouponBond>>>,

        contributors: HashMap<ComponentAddress, Decimal>,

        proposal_creation_right: ProposalCreationRight,

        liquidated_collateral: Vault,

        investment_record: HashMap<ComponentAddress, Decimal>,

        withdraw_requests: HashMap<u64, Decimal>,

        approval_details: HashMap<u64, ApprovalDetails>,

        executives: HashSet<ComponentAddress>,

        // executive_token_address: ResourceAddress

        executive_badge_resource_manager : NonFungibleResourceManager
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

            tags: Vec<String>,

            purpose: String,

            proposal_creation_right: ProposalCreationRight,

            token_name: String,
        ) -> (Global<TokenWeigtedDao>, Bucket) {
            // reserve an address for the DAO component
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(TokenWeigtedDao::blueprint_id());

            let owner_badge_description = format!("{}'s owner badge", &organization_name);

            let owner_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(0)
                .metadata(metadata!(
                    init{
                        "name"=>owner_badge_description,locked;
                        "icon_url" => Url::of(&org_ico_url), locked;
                    }
                ))
                .mint_roles(mint_roles! {
                    // A good minting rule is described in example 08
                    minter => rule!(allow_all);
                    minter_updater => rule!(deny_all);
                })
                .mint_initial_supply(1)
                .into();

            let executive_badge_description = format!("{}'s executive badge", &organization_name);

            // create a new Executive Badge resource manager
            let executive_badges_manager: NonFungibleResourceManager =  
                ResourceBuilder::new_integer_non_fungible::<ExecutiveBadge>(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => executive_badge_description, locked;
                        }
                    ))
                    .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(component_address)));
                        minter_updater => rule!(deny_all);
                    })
                    .recall_roles(recall_roles! {
                        recaller => rule!(require_any_of(vec![  
                                owner_badge.resource_address(),
                                // manager_badge.resource_address()
                            ]));
                        recaller_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require_any_of(vec![
                                owner_badge.resource_address(),
                                // manager_badge.resource_address(),
                            ]));
                        burner_updater => rule!(deny_all);
                    })
                    // starting with no initial supply means a resource manger is produced instead of a bucket
                    .create_with_no_initial_supply();


            

            // create nft to be sold for voting purpose
            let dao_token_description = format!("{} voting share", token_name);

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

            let component: Global<TokenWeigtedDao>;

            match proposal_creation_right {
                ProposalCreationRight::EVERYONE => {
                    component = Self {
                        token_price: token_price.clone(),

                        organization_name: organization_name.clone(),

                        dao_token_address: dao_token_address.clone(),

                        owner_token_addresss: owner_token_addresss.clone(),

                        current_praposals: HashMap::new(),

                        dao_token_resource_manager: voting_power_tokens.resource_manager(),

                        dao_token: Vault::with_bucket(voting_power_tokens),

                        buy_back_price: token_buy_back_price.clone(),

                        shares: Vault::new(XRD),

                        bonds: HashMap::new(),

                        // Initialize zero_coupon_bond as None
                        zero_coupon_bond: HashMap::new(),

                        contributors: HashMap::new(),

                        proposal_creation_right: ProposalCreationRight::EVERYONE,

                        liquidated_collateral: Vault::new(XRD),

                        investment_record: HashMap::new(),

                        withdraw_requests: HashMap::new(),

                        approval_details: HashMap::new(),

                        executives: HashSet::new(),

                        executive_badge_resource_manager : executive_badges_manager

                        // executive_token_address
                    }
                    .instantiate()
                    .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                        owner_token_addresss.clone()
                    ))))
                    .roles(roles!(
                        // manager => rule!(require(manager_badge.resource_address()));
                        executive => rule!(require(executive_badges_manager.address()));))
                    .with_address(address_reservation.clone())
                    .globalize();
                }
                ProposalCreationRight::TOKEN_HOLDER_THRESHOLD(threshold) => {
                    component = Self {
                        token_price: token_price.clone(),

                        organization_name: organization_name.clone(),

                        dao_token_address: dao_token_address.clone(),

                        owner_token_addresss: owner_token_addresss.clone(),

                        current_praposals: HashMap::new(),

                        dao_token_resource_manager: voting_power_tokens.resource_manager(),

                        dao_token: Vault::with_bucket(voting_power_tokens),

                        buy_back_price: token_buy_back_price.clone(),

                        shares: Vault::new(XRD),

                        bonds: HashMap::new(),

                        // Initialize zero_coupon_bond as None
                        zero_coupon_bond: HashMap::new(),

                        contributors: HashMap::new(),

                        proposal_creation_right: ProposalCreationRight::TOKEN_HOLDER_THRESHOLD(
                            threshold,
                        ),

                        liquidated_collateral: Vault::new(XRD),

                        investment_record: HashMap::new(),

                        withdraw_requests: HashMap::new(),

                        approval_details: HashMap::new(),

                        executives: HashSet::new(),

                        executive_badge_resource_manager : executive_badges_manager

                        // executive_token_address,
                    }
                    .instantiate()
                    .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                        owner_token_addresss.clone()
                    ))))
                    .roles(roles!(
                        // manager => rule!(require(manager_badge.resource_address()));
                        executive => rule!(require(executive_badges_manager.address()));))
                    .with_address(address_reservation.clone())
                    .globalize();
                }
                ProposalCreationRight::ADMIN => {
                    component = Self {
                        token_price: token_price.clone(),

                        organization_name: organization_name.clone(),

                        dao_token_address: dao_token_address.clone(),

                        owner_token_addresss: owner_token_addresss.clone(),

                        current_praposals: HashMap::new(),

                        dao_token_resource_manager: voting_power_tokens.resource_manager(),

                        dao_token: Vault::with_bucket(voting_power_tokens),

                        buy_back_price: token_buy_back_price.clone(),

                        shares: Vault::new(XRD),

                        bonds: HashMap::new(),

                        // Initialize zero_coupon_bond as None
                        zero_coupon_bond: HashMap::new(),

                        contributors: HashMap::new(),

                        proposal_creation_right: ProposalCreationRight::ADMIN,

                        liquidated_collateral: Vault::new(XRD),

                        investment_record: HashMap::new(),

                        withdraw_requests: HashMap::new(),

                        approval_details: HashMap::new(),

                        executives: HashSet::new(),

                        executive_badge_resource_manager : executive_badges_manager

                        // executive_token_address,
                    }
                    .instantiate()
                    .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                        owner_token_addresss.clone()
                    ))))
                    .roles(roles!(
                        // manager => rule!(require(manager_badge.resource_address()));
                        executive => rule!(require(executive_badges_manager.address()));))
                    .with_address(address_reservation.clone())
                    .globalize();
                }
            }

            let component_address = component.address();

            // Emit specific events based on the proposal creation right
            match proposal_creation_right {
                ProposalCreationRight::EVERYONE => {
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

                        tags: tags.clone(),

                        purpose: purpose.clone(),

                        proposal_creation_right: ProposalCreationRight::EVERYONE,

                        token_name,

                        // executive_token_address,
                    };

                    Runtime::emit_event(PandaoEvent {
                        // event_type: EventType::PROPOSAL_CREATION_RIGHT,
                        event_type: EventType::DEPLOYMENT,
                        dao_type: DaoType::Investment,
                        component_address,
                        meta_data: DaoEvent::TokenWeightedDEployment(event_metadata),
                    });
                }
                ProposalCreationRight::TOKEN_HOLDER_THRESHOLD(threshold) => {
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

                        tags: tags.clone(),

                        purpose: purpose.clone(),

                        proposal_creation_right: ProposalCreationRight::TOKEN_HOLDER_THRESHOLD(
                            threshold,
                        ),

                        token_name,

                        // executive_token_address,
                    };

                    Runtime::emit_event(PandaoEvent {
                        // event_type: EventType::PROPOSAL_CREATION_RIGHT,
                        event_type: EventType::DEPLOYMENT,
                        dao_type: DaoType::Investment,
                        component_address,
                        meta_data: DaoEvent::TokenWeightedDEployment(event_metadata),
                    });
                }
                ProposalCreationRight::ADMIN => {
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

                        tags: tags.clone(),

                        purpose: purpose.clone(),

                        proposal_creation_right: ProposalCreationRight::ADMIN,

                        token_name,

                        // executive_token_address,
                    };

                    Runtime::emit_event(PandaoEvent {
                        // event_type: EventType::PROPOSAL_CREATION_RIGHT,
                        event_type: EventType::DEPLOYMENT,
                        dao_type: DaoType::Investment,
                        component_address,
                        meta_data: DaoEvent::TokenWeightedDEployment(event_metadata),
                    });
                }
            }

            (component, owner_badge)
        }

        fn get_discriminator(&self) -> u64 {
            let current_epoch = Runtime::current_epoch();
            let unique_number = current_epoch.number();
            unique_number
        }

        // pub fn generate_uuid(&self){
        //     let a = Runtime::generate_ruid();
        // }

        pub fn mint_executive_badge(&mut self, name: String) -> NonFungibleBucket {

            // mint and receive a new staff badge. requires an owner badge

            let discriminator = self.get_discriminator();

            let executive_badge_bucket = self.executive_badge_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(discriminator),
                ExecutiveBadge {
                    executive_number: discriminator,
                    executive_name: name.clone()
                }   
            );

            let resource_address = executive_badge_bucket.resource_address();
            let local_id: NonFungibleLocalId = executive_badge_bucket.non_fungible_local_id();
            let global_id: NonFungibleGlobalId = executive_badge_bucket.non_fungible_global_id();
            

            let event_metadata = ExecutiveBadgeMinted { 
                name,
                number : discriminator,
                resource_address,
                local_id,
                global_id
            };

            Runtime::emit_event(events::PandaoEvent {
                event_type: EventType::EXECUTIVE_BADGE_MINTED,
                dao_type: DaoType::Investment,
                component_address: Runtime::global_address(),
                meta_data: DaoEvent::ExecutiveBadgeMinted(event_metadata),
            });

            executive_badge_bucket
        }

        pub fn make_an_executive(&mut self, mut to_account: Global<Account>, resource: NonFungibleBucket) {

            let resource_address = resource.resource_address();

            let local_id = resource.non_fungible_local_id();

            let receiver = to_account.address();

            // let new_bucket : &Bucket = resource.as_ref();
            let new_bucket : Bucket = resource.into();

            to_account.try_deposit_or_abort(new_bucket, None);

            self.executives.insert(receiver);

            let event_metadata = ExecutiveAppointed {
                account_address: to_account.address(),
                resource_address,
                local_id
            };

            Runtime::emit_event(events::PandaoEvent {
                event_type: EventType::EXECUTIVE_APPOINTED,
                dao_type: DaoType::Investment,
                component_address: Runtime::global_address(),
                meta_data: DaoEvent::ExecutiveAppointed(event_metadata),
            });
        }

        pub fn transfer_xrds_to_withdrawal_seeker(&self, mut to_account : Global<Account>, resource : Bucket) {
            to_account.try_deposit_or_abort(resource, None);
        }

        pub fn obtain_community_token(  
            &mut self,
            mut xrd: Bucket,
            token_amount: Decimal,
            account_address: ComponentAddress // minter_address: Option<String>,
        ) -> (Bucket, Bucket) {
            assert!(
                (self.token_price * token_amount) <= xrd.amount(),
                "you are paying an insufficient amount"
            );

            // let test = xrd.amount();

            let collected_xrd = xrd.take(self.token_price * token_amount);
            let power_share = self.dao_token.take(token_amount);
            let amount_paid = self.token_price * token_amount;
            self.shares.put(collected_xrd);

            //record investment

            if let Some(current_investment) = self.investment_record.get_mut(&account_address) {
                *current_investment += amount_paid;
            } else {
                self.investment_record.insert(account_address, amount_paid);
            }

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

                dao_type: DaoType::Investment,

                component_address,

                meta_data: DaoEvent::TokenWeightedTokenPurchase(event_metadata),
            });

            (xrd, power_share)
        }

        pub fn get_investment_details(&self, address: ComponentAddress) -> Result<Decimal, String> {
            if let Some(amt) = self.investment_record.get(&address) {
                Ok(amt.clone())
            } else {
                Err("No investment is made by this account address".to_string())
            }
        }

        fn generate_uuid() -> u64 {
            let current_epoch = Runtime::current_epoch();
            let unique_number: u64 = current_epoch.number();
            unique_number
        }

        pub fn request_withdrawal(
            &mut self,
            requester_address: ComponentAddress,
            requested_amount: Decimal,
        ) -> Result<(), String> {

            //Check if the sender has invested any amount
            let invested_amount =
                if let Some(amount) = self.investment_record.get(&requester_address) {
                    amount.clone()
                } else {
                    return Err("No investment found for the sender".to_string());
                };

            // Calculate the maximum allowed withdrawal amount (40% of invested amount)
            let max_withdrawal_amount = invested_amount * Decimal::from(40) / Decimal::from(100);

            // Ensure the requested amount does not exceed the maximum allowed withdrawal amount
            if requested_amount > max_withdrawal_amount {

                //event emission is required
                let event_metadata = WithdrawalRequested {
                    requester_address,
                    requested_amount,
                    max_withdrawal_amount,
                    withdrawal_request_occured : false,
                    requester_id : None
                };

                Runtime::emit_event(PandaoEvent{
                event_type: EventType::WITHDRAWAL_REQUEST_FAILED,
                dao_type: DaoType::Investment,
                component_address: Runtime::global_address(),
                meta_data: DaoEvent::WithdrawalRequestFailed(event_metadata)
                });

                return Err(format!(
                    "Requested amount {} exceeds the maximum allowed withdrawal amount {}",
                    requested_amount, max_withdrawal_amount
                ));
            }

            let uuid = Self::generate_uuid();
            // Store the withdrawal request
            self.withdraw_requests
                .insert(uuid, requested_amount);

            let event_metadata = WithdrawalRequested {
                requester_address,
                requested_amount,
                max_withdrawal_amount,
                withdrawal_request_occured : true,
                requester_id : Some(uuid)
            };

            Runtime::emit_event(PandaoEvent {
                event_type: EventType::WITHDRAWAL_REQUESTED_SUCCESSFULLY,
                dao_type: DaoType::Investment,
                component_address: Runtime::global_address(),
                meta_data: DaoEvent::WithdrawalRequested(event_metadata),
            });



            Ok(())
        }

        /* I need to store
        - address of an individual who is approving or denying the request
        - address of an individual whose request is being responded
        - store number of approvals
        - store number of denails

        and keep in your mind that
        there are only three individuals who will call this function

        case 1:
        if 2 approvals 1 denial
        this fn should call another fn which will transfer the XRD from component's Account
        to the requester's account

        case 2:
        if 1 approval 2 denials    
        this fn will not tranfer any XRD from Component's account to request's account    

        case 3:
        if only one individual responds or two individuals respond
        no transfer will occur

        because
        3 people must participate in responding the request
         */
        

        pub fn approve_or_deny_withdrawal_request(
            &mut self,
            approver_address: ComponentAddress,
            mut user_address: Global<Account>,
            request_id : u64,
            response: ApprovalResponse,
        ) -> Result<(), String> {
            // check whether a caller is executive or not

            // Check if the caller is an executive
            if !self.executives.contains(&approver_address) {
                return Err(format!("Caller is not an executive"));
            }

            // Check if the user has a withdrawal request
            if !self.withdraw_requests.contains_key(&request_id) {
                return Err(format!(
                    "No withdrawal request found for this request id {:?}",
                    user_address
                ));
            }

            // Get the approval details for the user
            let approval_details = self.approval_details.entry(request_id).or_insert(ApprovalDetails {
                approval_giver_addresses: Vec::new(),
                denial_giver_addresses: Vec::new(),
                approvals: 0,
                denials: 0,
            });

            match response{
                ApprovalResponse::Approve => {
                    // Check if the approver has already approved the request
                    if approval_details.approval_giver_addresses.contains(&approver_address) {
                        return Err(format!(
                            "Approver address {:?} has already approved the request",
                            approver_address
                        ));
                    }

                    approval_details.approvals += 1;
                    approval_details.approval_giver_addresses.push(approver_address);

                    // Add event emission for approval
                    let event_metadata = WithdrawalRequestApproved {
                        approver_address,
                        user_address: user_address.address(),
                        is_approved: true,
                        request_id
                    };

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::WITHDRAWAL_REQUEST_APPROVED,
                        dao_type: DaoType::Investment,
                        component_address: Runtime::global_address(),
                        meta_data: DaoEvent::WithdrawalRequestApproved(event_metadata),
                    });
                }

                ApprovalResponse::Deny => {
                    // Check if the approver has already denied the request
                    if approval_details.denial_giver_addresses.contains(&approver_address) {
                        return Err(format!(
                            "Approver address {:?} has already denied the request",
                            approver_address
                        ));
                    }

                    approval_details.denials += 1;
                    approval_details.denial_giver_addresses.push(approver_address);

                    // Add event emission for denial
                    let event_metadata = WithdrawalRequestDenied {
                        disapprover_address : approver_address,
                        user_address: user_address.address(),
                        is_approved: false,
                        request_id
                    };

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::WITHDRAWAL_REQUEST_DENIED,
                        dao_type: DaoType::Investment,
                        component_address: Runtime::global_address(),
                        meta_data: DaoEvent::WithdrawalRequestDenied(event_metadata),
                    });
                }
            }

            // Check if the request has been acknowledged by all executives
            if approval_details.approvals + approval_details.denials == 3 {
                // Check if the request has been approved by at least 2 executives
                if approval_details.approvals > approval_details.denials {
                    // Withdraw the money

                    let demanded_xrds = *self.withdraw_requests.get(&request_id).unwrap();
                    let bucket = self.shares.take(demanded_xrds);

                    user_address.try_deposit_or_abort(bucket, None);
                    
                    //as request is approved, remove the request
                    self.withdraw_requests.remove(&request_id);
                    //also remove the approval details
                    self.approval_details.remove(&request_id);

                    // Emit event
                    let event_metadata = FundsWithdrawn {
                        user_address: user_address.address(),
                        requested_amount: demanded_xrds,
                        request_id
                    };

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::FUNDS_WITHDRAWN,
                        dao_type: DaoType::Investment,
                        component_address: Runtime::global_address(),
                        meta_data: DaoEvent::FundsWithdrawn(event_metadata),
                    });


                } else {
                    // Emit event indicating the request is denied
                    let demanded_xrds = *self.withdraw_requests.get(&request_id).unwrap();

                    //as request is denied, remove the request
                    self.withdraw_requests.remove(&request_id);
                    //also remove the approval details
                    self.approval_details.remove(&request_id);

                    // Emit event
                    let event_metadata = FundsNotWithdrawn {
                        user_address: user_address.address(),
                        requested_amount: demanded_xrds,
                        request_id
                    };

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::FUNDS_NOT_WITHDRAWN,
                        dao_type: DaoType::Investment,
                        component_address: Runtime::global_address(),
                        meta_data: DaoEvent::FundsNotWithdrawn(event_metadata),
                    });
                }

                // Remove the approval details
                // self.approval_details.remove(&request_id);
                //also remove the withdrawal request
                // self.withdraw_requests.remove(&request_id);
            }

            Ok(())
        }


        

        // pub fn get_approval_details(
        //     &self,
        //     user_address: ComponentAddress,
        // ) -> Option<ApprovalDetails> {
        //     self.approval_details.get(&user_address).cloned()
        // }

        // pub fn add_executives(&mut self, new_executives: [ComponentAddress; 3]) {
        //     for address in new_executives {
        //         self.executives.insert(address);
        //     }
            // Emit event
            // let event_metadata = ExecutivesAdded {
            //     new_executives: new_executives.to_vec(),
            // };

            // let component_address = Runtime::global_address();

            // Runtime::emit_event(PandaoEvent {
            //     event_type: EventType::EXECUTIVES_ADDED,
            //     dao_type: DaoType::Investment,
            //     component_address,
            //     meta_data: DaoEvent::ExecutivesAdded(event_metadata),
            // });
        // }

        // pub fn withdraw_money(
        //     &mut self,
        //     mut user_address: Global<Account>, //ComponentAddress,
        // ) -> Result<(), String> {
        //     // Check if the user has a withdrawal request
        //     let requested_amount = self
        //         .withdraw_requests
        //         .get(&user_address.address())
        //         .ok_or_else(|| format!("No withdrawal request found for user address"))?;

        //     // Get the approval details for the user
        //     let approval_details = self
        //         .approval_details
        //         .get(&user_address.address())
        //         .ok_or_else(|| format!("No approval details found for user address"))?;

        //     // Check the approval and denial counts
        //     if approval_details.approvals > approval_details.denials {
        //         // Allow the withdrawal
        //         // let event_metadata = WithdrawalApproved {
        //         //     user_address,
        //         //     requested_amount: *requested_amount,
        //         // };

        //         // let component_address = Runtime::global_address();

        //         // Runtime::emit_event(PandaoEvent {
        //         //     event_type: EventType::WITHDRAWAL_APPROVED,
        //         //     dao_type: DaoType::Investment,
        //         //     component_address,
        //         //     meta_data: DaoEvent::WithdrawalApproved(event_metadata),
        //         // });

        //         // Transfer the requested amount from the treasury to a new bucket
        //         let mut withdrawal_bucket = self.shares.take(*requested_amount);

        //         user_address.try_deposit_or_abort(withdrawal_bucket, None);

        //         // Remove the withdrawal request and approval details
        //         self.withdraw_requests.remove(&user_address.address());
        //         self.approval_details.remove(&user_address.address());

        //         // Return the bucket with the requested amount
        //         Ok(())
        //     } else {
        //         // Emit event indicating the request is still not acknowledged or denied
        //         // let event_metadata = WithdrawalNotAcknowledged {
        //         //     user_address,
        //         //     approvals: approval_details.approvals,
        //         //     denials: approval_details.denials,
        //         // };

        //         // let component_address = Runtime::global_address();

        //         // Runtime::emit_event(PandaoEvent {
        //         //     event_type: EventType::WITHDRAWAL_NOT_ACKNOWLEDGED,
        //         //     dao_type: DaoType::Investment,
        //         //     component_address,
        //         //     meta_data: DaoEvent::WithdrawalNotAcknowledged(event_metadata),
        //         // });

        //         // Return an error message
        //         Err(format!(
        //             "Withdrawal request for user address is still not acknowledged by executives"
        //         ))
        //     }
        // }

        pub fn get_executives(&self) -> Vec<ComponentAddress> {
            self.executives.iter().cloned().collect()
        }

        pub fn withdraw_power(&mut self, voting_power: Bucket) -> Bucket {
            // put the voting power back
            assert!(
                self.current_praposals.is_empty(),
                "token can not be sold when there are active praposals or incomplete proposals"
            );

            let power_amount = voting_power.amount();

            self.dao_token.put(voting_power);

            let event_metadata = TokenWeightBuyToken {
                amount: power_amount,

                resource_address: self.dao_token_address.clone(),

                amount_paid: power_amount * self.buy_back_price,

                current_component_share: self.shares.amount(),
            };

            let component_address = Runtime::global_address();

            Runtime::emit_event(PandaoEvent {
                event_type: EventType::TOKEN_SELL,
                dao_type: DaoType::Investment,
                component_address,
                meta_data: DaoEvent::TokenWeightedTokenPurchase(event_metadata),
            });

            self.shares.take(power_amount * self.buy_back_price)
        }


        //get_unique_random_number
        fn get_proposal_id() -> u64 {
            let current_epoch = Runtime::current_epoch();
            let unique_number: u64 = current_epoch.number();
            unique_number
        }

        pub fn create_praposal(
            &mut self,
            title: String,
            description: String,
            minimun_quorum: u8,
            start_time: scrypto::time::UtcDateTime,
            end_time: scrypto::time::UtcDateTime,
            address_issued_bonds_to_sell: Option<ComponentAddress>,
            target_xrd_amount: Option<Decimal>,
            proposal_creator_address: Option<ComponentAddress>,
            governance_token_or_owner_token_address: Bucket,
            voting_type: VotingType,
        ) -> (
            Global<crate::proposal::pandao_praposal::TokenWeightProposal>,
            String,
            Bucket,
        ) {
            //implement proposal creation rights
            match self.proposal_creation_right {
                ProposalCreationRight::EVERYONE => {
                    assert_eq!(
                        governance_token_or_owner_token_address.resource_address(),
                        self.dao_token_address,
                        "wrong voting token supplied! please make sure that you supply DAO Governance Token"
                    );

                    assert!(
                        governance_token_or_owner_token_address.amount() >= Decimal::one(),
                        "Proposal creator must have at least one governance token to create a proposal"
                    );

                    //allow proposal creation
                }
                ProposalCreationRight::TOKEN_HOLDER_THRESHOLD(threshold) => {
                    assert_eq!(
                        governance_token_or_owner_token_address.resource_address(),
                        self.dao_token_address,
                        "wrong voting token supplied! please make sure that you supply DAO Governance Token"
                    );

                    assert!(
                        governance_token_or_owner_token_address.amount() >= threshold,
                        "Proposal creator does not have enough tokens to meet the threshold"
                    );
                }
                ProposalCreationRight::ADMIN => {
                    assert_eq!(
                        governance_token_or_owner_token_address.resource_address(),
                        self.owner_token_addresss,
                        "Only the admin can create a proposal and If you are an Admin please make sure you pass OWNER TOKEN ADDRESS"
                    );

                    assert!(
                        governance_token_or_owner_token_address.amount() >= Decimal::one(),
                        "ADMIN must pass his/her OWNER TOKEN to create proposal"
                    );
                }
            }

            use crate::proposal::pandao_praposal::TokenWeightProposal;

            if let Some(address_selling_bonds) = address_issued_bonds_to_sell {
                assert!(
                    self.zero_coupon_bond.contains_key(&address_selling_bonds),
                    "The Address you have specified has not created any bond"
                );
            }

            let amount_of_tokens_should_be_minted: Option<usize> = None;
            let desired_token_price: Option<Decimal> = None;
            let desired_token_buy_back_price: Option<Decimal> = None;

            let global_proposal_component: Global<TokenWeightProposal>;

            match voting_type {
                VotingType::ResourceHold => {
                    (global_proposal_component, _) = TokenWeightProposal::new(
                        title.clone(),
                        description.clone(),
                        minimun_quorum,
                        start_time,
                        end_time,
                        self.owner_token_addresss.clone(),
                        self.dao_token_address.clone(),
                        address_issued_bonds_to_sell.clone(),
                        target_xrd_amount.clone(),
                        proposal_creator_address,
                        amount_of_tokens_should_be_minted,
                        VotingType::ResourceHold,
                        desired_token_price,
                        desired_token_buy_back_price,
                    );
                }
                VotingType::Equality => {
                    (global_proposal_component, _) = TokenWeightProposal::new(
                        title.clone(),
                        description.clone(),
                        minimun_quorum,
                        start_time,
                        end_time,
                        self.owner_token_addresss.clone(),
                        self.dao_token_address.clone(),
                        address_issued_bonds_to_sell.clone(),
                        target_xrd_amount.clone(),
                        proposal_creator_address,
                        amount_of_tokens_should_be_minted,
                        VotingType::Equality,
                        desired_token_price,
                        desired_token_buy_back_price,
                    );
                }
            }

            let start_time_ts: i64 = start_time.to_instant().seconds_since_unix_epoch;
            let end_time_ts: i64 = end_time.to_instant().seconds_since_unix_epoch;

            //unique-id-generation
            let proposal_id: usize = Self::get_proposal_id()
                .try_into()
                .expect("couldn't get called successfully");
            //populate HashMap with newly created proposal

            let inner_map = self
                .current_praposals
                .entry(proposal_creator_address.unwrap())
                .or_insert_with(HashMap::new);

            inner_map.insert(proposal_id, global_proposal_component);

            match voting_type {
                VotingType::ResourceHold => {
                    let praposal_metadata = PraposalMetadata {
                        title,
                        description,
                        minimum_quorum: minimun_quorum.into(),
                        end_time_ts,
                        start_time_ts,
                        owner_token_address: self.owner_token_addresss.clone(),
                        component_address: global_proposal_component.address(),
                        address_issued_bonds_to_sell,
                        target_xrd_amount,
                        proposal_creator_address,
                        amount_of_tokens_should_be_minted,
                        proposal_id,
                        governance_token_or_owner_token_address:
                            governance_token_or_owner_token_address.resource_address(),
                        token_type: VotingType::ResourceHold,
                        desired_token_price,
                        desired_token_buy_back_price,
                    };

                    let component_address = Runtime::global_address();

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::PROPOSAL_TO_PURCHASE_BOND,
                        dao_type: DaoType::Investment,
                        meta_data: DaoEvent::PraposalDeployment(praposal_metadata),
                        component_address,
                    });
                }
                VotingType::Equality => {
                    let praposal_metadata = PraposalMetadata {
                        title,
                        description,
                        minimum_quorum: minimun_quorum.into(),
                        end_time_ts,
                        start_time_ts,
                        owner_token_address: self.owner_token_addresss.clone(),
                        component_address: global_proposal_component.address(),
                        address_issued_bonds_to_sell,
                        target_xrd_amount,
                        proposal_creator_address,
                        amount_of_tokens_should_be_minted,
                        proposal_id,
                        governance_token_or_owner_token_address:
                            governance_token_or_owner_token_address.resource_address(),
                        token_type: VotingType::Equality,
                        desired_token_price,
                        desired_token_buy_back_price,
                    };

                    let component_address = Runtime::global_address();

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::PROPOSAL_TO_PURCHASE_BOND,
                        dao_type: DaoType::Investment,
                        meta_data: DaoEvent::PraposalDeployment(praposal_metadata),
                        component_address,
                    });
                }
            }

            let mut message = String::new();
            message = format!("Proposal created with id : {}", proposal_id);

            (
                global_proposal_component,
                message,
                governance_token_or_owner_token_address,
            )
        }

        pub fn get_created_proposals(
            &self,
            your_address: ComponentAddress,
        ) -> Result<HashMap<usize, Global<TokenWeightProposal>>, String> {
            let inner_map = self.current_praposals.get(&your_address);
            match inner_map {
                Some(map) => {
                    let map = map;
                    Ok(map.clone())
                }
                None => Err(format!("this addres has no created proposals")),
            }
        }

        pub fn get_proposal_using_proposal_id(
            &self,
            proposal_id: usize,
        ) -> Result<Global<TokenWeightProposal>, String> {
            for (_, inner_map) in &self.current_praposals {
                let proposal = inner_map.get(&proposal_id);
                match proposal {
                    Some(proposal) => return Ok(proposal.clone()),
                    None => return Err(format!("proposal with id : {proposal_id} not found")),
                }
            }
            return Err(format!("proposal with id : {proposal_id} not found"));
        }

        pub fn get_all_proposals(&self) -> Vec<Global<TokenWeightProposal>> {
            let mut all_proposals: Vec<Global<TokenWeightProposal>> = Vec::new();
            for (_, inner_map) in &self.current_praposals {
                for (_, proposal) in inner_map {
                    all_proposals.push(proposal.clone());
                }
            }
            all_proposals
        }

        pub fn execute_proposal(&mut self, proposal_id: usize) {
            // First, find the proposal
            let mut proposal_option = None;
            let mut bond_creator_address_option = None;
            let mut target_xrd_amount_option = None;

            for (_, inner_map) in &self.current_praposals {
                if let Some(proposal) = inner_map.get(&proposal_id) {
                    proposal_option = Some(proposal.clone());
                    bond_creator_address_option = Some(proposal.get_address_issued_bonds());
                    target_xrd_amount_option = Some(proposal.get_target_xrd_amount());
                    break;
                }
            }

            // If the proposal is found, execute it
            if let (Some(proposal), Some(bond_creator_address), Some(target_xrd_amount)) = (
                proposal_option,
                bond_creator_address_option,
                target_xrd_amount_option,
            ) {
                // let current_epoch = Runtime::current_epoch();
                // let current_time_seconds = current_epoch.number() as i64;
                let now: Instant = Clock::current_time_rounded_to_seconds();
                let current_time_seconds: i64 = now.seconds_since_unix_epoch;

                let last_time = proposal.get_last_time();
                let end_time_seconds = last_time.to_instant().seconds_since_unix_epoch;

                // Debug statements to verify the times
                println!("Current time (epoch seconds): {}", current_time_seconds);
                println!("Proposal end time (epoch seconds): {}", end_time_seconds);

                assert!(
                    current_time_seconds > end_time_seconds,
                    "Proposal can only be executed after the specified end time"
                );

                let bond_components = self
                    .zero_coupon_bond
                    .get_mut(&bond_creator_address)
                    .unwrap();

                //*we can restrict a creator in terms of bond creation
                let latest_bond_component =
                    bond_components.last_mut().expect("No bond component found");

                let bond_uid = latest_bond_component.get_bond_u_id();

                // Check if the minimum quorum is met
                let number_of_voters = proposal.get_number_of_voters(); //inline attribute on fn definition
                let minimum_quorum = proposal.get_minimum_quorum();
                // let minimum_quorum = min_quo.0.to_u64().expect("Invalid minimum quorum value") as usize;

                if number_of_voters < minimum_quorum {
                    // Emit an event indicating that the proposal cannot be executed due to insufficient participation
                    let event_metadata = ProposalQuorumNotMet {
                        proposal_id,
                        minimum_quorum: proposal.get_minimum_quorum(),
                        number_of_voters,
                        bond_creator_address,
                        contract_identity: bond_uid.clone(),
                        proposal_type: EventType::PROPOSAL_TO_PURCHASE_BOND,
                    };

                    let component_address = Runtime::global_address();

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::QUORUM_NOT_MET_AND_FAILED,
                        dao_type: DaoType::Investment,
                        component_address,
                        meta_data: DaoEvent::ProposalQuorumNotMet(event_metadata),
                    });

                    // panic!("The proposal cannot be executed due to insufficient participation");
                }

                // Check if the treasury has enough XRD
                let treasury_balance = self.shares.amount();

                assert!(
                    treasury_balance >= target_xrd_amount,
                    "Insufficient funds in the treasury to execute the proposal."
                );

                // Create a bucket with the exact XRD amount needed for the purchase
                let payment = self.shares.take(target_xrd_amount);

                // Call the purchase_bond function
                let remaining = self.purchase_bond(bond_creator_address.clone(), payment);

                // Handle remaining funds and received bond NFT
                self.shares.put(remaining);

                let event_metadata = ProposalQuorumMet {
                    proposal_id,
                    minimum_quorum: proposal.get_minimum_quorum(),
                    number_of_voters,
                    //bond creator address
                    bond_creator_address,
                    contract_identity: bond_uid,
                    proposal_type: EventType::PROPOSAL_TO_PURCHASE_BOND,
                };

                let component_address = Runtime::global_address();

                //what to emit?
                //1. success or unsuccess
                //2. bond creator address + contract || bond id

                Runtime::emit_event(PandaoEvent {
                    event_type: EventType::QUORUM_MET_AND_SUCCESS,
                    dao_type: DaoType::Investment,
                    component_address,
                    meta_data: DaoEvent::ProposalQuorumMet(event_metadata),
                });

                // Optionally, you can remove the executed proposal from the current proposals
                for (_, inner_map) in &mut self.current_praposals {
                    inner_map.remove(&proposal_id);
                }
            } else {
                panic!("there is no current active proposal with the given ID");
            }
        }

        pub fn vote(
            &mut self,
            token: Bucket,
            againsts: bool,
            // account: Global<Account>,
            your_address: ComponentAddress,
            proposal_id: usize,
        ) -> Bucket {
            // let owner_role_of_voter = account.get_owner_role();
            // Runtime::assert_access_rule(owner_role_of_voter.rule);

            // Find the proposal by proposal_id
            let mut proposal_option = None;

            for (_, inner_map) in &self.current_praposals {
                if let Some(proposal) = inner_map.get(&proposal_id) {
                    proposal_option = Some(proposal.clone());
                    break;
                }
            }

            if let Some(proposal) = proposal_option {
                assert_eq!(
                    token.resource_address(),
                    self.dao_token_address,
                    "wrong voting token supplied"
                );

                // Get the voter address from the account
                let voter_address = your_address;

                let mut vote_caster_addresses = proposal.get_vote_caster_addresses();

                // Check if the voter has already voted
                assert!(
                    !vote_caster_addresses.contains(&voter_address),
                    "You have already voted on this proposal."
                );

                let amount = token.amount();

                let event_metadata = ProposalVote {
                    praposal_address: proposal.address(),
                    voting_amount: amount,
                    againts: againsts,
                    voter_address,
                    proposal_id,
                };

                Runtime::emit_event(PandaoEvent {
                    event_type: EventType::VOTE,
                    dao_type: DaoType::Investment,
                    component_address: Runtime::global_address(),
                    meta_data: DaoEvent::PraposalVote(event_metadata),
                });

                let result = proposal.vote(token, againsts);

                // Mark this voter as having voted
                proposal.set_vote_caster_address(voter_address);

                result
            } else {
                assert!(false, "no active proposal with the given ID");
                panic!();
            }
        }

        pub fn create_zero_coupon_bond(
            &mut self,
            contract_type: String,
            contract_role: String,
            contract_identifier: String,
            nominal_interest_rate: Decimal,
            currency: String,
            initial_exchange_date: u64,
            maturity_date: u64,
            notional_principal: Decimal,
            discount: u64,
            bond_position: String,
            price: u64,
            number_of_bonds: Decimal,
            your_address: ComponentAddress,
            nft_as_collateral: Bucket, //OK -> Account address is of ComponentAddress Type
        ) -> Global<ZeroCouponBond> {
            let collateral_resource_address = nft_as_collateral.resource_address();

            let bond_component = ZeroCouponBond::instantiate_zerocouponbond(
                contract_type.clone(),
                contract_role.clone(),
                contract_identifier.clone(),
                nominal_interest_rate,
                currency.clone(),
                initial_exchange_date,
                maturity_date,
                notional_principal,
                discount,
                bond_position.clone(),
                price,
                number_of_bonds,
                nft_as_collateral,
            );

            self.zero_coupon_bond
                .entry(your_address)
                .or_insert_with(Vec::new)
                .push(bond_component);
            // self.zero_coupon_bond = Some(bond_component);

            // Emit the ZeroCouponBondCreation event
            let event_metadata = ZeroCouponBondCreation {
                component_address: bond_component.address(),
                contract_type,
                contract_role,
                contract_identifier,
                nominal_interest_rate,
                currency,
                initial_exchange_date,
                maturity_date,
                notional_principal,
                discount,
                bond_position,
                price,
                number_of_bonds,
                creator_address: your_address,
                collateral_resource_address,
            };

            Runtime::emit_event(PandaoEvent {
                event_type: EventType::ZERO_COUPON_BOND_CREATION, // You can define a specific event type for bond creation if needed
                dao_type: DaoType::Investment,
                component_address: Runtime::global_address(),
                meta_data: DaoEvent::ZeroCouponBondCreation(event_metadata),
            });

            bond_component
        }

        // New method to purchase a bond
        // pub fn purchase_bond(&mut self, payment: Bucket) -> (Bucket, Bucket) {

        //     assert!(self.zero_coupon_bond.is_some(), "ZeroCouponBond not initialized");

        //     self.zero_coupon_bond.as_mut().unwrap().purchase_bond(payment)
        // }

        pub fn purchase_bond(
            &mut self,
            bond_creator_address: ComponentAddress,
            // uid : Uid,
            payment: Bucket,
        ) -> Bucket {
            assert!(
                self.zero_coupon_bond.contains_key(&bond_creator_address),
                "No bonds created by the specified address."
            );

            // Retrieve the most recent bond component created by the bond creator
            let bond_components = self
                .zero_coupon_bond
                .get_mut(&bond_creator_address)
                .unwrap();

            //*we can restrict a creator in terms of bond creation
            let latest_bond_component =
                bond_components.last_mut().expect("No bond component found");

            // Purchase bond from the latest bond component
            let (purchased_bond, payment) = latest_bond_component.purchase_bond(payment);
            self.update_bond_vault_and_store(purchased_bond);
            payment
        }

        // New method to sell a bond
        pub fn sell_bond(
            &mut self,
            bond_creator_address: ComponentAddress,
            // bond: Bucket,
        ) {
            assert!(
                self.zero_coupon_bond.contains_key(&bond_creator_address),
                "No bonds created by the specified address."
            );

            // Retrieve the most recent bond component created by the bond creator
            let bond_components = self
                .zero_coupon_bond
                .get_mut(&bond_creator_address)
                .unwrap();

            let latest_bond_component =
                bond_components.last_mut().expect("No bond component found");

            // Sell bond from the latest bond component

            //access the bond resouce address
            let bond_resource_address = latest_bond_component.get_resource_address();

            //access the bond
            let vault = self.bonds.get_mut(&bond_resource_address).unwrap();

            let purchased_bond = vault.take(1);

            let principal_plus_interest = latest_bond_component.sell_the_bond(purchased_bond);

            self.shares.put(principal_plus_interest);
        }

        // New method to check bond maturity
        pub fn check_bond_maturity(&self, bond_creator_address: ComponentAddress) -> i64 {
            assert!(
                self.zero_coupon_bond.contains_key(&bond_creator_address),
                "No bonds created by the specified address."
            );

            // Retrieve the most recent bond component created by the bond creator
            let bond_components = self.zero_coupon_bond.get(&bond_creator_address).unwrap();
            let latest_bond_component = bond_components.last().expect("No bond component found");

            // Check bond maturity of the latest bond component
            latest_bond_component.check_the_maturity_of_bonds()
        }

        // New method to get bond details
        pub fn get_bond_details(&self, bond_creator_address: ComponentAddress) -> BondDetails {
            assert!(
                self.zero_coupon_bond.contains_key(&bond_creator_address),
                "No bonds created by the specified address."
            );

            // Retrieve the most recent bond component created by the bond creator
            let bond_components = self.zero_coupon_bond.get(&bond_creator_address).unwrap();
            let latest_bond_component = bond_components.last().expect("No bond component found");

            // Get bond details of the latest bond component
            latest_bond_component.get_bond_details()
        }

        // Function to retrieve bond creators and their bond component addresses
        pub fn get_bond_creators(&self) -> HashMap<ComponentAddress, Vec<Global<ZeroCouponBond>>> {
            self.zero_coupon_bond.clone() // Return the HashMap of bond creators and their bonds
        }

        // New function to get all bond creator addresses
        pub fn get_bond_creator_addresses(&self) -> Vec<ComponentAddress> {
            self.zero_coupon_bond.keys().cloned().collect() // Return a list of bond creator addresses
        }

        // Function to get bond creator address and bond details
        pub fn get_bond_creator_and_details(&self) -> Vec<(ComponentAddress, Vec<BondDetails>)> {
            let mut result = Vec::new();

            // Iterate through each bond creator address and their bond components
            for (creator_address, bonds) in &self.zero_coupon_bond {
                let mut bond_details = Vec::new();
                for bond in bonds {
                    bond_details.push(bond.get_bond_details());
                }
                // Push the creator address and corresponding bond details to the result
                result.push((*creator_address, bond_details));
            }

            result
        }

        pub fn send_money_to_dao_treasury(
            &mut self,
            payment: Bucket,
            account: Global<Account>,
        ) -> Bucket {
            // Ensure the payment is in XRD
            assert_eq!(
                payment.resource_address(),
                XRD,
                "Only XRD tokens are accepted for treasury contributions"
            );

            // Get the amount being sent
            let amount = payment.amount();

            // Get the sender's address
            let sender_address = account.address();

            // Store the payment in the dao_token vault
            self.dao_token.put(payment);

            // Update the contributor's record
            self.update_contributor_record(sender_address, amount);

            // Emit an event for the contribution
            self.emit_contribution_event(sender_address, amount);

            // Return an empty bucket
            Bucket::new(XRD)
        }

        // Helper method to update the contributor's record
        fn update_contributor_record(&mut self, address: ComponentAddress, amount: Decimal) {
            *self.contributors.entry(address).or_insert(Decimal::zero()) += amount;
        }

        // Helper method to emit a contribution event
        fn emit_contribution_event(&self, address: ComponentAddress, amount: Decimal) {
            let event_metadata = TreasuryContribution {
                contributor: address,
                amount: amount,
                timestamp: Runtime::current_epoch().number(),
            };

            Runtime::emit_event(PandaoEvent {
                event_type: EventType::TREASURY_CONTRIBUTION,
                dao_type: DaoType::Investment,
                component_address: Runtime::global_address(),
                meta_data: DaoEvent::TreasuryContribution(event_metadata),
            });
        }

        // Method to get all contributors and their total contributions
        pub fn get_all_contributors(&self) -> HashMap<ComponentAddress, Decimal> {
            self.contributors.clone()
        }

        fn update_bond_vault_and_store(&mut self, desired_bond: Bucket) {
            let desired_resource_address: ResourceAddress = desired_bond.resource_address();
            if !self.bonds.contains_key(&desired_resource_address) {
                self.bonds.insert(
                    desired_resource_address,
                    Vault::new(desired_resource_address),
                );
            }
            let vault = self.bonds.get_mut(&desired_resource_address).unwrap();
            vault.put(desired_bond);
        }

        // pub fn execute_proposal_for_pandao(&mut self){
        //     match self.current_praposal{
        //         Some(current_proposal) =>{
        //             //earlier execute proposal was not taking any action but it was made to take action in financial dao case
        //             println!("your proposal is executed successfully");
        //             self.current_praposal = None;
        //         },
        //         // None => println!("there is not any proposal created")
        //         None => assert!(false, "there is no any created proposal")
        //     }
        // }

        pub fn create_proposal_to_mint_more_dao_tokens(
            &mut self,
            title: String,
            description: String,
            minimun_quorum: u8,
            amount_of_tokens_should_be_minted: Option<usize>,
            start_time: scrypto::time::UtcDateTime,
            end_time: scrypto::time::UtcDateTime,
            proposal_creator_address: Option<ComponentAddress>,
            governance_token_or_owner_token_address: Bucket,
            voting_type: VotingType,
        ) -> (
            Global<crate::proposal::pandao_praposal::TokenWeightProposal>,
            String,
            Bucket,
        ) {
            match self.proposal_creation_right {
                ProposalCreationRight::EVERYONE => {
                    assert_eq!(
                        governance_token_or_owner_token_address.resource_address(),
                        self.dao_token_address,
                        "wrong voting token supplied! please make sure that you supply DAO Governance Token"
                    );

                    assert!(
                        governance_token_or_owner_token_address.amount() >= Decimal::one(),
                        "Proposal creator must have at least one governance token to create a proposal"
                    );

                    //allow proposal creation
                }
                ProposalCreationRight::TOKEN_HOLDER_THRESHOLD(threshold) => {
                    assert_eq!(
                        governance_token_or_owner_token_address.resource_address(),
                        self.dao_token_address,
                        "wrong voting token supplied! please make sure that you supply DAO Governance Token"
                    );

                    assert!(
                        governance_token_or_owner_token_address.amount() >= threshold,
                        "Proposal creator does not have enough tokens to meet the threshold"
                    );
                }
                ProposalCreationRight::ADMIN => {
                    assert_eq!(
                        governance_token_or_owner_token_address.resource_address(),
                        self.owner_token_addresss,
                        "Only the admin can create a proposal and If you are an Admin please make sure you pass OWNER TOKEN ADDRESS"
                    );

                    assert!(
                        governance_token_or_owner_token_address.amount() >= Decimal::one(),
                        "ADMIN must pass his/her OWNER TOKEN to create proposal"
                    );
                }
            }

            let address_issued_bonds_to_sell: Option<ComponentAddress> = None;
            let target_xrd_amount: Option<Decimal> = None;
            let desired_token_price: Option<Decimal> = None;
            let desired_token_buy_back_price: Option<Decimal> = None;

            let global_proposal_component: Global<TokenWeightProposal>;

            match voting_type {
                VotingType::ResourceHold => {
                    (global_proposal_component, _) = TokenWeightProposal::new(
                        title.clone(),
                        description.clone(),
                        minimun_quorum,
                        start_time,
                        end_time,
                        self.owner_token_addresss.clone(),
                        self.dao_token_address.clone(),
                        address_issued_bonds_to_sell.clone(),
                        target_xrd_amount.clone(),
                        proposal_creator_address,
                        amount_of_tokens_should_be_minted,
                        VotingType::ResourceHold,
                        desired_token_price,
                        desired_token_buy_back_price,
                    );
                }
                VotingType::Equality => {
                    (global_proposal_component, _) = TokenWeightProposal::new(
                        title.clone(),
                        description.clone(),
                        minimun_quorum,
                        start_time,
                        end_time,
                        self.owner_token_addresss.clone(),
                        self.dao_token_address.clone(),
                        address_issued_bonds_to_sell.clone(),
                        target_xrd_amount.clone(),
                        proposal_creator_address,
                        amount_of_tokens_should_be_minted,
                        VotingType::Equality,
                        desired_token_price,
                        desired_token_buy_back_price,
                    );
                }
            }

            //start date
            //end date
            let start_time_ts: i64 = start_time.to_instant().seconds_since_unix_epoch;
            let end_time_ts: i64 = end_time.to_instant().seconds_since_unix_epoch;

            let proposal_id: usize = Self::get_proposal_id()
                .try_into()
                .expect("couldn't get called successfully");

            let inner_map = self
                .current_praposals
                .entry(proposal_creator_address.unwrap())
                .or_insert_with(HashMap::new);

            inner_map.insert(proposal_id, global_proposal_component);

            match voting_type {
                VotingType::ResourceHold => {
                    let praposal_metadata = PraposalMetadata {
                        title,
                        description,
                        minimum_quorum: minimun_quorum.into(),
                        end_time_ts,
                        start_time_ts,
                        owner_token_address: self.owner_token_addresss.clone(),
                        component_address: global_proposal_component.address(),
                        address_issued_bonds_to_sell,
                        target_xrd_amount,
                        proposal_creator_address,
                        amount_of_tokens_should_be_minted,
                        proposal_id,
                        governance_token_or_owner_token_address:
                            governance_token_or_owner_token_address.resource_address(),
                        token_type: VotingType::ResourceHold,
                        desired_token_price,
                        desired_token_buy_back_price,
                    };

                    let component_address = Runtime::global_address();

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::PROPOSAL_TO_MINT_MORE_TOKENS,
                        dao_type: DaoType::Investment,
                        meta_data: DaoEvent::PraposalDeployment(praposal_metadata),
                        component_address,
                    });
                }
                VotingType::Equality => {
                    let praposal_metadata = PraposalMetadata {
                        title,
                        description,
                        minimum_quorum: minimun_quorum.into(),
                        end_time_ts,
                        start_time_ts,
                        owner_token_address: self.owner_token_addresss.clone(),
                        component_address: global_proposal_component.address(),
                        address_issued_bonds_to_sell,
                        target_xrd_amount,
                        proposal_creator_address,
                        amount_of_tokens_should_be_minted,
                        proposal_id,
                        governance_token_or_owner_token_address:
                            governance_token_or_owner_token_address.resource_address(),
                        token_type: VotingType::Equality,
                        desired_token_price,
                        desired_token_buy_back_price,
                    };

                    let component_address = Runtime::global_address();

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::PROPOSAL_TO_MINT_MORE_TOKENS,
                        dao_type: DaoType::Investment,
                        meta_data: DaoEvent::PraposalDeployment(praposal_metadata),
                        component_address,
                    });
                }
            }

            let mut message = String::new();
            message = format!("Proposal created with id : {}", proposal_id);

            (
                global_proposal_component,
                message,
                governance_token_or_owner_token_address,
            )
        }

        pub fn mint_more_tokens(&mut self, token_number_to_mint: usize) {
            self.dao_token
                .put(self.dao_token_resource_manager.mint(token_number_to_mint));
        }

        pub fn set_price(&mut self, desired_token_price: Decimal, desired_buy_back_price: Decimal) {
            self.token_price = desired_token_price;
            self.buy_back_price = desired_buy_back_price;
        }

        pub fn execute_proposal_to_mint_more_tokens(
            &mut self,
            proposal_id: usize,
        ) -> Result<String, String> {
            for (_, inner_map) in &self.current_praposals {
                let proposal = inner_map.get(&proposal_id);

                let now: Instant = Clock::current_time_rounded_to_seconds();
                let current_time_seconds: i64 = now.seconds_since_unix_epoch;

                let last_time = proposal.unwrap().get_last_time();
                let end_time_seconds = last_time.to_instant().seconds_since_unix_epoch;

                // Debug statements to verify the times
                println!("Current time (epoch seconds): {}", current_time_seconds);
                println!("Proposal end time (epoch seconds): {}", end_time_seconds);

                assert!(
                    current_time_seconds > end_time_seconds,
                    "Proposal can only be executed after the specified end time"
                );

                match proposal {
                    Some(proposal) => {
                        if let Some(how_much_amount) = proposal.get_token_mint_amount() {
                            self.mint_more_tokens(how_much_amount);

                            let message = "proposal executed successfully".to_string();

                            return Ok(message);
                        } else {
                            return Err(format!("token_mint_amount is not present in a proposal with id : {proposal_id} and it seems not to be a proposal to mint more tokens"));
                        }
                    }
                    None => return Err(format!("proposal with id : {proposal_id} not found")),
                }
            }
            Ok("execute fn called successfully".to_string())
        }

        pub fn get_back_the_collateral(
            &mut self,
            bond_creator_address: ComponentAddress,
        ) -> Bucket {
            assert!(
                self.zero_coupon_bond.contains_key(&bond_creator_address),
                "No bonds created by the specified address."
            );

            // Retrieve the most recent bond component created by the bond creator
            let bond_components = self
                .zero_coupon_bond
                .get_mut(&bond_creator_address)
                .unwrap();

            let latest_bond_component =
                bond_components.last_mut().expect("No bond component found");

            let money_claim_status_by_community = latest_bond_component.get_money_claim_status();

            let collateral_resource_address =
                latest_bond_component.get_resource_address_of_collateral();

            if money_claim_status_by_community == true {
                let meta_data = GetBackTheCollateralEvent {
                    bond_creator_address,
                    is_given_money_claimed_by_community: money_claim_status_by_community,
                    resource_address_of_collateral: collateral_resource_address,
                    message: "collateral taken back successfully".to_string(),
                };

                Runtime::emit_event(PandaoEvent {
                    event_type: EventType::COLLATERAL_GOT_BACK,
                    dao_type: DaoType::Investment,
                    meta_data: DaoEvent::GetBackTheCollateral(meta_data),
                    component_address: Runtime::global_address(),
                });

                latest_bond_component.get_back_the_collateral()
            } else {
                let meta_data = GetBackTheCollateralEvent {
                    bond_creator_address,
                    is_given_money_claimed_by_community : money_claim_status_by_community,
                    resource_address_of_collateral : collateral_resource_address,
                    message : "you can not take your collateral back because community has not claimed the amount".to_string()
                };

                Runtime::emit_event(PandaoEvent {
                    event_type: EventType::FAILED_IN_GETTING_BACK_COLLATERAL,
                    dao_type: DaoType::Investment,
                    meta_data: DaoEvent::GetBackTheCollateral(meta_data),
                    component_address: Runtime::global_address(),
                });

                latest_bond_component.get_back_the_collateral()
            }
        }

        pub fn liquidate_collateral(&mut self, bond_creator_address: ComponentAddress) {
            assert!(
                self.zero_coupon_bond.contains_key(&bond_creator_address),
                "No bonds created by the specified address."
            );

            // Retrieve the most recent bond component created by the bond creator
            let bond_components = self
                .zero_coupon_bond
                .get_mut(&bond_creator_address)
                .unwrap();

            let latest_bond_component =
                bond_components.last_mut().expect("No bond component found");

            //condition
            let now: Instant = Clock::current_time_rounded_to_seconds();
            let current_time_seconds: u64 = now.seconds_since_unix_epoch as u64;

            let maturity_date = latest_bond_component.get_maturity_data();

            if maturity_date < current_time_seconds {
                let redeemed_collateral = latest_bond_component.liquidate_collateral();

                let collateral_resource_address =
                    latest_bond_component.get_resource_address_of_collateral();

                let liquidated_amount = redeemed_collateral.amount();

                self.liquidated_collateral = Vault::new(redeemed_collateral.resource_address());

                self.liquidated_collateral.put(redeemed_collateral);

                let event_metadata = LiquidatedCollateralEvent {
                    bond_creator_address,
                    liquidated_amount,
                    collateral_resource_address,
                };

                Runtime::emit_event(PandaoEvent {
                    event_type: EventType::LIQUIDATED_COLLATERAL,
                    dao_type: DaoType::Investment,
                    component_address: Runtime::global_address(),
                    meta_data: DaoEvent::LiquidatedCollateral(event_metadata),
                });
            } else {
                let event_metadata = CollateralLiquidationFailedEvent {
                    bond_creator_address,
                };

                Runtime::emit_event(PandaoEvent {
                    event_type: EventType::COLLATERAL_LIQUIDATION_FAILED,
                    dao_type: DaoType::Investment,
                    component_address: Runtime::global_address(),
                    meta_data: DaoEvent::CollateralLiquidationFailed(event_metadata),
                });
            }
        }

        pub fn claim_the_invested_XRDs_plus_interest(
            &mut self,
            bond_creator_address: ComponentAddress,
        ) {
            assert!(
                self.zero_coupon_bond.contains_key(&bond_creator_address),
                "No bonds created by the specified address."
            );

            // Retrieve the most recent bond component created by the bond creator
            let bond_components = self
                .zero_coupon_bond
                .get_mut(&bond_creator_address)
                .unwrap();

            let latest_bond_component =
                bond_components.last_mut().expect("No bond component found");

            // how much XRDs are required by the communty?
            let balance_required_by_the_community =
                latest_bond_component.balance_required_by_the_community();

            //balance in bond component
            let balance_in_latest_bond_component =
                latest_bond_component.check_the_balance_of_bond_issuer();

            if balance_in_latest_bond_component < balance_required_by_the_community {
                //perform liquidation
                let redeemed_collateral = latest_bond_component.liquidate_collateral();

                let collateral_resource_address =
                    latest_bond_component.get_resource_address_of_collateral();

                let liquidated_amount = redeemed_collateral.amount();

                self.liquidated_collateral = Vault::new(redeemed_collateral.resource_address());

                self.liquidated_collateral.put(redeemed_collateral);

                let event_metadata = ClaimInvestedXRDsPlusInterestErrorEvent {
                    bond_creator_address,
                    required_amount_by_the_community: balance_required_by_the_community,
                    balance_of_bond_issuer: balance_in_latest_bond_component,
                    collateral_liquidated: true,
                    collateral_resource_address,
                    liquidated_amount,
                };

                Runtime::emit_event(PandaoEvent {
                    event_type: EventType::FAILED_CLAIM_INVESTED_XRDs_PLUS_INTEREST,
                    dao_type: DaoType::Investment,
                    component_address: Runtime::global_address(),
                    meta_data: DaoEvent::ClaimInvestedXRDsPlusInterestError(event_metadata),
                });
            } else {
                let claimed_invested_xrd_plus_interest =
                    latest_bond_component.claim_the_invested_XRDs_plus_interest();

                let claimed_amount = claimed_invested_xrd_plus_interest.amount();

                self.shares.put(claimed_invested_xrd_plus_interest);

                //claimed successful
                latest_bond_component.change_community_claim_status(true);

                let event_metadata = ClaimInvestedXRDsPlusInterestEvent {
                    bond_creator_address,
                    claimed_amount,
                    amount_required_by_the_community: balance_required_by_the_community,
                    collateral_liquidated: false,
                };

                Runtime::emit_event(PandaoEvent {
                    event_type: EventType::CLAIM_INVESTED_XRDs_PLUS_INTEREST,
                    dao_type: DaoType::Investment,
                    component_address: Runtime::global_address(),
                    meta_data: DaoEvent::ClaimInvestedXRDsPlusInterest(event_metadata),
                });
            }
        }

        //FOR BOND ISSUER TO TAKE OUT COMMUNITY INVESTMENT
        pub fn take_out_the_invested_XRDs_by_the_community(
            &mut self,
            bond_creator_address: ComponentAddress,
        ) -> Bucket {
            assert!(
                self.zero_coupon_bond.contains_key(&bond_creator_address),
                "No bonds created by the specified address."
            );

            // Retrieve the most recent bond component created by the bond creator
            let bond_components = self
                .zero_coupon_bond
                .get_mut(&bond_creator_address)
                .unwrap();

            let latest_bond_component =
                bond_components.last_mut().expect("No bond component found");

            // let bond_creator_money_taken_status = latest_bond_component.bond_creator_money_status();

            let taken_out_invested_amount =
                latest_bond_component.take_out_the_invested_XRDs_by_the_community();

            let event_metadata = TakeOutInvestedXRDsEvent {
                bond_creator_address,
                taken_out_amount: taken_out_invested_amount.amount(),
            };

            Runtime::emit_event(PandaoEvent {
                event_type: EventType::TAKE_OUT_INVESTED_XRDs,
                dao_type: DaoType::Investment,
                component_address: Runtime::global_address(),
                meta_data: DaoEvent::TakeOutInvestedXRDs(event_metadata),
            });

            taken_out_invested_amount
        }

        pub fn put_in_money_plus_interest_for_the_community_to_redeem(
            &mut self,
            bond_creator_address: ComponentAddress,
            borrowed_xrd_with_interest: Bucket,
        ) -> (Bucket, Bucket) {
            assert!(
                self.zero_coupon_bond.contains_key(&bond_creator_address),
                "No bonds created by the specified address."
            );

            // Retrieve the most recent bond component created by the bond creator
            let bond_components = self
                .zero_coupon_bond
                .get_mut(&bond_creator_address)
                .unwrap();

            let latest_bond_component =
                bond_components.last_mut().expect("No bond component found");

            let amount_getting_deposited = borrowed_xrd_with_interest.amount();

            // Get Required Amount
            let required_amount = latest_bond_component.balance_required_by_the_community();

            let extra_money = latest_bond_component
                .put_in_money_plus_interest_for_the_community_to_redeem(borrowed_xrd_with_interest);

            let balance_of_bond_component =
                latest_bond_component.check_the_balance_of_bond_issuer();

            //required amount?
            let balance_required_by_community =
                latest_bond_component.balance_required_by_the_community();

            if balance_of_bond_component >= balance_required_by_community {
                let collateral_being_taken_back = latest_bond_component.get_back_the_collateral();

                let extra_money_amount = extra_money.amount();

                // let required_now = latest_bond_component.balance_required_by_the_community();

                let event_metadata_if = PutInMoneyPlusInterestEvent {
                    bond_creator_address,
                    amount_getting_deposited,
                    amount_required_by_the_community: required_amount,
                    amount_taken_by_the_community: required_amount,
                    extra_amount_given_back_to_the_sender: extra_money_amount,
                    more_xrd_amount_required_by_the_community: Decimal::zero(),
                    collateral_given_back: true,
                };

                Runtime::emit_event(PandaoEvent {
                    event_type: EventType::PUT_IN_MONEY_PLUS_INTEREST,
                    dao_type: DaoType::Investment,
                    component_address: Runtime::global_address(),
                    meta_data: DaoEvent::PutInMoneyPlusInterest(event_metadata_if),
                });

                (extra_money, collateral_being_taken_back)
            } else {
                let extra_money_amount = extra_money.amount();

                //event emission

                if amount_getting_deposited >= required_amount {
                    let event_metadata_if = PutInMoneyPlusInterestEvent {
                        bond_creator_address,
                        amount_getting_deposited,
                        amount_required_by_the_community: required_amount,
                        amount_taken_by_the_community: required_amount,
                        extra_amount_given_back_to_the_sender: extra_money_amount,
                        more_xrd_amount_required_by_the_community: Decimal::zero(),
                        collateral_given_back: false,
                    };

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::PUT_IN_MONEY_PLUS_INTEREST,
                        dao_type: DaoType::Investment,
                        component_address: Runtime::global_address(),
                        meta_data: DaoEvent::PutInMoneyPlusInterest(event_metadata_if),
                    });
                } else {
                    let more_xrd_amount_required_by_the_community =
                        required_amount - amount_getting_deposited;

                    let event_metadata_else = PutInMoneyPlusInterestEvent {
                        bond_creator_address,
                        amount_getting_deposited,
                        amount_required_by_the_community: required_amount,
                        amount_taken_by_the_community: amount_getting_deposited,
                        extra_amount_given_back_to_the_sender: extra_money_amount,
                        more_xrd_amount_required_by_the_community,
                        collateral_given_back: false,
                    };

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::PUT_IN_LESS_MONEY_PLUS_INTEREST,
                        dao_type: DaoType::Investment,
                        component_address: Runtime::global_address(),
                        meta_data: DaoEvent::PutInMoneyPlusInterest(event_metadata_else),
                    });
                }

                let collateral_resource_address =
                    latest_bond_component.get_resource_address_of_collateral();

                let empty_bucket = Bucket::new(collateral_resource_address);

                (extra_money, empty_bucket)
            }
        }

        pub fn check_the_balance_of_bond_issuer(
            &mut self,
            bond_creator_address: ComponentAddress,
        ) -> Decimal {
            assert!(
                self.zero_coupon_bond.contains_key(&bond_creator_address),
                "No bonds created by the specified address."
            );

            // Retrieve the most recent bond component created by the bond creator
            let bond_components = self
                .zero_coupon_bond
                .get_mut(&bond_creator_address)
                .unwrap();

            let latest_bond_component =
                bond_components.last_mut().expect("No bond component found");

            let balance = latest_bond_component.check_the_balance_of_bond_issuer();

            let event_metadata = CheckBondIssuerBalanceEvent {
                bond_creator_address,
                balance,
            };

            Runtime::emit_event(PandaoEvent {
                event_type: EventType::CHECK_BOND_ISSUER_BALANCE,
                dao_type: DaoType::Investment,
                component_address: Runtime::global_address(),
                meta_data: DaoEvent::CheckBondIssuerBalance(event_metadata),
            });

            balance
        }

        //force transfer XRDs to community vault after collateral liquidation
        //if the xrd requirement doesn't meet
        pub fn transfer_xrds_to_community_vault(&mut self, bond_creator_address: ComponentAddress) {
            assert!(
                self.zero_coupon_bond.contains_key(&bond_creator_address),
                "No bonds created by the specified address."
            );

            // Retrieve the most recent bond component created by the bond creator
            let bond_components = self
                .zero_coupon_bond
                .get_mut(&bond_creator_address)
                .unwrap();

            let latest_bond_component =
                bond_components.last_mut().expect("No bond component found");

            //required xrds
            let required_xrds = latest_bond_component.balance_required_by_the_community();
            let bond_component_balance = latest_bond_component.check_the_balance_of_bond_issuer();

            let creator_xrds = latest_bond_component.force_transfer_deposited_xrds();

            let event = ForceTransferFunds {
                bond_creator_address,
                required_amount: required_xrds,
                bond_component_balance,
                transferred_amount_to_community_vault: bond_component_balance,
            };

            Runtime::emit_event(PandaoEvent {
                event_type: EventType::FORCE_TRANSFER_OF_FUNDS,
                dao_type: DaoType::Investment,
                component_address: Runtime::global_address(),
                meta_data: DaoEvent::ForceTransferFunds(event),
            });

            self.shares.put(creator_xrds);
        }

        pub fn create_proposal_to_change_token_price(
            &mut self,
            title: String,
            description: String,
            minimun_quorum: u8,
            start_time: scrypto::time::UtcDateTime,
            end_time: scrypto::time::UtcDateTime,
            proposal_creator_address: Option<ComponentAddress>,
            governance_token_or_owner_token_address: Bucket,
            voting_type: VotingType,
            desired_token_price: Option<Decimal>,
            desired_token_buy_back_price: Option<Decimal>,
        ) -> (
            Global<crate::proposal::pandao_praposal::TokenWeightProposal>,
            String,
            Bucket,
        ) {
            match self.proposal_creation_right {
                ProposalCreationRight::EVERYONE => {
                    assert_eq!(
                        governance_token_or_owner_token_address.resource_address(),
                        self.dao_token_address,
                        "wrong voting token supplied! please make sure that you supply DAO Governance Token"
                    );

                    assert!(
                        governance_token_or_owner_token_address.amount() >= Decimal::one(),
                        "Proposal creator must have at least one governance token to create a proposal"
                    );

                    //allow proposal creation
                }
                ProposalCreationRight::TOKEN_HOLDER_THRESHOLD(threshold) => {
                    assert_eq!(
                        governance_token_or_owner_token_address.resource_address(),
                        self.dao_token_address,
                        "wrong voting token supplied! please make sure that you supply DAO Governance Token"
                    );

                    assert!(
                        governance_token_or_owner_token_address.amount() >= threshold,
                        "Proposal creator does not have enough tokens to meet the threshold"
                    );
                }
                ProposalCreationRight::ADMIN => {
                    assert_eq!(
                        governance_token_or_owner_token_address.resource_address(),
                        self.owner_token_addresss,
                        "Only the admin can create a proposal and If you are an Admin please make sure you pass OWNER TOKEN ADDRESS"
                    );

                    assert!(
                        governance_token_or_owner_token_address.amount() >= Decimal::one(),
                        "ADMIN must pass his/her OWNER TOKEN to create proposal"
                    );
                }
            }

            let address_issued_bonds_to_sell: Option<ComponentAddress> = None;
            let target_xrd_amount: Option<Decimal> = None;
            let amount_of_tokens_should_be_minted: Option<usize> = None;

            let global_proposal_component: Global<TokenWeightProposal>;

            match voting_type {
                VotingType::ResourceHold => {
                    (global_proposal_component, _) = TokenWeightProposal::new(
                        title.clone(),
                        description.clone(),
                        minimun_quorum,
                        start_time,
                        end_time,
                        self.owner_token_addresss.clone(),
                        self.dao_token_address.clone(),
                        address_issued_bonds_to_sell.clone(),
                        target_xrd_amount.clone(),
                        proposal_creator_address,
                        amount_of_tokens_should_be_minted,
                        VotingType::ResourceHold,
                        desired_token_price,
                        desired_token_buy_back_price,
                    );
                }
                VotingType::Equality => {
                    (global_proposal_component, _) = TokenWeightProposal::new(
                        title.clone(),
                        description.clone(),
                        minimun_quorum,
                        start_time,
                        end_time,
                        self.owner_token_addresss.clone(),
                        self.dao_token_address.clone(),
                        address_issued_bonds_to_sell.clone(),
                        target_xrd_amount.clone(),
                        proposal_creator_address,
                        amount_of_tokens_should_be_minted,
                        VotingType::Equality,
                        desired_token_price,
                        desired_token_buy_back_price,
                    );
                }
            }

            //start date
            //end date
            let start_time_ts: i64 = start_time.to_instant().seconds_since_unix_epoch;
            let end_time_ts: i64 = end_time.to_instant().seconds_since_unix_epoch;

            let proposal_id: usize = Self::get_proposal_id()
                .try_into()
                .expect("couldn't get called successfully");

            let inner_map = self
                .current_praposals
                .entry(proposal_creator_address.unwrap())
                .or_insert_with(HashMap::new);

            inner_map.insert(proposal_id, global_proposal_component);

            match voting_type {
                VotingType::ResourceHold => {
                    let praposal_metadata = PraposalMetadata {
                        title,
                        description,
                        minimum_quorum: minimun_quorum.into(),
                        end_time_ts,
                        start_time_ts,
                        owner_token_address: self.owner_token_addresss.clone(),
                        component_address: global_proposal_component.address(),
                        address_issued_bonds_to_sell,
                        target_xrd_amount,
                        proposal_creator_address,
                        amount_of_tokens_should_be_minted,
                        proposal_id,
                        governance_token_or_owner_token_address:
                            governance_token_or_owner_token_address.resource_address(),
                        token_type: VotingType::ResourceHold,
                        desired_token_price,
                        desired_token_buy_back_price,
                    };

                    let component_address = Runtime::global_address();

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::PROPOSAL_TO_CHANGE_TOKEN_PRICE,
                        dao_type: DaoType::Investment,
                        meta_data: DaoEvent::PraposalDeployment(praposal_metadata),
                        component_address,
                    });
                }
                VotingType::Equality => {
                    let praposal_metadata = PraposalMetadata {
                        title,
                        description,
                        minimum_quorum: minimun_quorum.into(),
                        end_time_ts,
                        start_time_ts,
                        owner_token_address: self.owner_token_addresss.clone(),
                        component_address: global_proposal_component.address(),
                        address_issued_bonds_to_sell,
                        target_xrd_amount,
                        proposal_creator_address,
                        amount_of_tokens_should_be_minted,
                        proposal_id,
                        governance_token_or_owner_token_address:
                            governance_token_or_owner_token_address.resource_address(),
                        token_type: VotingType::Equality,
                        desired_token_price,
                        desired_token_buy_back_price,
                    };

                    let component_address = Runtime::global_address();

                    Runtime::emit_event(PandaoEvent {
                        event_type: EventType::PROPOSAL_TO_CHANGE_TOKEN_PRICE,
                        dao_type: DaoType::Investment,
                        meta_data: DaoEvent::PraposalDeployment(praposal_metadata),
                        component_address,
                    });
                }
            }

            let mut message = String::new();
            message = format!("Proposal created with id : {}", proposal_id);

            (
                global_proposal_component,
                message,
                governance_token_or_owner_token_address,
            )
        }

        pub fn execute_proposal_to_change_token_price(
            &mut self,
            proposal_id: usize,
        ) -> Result<String, String> {
            for (_, inner_map) in self.current_praposals.clone() {
                let proposal = inner_map.get(&proposal_id);

                let now: Instant = Clock::current_time_rounded_to_seconds();
                let current_time_seconds: i64 = now.seconds_since_unix_epoch;

                let last_time = proposal.unwrap().get_last_time();
                let end_time_seconds = last_time.to_instant().seconds_since_unix_epoch;

                // Debug statements to verify the times
                println!("Current time (epoch seconds): {}", current_time_seconds);
                println!("Proposal end time (epoch seconds): {}", end_time_seconds);

                assert!(
                    current_time_seconds > end_time_seconds,
                    "Proposal can only be executed after the specified end time"
                );

                match proposal {
                    Some(proposal) => {
                        let number_of_voters = proposal.get_number_of_voters(); //inline attribute on fn definition
                        let minimum_quorum = proposal.get_minimum_quorum();

                        if let Some(desired_price) = proposal.get_desired_token_price() {
                            if let Some(buy_back) = proposal.get_desired_token_buy_back_price() {
                                if number_of_voters < minimum_quorum {
                                    // Emit an event indicating that the proposal cannot be executed due to insufficient participation
                                    let event_metadata = PriceChangeProposalQuorumNotMet {
                                        proposal_id,
                                        minimum_quorum: proposal.get_minimum_quorum(),
                                        number_of_voters,
                                        desired_price,
                                        desired_token_buy_back_price: buy_back,
                                        proposal_type: EventType::PROPOSAL_TO_CHANGE_TOKEN_PRICE,
                                    };

                                    let component_address = Runtime::global_address();

                                    Runtime::emit_event(PandaoEvent {
                                        event_type:
                                            EventType::PRICE_CHANGE_QUORUM_NOT_MET_AND_FAILED,
                                        dao_type: DaoType::Investment,
                                        component_address,
                                        meta_data: DaoEvent::PriceChangeProposalQuorumNotMet(
                                            event_metadata,
                                        ),
                                    });
                                }

                                self.set_price(desired_price, buy_back);

                                let event_metadata = PriceChangeProposalQuorumMet {
                                    proposal_id,
                                    minimum_quorum: proposal.get_minimum_quorum(),
                                    number_of_voters,
                                    desired_token_price: desired_price,
                                    desired_token_buy_back_price: buy_back,
                                    proposal_type: EventType::PROPOSAL_TO_CHANGE_TOKEN_PRICE,
                                };

                                let component_address = Runtime::global_address();

                                Runtime::emit_event(PandaoEvent {
                                    event_type: EventType::PRICE_CHANGE_QUORUM_MET_AND_SUCCESS,
                                    dao_type: DaoType::Investment,
                                    component_address,
                                    meta_data: DaoEvent::PriceChangeProposalQuorumMet(
                                        event_metadata,
                                    ),
                                });

                                let message = "proposal executed successfully".to_string();

                                return Ok(message);
                            } else {
                                return Err(format!("desired token buy back price is not present in a proposal with id : {proposal_id}"));
                            }
                        } else {
                            return Err(format!("desired token price is not present in a proposal with id : {proposal_id}"));
                        }
                    }
                    None => return Err(format!("proposal with id : {proposal_id} not found")),
                }
            }
            Ok("execute fn called successfully".to_string())
        }
    }
}

//push

//*initialize
// resim call-function package_sim1p4nk9h5kw2mcmwn5u2xcmlmwap8j6dzet7w7zztzz55p70rgqs4vag TokenWeigtedDao initiate "Panjab Investment DAO" 100 0 5 2 "https://pbs.twimg.com/profile_images/1643159245389713408/47gnTbms_200x200.jpg" "https://pbs.twimg.com/profile_images/1548373397289455616/OFhGnboY_400x400.jpg" "This is a DAO for managing community projects"
// resim call-function package_sim1pk3cmat8st4ja2ms8mjqy2e9ptk8y6cx40v4qnfrkgnxcp2krkpr92 TokenWeigtedDao initiate "Panjab Investment DAO" 100 0 5 2 "https://pbs.twimg.com/profile_images/1643159245389713408/47gnTbms_200x200.jpg" "https://pbs.twimg.com/profile_images/1548373397289455616/OFhGnboY_400x400.jpg" "This is a DAO for managing community projects" --manifest instantiate_pandao.rtm

// account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma
// component_sim1czwnyl3pfn955s45a2js64w8zjlptwz4y3w4wwwl944rk2l2ceapsc
//

//*obtain_token
// resim call-method component_sim1czwnyl3pfn955s45a2js64w8zjlptwz4y3w4wwwl944rk2l2ceapsc obtain_token resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:5 1
// resim call-method component_sim1cpwu4wc6rg0am8l9prnh2lzqkk6hue6stzqhdx48rzvek2mmm5vp0p obtain_community_token resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:5 1

//*create_zero_coupon_bonds
// resim call-method component_sim1cpwu4wc6rg0am8l9prnh2lzqkk6hue6stzqhdx48rzvek2mmm5vp0p create_zero_coupon_bond "Corporate Bond" "Issuer" "Contract ID 123" 0.05 "USD" 1694774400 1695052800 1000000 5 "Secondary Market" 105 100

//*purchase_a_bond
// resim call-method component_sim1cpwu4wc6rg0am8l9prnh2lzqkk6hue6stzqhdx48rzvek2mmm5vp0p purchase_bond resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:105

//*sell_a_bond
// resim call-method component_sim1cpwu4wc6rg0am8l9prnh2lzqkk6hue6stzqhdx48rzvek2mmm5vp0p sell_bond resource_sim1tklvuzvc60lvdc2dmrszpa20n2tu3vw839x97gtq6ezvx2qu04k5yz:1

//*check_bond_maturity
// resim call-method component_sim1cpwu4wc6rg0am8l9prnh2lzqkk6hue6stzqhdx48rzvek2mmm5vp0p check_bond_maturity

//*get_bond_details
// resim call-method component_sim1cpwu4wc6rg0am8l9prnh2lzqkk6hue6stzqhdx48rzvek2mmm5vp0p get_bond_details

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
