use abstract_app::abstract_core::AbstractError;
use abstract_app::abstract_sdk::AbstractSdkError;
use abstract_app::AppError as AbstractAppError;
use cosmwasm_std::{Addr, StdError};
use cw_asset::AssetError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AppError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    DappError(#[from] AbstractAppError),

    #[error("Native asset should be native")]
    NotNativeAsset {},

    #[error("Convert can be called only by the croncat manager {sender} vs {manager}")]
    NotManagerConvert { sender: Addr, manager: Addr},
}
