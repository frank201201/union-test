use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, ContractInfoResponse, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128, WasmQuery,
};
use cw2::set_contract_version;
use cw20::{
    BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20ReceiveMsg, DownloadLogoResponse, EmbeddedLogo,
    Logo, LogoInfo, MarketingInfoResponse, MinterResponse, TokenInfoResponse,
};
use frissitheto::UpgradeMsg;

use crate::{
    allowances::{
        execute_burn_from, execute_decrease_allowance, execute_increase_allowance,
        execute_send_from, execute_transfer_from, query_allowance,
    },
    enumerable::{query_all_accounts, query_owner_allowances, query_spender_allowances},
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{MinterData, TokenInfo, BALANCES, LOGO, MARKETING_INFO, TOKEN_INFO},
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // create initial accounts
    let total_supply = create_accounts(&mut deps, &msg.initial_balances)?;

    if let Some(limit) = msg.get_cap() {
        if total_supply > limit {
            return Err(StdError::generic_err("Initial supply greater than cap").into());
        }
    }

    let mint = match msg.mint {
        Some(m) => Some(MinterData {
            minter: deps.api.addr_validate(&m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    // store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply,
        mint,
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    if let Some(marketing) = msg.marketing {
        let logo = if let Some(logo) = marketing.logo {
            LOGO.save(deps.storage, &logo)?;

            match logo {
                Logo::Url(url) => Some(LogoInfo::Url(url)),
                Logo::Embedded(_) => Some(LogoInfo::Embedded),
            }
        } else {
            None
        };

        let data = MarketingInfoResponse {
            project: marketing.project,
            description: marketing.description,
            marketing: marketing
                .marketing
                .map(|addr| deps.api.addr_validate(&addr))
                .transpose()?,
            logo,
        };
        MARKETING_INFO.save(deps.storage, &data)?;
    }

    Ok(Response::default())
}

pub fn create_accounts(
    deps: &mut DepsMut,
    accounts: &[Cw20Coin],
) -> Result<Uint128, ContractError> {
    validate_accounts(accounts)?;

    let mut total_supply = Uint128::zero();
    for row in accounts {
        let address = deps.api.addr_validate(&row.address)?;
        BALANCES.save(deps.storage, &address, &row.amount)?;
        total_supply += row.amount;
    }

    Ok(total_supply)
}

pub fn validate_accounts(accounts: &[Cw20Coin]) -> Result<(), ContractError> {
    let mut addresses = accounts.iter().map(|c| &c.address).collect::<Vec<_>>();
    addresses.sort();
    addresses.dedup();

    if addresses.len() != accounts.len() {
        Err(ContractError::DuplicateInitialBalanceAddresses {})
    } else {
        Ok(())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Cw20ExecuteMsg(Cw20ExecuteMsg::Transfer { recipient, amount }) => {
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Cw20ExecuteMsg(Cw20ExecuteMsg::Burn { amount }) => {
            execute_burn(deps, env, info, amount)
        }
        ExecuteMsg::Cw20ExecuteMsg(Cw20ExecuteMsg::Send {
            contract,
            amount,
            msg,
        }) => execute_send(deps, env, info, contract, amount, msg),
        ExecuteMsg::Cw20ExecuteMsg(Cw20ExecuteMsg::Mint { recipient, amount }) => {
            execute_mint(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Cw20ExecuteMsg(Cw20ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        }) => execute_increase_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::Cw20ExecuteMsg(Cw20ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        }) => execute_decrease_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::Cw20ExecuteMsg(Cw20ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        }) => execute_transfer_from(deps, env, info, owner, recipient, amount),
        ExecuteMsg::Cw20ExecuteMsg(Cw20ExecuteMsg::BurnFrom { owner, amount }) => {
            execute_burn_from(deps, env, info, owner, amount)
        }
        ExecuteMsg::Cw20ExecuteMsg(Cw20ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        }) => execute_send_from(deps, env, info, owner, contract, amount, msg),
        ExecuteMsg::Cw20ExecuteMsg(Cw20ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        }) => execute_update_marketing(deps, env, info, project, description, marketing),
        ExecuteMsg::Cw20ExecuteMsg(Cw20ExecuteMsg::UploadLogo(logo)) => {
            execute_upload_logo(deps, env, info, logo)
        }
        ExecuteMsg::Cw20ExecuteMsg(Cw20ExecuteMsg::UpdateMinter { new_minter }) => {
            execute_update_minter(deps, env, info, new_minter)
        }
        ExecuteMsg::UpdateMetadata {
            name,
            symbol,
            decimals,
        } => {
            let res: ContractInfoResponse = deps.querier.query(
                &WasmQuery::ContractInfo {
                    contract_addr: env.contract.address.to_string(),
                }
                .into(),
            )?;

            if info.sender != res.admin.ok_or(ContractError::Unauthorized {})? {
                return Err(ContractError::Unauthorized {});
            }
            update_metadata(deps, name, symbol, decimals)
        }
    }
}

fn update_metadata(
    deps: DepsMut,
    name: String,
    symbol: String,
    decimals: u8,
) -> Result<Response, ContractError> {
    TOKEN_INFO.update::<_, StdError>(deps.storage, |info| {
        Ok(TokenInfo {
            name,
            symbol,
            decimals,
            total_supply: info.total_supply,
            mint: info.mint,
        })
    })?;

    Ok(Response::new())
}

pub fn execute_transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&recipient)?;

    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", info.sender)
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // lower balance
    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    // reduce total_supply
    TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(amount)?;
        Ok(info)
    })?;

    let res = Response::new()
        .add_attribute("action", "burn")
        .add_attribute("from", info.sender)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut config = TOKEN_INFO
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    if config
        .mint
        .as_ref()
        .ok_or(ContractError::Unauthorized {})?
        .minter
        != info.sender
    {
        return Err(ContractError::Unauthorized {});
    }

    // update supply and enforce cap
    config.total_supply += amount;
    if let Some(limit) = config.get_cap() {
        if config.total_supply > limit {
            return Err(ContractError::CannotExceedCap {});
        }
    }
    TOKEN_INFO.save(deps.storage, &config)?;

    // add amount to recipient balance
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new()
        .add_attribute("action", "mint")
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_send(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&contract)?;

    // move the tokens to the contract
    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new()
        .add_attribute("action", "send")
        .add_attribute("from", &info.sender)
        .add_attribute("to", &contract)
        .add_attribute("amount", amount)
        .add_message(
            Cw20ReceiveMsg {
                sender: info.sender.into(),
                amount,
                msg,
            }
            .into_cosmos_msg(contract)?,
        );
    Ok(res)
}

pub fn execute_update_minter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_minter: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = TOKEN_INFO
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    let mint = config.mint.as_ref().ok_or(ContractError::Unauthorized {})?;
    if mint.minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let minter_data = new_minter
        .map(|new_minter| deps.api.addr_validate(&new_minter))
        .transpose()?
        .map(|minter| MinterData {
            minter,
            cap: mint.cap,
        });

    config.mint = minter_data;

    TOKEN_INFO.save(deps.storage, &config)?;

    Ok(Response::default()
        .add_attribute("action", "update_minter")
        .add_attribute(
            "new_minter",
            config
                .mint
                .map(|m| m.minter.into_string())
                .unwrap_or_else(|| "None".to_string()),
        ))
}

pub fn execute_update_marketing(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    project: Option<String>,
    description: Option<String>,
    marketing: Option<String>,
) -> Result<Response, ContractError> {
    let mut marketing_info = MARKETING_INFO
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    if marketing_info
        .marketing
        .as_ref()
        .ok_or(ContractError::Unauthorized {})?
        != info.sender
    {
        return Err(ContractError::Unauthorized {});
    }

    match project {
        Some(empty) if empty.trim().is_empty() => marketing_info.project = None,
        Some(project) => marketing_info.project = Some(project),
        None => (),
    }

    match description {
        Some(empty) if empty.trim().is_empty() => marketing_info.description = None,
        Some(description) => marketing_info.description = Some(description),
        None => (),
    }

    match marketing {
        Some(empty) if empty.trim().is_empty() => marketing_info.marketing = None,
        Some(marketing) => marketing_info.marketing = Some(deps.api.addr_validate(&marketing)?),
        None => (),
    }

    if marketing_info.project.is_none()
        && marketing_info.description.is_none()
        && marketing_info.marketing.is_none()
        && marketing_info.logo.is_none()
    {
        MARKETING_INFO.remove(deps.storage);
    } else {
        MARKETING_INFO.save(deps.storage, &marketing_info)?;
    }

    let res = Response::new().add_attribute("action", "update_marketing");
    Ok(res)
}

pub fn execute_upload_logo(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    logo: Logo,
) -> Result<Response, ContractError> {
    let mut marketing_info = MARKETING_INFO
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    if marketing_info
        .marketing
        .as_ref()
        .ok_or(ContractError::Unauthorized {})?
        != info.sender
    {
        return Err(ContractError::Unauthorized {});
    }

    LOGO.save(deps.storage, &logo)?;

    let logo_info = match logo {
        Logo::Url(url) => LogoInfo::Url(url),
        Logo::Embedded(_) => LogoInfo::Embedded,
    };

    marketing_info.logo = Some(logo_info);
    MARKETING_INFO.save(deps.storage, &marketing_info)?;

    let res = Response::new().add_attribute("action", "upload_logo");
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_json_binary(&query_balance(deps, address)?),
        QueryMsg::TokenInfo {} => to_json_binary(&query_token_info(deps)?),
        QueryMsg::Minter {} => to_json_binary(&query_minter(deps)?),
        QueryMsg::Allowance { owner, spender } => {
            to_json_binary(&query_allowance(deps, owner, spender)?)
        }
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => to_json_binary(&query_owner_allowances(deps, owner, start_after, limit)?),
        QueryMsg::AllSpenderAllowances {
            spender,
            start_after,
            limit,
        } => to_json_binary(&query_spender_allowances(
            deps,
            spender,
            start_after,
            limit,
        )?),
        QueryMsg::AllAccounts { start_after, limit } => {
            to_json_binary(&query_all_accounts(deps, start_after, limit)?)
        }
        QueryMsg::MarketingInfo {} => to_json_binary(&query_marketing_info(deps)?),
        QueryMsg::DownloadLogo {} => to_json_binary(&query_download_logo(deps)?),
    }
}

pub fn query_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = BALANCES
        .may_load(deps.storage, &address)?
        .unwrap_or_default();
    Ok(BalanceResponse { balance })
}

pub fn query_token_info(deps: Deps) -> StdResult<TokenInfoResponse> {
    let info = TOKEN_INFO.load(deps.storage)?;
    let res = TokenInfoResponse {
        name: info.name,
        symbol: info.symbol,
        decimals: info.decimals,
        total_supply: info.total_supply,
    };
    Ok(res)
}

pub fn query_minter(deps: Deps) -> StdResult<Option<MinterResponse>> {
    let meta = TOKEN_INFO.load(deps.storage)?;
    let minter = match meta.mint {
        Some(m) => Some(MinterResponse {
            minter: m.minter.into(),
            cap: m.cap,
        }),
        None => None,
    };
    Ok(minter)
}

pub fn query_marketing_info(deps: Deps) -> StdResult<MarketingInfoResponse> {
    Ok(MARKETING_INFO.may_load(deps.storage)?.unwrap_or_default())
}

pub fn query_download_logo(deps: Deps) -> StdResult<DownloadLogoResponse> {
    let logo = LOGO.load(deps.storage)?;
    match logo {
        Logo::Embedded(EmbeddedLogo::Svg(logo)) => Ok(DownloadLogoResponse {
            mime_type: "image/svg+xml".to_owned(),
            data: logo,
        }),
        Logo::Embedded(EmbeddedLogo::Png(logo)) => Ok(DownloadLogoResponse {
            mime_type: "image/png".to_owned(),
            data: logo,
        }),
        Logo::Url(_) => Err(StdError::not_found("logo")),
    }
}

#[cw_serde]
pub struct MigrateMsg {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: UpgradeMsg<InstantiateMsg, MigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(
        deps,
        |deps, init_msg| {
            let res = instantiate(
                deps,
                env,
                MessageInfo {
                    sender: cosmwasm_std::Addr::unchecked("sender"),
                    funds: vec![],
                },
                init_msg,
            )?;

            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{message_info, mock_env, MockApi, MockStorage},
        Addr, ContractResult, OwnedDeps, Querier, QuerierResult,
    };

    use super::*;

    struct CustomQuerier;

    impl Querier for CustomQuerier {
        fn raw_query(&self, _: &[u8]) -> cosmwasm_std::QuerierResult {
            QuerierResult::Ok(ContractResult::Ok(
                to_json_binary(&ContractInfoResponse::new(
                    1,
                    Addr::unchecked("helo"),
                    Some(Addr::unchecked("some_admin")),
                    false,
                    None,
                ))
                .unwrap(),
            ))
        }
    }

    #[test]
    fn upgrade_token_info() {
        let token_name = "Union";
        let token_symbol = "UNO";
        let decimals = 6;

        let mut deps = OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: CustomQuerier,
            custom_query_type: std::marker::PhantomData,
        };
        instantiate(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("sender"), &[]),
            InstantiateMsg {
                name: token_name.into(),
                symbol: token_symbol.into(),
                decimals,
                initial_balances: vec![],
                mint: None,
                marketing: None,
            },
        )
        .unwrap();

        let token_name = "Union New";
        let token_symbol = "UNONew";
        let decimals = 10;

        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("some_admin"), &[]),
            ExecuteMsg::UpdateMetadata {
                name: token_name.into(),
                symbol: token_symbol.into(),
                decimals,
            },
        )
        .unwrap();

        let token_info = TOKEN_INFO.load(deps.as_ref().storage).unwrap();

        assert_eq!(token_info.name, token_name);
        assert_eq!(token_info.symbol, token_symbol);
        assert_eq!(token_info.decimals, decimals);

        assert_eq!(
            execute(
                deps.as_mut(),
                mock_env(),
                message_info(&Addr::unchecked("invalid_admin"), &[]),
                ExecuteMsg::UpdateMetadata {
                    name: token_name.into(),
                    symbol: token_symbol.into(),
                    decimals,
                },
            ),
            Err(ContractError::Unauthorized {})
        );
    }
}
