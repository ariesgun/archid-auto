use abstract_app::objects::AssetEntry;
use archid_registry::msg::ResolveRecordResponse;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Uint128};

use crate::{contract::App, state::TaskId};

// This is used for type safety and re-exporting the contract endpoint structs.
abstract_app::app_msg_types!(App, AppExecuteMsg, AppQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {
    /// Initial count
    pub count: i32,
    pub native_asset: AssetEntry,
    pub task_creation_amount: Uint128,
    pub refill_threshold: Uint128,
}

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum AppExecuteMsg {
    /// Increment count by 1
    Increment {},
    /// Admin method - reset count
    Reset {
        /// Count value after reset
        count: i32,
    },
    UpdateConfig {},
    UpdateDefaultID {},
    #[cfg_attr(feature = "interface", payable)]
    RegisterDomain {
        desired_name: String,
    },
    RegisterDomain2 {
        desired_name: String,
    },
    CreateAutoRenewalTask {
        frequency: String,
        domain_name: String,
    },
    RenewDomain {
        task_id: TaskId
    }
}

/// App query messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum AppQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(CountResponse)]
    Count {},
    #[returns(NameResolutionResponse)]
    NameResolution {
        domain_name: String,
    },
    #[returns(DefaultIdResponse)]
    DefaultId {
        address: Addr,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct AppMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    /// Native gas/stake asset that used for attaching to croncat task
    pub native_asset: AssetEntry,
    /// Initial amount in native asset that sent on creating/refilling DCA
    /// to croncat to cover gas usage of agents
    pub task_creation_amount: Uint128,
    /// Threshold when task refill should happen
    /// if it's lower during [`DCAExecuteMsg::Convert`] DCA will refill croncat task
    pub refill_threshold: Uint128
}

#[cosmwasm_schema::cw_serde]
pub struct CountResponse {
    pub count: i32,
}

#[cosmwasm_schema::cw_serde]
pub struct NameResolutionResponse {
    pub query_resp: ResolveRecordResponse,
}

#[cosmwasm_schema::cw_serde]
pub struct DefaultIdResponse {
    pub default_id: String,
}