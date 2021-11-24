use crate::state::{LootopiaNFTContract}; 
use cosmwasm_std::entry_point;
use cosmwasm_std::{from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};

use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ReceiveMsg};
pub use cw721_base::{MinterResponse};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg, ReceiveMsg, MintMsg};
use crate::errors::ContractError;
use crate::state::{Config, CONFIG, Loot, LOOT, Metadata, Trait};
use terraswap::asset::{Asset, AssetInfo};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    LOOT.save(
        deps.storage,
        &Loot {
            names: msg.names.clone(),
            origins: msg.origins.clone(),
            professions: msg.professions.clone(),
            obsessions: msg.obsessions.clone(),
            talents: msg.talents.clone(),
            skills: msg.skills.clone(),
            alignments: msg.alignments.clone(),
            num_items: msg.num_items,
            curr_num_items: 0,
        },
    )?;
    let config = Config {
        payment_token: msg.payment_token.clone(),
        price: msg.price,
        treasury: msg.treasury.clone(),
        limit_per_address: msg.limit_per_address,
        nft_limit: msg.nft_limit,
    };

    CONFIG.save(deps.storage, &config)?;

    LootopiaNFTContract::default().instantiate(deps, env, info, msg.into())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive(deps, env, info, msg),
        ExecuteMsg::WithdrawSales { amount } => withdraw_sales(deps, amount),
        // CW721 methods
        _ => LootopiaNFTContract::default()
            .execute(deps, env, info, msg.into())
            .map_err(|err| err.into()),
    }
}

fn get_hash<T: Hash>(seed: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    hasher.finish()
}

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg)? {
        ReceiveMsg::Buy {
            mint_msg,
        } => execute_buy(
            deps, 
            env,
            info.clone(),
            info.sender.to_string().clone(), // the token sent
            cw20_msg.amount, // the amount sent
            cw20_msg.sender, // address of the buyer
            mint_msg
        ),
        _ => Err(ContractError::Unauthorized {}),
    }
}

pub fn withdraw_sales(deps: DepsMut, amount: Uint128) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    let to_withdraw = Asset {
        info: AssetInfo::Token {
                contract_addr: cfg.payment_token.clone(),
        },
        amount: amount.clone(),
    };
    let treasury = deps.api.addr_validate(&cfg.treasury)?;
    Ok(Response::new().add_message(to_withdraw.into_msg(&deps.querier, treasury)?))
}

pub fn execute_buy(
    deps: DepsMut, 
    env: Env,
    mut info: MessageInfo,
    token_sent: String, 
    amount_sent: Uint128, 
    buyer: String,
    mut mint_msg: MintMsg,
) -> Result<Response, ContractError> {


    let cw721_contract = LootopiaNFTContract::default();
    // verify token_sent == payment_token
    let config = CONFIG.load(deps.storage)?;
    if config.payment_token != token_sent {
        return Err(ContractError::Unauthorized {});
    } // verify amount_sent >= price
    if amount_sent < config.price {
        return Err(ContractError::Unauthorized {});
    }

    let mut loot = LOOT.load(deps.storage)?;

    // make sure total minted <= total num items
    if loot.curr_num_items > loot.num_items {
        return Err(ContractError::MaxTokensMinted {});
    }
    // pick random name
        let rng_seed = &[
            info.sender.to_string().as_bytes(),
            env.block.height.to_string().as_bytes(),
        ]
        .concat();
        let hash = get_hash(&rng_seed);
        // pick random
        let selected_name = loot
            .names
            .get((hash % loot.names.len() as u64) as usize)
            .ok_or(ContractError::Failed {})?;
        let selected_origin = loot
            .origins
            .get((hash % loot.origins.len() as u64) as usize)
            .ok_or(ContractError::Failed {})?;
        let selected_profession = loot
            .professions
            .get((hash % loot.professions.len() as u64) as usize)
            .ok_or(ContractError::Failed {})?;
        let selected_obsession = loot
            .obsessions
            .get((hash % loot.obsessions.len() as u64) as usize)
            .ok_or(ContractError::Failed {})?;
        let selected_talent = loot
            .talents
            .get((hash % loot.talents.len() as u64) as usize)
            .ok_or(ContractError::Failed {})?;
        let selected_skill = loot
            .skills
            .get((hash % loot.skills.len() as u64) as usize)
            .ok_or(ContractError::Failed {})?;
        let selected_alignment = loot
            .alignments
            .get((hash % loot.alignments.len() as u64) as usize)
            .ok_or(ContractError::Failed {})?;
        // increase number of items
        loot.curr_num_items += 1;
        LOOT.save(deps.storage, &loot)?;
 
        info.sender = deps.api.addr_validate(&config.payment_token)?;
    
        // We need to construct the NFT mint message here.
 
      mint_msg.owner = buyer;
      mint_msg.token_id = loot.curr_num_items.to_string();
      mint_msg.token_uri = Some(loot.curr_num_items.to_string());
      let extension = Metadata { 
        name: Some(selected_name.clone()),
        image: None,
        animation_url: None,
        description: Some("Character Sheet Loot for the Lootopia Metaverse".to_string()),
        background_color: None,
        youtube_url: None,
        image_data: None,
        external_url: None,
        attributes: Some(vec![
            Trait {
                display_type: None,
                trait_type: "Origin".to_string(),
                value: selected_origin.clone(),
            },
            Trait {
                display_type: None,
                trait_type: "Profession".to_string(),
                value: selected_profession.clone(),
            },
            Trait {
                display_type: None,
                trait_type: "Obsession".to_string(),
                value: selected_obsession.clone(),
            },
            Trait {
                display_type: None,
                trait_type: "Talent".to_string(),
                value: selected_talent.clone(),
            },
            Trait {
                display_type: None,
                trait_type: "Skill".to_string(),
                value: selected_skill.clone(),
            },
            Trait {
                display_type: None,
                trait_type: "Alignment".to_string(),
                value: selected_alignment.clone(),
            },
        ]),
      };
    mint_msg.extension = Some(extension);
    // if both ok, mint buyer a token

    let cfg = CONFIG.load(deps.storage)?;

    let to_withdraw = Asset {
        info: AssetInfo::Token {
                contract_addr: cfg.payment_token.clone(),
        },
        amount: amount_sent.clone(),
    };
    let treasury = deps.api.addr_validate(&cfg.treasury)?;
    Response::new().add_message(to_withdraw.into_msg(&deps.querier, treasury)?);
    let response = cw721_contract.mint(deps, env, info, mint_msg)?;
    Ok(response)
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // CW721 methods
        _ => LootopiaNFTContract::default().query(deps, env, msg.into()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: MigrateMsg<Config>,
) -> Result<Response, ContractError> {
    match msg {
        MigrateMsg { version, config } => try_migrate(deps, version, config),
    }
}

fn try_migrate(
    deps: DepsMut,
    version: String,
    config: Option<Config>,
) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;
    set_contract_version(deps.storage, contract_version.contract, version)?;

    if config.is_some() {
        CONFIG.save(deps.storage, &config.unwrap())?
    }

    Ok(Response::new()
        .add_attribute("method", "try_migrate")
        .add_attribute("version", contract_version.version))
}