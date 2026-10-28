use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub goal: Uint128,
    pub denom: String,
    pub duration_seconds: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Contribute {},
    Withdraw {},
    Refund {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    GetConfig {},
    #[returns(StatusResponse)]
    GetStatus {},
    #[returns(Uint128)]
    GetContribution { address: String },
}

#[cw_serde]
pub enum Status {
    Open,
    Succeeded,
    Failed,
}

#[cw_serde]
pub struct StatusResponse {
    pub status: Status,
    pub total: Uint128,
}