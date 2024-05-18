use abstract_app::abstract_sdk::AbstractSdkResult;
use abstract_app::traits::{AbstractResponse, Execution};
use cosmwasm_std::{to_json_binary, wasm_execute, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Uint128, WasmMsg};
use cw_asset::{Asset, AssetList};

use crate::contract::{App, AppResult};

use crate::error::AppError;
use crate::msg::{AppExecuteMsg, ExecuteMsg};
use crate::state::{Config, TaskEntry, TaskId, CONFIG, COUNT, DEFAULT_ID_MAP, NEXT_ID, TASK_LIST};

use croncat_app::{
    croncat_integration_utils::{CronCatAction, CronCatTaskRequest, CronCatInterval},
    CronCat, CronCatInterface,
};


pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: App,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::Increment {} => increment(deps, app),
        AppExecuteMsg::Reset { count } => reset(deps, info, count, app),
        AppExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
        AppExecuteMsg::UpdateDefaultID {} =>update_default_id(deps, info, app),
        AppExecuteMsg::RegisterDomain { desired_name} => register_domain(deps, info, app, desired_name),
        AppExecuteMsg::RegisterDomain2 { desired_name} => register_domain2(deps, info, app, desired_name),
        AppExecuteMsg::RenewDomain { task_id} => renew_domain(deps, env, info, app, task_id),
        AppExecuteMsg::CreateAutoRenewalTask { frequency, domain_name} => create_auto_renewal_task(deps, env, info, app, frequency, domain_name),
    }
}

/// Helper to for task creation message
fn create_convert_task_internal(
    env: Env,
    dca: TaskEntry,
    task_id: TaskId,
    cron_cat: CronCat<App>,
    config: Config,
) -> AbstractSdkResult<CosmosMsg> {
    let interval = CronCatInterval::Cron(dca.frequency);

    let task = CronCatTaskRequest {
        interval,
        boundary: None,
        stop_on_fail: true,
        actions: vec![CronCatAction {
            msg: wasm_execute(
                env.contract.address,
                &ExecuteMsg::from(AppExecuteMsg::RenewDomain { task_id }),
                vec![],
            )?
            .into(),
            gas_limit: Some(300_000),
        }],
        queries: None,
        transforms: None,
        cw20: None,
    };
    let assets = AssetList::from(vec![Asset::native(
        config.native_denom,
        config.task_creation_amount,
    )])
    .into();

    cron_cat.create_task(task, task_id, assets)
}

fn increment(deps: DepsMut, app: App) -> AppResult {
    COUNT.update(deps.storage, |count| AppResult::Ok(count + 1))?;

    Ok(app.response("increment"))
}

fn reset(deps: DepsMut, info: MessageInfo, count: i32, app: App) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;
    COUNT.save(deps.storage, &count)?;

    Ok(app.response("reset"))
}

/// Update the configuration of the app
fn update_config(deps: DepsMut, msg_info: MessageInfo, app: App) -> AppResult {
    // Only the admin should be able to call this
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.response("update_config"))
}

fn update_default_id(deps: DepsMut, msg_info: MessageInfo, app: App) -> AppResult {
    let value = "hello.arch".to_string();
    let _ = DEFAULT_ID_MAP.save(deps.storage, msg_info.sender.to_owned(), &value);

    // TODO: Only the owner can do this.

    Ok(app
        .response("update_default_id")
        .add_attribute("default_id", value)
    )
}

fn register_domain(deps: DepsMut, msg_info: MessageInfo, app: App, desired_name: String) -> AppResult {

    let registry_contract = "archway1lr8rstt40s697hqpedv2nvt27f4cuccqwvly9gnvuszxmcevrlns60xw4r";

    let cost_per_year: u128 = 1000000000000000000;
    let denom = "aarch"; // (Or "aconst" for testnet)

    let register_msg = archid_registry::msg::ExecuteMsg::Register {
        name: desired_name,
    };

    let register_resp =  WasmMsg::Execute {
        contract_addr: registry_contract.into(),
        msg: to_json_binary(&register_msg)?,
        funds: vec![Coin {
            denom: "aconst".into(),
            amount: Uint128::from(cost_per_year),
        }]
    };

    let messages = vec![register_resp];

    Ok(
        app.response("register_domain").add_messages(messages).add_attribute("sender", msg_info.sender)
    )
}

fn register_domain2(deps: DepsMut, msg_info: MessageInfo, app: App, desired_name: String) -> AppResult {

    let registry_contract = "archway1lr8rstt40s697hqpedv2nvt27f4cuccqwvly9gnvuszxmcevrlns60xw4r";

    let cost_per_year: u128 = 1000000000000000000;
    let denom = "aarch"; // (Or "aconst" for testnet)

    let register_msg = archid_registry::msg::ExecuteMsg::Register {
        name: desired_name,
    };

    let register_resp =  WasmMsg::Execute {
        contract_addr: registry_contract.into(),
        msg: to_json_binary(&register_msg)?,
        funds: vec![Coin {
            denom: "aconst".into(),
            amount: Uint128::from(cost_per_year),
        }]
    };

    let executor = app.executor(deps.as_ref());
    let account_message = executor.execute(vec![register_resp.into()]).unwrap();

    Ok(
        app.response("register_domain")
            .add_message(account_message)
            .add_attribute("sender", msg_info.sender)
    )
}

// To be called by cron-cat only
fn renew_domain(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    app: App,
    task_id: TaskId) -> AppResult
{
    let registry_contract = "archway1lr8rstt40s697hqpedv2nvt27f4cuccqwvly9gnvuszxmcevrlns60xw4r";

    let cron_cat = app.cron_cat(deps.as_ref());

    let manager_addr = cron_cat.query_manager_addr(env.contract.address.clone(), task_id)?;
    if manager_addr != msg_info.sender {
        return Err(AppError::NotManagerConvert { sender: msg_info.sender, manager: manager_addr });
    }

    let cost_per_year: u128 = 1000000000000000000;
    let denom = "aarch"; // (Or "aconst" for testnet)

    let task_entry = TASK_LIST.load(deps.storage, task_id)?;

    let renew_msg = archid_registry::msg::ExecuteMsg::RenewRegistration {
        name: task_entry.domain_name,
    };

    let renew_resp =  WasmMsg::Execute {
        contract_addr: registry_contract.into(),
        msg: to_json_binary(&renew_msg)?,
        funds: vec![Coin {
            denom: "aconst".into(),
            amount: Uint128::from(cost_per_year),
        }]
    };

    let executor = app.executor(deps.as_ref());
    let account_message = executor.execute(vec![renew_resp.into()]).unwrap();

    Ok(
        app.response("renew_domain")
            .add_message(account_message)
            .add_attribute("sender", msg_info.sender)
    )

}

// Auto-extend domain create
fn create_auto_renewal_task(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    app: App,
    frequency: String,
    domain_name: String) -> AppResult
{

    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let config = CONFIG.load(deps.storage)?;
    let task_id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id.next_id()))?;
    let task_entry = TaskEntry {
        frequency,
        domain_name
    };
    let cron_cat = app.cron_cat(deps.as_ref());

    let task_msg = create_convert_task_internal(env, task_entry, task_id, cron_cat, config)?;

    Ok(
        app.response("create_auto_renewal_task")
            .add_message(task_msg)
            .add_attribute("sender", msg_info.sender)
    )
}

// Cancel auto-extend + refund

// Update auto-extend


