use crate::contract::{App, AppResult};
use crate::msg::{AppQueryMsg, ConfigResponse, CountResponse, DefaultIdResponse, NameResolutionResponse};
use crate::state::{CONFIG, COUNT, DEFAULT_ID_MAP};
use archid_registry::msg::QueryMsg::ResolveRecord;
use archid_registry::msg::ResolveRecordResponse;
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, QueryRequest, StdResult, WasmQuery};


pub fn query_handler(deps: Deps, _env: Env, _app: &App, msg: AppQueryMsg) -> AppResult<Binary> {
    match msg {
        AppQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        AppQueryMsg::Count {} => to_json_binary(&query_count(deps)?),
        AppQueryMsg::NameResolution { domain_name } => to_json_binary(&query_name_resolution(deps, &domain_name)?),
        AppQueryMsg::DefaultId { address } => to_json_binary(&query_default_id(deps, address)?),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let _config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {})
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let count = COUNT.load(deps.storage)?;
    Ok(CountResponse { count })
}

fn query_name_resolution(deps: Deps, domain_name: &str) -> StdResult<NameResolutionResponse> {

    let registry_contract = "archway1lr8rstt40s697hqpedv2nvt27f4cuccqwvly9gnvuszxmcevrlns60xw4r";
    // Testnet: archway1lr8rstt40s697hqpedv2nvt27f4cuccqwvly9gnvuszxmcevrlns60xw4r
    // Mainnet: archway1275jwjpktae4y4y0cdq274a2m0jnpekhttnfuljm6n59wnpyd62qppqxq0

    // Create the query msg
    let query_msg = ResolveRecord {
        name: domain_name.to_string(),
    };

    // Do the query request
    let query_req = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: registry_contract.into(),
        msg: to_json_binary(&query_msg).unwrap(),
    });
    // Get the query result
    let query_resp: ResolveRecordResponse = deps.querier.query(&query_req)?;

    Ok(NameResolutionResponse {query_resp})
}

fn query_default_id(deps: Deps, address: Addr) -> StdResult<DefaultIdResponse> {

    // TODO: Check if the address can be found in the MAP.
    let default_id_map = DEFAULT_ID_MAP.load(deps.storage, address)?;
    
    Ok(
        DefaultIdResponse { 
            default_id: default_id_map 
        }
    )
}