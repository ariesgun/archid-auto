//! Publishes the module to the Abstract platform by uploading it and registering it on the app store.
//!
//! Info: The mnemonic used to register the module must be the same as the owner of the account that claimed the namespace.
//!
//! ## Example
//!
//! ```bash
//! $ just publish uni-6 osmo-test-5
//! ```
use abstract_app::{abstract_core::ans_host::QueryMsgFns, objects::{namespace::Namespace, AccountId, AssetEntry}};
use abstract_client::{AbstractClient, Publisher};
use app::{contract::{App, APP_ID}, msg::{AppExecuteMsg, AppInstantiateMsg, AppMigrateMsg, ExecuteMsg}, state::TaskId, AppExecuteMsgFns, AppInterface, AppQueryMsgFns};
use clap::Parser;
use cosmwasm_std::{BankMsg, Uint128};
use cw_asset::Asset;
use cw_orch::{
    anyhow, daemon::ChainInfo, prelude::{networks::parse_network, DaemonBuilder}, tokio::runtime::Runtime
};
use cw_orch::prelude::*;
use abstract_interface::{AppDeployer, DeployStrategy};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn publish(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    // run for each requested network
    for network in networks {
        // Setup
        let rt = Runtime::new()?;    
        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network)
            .build()?;

        let app_namespace = Namespace::from_id(APP_ID)?;
        println!("Namespace {}", app_namespace);

        // Create an [`AbstractClient`]
        let abstract_client: AbstractClient<Daemon> = AbstractClient::new(chain.clone())?;

        // Get the [`Publisher`] that owns the namespace, otherwise create a new one and claim the namespace
        let publisher: Publisher<_> = abstract_client.publisher_builder(app_namespace).install_on_sub_account(false).build()?;
        println!("Account pre {:?}", publisher.account().module_infos());
        println!("Account pre {:?}", publisher.account());

        if publisher.account().owner()? != chain.sender() {
            panic!("The current sender can not publish to this namespace. Please use the wallet that owns the Account that owns the Namespace.")
        }

        println!("Publishing...");
        // // Publish the App to the Abstract Platform
        publisher.publish_app::<AppInterface<Daemon>>()?;

        let res = abstract_client.name_service().contract_list(None, Some(10), None)?;
        println!("Res {:?}", res);

        println!("Account builder");
        let account = abstract_client.account_builder()
            .install_app_with_dependencies::<AppInterface<Daemon>>(
                &AppInstantiateMsg { 
                    count: 0,
                    native_asset: AssetEntry::new("osmosis>osmo"),
                    refill_threshold: Uint128::new(0),
                    task_creation_amount: Uint128::new(0),
                },
                Empty {}
            )?
            .build()?;
        println!("Done");

        // let account = abstract_client.account_from(AccountId::local(89))?;

        // let app = account.install_app_with_dependencies::<AppInterface<Daemon>>(
        //     &AppInstantiateMsg { 
        //         count: 0,
        //         native_asset: AssetEntry::new("osmosis>osmo"),
        //         refill_threshold: Uint128::new(0),
        //         task_creation_amount: Uint128::new(30000000000000000),
        //     }, 
        //     Empty {},
        //     &[]
        // )?;

        // let app = account.install_app_with_dependencies::<AppInterface<Daemon>>(
        //     &AppInstantiateMsg { 
        //         count: 0,
        //         native_asset: AssetEntry::new("archway>const"),
        //         refill_threshold: Uint128::new(0),
        //         task_creation_amount: Uint128::new(30000000000000000),
        //     }, 
        //     Empty {}, 
        //     &[]
        // )?;
        
        // account.install_app::<AppInterface<Daemon>>(
        //     &AppInstantiateMsg { 
        //         count: 0,
        //         native_asset: AssetEntry::new("archway>const"),
        //         refill_threshold: Uint128::new(0),
        //         task_creation_amount: Uint128::new(30000000000000000),
        //     }, &[])?;

        // chain.execute(
        //     BankMsg::Send {
        //         to_address: account.manager(),,
        //         amount: vec![Coin::new(10000000000000000, "aconst")],
        //     }, 
        //     coins, 
        //     contract_address
        // );

        // let account = publisher.account();
        // let app = publisher.account()
        //     .install_app::<AppInterface<Daemon>>(&AppInstantiateMsg { count: 123 }, &[])?;
        let app = account.application::<AppInterface<Daemon>>()?;

        // chain.wallet().

        println!("Account {:?}", account.module_infos());
        println!("Account {:?}", account);

        println!("Incrementing...");
        app.increment();
        println!("Count {}", app.count()?.count);
        
        // let result = app.register_domain_2("xenos3".to_string())?;
        // println!("Result {:?}", result);

        let result = app.name_resolution("xenos3.arch".to_string())?;
        println!("Name resolution {:?}", result);

        let result = app.renew_domain(TaskId(1))?;

        // account.manager().execute_on_module(
        //     "abstract",
        //     Into::<ExecuteMsg>::into(AppExecuteMsg::Increment {}),
        // )?;

        // let result = app.register_domain("xenos1.arch".to_string())?;
        // println!("Result {:?}", result);
        
        // let result = app.register_domain_2("xenos1".to_string())?;
        // println!("Result {:?}", result);
        // let result = app.name_resolution("xenos1.arch".to_string());
        // println!("Result {:?}", result);
        // let result = app.name_resolution("xenos2.arch".to_string());
        // println!("Result {:?}", result);

        // Renew domain


    }
    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to publish on
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    network_ids: Vec<String>,
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let args = Arguments::parse();
    let networks = args
        .network_ids
        .iter()
        .map(|n| parse_network(n).unwrap())
        .collect();
    publish(networks).unwrap();
}
