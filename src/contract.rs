use crate::msg::AppMigrateMsg;
use crate::{
    error::AppError,
    handlers,
    msg::{AppExecuteMsg, AppInstantiateMsg, AppQueryMsg},
    replies::{self, INSTANTIATE_REPLY_ID},
};
use abstract_app::abstract_core::objects::dependency::StaticDependency;
use abstract_app::AppContract;
use cosmwasm_std::Response;

#[cfg(feature = "interface")]
use abstract_app::abstract_core::{manager::ModuleInstallConfig, objects::module::ModuleInfo};

#[cfg(feature = "interface")]
use croncat_app::contract::interface::Croncat;

use croncat_app::contract::{CRONCAT_ID, CRONCAT_MODULE_VERSION};

/// The version of your app
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const APP_ID: &str = "enos-osmo-5:archi-auto";

/// The type of the result returned by your app's entry points.
pub type AppResult<T = Response> = Result<T, AppError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type App = AppContract<AppError, AppInstantiateMsg, AppExecuteMsg, AppQueryMsg, AppMigrateMsg>;

const APP: App = App::new(APP_ID, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_migrate(handlers::migrate_handler)
    .with_replies(&[(INSTANTIATE_REPLY_ID, replies::instantiate_reply)])
    .with_dependencies(&[
        StaticDependency::new(CRONCAT_ID, &[CRONCAT_MODULE_VERSION]),
    ]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(APP, App);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(APP, App, AppInterface);

#[cfg(feature = "interface")]
impl<Chain: cw_orch::environment::CwEnv> abstract_app::abstract_interface::DependencyCreation
    for crate::AppInterface<Chain>
{
    type DependenciesConfig = cosmwasm_std::Empty;

    fn dependency_install_configs(
        _configuration: Self::DependenciesConfig,
    ) -> Result<Vec<ModuleInstallConfig>, abstract_app::abstract_interface::AbstractInterfaceError> {
        let croncat_dependency_install_configs: Vec<ModuleInstallConfig> =
            <Croncat<Chain> as abstract_app::abstract_interface::DependencyCreation>::dependency_install_configs(
                cosmwasm_std::Empty {},
            )?;
        let croncat_install_config =
            <Croncat<Chain> as abstract_app::abstract_interface::InstallConfig>::install_config(
                &croncat_app::msg::AppInstantiateMsg {},
            )?;

        println!("hello {:?} {:?}", croncat_dependency_install_configs, croncat_install_config);

        Ok([
            croncat_dependency_install_configs,
            vec![croncat_install_config],
        ]
        .concat())
    }
}