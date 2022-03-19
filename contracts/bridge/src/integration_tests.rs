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
            .unwrap();

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
        use crate::msg::ExecuteMsg;

        #[test]
        fn initiate_transfer() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let msg = ExecuteMsg::InitiateTransfer {};
            let funds = vec![
                coin(1_000_000, "uluna"), // transfer subject
            ];
            let cosmos_msg = cw_template_contract.call(msg, Some(funds)).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
        }
    }
}
