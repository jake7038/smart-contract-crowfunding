use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Uint128,
};
use cw_utils::must_pay;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, Status, StatusResponse};
use crate::state::{Config, CONFIG, CONTRIBUTIONS, TOTAL, WITHDRAWN};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: info.sender.clone(),
        goal: msg.goal,
        denom: msg.denom,
        deadline: env.block.time.plus_seconds(msg.duration_seconds),
    };
    CONFIG.save(deps.storage, &config)?;
    TOTAL.save(deps.storage, &Uint128::zero())?;
    WITHDRAWN.save(deps.storage, &false)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Contribute {} => contribute(deps, env, info),
        ExecuteMsg::Withdraw {} => withdraw(deps, env, info),
        ExecuteMsg::Refund {} => refund(deps, env, info),
    }
}

fn contribute(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if env.block.time >= config.deadline {
        return Err(ContractError::CampaignEnded {});
    }

    // Exige exatamente um tipo de moeda, a da campanha, com valor > 0
    let amount = must_pay(&info, &config.denom)?;

    CONTRIBUTIONS.update(deps.storage, &info.sender, |prev| -> StdResult<_> {
        Ok(prev.unwrap_or_default() + amount)
    })?;
    TOTAL.update(deps.storage, |t| -> StdResult<_> { Ok(t + amount) })?;

    Ok(Response::new()
        .add_attribute("action", "contribute")
        .add_attribute("from", info.sender)
        .add_attribute("amount", amount))
}

fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    if env.block.time < config.deadline {
        return Err(ContractError::CampaignOpen {});
    }
    if WITHDRAWN.load(deps.storage)? {
        return Err(ContractError::AlreadyWithdrawn {});
    }
    let total = TOTAL.load(deps.storage)?;
    if total < config.goal {
        return Err(ContractError::GoalNotReached {});
    }

    WITHDRAWN.save(deps.storage, &true)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: config.owner.to_string(),
            amount: vec![Coin::new(total, config.denom)],
        })
        .add_attribute("action", "withdraw"))
}

fn refund(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if env.block.time < config.deadline {
        return Err(ContractError::CampaignOpen {});
    }
    if TOTAL.load(deps.storage)? >= config.goal {
        return Err(ContractError::GoalReached {});
    }

    let amount = CONTRIBUTIONS
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();
    if amount.is_zero() {
        return Err(ContractError::NothingToRefund {});
    }

    // Zera antes de enviar: evita reembolso duplo
    CONTRIBUTIONS.remove(deps.storage, &info.sender);

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin::new(amount, config.denom)],
        })
        .add_attribute("action", "refund"))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::GetStatus {} => {
            let config = CONFIG.load(deps.storage)?;
            let total = TOTAL.load(deps.storage)?;
            let status = if env.block.time < config.deadline {
                Status::Open
            } else if total >= config.goal {
                Status::Succeeded
            } else {
                Status::Failed
            };
            to_json_binary(&StatusResponse { status, total })
        }
        QueryMsg::GetContribution { address } => {
            let addr = deps.api.addr_validate(&address)?;
            let amount = CONTRIBUTIONS.may_load(deps.storage, &addr)?.unwrap_or_default();
            to_json_binary(&amount)
        }
    }
}

//testes 

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{Addr, OwnedDeps};

    fn setup() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, Addr) {
        let mut deps = mock_dependencies();
        let owner = deps.api.addr_make("owner");
        let msg = InstantiateMsg {
            goal: Uint128::new(100),
            denom: "untrn".into(),
            duration_seconds: 1000,
        };
        instantiate(deps.as_mut(), mock_env(), message_info(&owner, &[]), msg).unwrap();
        (deps, owner)
    }

    #[test]
    fn goal_reached_owner_withdraws() {
        let (mut deps, owner) = setup();
        let alice = deps.api.addr_make("alice");

        execute(deps.as_mut(), mock_env(),
            message_info(&alice, &coins(100, "untrn")), ExecuteMsg::Contribute {}).unwrap();

        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(1001);
        let res = execute(deps.as_mut(), env, message_info(&owner, &[]), ExecuteMsg::Withdraw {}).unwrap();
        assert_eq!(res.messages.len(), 1);
    }

    #[test]
    fn goal_missed_contributor_refunds() {
        let (mut deps, _) = setup();
        let bob = deps.api.addr_make("bob");

        execute(deps.as_mut(), mock_env(),
            message_info(&bob, &coins(40, "untrn")), ExecuteMsg::Contribute {}).unwrap();

        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(1001);
        execute(deps.as_mut(), env.clone(), message_info(&bob, &[]), ExecuteMsg::Refund {}).unwrap();

        // segundo reembolso deve falhar
        let err = execute(deps.as_mut(), env, message_info(&bob, &[]), ExecuteMsg::Refund {}).unwrap_err();
        assert!(matches!(err, ContractError::NothingToRefund {}));
    }

    #[test]
    fn cannot_withdraw_before_deadline() {
        let (mut deps, owner) = setup();
        let err = execute(deps.as_mut(), mock_env(), message_info(&owner, &[]), ExecuteMsg::Withdraw {}).unwrap_err();
        assert!(matches!(err, ContractError::CampaignOpen {}));
    }
}