#[cfg(test)]
mod tests {
    use crate::helpers::Contract as CwBorrowerContract;
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::{coin, Addr, Coin, Empty};
    use cw_flash_loan_gateway::{self, helpers::Contract as CwGatewayContract};
    use cw_flash_loan_vault::{self, helpers::Contract as CwVaultContract};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn cw_borrower_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );

        Box::new(contract)
    }

    pub fn cw_gateway_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw_flash_loan_gateway::contract::execute,
            cw_flash_loan_gateway::contract::instantiate,
            cw_flash_loan_gateway::contract::query,
        )
        .with_reply(cw_flash_loan_gateway::contract::reply);

        Box::new(contract)
    }

    pub fn cw_vault_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw_flash_loan_vault::contract::execute,
            cw_flash_loan_vault::contract::instantiate,
            cw_flash_loan_vault::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";

    fn mock_app(initial_user_balance: Coin) -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(USER), vec![initial_user_balance])
                .unwrap();

            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(ADMIN),
                    vec![coin(1_000_000_000_000_000_000, "uluna")],
                )
                .unwrap();
        })
    }

    struct ProperInstantiateProps {
        initial_user_coins: Coin,
        initial_vault_coins: Coin,
    }

    fn proper_instantiate(
        props: ProperInstantiateProps,
    ) -> (App, CwBorrowerContract, CwVaultContract, CwGatewayContract) {
        let ProperInstantiateProps {
            initial_user_coins,
            initial_vault_coins,
        } = props;
        let mut app = mock_app(initial_user_coins.clone());

        let cw_vault_id = app.store_code(cw_vault_contract());
        let msg = cw_flash_loan_vault::msg::InstantiateMsg {};
        let cw_vault_contract_addr = app
            .instantiate_contract(
                cw_vault_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[initial_vault_coins],
                "test",
                None,
            )
            .unwrap();

        let cw_vault_contract = CwVaultContract(cw_vault_contract_addr.into());

        let cw_gateway_id = app.store_code(cw_gateway_contract());
        let msg = cw_flash_loan_gateway::msg::InstantiateMsg {
            cw_vault_contract_addr: cw_vault_contract.addr().to_string(),
        };
        let cw_gateway_contract_addr = app.instantiate_contract(
            cw_gateway_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "test",
            None,
        );

        let cw_gateway_contract_addr = cw_gateway_contract_addr.unwrap();

        let cw_gateway_contract = CwGatewayContract(cw_gateway_contract_addr);

        let cw_borrower_id = app.store_code(cw_borrower_contract());
        let msg = InstantiateMsg {
            cw_gateway_contract_addr: cw_gateway_contract.addr().to_string(),
        };
        let cw_borrower_contract_addr = app
            .instantiate_contract(
                cw_borrower_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[initial_user_coins],
                "test",
                None,
            )
            .unwrap();

        let cw_borrower_contract = CwBorrowerContract(cw_borrower_contract_addr);

        (
            app,
            cw_borrower_contract,
            cw_vault_contract,
            cw_gateway_contract,
        )
    }

    mod transfers {
        use super::*;

        #[test]
        fn borrower_request_flash_loan() {
            let initial_vault_coins = coin(200_000_000_000, "uluna");
            let initial_user_coins = coin(50_250, "uluna");
            let expected_coin_to_borrow = coin(175_000_000, "uluna");

            let (mut app, cw_borrower_contract, cw_vault_contract, cw_gateway_contract) =
                proper_instantiate(ProperInstantiateProps {
                    initial_vault_coins: initial_vault_coins.clone(),
                    initial_user_coins: initial_user_coins.clone(),
                });
            
            print_balances(
                "Initial balances",
                &app,
                &cw_borrower_contract,
                &cw_gateway_contract,
                &cw_vault_contract,
            );

            let cosmos_msg = cw_borrower_contract
                .call(
                    crate::msg::ExecuteMsg::OpenFlashLoan {
                        asset_to_borrow: expected_coin_to_borrow.clone(),
                    },
                    None,
                )
                .unwrap();

            let r = app.execute(cw_gateway_contract.addr(), cosmos_msg);
            println!("Response: {:?}", &r);

            print_balances(
                "End balances",
                &app,
                &cw_borrower_contract,
                &cw_gateway_contract,
                &cw_vault_contract,
            );

            // assert!(r.is_ok());

            // assert_eq!(
            //     bridge_coins.get(0).unwrap().amount,
            //     expected_coin_to_transfer.amount - expected_broadcast_fee.amount,
            //     "bridge locked transferred assets MINUS the broadcast fee on the Gateway"
            // );

            // let state_resposnse: StateResponse = app
            //     .wrap()
            //     .query_wasm_smart(&cw_bridge_contract.addr(), &crate::msg::QueryMsg::State)
            //     .unwrap();
            // let state = state_resposnse.state;

            // assert_eq!(state.reentrancy_prevention_flag, 2);
        }

        fn print_balances(
            label: &str,
            app: &App,
            cw_borrower_contract: &CwBorrowerContract,
            cw_gateway_contract: &CwGatewayContract,
            cw_vault_contract: &CwVaultContract,
        ) {
            let borrower_coins = app
                .wrap()
                .query_all_balances(cw_borrower_contract.addr())
                .unwrap();

            println!("[{}]: Borrower = {:?}", label, &borrower_coins);

            let gateway_coins = app
                .wrap()
                .query_all_balances(cw_gateway_contract.addr())
                .unwrap();

            println!("[{}]: Gateway = {:?}", label, &gateway_coins);

            let vault_coins = app
                .wrap()
                .query_all_balances(cw_vault_contract.addr())
                .unwrap();

            println!("[{}]: Vault = {:?}", label, &vault_coins);
        }

        #[test]
        fn gateway_requests_funds_from_vault() {
            let initial_vault_coins = coin(200_000_000_000, "uluna");
            let initial_user_coins = coin(50_250, "uluna");
            let expected_coin_to_borrow = coin(175_000_000, "uluna");

            let (mut app, _cw_borrower_contract, cw_vault_contract, cw_gateway_contract) =
                proper_instantiate(ProperInstantiateProps {
                    initial_vault_coins: initial_vault_coins.clone(),
                    initial_user_coins: initial_user_coins.clone(),
                });

            let cosmos_msg = cw_vault_contract
                .call(
                    cw_flash_loan_vault::msg::ExecuteMsg::LendAsset {
                        asset: expected_coin_to_borrow.clone(),
                        borrower_addr: String::from(USER),
                    },
                    None,
                )
                .unwrap();

            let _r = app.execute(cw_gateway_contract.addr(), cosmos_msg);
            // println!("Response: {:?}", &r);

            // assert!(r.is_ok());

            let user_coins = app.wrap().query_all_balances(USER).unwrap();

            println!("User coins: {:?}", &user_coins);

            // assert_eq!(
            //     user_coins.get(0).unwrap().amount,
            //     initial_user_coins.amount - expected_coin_to_transfer.amount,
            //     "gatway charged the broadcast fee"
            // );

            let gateway_coins = app
                .wrap()
                .query_all_balances(&cw_gateway_contract.addr())
                .unwrap();

            println!("Gateway coins: {:?}", &gateway_coins);

            // assert_eq!(
            //     gateway_coins.get(0).unwrap().amount,
            //     expected_broadcast_fee.amount,
            //     "gatway charged the broadcast fee"
            // );

            let vault = app
                .wrap()
                .query_all_balances(&cw_vault_contract.addr())
                .unwrap();

            println!("Vault coins: {:?}", &vault);

            // assert_eq!(
            //     bridge_coins.get(0).unwrap().amount,
            //     expected_coin_to_transfer.amount - expected_broadcast_fee.amount,
            //     "bridge locked transferred assets MINUS the broadcast fee on the Gateway"
            // );

            // let state_resposnse: StateResponse = app
            //     .wrap()
            //     .query_wasm_smart(&cw_bridge_contract.addr(), &crate::msg::QueryMsg::State)
            //     .unwrap();
            // let state = state_resposnse.state;

            // assert_eq!(state.reentrancy_prevention_flag, 2);
        }
    }
}
