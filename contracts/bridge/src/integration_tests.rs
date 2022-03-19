#[cfg(test)]
mod tests {
    use crate::helpers::Contract as CwContract;
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::{coin, Addr, Coin, Empty, ReplyOn};
    use cw_gateway;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn cw_bridge_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
        Box::new(contract)
    }

    pub fn cw_gateway_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw_gateway::contract::execute,
            cw_gateway::contract::instantiate,
            cw_gateway::contract::query,
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
        })
    }

    struct ProperInstantiateProps {
        initial_user_coins: Coin,
        reentrancy_prevention_flag: u8,
        minimal_transfer_requirement: Option<Coin>,
        broadcast_fee: Option<Coin>,
        reply_on_mode: ReplyOn,
    }

    fn proper_instantiate(props: ProperInstantiateProps) -> (App, CwContract, CwContract) {
        let ProperInstantiateProps {
            initial_user_coins,
            broadcast_fee,
            reentrancy_prevention_flag,
            minimal_transfer_requirement,
            reply_on_mode,
        } = props;
        let mut app = mock_app(initial_user_coins.clone());

        let cw_gateway_id = app.store_code(cw_gateway_contract());
        let msg = cw_gateway::msg::InstantiateMsg { broadcast_fee };
        let cw_gateway_contract_addr = app
            .instantiate_contract(
                cw_gateway_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_gateway_contract = CwContract(cw_gateway_contract_addr.into());

        let cw_bridge_id = app.store_code(cw_bridge_contract());
        let msg = InstantiateMsg {
            cw_gateway_contract_addr: cw_gateway_contract.addr().to_string(),
            reentrancy_prevention_flag,
            minimal_transfer_requirement,
            reply_on_mode,
        };
        let cw_bridge_contract_addr = app
            .instantiate_contract(
                cw_bridge_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_bridge_contract = CwContract(cw_bridge_contract_addr);

        (app, cw_gateway_contract, cw_bridge_contract)
    }

    mod transfers {
        use super::*;
        use crate::msg::{ExecuteMsg, StateResponse};

        #[test]
        fn initiate_transfer_when_locking_funds_submessage_fails() {
            let initial_user_coins = coin(100_000_000, "uluna");
            let expected_broadcast_fee = coin(1_500, "uluna");
            let expected_coin_to_transfer = coin(10_000, "uluna");

            let (mut app, cw_gateway_contract, cw_bridge_contract) =
                proper_instantiate(ProperInstantiateProps {
                    initial_user_coins: initial_user_coins.clone(),
                    reentrancy_prevention_flag: 0,
                    minimal_transfer_requirement: None,
                    broadcast_fee: Some(expected_broadcast_fee.clone()),
                    reply_on_mode: ReplyOn::Success,
                });
            let coins_to_transfer = vec![expected_coin_to_transfer.clone()];

            let cosmos_msg = cw_bridge_contract
                .call(
                    ExecuteMsg::InitiateTransfer {},
                    Some(coins_to_transfer.clone()),
                )
                .unwrap();
            let r = app.execute(Addr::unchecked(USER), cosmos_msg);

            assert!(r.is_ok());

            let user_coins = app.wrap().query_all_balances(USER).unwrap();

            println!("User coins: {:?}", &user_coins);

            assert_eq!(
                user_coins.get(0).unwrap().amount,
                initial_user_coins.amount - expected_coin_to_transfer.amount,
                "gatway charged the broadcast fee"
            );

            let gateway_coins = app
                .wrap()
                .query_all_balances(&cw_gateway_contract.addr())
                .unwrap();

            println!("Gateway coins: {:?}", &gateway_coins);

            assert_eq!(
                gateway_coins.get(0).unwrap().amount,
                expected_broadcast_fee.amount,
                "gatway charged the broadcast fee"
            );

            let bridge_coins = app
                .wrap()
                .query_all_balances(&cw_bridge_contract.addr())
                .unwrap();

            println!("Bridge coins: {:?}", &bridge_coins);

            assert_eq!(
                bridge_coins.get(0).unwrap().amount,
                expected_coin_to_transfer.amount - expected_broadcast_fee.amount,
                "bridge locked transferred assets MINUS the broadcast fee on the Gateway"
            );

            let state_resposnse: StateResponse = app
                .wrap()
                .query_wasm_smart(&cw_bridge_contract.addr(), &crate::msg::QueryMsg::State)
                .unwrap();
            let state = state_resposnse.state;

            assert_eq!(state.reentrancy_prevention_flag, 2);
        }
    }
}
