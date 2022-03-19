#[cfg(test)]
mod tests {
    use crate::helpers::CwBridgeContract;
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::{coin, Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    use cw_gateway;

    pub fn cw_bridge_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ).with_reply(crate::contract::reply);
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
    const NATIVE_DENOM: &str = "uluna";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(100_000_000),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, CwBridgeContract) {
        let mut app = mock_app();

        let cw_gateway_id = app.store_code(cw_gateway_contract());
        let msg = cw_gateway::msg::InstantiateMsg { };
        let cw_gateway_contract_addr = app
            .instantiate_contract(
                cw_gateway_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap()
            .to_string();

        let cw_bridge_id = app.store_code(cw_bridge_contract());
        let msg = InstantiateMsg { cw_gateway_contract_addr };
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

        let cw_bridge_contract = CwBridgeContract(cw_bridge_contract_addr);

        (app, cw_bridge_contract)
    }

    mod transfers {
        use super::*;
        use crate::{msg::ExecuteMsg, state::State};

        #[test]
        fn initiate_transfer() {
            let (mut app, cw_bridge_contract) = proper_instantiate();

            let msg = ExecuteMsg::InitiateTransfer {};
            let funds = vec![
                coin(1_000_000, "uluna"), // transfer subject
            ];
            let cosmos_msg = cw_bridge_contract.call(msg, Some(funds)).unwrap();
            let r = app.execute(Addr::unchecked(USER), cosmos_msg);

            println!("YOOO {:?}", r);

            let r = app.wrap().query_all_balances(USER).unwrap();
            println!("USER coins {:?}", r);

            let r = app.wrap().query_all_balances("Contract #0").unwrap();
            println!("GATEWAY coins {:?}", r);

            let r = app.wrap().query_all_balances("Contract #1").unwrap();
            println!("BRIDGE coins {:?}", r);

            let s: State = app
                .wrap()
                .query_wasm_smart("Contract #1", &crate::msg::QueryMsg::State)
                .unwrap();

            println!("BRIDGE state {:?}", s);
        }
    }
}
