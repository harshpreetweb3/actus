    use scrypto::prelude::*;

    #[derive(ScryptoSbor, Debug)]
    pub struct BondDetails {
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
        pub amount: Decimal,
        pub maturity_days_left: i64,
       
    }

    #[blueprint]
    mod zerocouponbond {

        struct ZeroCouponBond {
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
            bonds: Vault,
            collected_xrd: Vault,
            price: u64,
            pub bond_resourse_address : ResourceAddress,
            pub collateral : Vault,
            pub money_taken_by_creator : bool,
            pub successful_claim_by_community : bool
        }

        impl ZeroCouponBond {
            // PAM issuer CONTRACT1234 0.04 30/360 USD 4724 4729 1000 100

            // No of bonds being issued by the bond issuer / creator ?
            // I think we will make bond a Resource (either Fungible / Non Fungible)
            // Bonds would be nothing but NFTs/ you can say tokens
            // like gumball, bond creator will set the price and mint some amount of bonds so that they can be purchased by the investors
            // XRD will be paid to purchase/invest in the bonds
            // Token will represent a bond
            // Token would be of some face value eg 1000; we do have to give some face value to the bond

            // PAM issuer CONTRACT1234 0.04 USD 1720100602 1727876602 1000 100 long 900 100
            pub fn instantiate_zerocouponbond(
                contract_type: String,          // PAM
                contract_role: String,          // issuer
                contract_identifier: String,    // unique id for a contract
                nominal_interest_rate: Decimal, // how much interest is being given by a bond
                currency: String,               // for example : XRD
                initial_exchange_date: u64,     // initial exchange date
                maturity_date: u64,             // date when bond matures
                notional_principal: Decimal,    // price defined by bond creator
                discount: u64,                  // discount on bond
                bond_position: String,          // long or short position
                price: u64,                 // price per bond
                number_of_bonds: Decimal,       // number of bonds to mint
                nft_as_collateral: Bucket,      // collateral for bonds
            ) -> Global<ZeroCouponBond> {
                let bucket_of_bonds: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                    .divisibility(DIVISIBILITY_NONE)
                    .metadata(metadata!(
                        init {
                            "name" => "ZeroCouponBond", locked;
                            "symbol" => "ZCB", locked;
                            "description" => "A Zero Coupon Bond", locked;
                        }
                    ))  
                    .mint_initial_supply(number_of_bonds)
                    .into();

                let bond_resourse_address = bucket_of_bonds.resource_address();

                Self {
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
                    bonds: Vault::with_bucket(bucket_of_bonds),
                    collected_xrd: Vault::new(XRD),
                    price,
                    bond_resourse_address,
                    collateral: Vault::with_bucket(nft_as_collateral),
                    money_taken_by_creator : false,
                    successful_claim_by_community : false
                }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()

            }

            pub fn get_resource_address(&self) -> ResourceAddress{
                return self.bond_resourse_address;
            }

            pub fn purchase_bond(&mut self, mut payment: Bucket) -> (Bucket, Bucket) {
                let our_share = payment.take(self.price);
                self.collected_xrd.put(our_share);
                (self.bonds.take(1), payment)
            }

            pub fn sell_the_bond(&mut self, bond: Bucket) -> Bucket {
                assert!(
                    bond.amount() == Decimal::one(),
                    "You can only sell one bond at a time."
                );
                assert!(
                    bond.resource_address() == self.bonds.resource_address(),
                    "Invalid bond resource."
                );

                // Bond should only be sold after the maturity period
                // Appllllllllly the condition


                self.bonds.put(bond);
                let refund_amount = self.notional_principal + Decimal::from(self.discount);
                self.collected_xrd.take(refund_amount)
            }

            pub fn check_the_maturity_of_bonds(&self) -> i64 {
                
                let current_epoch = Runtime::current_epoch().number();
                let seconds_in_day = 24 * 60 * 60;
                let days_left = (self.maturity_date as i64 - current_epoch as i64) / seconds_in_day;
                days_left
            }

            pub fn get_bond_details(&self) -> BondDetails {
                let current_epoch = Runtime::current_epoch().number();
                let seconds_in_day = 24 * 60 * 60;
                let days_left = (self.maturity_date as i64 - current_epoch as i64) / seconds_in_day;
            
                BondDetails {
                    contract_type: self.contract_type.clone(),
                    contract_role: self.contract_role.clone(),
                    contract_identifier: self.contract_identifier.clone(),
                    nominal_interest_rate: self.nominal_interest_rate,
                    currency: self.currency.clone(),
                    initial_exchange_date: self.initial_exchange_date,
                    maturity_date: self.maturity_date,
                    notional_principal: self.notional_principal,
                    discount: self.discount,
                    bond_position: self.bond_position.clone(),
                    price: self.price,
                    amount: self.bonds.amount(),
                    maturity_days_left: days_left,
                }
            }

            // LIQUIDATE COLLATERAL
            // ONLY COMMUNITY CAN CALL THIS
            pub fn liquidate_collateral(&mut self) -> Bucket {

                // AFTER MATURITY DATE
                let now: Instant = Clock::current_time_rounded_to_seconds();
                let current_time_seconds: u64 = now.seconds_since_unix_epoch as u64;
                //CHECK IF MATURITY DATE PASSED
                assert!(self.maturity_date < current_time_seconds, "you cannot redeem the collateral because maturity date is not passed yet");
                
                self.collateral.take(1)
            }

            pub fn get_maturity_data(&self) -> u64{
                self.maturity_date
            }

            pub fn get_back_the_collateral(&mut self) -> Bucket {
                
                // if self.successful_claim_by_community == true{
                    //now that community claimed their interest plus principal amount
                    //now we can let the creator take back his collateral
                    self.collateral.take(1)
                // }else{
                //     Bucket::new(self.collateral.resource_address())
                // }
            }

            pub fn get_money_claim_status(&self) -> bool{
                self.successful_claim_by_community
            }

            pub fn get_resource_address_of_collateral(&self) -> ResourceAddress{
                self.collateral.resource_address()
            }

            //GET BACK THE INVESTED XRD + INTEREST RATE
            //FOR A COMMUNITY
            pub fn claim_the_invested_XRDs_plus_interest(&mut self) -> Bucket{

                let bond_price = self.price;

                let interest_amount = (self.nominal_interest_rate/100)*bond_price;

                let total_amount = bond_price + interest_amount;

                self.collected_xrd.take(total_amount)

            }

            //XRDs required by the community
            pub fn balance_required_by_the_community(&self) -> Decimal{
                
                let bond_price = self.price;

                let interest_amount = (self.nominal_interest_rate/100)*bond_price;

                let total_amount = bond_price + interest_amount;

                total_amount
            }

            pub fn change_community_claim_status(&mut self, value : bool){
                self.successful_claim_by_community = value;
            }

            pub fn take_out_the_invested_XRDs_by_the_community(&mut self)
            -> Bucket
            {
                let bond_price = self.price;
                self.collected_xrd.take(bond_price)
            }

            pub fn bond_creator_money_status(&self) -> bool{
                self.money_taken_by_creator
            }
            

            pub fn put_in_money_plus_interest_for_the_community_to_redeem(&mut self, mut borrowed_xrd_with_interest : Bucket) -> Bucket {

                let required_amount_by_the_community = self.balance_required_by_the_community();

                let resource_address_of_xrds = borrowed_xrd_with_interest.resource_address();

                let amount_getting_deposited = borrowed_xrd_with_interest.amount();

                if amount_getting_deposited >= required_amount_by_the_community{

                    let taken_out_required_amount = borrowed_xrd_with_interest.take(required_amount_by_the_community);

                    self.collected_xrd.put(taken_out_required_amount);
    
                    borrowed_xrd_with_interest

                }else{

                    self.collected_xrd.put(borrowed_xrd_with_interest);

                    // this is an emtpy bucket 
                    Bucket::new(resource_address_of_xrds)
                }

            }

            pub fn check_the_balance_of_bond_issuer(&self) 
            -> Decimal
            {
                let balance = self.collected_xrd.amount();
                balance
            }

            pub fn get_bond_u_id(&self) -> String{
                let bond_u_id = self.contract_identifier.clone();
                bond_u_id
            }

            pub fn force_transfer_deposited_xrds(&mut self) -> Bucket{
                
                let balance = self.check_the_balance_of_bond_issuer();

                self.collected_xrd.take(balance)
            }
        }
    }

    // resim call-function package_sim1pk3cmat8st4ja2ms8mjqy2e9ptk8y6cx40v4qnfrkgnxcp2krkpr92 ZeroCouponBond instantiate_zerocouponbond PAM issuer CONTRACT1234 0.04 USD 1720100602 1727876602 1000 100 long 900 100
    // component_sim1cp4qmcqlmtsqns8ckwjttvffjk4j4smkhlkt0qv94caftlj5u2xve2
    // resim show component_sim1cp4qmcqlmtsqns8ckwjttvffjk4j4smkhlkt0qv94caftlj5u2xve2
    // resim show account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma

    // resim call-method component_sim1cp4qmcqlmtsqns8ckwjttvffjk4j4smkhlkt0qv94caftlj5u2xve2 get_bond_details 
    // resim call-method component_sim1cp4qmcqlmtsqns8ckwjttvffjk4j4smkhlkt0qv94caftlj5u2xve2 purchase_bond resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:900
    // resim call-method component_sim1cp4qmcqlmtsqns8ckwjttvffjk4j4smkhlkt0qv94caftlj5u2xve2 check_the_maturity_of_bonds

    



    // idea
    // 1. instead of the IED and MD, we can take maturity_period such as 3M/90D, then after 3M/90D, amount will be repaid ...

    // 2. Zero Coupon Contract with an owner 

    // bond creator can set_bond_price post issuing/instantiating a contract but here it is on him to decide the face value of bond
    // I believe bond price must be dependent on multiple factors so what those factors are, how can I/an investor dynamically see changes in the bond prices . . . 

    // owner would be able to withdraw the earning he earned post selling the bonds
    // but he would have to dispense the earning back to the contract before the maturity date

    // 3. ReMint more bonds

    // there would be function to check how many bonds are remaining in the contract
    // contract owner will mint more bonds if wants to increase up the no_of_zero_coupon_bonds


    //*new
    // component_sim1cr9ep7596uvmt45kmxczrkxf8a5vevsnl5xw42heven9qkkg5xukme

    // resource_sim1tklvuzvc60lvdc2dmrszpa20n2tu3vw839x97gtq6ezvx2qu04k5yz: 100 ZeroCouponBond (ZCB)

    // Blueprint ID: { package_address: package_sim1pk3cmat8st4ja2ms8mjqy2e9ptk8y6cx40v4qnfrkgnxcp2krkpr92, blueprint_name: "ZeroCouponBond" }
