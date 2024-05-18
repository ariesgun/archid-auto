mod common;

use abstract_app::abstract_core::{
    app::BaseQueryMsgFns,
    objects::{
        ans_host::AnsHostError, dependency::DependencyResponse, module_version::ModuleDataResponse,
        AnsAsset, AssetEntry, DexAssetPairing, PoolAddress, PoolReference, UncheckedContractEntry,
        UniquePoolId,
    },
};
use abstract_app::abstract_interface::*;
use abstract_app::abstract_sdk::AbstractSdkError;
use abstract_client::{AbstractClient, Account, Application, Namespace};
use common::contracts;
use cosmwasm_std::{coin, coins, to_json_binary, Addr, Decimal, StdError, Uint128};
use croncat_app::{
    contract::{CRONCAT_ID, CRONCAT_MODULE_VERSION},
    croncat_integration_utils::{AGENTS_NAME, MANAGER_NAME, TASKS_NAME},
    AppQueryMsgFns as CronCatAppQueryMsgFns, Croncat, CRON_CAT_FACTORY,
};
use croncat_sdk_agents::msg::InstantiateMsg as AgentsInstantiateMsg;
use croncat_sdk_factory::msg::{
    ContractMetadataResponse, FactoryInstantiateMsg, FactoryQueryMsg, ModuleInstantiateInfo,
    VersionKind,
};
use croncat_sdk_manager::msg::ManagerInstantiateMsg;
use croncat_sdk_tasks::msg::TasksInstantiateMsg;
use cw20::Cw20Coin;
use cw_asset::AssetInfo;
// Use prelude to get all the necessary imports
use cw_orch::mock::cw_multi_test::Executor;
use cw_orch::{anyhow, prelude::*};
use app::{
    contract::{APP_ID, APP_VERSION},
    msg::{AppInstantiateMsg, ConfigResponse},
    state::{TaskEntry, TaskId},
    *,
};
use wyndex_bundle::{WynDex, EUR, USD, WYNDEX};

#[allow(unused)]
struct CronCatAddrs {
    factory: Addr,
    manager: Addr,
    tasks: Addr,
    agents: Addr,
}

#[allow(unused)]
struct DeployedApps {
    arch_app: Application<MockBech32, AppInterface<MockBech32>>,
    cron_cat_app: Application<MockBech32, Croncat<MockBech32>>,
    wyndex: WynDex,
}
// consts for testing
const AGENT: &str = "agent";
const VERSION: &str = "1.0";
const DENOM: &str = "abstr";
const PAUSE_ADMIN: &str = "cosmos338dwgj5wm2tuahvfjdldz5s8hmt7l5aznw8jz9s2mmgj5c52jqgfq000";

/// A low-level cw-orchestrator setup script
/// "low-level" because cron-cat doesn't use cw-orch itself.
fn setup_croncat_contracts(
    mock: MockBech32,
    proxy_addr: String,
) -> anyhow::Result<(CronCatAddrs, Addr)> {
    let sender = mock.sender();
    let pause_admin = mock.addr_make(PAUSE_ADMIN);
    let agent_addr = mock.addr_make(AGENT);

    let mut app = mock.app.borrow_mut();
    // Instantiate cw20
    let cw20_code_id = app.store_code(contracts::cw20_contract());
    let cw20_addr = app.instantiate_contract(
        cw20_code_id,
        sender.clone(),
        &cw20_base::msg::InstantiateMsg {
            name: "croncatcoins".to_owned(),
            symbol: "ccc".to_owned(),
            decimals: 6,
            initial_balances: vec![Cw20Coin {
                address: proxy_addr,
                amount: Uint128::new(105),
            }],
            mint: None,
            marketing: None,
        },
        &[],
        "cw20-contract".to_owned(),
        None,
    )?;

    let factory_code_id = app.store_code(contracts::croncat_factory_contract());
    let factory_addr = app.instantiate_contract(
        factory_code_id,
        sender.clone(),
        &FactoryInstantiateMsg {
            owner_addr: Some(sender.to_string()),
        },
        &[],
        "croncat-factory",
        None,
    )?;

    // Instantiate manager
    let code_id = app.store_code(contracts::croncat_manager_contract());
    let msg = ManagerInstantiateMsg {
        version: Some("1.0".to_owned()),
        croncat_tasks_key: (TASKS_NAME.to_owned(), [1, 0]),
        croncat_agents_key: (AGENTS_NAME.to_owned(), [1, 0]),
        pause_admin: pause_admin.clone(),
        gas_price: None,
        treasury_addr: None,
        cw20_whitelist: Some(vec![cw20_addr.to_string()]),
    };
    let module_instantiate_info = ModuleInstantiateInfo {
        code_id,
        version: [1, 0],
        commit_id: "commit1".to_owned(),
        checksum: "checksum123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_json_binary(&msg).unwrap(),
        contract_name: MANAGER_NAME.to_owned(),
    };
    app.execute_contract(
        sender.clone(),
        factory_addr.clone(),
        &croncat_factory::msg::ExecuteMsg::Deploy {
            kind: VersionKind::Agents,
            module_instantiate_info,
        },
        &[Coin {
            denom: DENOM.to_owned(),
            amount: Uint128::new(1),
        }],
    )
    .unwrap();

    // Instantiate agents
    let code_id = app.store_code(contracts::croncat_agents_contract());
    let msg = AgentsInstantiateMsg {
        version: Some(VERSION.to_owned()),
        croncat_manager_key: (MANAGER_NAME.to_owned(), [1, 0]),
        croncat_tasks_key: (TASKS_NAME.to_owned(), [1, 0]),
        pause_admin: pause_admin.clone(),
        agent_nomination_duration: None,
        min_tasks_per_agent: None,
        min_coins_for_agent_registration: None,
        agents_eject_threshold: None,
        min_active_agent_count: None,
        allowed_agents: Some(vec![agent_addr.to_string()]),
        public_registration: true,
    };
    let module_instantiate_info = ModuleInstantiateInfo {
        code_id,
        version: [1, 0],
        commit_id: "commit123".to_owned(),
        checksum: "checksum321".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_json_binary(&msg).unwrap(),
        contract_name: AGENTS_NAME.to_owned(),
    };
    app.execute_contract(
        sender.clone(),
        factory_addr.to_owned(),
        &croncat_factory::msg::ExecuteMsg::Deploy {
            kind: VersionKind::Agents,
            module_instantiate_info,
        },
        &[],
    )
    .unwrap();

    // Instantiate tasks
    let code_id = app.store_code(contracts::croncat_tasks_contract());
    let msg = TasksInstantiateMsg {
        version: Some(VERSION.to_owned()),
        chain_name: "atom".to_owned(),
        pause_admin,
        croncat_manager_key: (MANAGER_NAME.to_owned(), [1, 0]),
        croncat_agents_key: (AGENTS_NAME.to_owned(), [1, 0]),
        slot_granularity_time: None,
        gas_base_fee: None,
        gas_action_fee: None,
        gas_query_fee: None,
        gas_limit: None,
    };
    let module_instantiate_info = ModuleInstantiateInfo {
        code_id,
        version: [1, 0],
        commit_id: "commit1".to_owned(),
        checksum: "checksum2".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_json_binary(&msg).unwrap(),
        contract_name: TASKS_NAME.to_owned(),
    };
    app.execute_contract(
        sender,
        factory_addr.to_owned(),
        &croncat_factory::msg::ExecuteMsg::Deploy {
            kind: VersionKind::Tasks,
            module_instantiate_info,
        },
        &[],
    )
    .unwrap();

    let metadata: ContractMetadataResponse = app
        .wrap()
        .query_wasm_smart(
            factory_addr.clone(),
            &croncat_sdk_factory::msg::FactoryQueryMsg::LatestContract {
                contract_name: MANAGER_NAME.to_owned(),
            },
        )
        .unwrap();
    let manager_address = metadata.metadata.unwrap().contract_addr;

    let metadata: ContractMetadataResponse = app
        .wrap()
        .query_wasm_smart(
            factory_addr.clone(),
            &croncat_sdk_factory::msg::FactoryQueryMsg::LatestContract {
                contract_name: TASKS_NAME.to_owned(),
            },
        )
        .unwrap();

    let tasks_address = metadata.metadata.unwrap().contract_addr;

    let response: ContractMetadataResponse = app.wrap().query_wasm_smart(
        &factory_addr,
        &FactoryQueryMsg::LatestContract {
            contract_name: AGENTS_NAME.to_string(),
        },
    )?;
    let agents_addr = response.metadata.unwrap().contract_addr;
    app.execute_contract(
        agent_addr,
        agents_addr.clone(),
        &croncat_sdk_agents::msg::ExecuteMsg::RegisterAgent {
            payable_account_id: None,
        },
        &[],
    )?;


    Ok((
        CronCatAddrs {
            factory: factory_addr,
            manager: manager_address,
            tasks: tasks_address,
            agents: agents_addr,
        },
        cw20_addr,
    ))
}

/// Set up the test environment with the contract installed
#[allow(clippy::type_complexity)]
fn setup() -> anyhow::Result<(
    MockBech32,
    Account<MockBech32>,
    AbstractClient<MockBech32>,
    DeployedApps,
    CronCatAddrs,
)> {
    // Create the mock
    let mock = MockBech32::new("mock");
    let sender = mock.sender();

    // With funds
    mock.add_balance(&sender, coins(6_000_000_000, DENOM))?;
    mock.add_balance(&mock.addr_make(AGENT), coins(6_000_000_000, DENOM))?;

    let (cron_cat_addrs, _) = setup_croncat_contracts(mock.clone(), sender.to_string())?;

    // Construct the DCA interface

    // QUEST #4 You need to deploy Abstract and the dependencies before deploying your app.
    // We made this super easy! Just use the `AbstractClient` and `Publisher` to deploy the dependencies.

    // Deploy Abstract to the mock with the client
    let abstract_client = AbstractClient::builder(mock.clone())
        .assets(vec![("denom".to_owned(), AssetInfo::native(DENOM).into())])
        .contract(
            UncheckedContractEntry::try_from(CRON_CAT_FACTORY)?,
            cron_cat_addrs.factory.to_string(),
        )
        .build()?;

    // Deploy wyndex to the mock
    let wyndex = wyndex_bundle::WynDex::deploy_on(mock.clone(), Empty {})?;

    let abstract_publisher = abstract_client
        .publisher_builder(Namespace::from_id(APP_ID)?)
        .build()?;

    // Create account for croncat namespace
    let cron_cat_publisher = abstract_client
        .publisher_builder(Namespace::from_id(CRONCAT_ID)?)
        .build()?;

    // Publish croncat
    cron_cat_publisher.publish_app::<Croncat<MockBech32>>()?;

    // Publish dca app to the mock
    abstract_publisher.publish_app::<AppInterface<MockBech32>>()?;

    // Create a new account and install the app onto
    let account = abstract_client
        .account_builder()
        // Note: Dex adapter and croncat app is a dependency of the DCA
        .install_app_with_dependencies::<AppInterface<MockBech32>>(
            &AppInstantiateMsg {
                count: 0,
                native_asset: AssetEntry::new("denom"),
                task_creation_amount: Uint128::new(5_000_000),
                refill_threshold: Uint128::new(1_00_000),
            },
            Empty {},
        )?
        .build()?;
    let arch_app = account.application::<AppInterface<MockBech32>>()?;

    mock.set_balance(
        &account.proxy()?,
        vec![coin(50_000_000, DENOM), coin(10_000, EUR)],
    )?;

    let cron_cat_app = account.application::<Croncat<MockBech32>>()?;
    let deployed_apps = DeployedApps {
        arch_app,
        cron_cat_app,
        wyndex,
    };
    Ok((
        mock,
        account,
        abstract_client,
        deployed_apps,
        cron_cat_addrs,
    ))
}

fn assert_querrier_err_eq<E: std::fmt::Display>(left: CwOrchError, right: E) {
    let querier_contract_err = || AbstractSdkError::ApiQuery {
        api: "Adapters".to_owned(),
        module_id: APP_ID.to_owned(),
        error: Box::new(StdError::generic_err(format!("Querier contract error: {right}")).into()),
    };
    assert_eq!(left.root().to_string(), querier_contract_err().to_string())
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_mock, _account, _abstr, apps, _manager_addr) = setup()?;

    let config: ConfigResponse = apps.arch_app.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            native_asset: AssetEntry::from("abstr"),
            task_creation_amount: Uint128::new(5_000_000),
            refill_threshold: Uint128::new(1_000_000),
        }
    );

    let module_data = apps.arch_app.module_data()?;
    assert_eq!(
        module_data,
        ModuleDataResponse {
            module_id: APP_ID.to_owned(),
            version: APP_VERSION.to_owned(),
            dependencies: vec![
                DependencyResponse {
                    id: CRONCAT_ID.to_owned(),
                    version_req: vec![format!("^{}", CRONCAT_MODULE_VERSION)]
                },
            ],
            metadata: None
        }
    );
    Ok(())
}

// #[test]
// fn successful_install() -> anyhow::Result<()> {
//     let (_, app) = setup(0)?;

//     let config = app.config()?;
//     assert_eq!(config, ConfigResponse {});
//     Ok(())
// }

// #[test]
// fn successful_increment() -> anyhow::Result<()> {
//     let (_, app) = setup(0)?;

//     app.increment()?;
//     let count: CountResponse = app.count()?;
//     assert_eq!(count.count, 1);
//     Ok(())
// }

// #[test]
// fn successful_reset() -> anyhow::Result<()> {
//     let (_, app) = setup(0)?;

//     app.reset(42)?;
//     let count: CountResponse = app.count()?;
//     assert_eq!(count.count, 42);
//     Ok(())
// }

// #[test]
// fn failed_reset() -> anyhow::Result<()> {
//     let (_, app) = setup(0)?;

//     let err: AppError = app
//         .call_as(&Addr::unchecked("NotAdmin"))
//         .reset(9)
//         .unwrap_err()
//         .downcast()
//         .unwrap();
//     assert_eq!(err, AppError::Admin(AdminError::NotAdmin {}));
//     Ok(())
// }
