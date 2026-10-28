use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Apenas o dono pode executar esta ação")]
    Unauthorized {},

    #[error("A campanha já terminou")]
    CampaignEnded {},

    #[error("A campanha ainda está aberta")]
    CampaignOpen {},

    #[error("A meta não foi atingida")]
    GoalNotReached {},

    #[error("A meta foi atingida, reembolso indisponível")]
    GoalReached {},

    #[error("Nada a reembolsar")]
    NothingToRefund {},

    #[error("Fundos já sacados")]
    AlreadyWithdrawn {},
}