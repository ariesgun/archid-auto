//! Publishes the module to the Abstract platform by uploading it and registering it on the app store.
//!
//! Info: The mnemonic used to register the module must be the same as the owner of the account that claimed the namespace.
//!
//! ## Example
//!
//! ```bash
//! $ just publish uni-6 osmo-test-5
//! ```
use abstract_app::{objects::namespace::Namespace};
use abstract_client::{AbstractClient, Publisher};
use app::{contract::{App, APP_ID}, msg::{AppInstantiateMsg, AppMigrateMsg}, AppExecuteMsgFns, AppInterface, AppQueryMsgFns};
use clap::Parser;
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
        println!("Hey {}", app_namespace);

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

        println!("Account builder");
        let account = abstract_client.account_builder()
            .install_app::<AppInterface<Daemon>>(&AppInstantiateMsg { count: 123 })?
            .build()?;

        // let account = publisher.account();
        // let app = publisher.account()
        //     .install_app::<AppInterface<Daemon>>(&AppInstantiateMsg { count: 123 }, &[])?;
        let app = account.application::<AppInterface<Daemon>>()?;

        println!("Account {:?}", account.module_infos());
        println!("Account {:?}", account);

        println!("Incrementing...");
        // app.increment();
        println!("Count {}", app.count()?.count);

        let result = app.name_resolution("archid.arch".to_string());
        println!("Result {:?}", result);
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
