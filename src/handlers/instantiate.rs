use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use abstract_app::abstract_sdk::features::AbstractNameService;
use cw_asset::AssetInfoBase;

use crate::contract::{App, AppResult};
use crate::error::AppError;
use crate::msg::AppInstantiateMsg;
use crate::state::{Config, CONFIG, COUNT};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: App,
    msg: AppInstantiateMsg,
) -> AppResult {
    println!("Hey init");
    let name_service = app.name_service(deps.as_ref());
    let asset = name_service.query(&msg.native_asset)?;
    let native_denom = match asset {
        AssetInfoBase::Native(denom) => denom,
        _ => return Err(AppError::NotNativeAsset {}),
    };

    println!("Initiating...");

    let config: Config = Config {
        native_denom,
        task_creation_amount: msg.task_creation_amount,
        refill_threshold: msg.refill_threshold,
    };

    println!("Initiating...");

    CONFIG.save(deps.storage, &config)?;
    COUNT.save(deps.storage, &msg.count)?;

    println!("Initiating...");

    Ok(Response::new())
}
