use crate::state::{LootopiaNFTContract}; 
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Order};

use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ReceiveMsg};
pub use cw721_base::{MinterResponse};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg, ReceiveMsg, MintMsg, AllWhitelisted};
use crate::errors::ContractError;
use crate::state::{Config, CONFIG, Loot, LOOT, Metadata, Trait, MINTS_BY_ADDRESS, WHITELIST_BY_ADDRESS, Extension};
use terraswap::asset::{Asset, AssetInfo};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use cw_storage_plus::Bound;

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
        whitelist: msg.whitelist,
        whitelist_admin: msg.whitelist_admin.clone(),
        admin: msg.admin.clone(),
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
        ExecuteMsg::AddWhitelistAddresses { addresses } => add_whitelist_addresses(deps, info, addresses),
        ExecuteMsg::ToggleWhitelist { whitelist } => toggle_whitelist(deps, info, whitelist),
        ExecuteMsg::SetWhitelistAdmin { whitelist_admin } => set_whitelist_admin(deps, info, whitelist_admin),
        ExecuteMsg::SetAdmin { admin } => set_admin(deps, info, admin),
        ExecuteMsg::UpdateAllMetadata {
            token_id,
            extension,
        } => execute_update_all_metadata(deps, env, info, token_id, extension),
        //ExecuteMsg::UpdateName { token_id, name } => execute_update_name(deps, info, token_id, name),
        // CW721 methods
        _ => LootopiaNFTContract::default()
            .execute(deps, env, info, msg.into())
            .map_err(|err| err.into()),
    }
}
/*
pub fn execute_update_name(
    deps: DepsMut,
    info: MessageInfo,
    token_id: String,
    name: String,
) -> Result<Response, ContractError> {

}
//
*/
pub fn execute_update_all_metadata(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    extension: Extension,
) -> Result<Response, ContractError> {
    let cw721_contract = LootopiaNFTContract::default();
    let config = CONFIG.load(deps.storage)?; 
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    cw721_contract
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                token_info.extension = extension;
                Ok(token_info)
            }
            None => return Err(ContractError::TokenNotFound {}),
        })?;

    Ok(Response::new()
        .add_attribute("action", "update")
        .add_attribute("token_id", token_id))
}

fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    config.admin = admin;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

fn set_whitelist_admin(deps: DepsMut, info: MessageInfo, whitelist_admin: String) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // check if person has admin rights
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    config.whitelist_admin = whitelist_admin;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

fn toggle_whitelist(deps: DepsMut, info: MessageInfo, whitelist: bool) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // check if person has admin rights
    let whitelist_admin = config.whitelist_admin.clone();
    if info.sender != whitelist_admin || info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    config.whitelist = whitelist;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

fn add_whitelist_addresses(deps: DepsMut, info: MessageInfo, addresses: Vec<String>) -> Result<Response, ContractError> {
    // check if the person executing this is the whitelist admin
    let config = CONFIG.load(deps.storage)?;
    let whitelist_admin = config.whitelist_admin.clone();
    if info.sender != whitelist_admin || info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // iterate addresses and add to whitelist
    for address in addresses {
        let mut by_address = WHITELIST_BY_ADDRESS
        .load(deps.storage, &address.as_bytes())
        .unwrap_or(vec![]);
    
    
        if by_address.len() < 1 as usize {
            by_address.push(1);
        }
        WHITELIST_BY_ADDRESS.save(deps.storage, address.as_bytes(), &by_address)?;
    }
    
    Ok(Response::new())
}

fn get_hash<T: Hash>(seed: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    hasher.finish()
}

fn receive(
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

fn withdraw_sales(deps: DepsMut, amount: Uint128) -> Result<Response, ContractError> {
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

fn execute_buy(
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
    let admin = config.admin.clone(); 
    if config.payment_token != token_sent && &buyer != &admin {
        return Err(ContractError::Unauthorized {});
    } // verify amount_sent >= price
    if amount_sent < config.price && &buyer != &admin {
        return Err(ContractError::Unauthorized {});
    }

    let mut loot = LOOT.load(deps.storage)?;

    // make sure total minted <= total num items
    if loot.curr_num_items > loot.num_items && &buyer != &admin {
        return Err(ContractError::MaxTokensMinted {});
    }

    // check limit per address
    let mut by_address = MINTS_BY_ADDRESS
    .load(deps.storage, &buyer.as_bytes())
    .unwrap_or(vec![]);

    if by_address.len() >= config.limit_per_address as usize && &buyer != &admin {
        return Err(ContractError::MaxMintsPerAddress {});
    }
    by_address.push(loot.curr_num_items);
    MINTS_BY_ADDRESS.save(deps.storage, buyer.as_bytes(), &by_address)?;

    // check if whitelist is enabled
    if config.whitelist == true && &buyer != &admin {
        let whitelist_by_address = WHITELIST_BY_ADDRESS
        .load(deps.storage, &buyer.as_bytes())
        .unwrap_or(vec![]);


        // if whitelist_by_address doesn't exist, then it's not whitelisted
        if whitelist_by_address.len() > 0 as usize {
            // check if whitelisted
            if whitelist_by_address[0] != 1 {
                return Err(ContractError::NotWhitelisted {});
            }
        } else {
            return Err(ContractError::NotWhitelisted {});
        }
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
 
    mint_msg.owner = buyer.clone();
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

    
      // TO FIX:

    let cfg = CONFIG.load(deps.storage)?;
    let to_withdraw = Asset {
        info: AssetInfo::Token {
                contract_addr: cfg.payment_token.clone(),
        },
        amount: amount_sent.clone(),
    };
    let treasury = deps.api.addr_validate(&cfg.treasury)?;
    
    let querier = deps.querier.clone();
    let response = cw721_contract.mint(deps, env, info, mint_msg)?;
    Ok(response.add_message(to_withdraw.into_msg(&querier, treasury)?))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Whitelisted { start_after, limit } => {
            to_binary(&try_whitelisted(deps, start_after, limit)?)
        }
        // CW721 methods
        _ => LootopiaNFTContract::default().query(deps, env, msg.into()),
    }
}

fn try_whitelisted(deps: Deps, start_after: Option<String>, limit: Option<u32>) -> StdResult<AllWhitelisted> {
    // settings for pagination
    const MAX_LIMIT: u32 = 30;
    const DEFAULT_LIMIT: u32 = 10;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let accounts: Result<Vec<_>, _> = WHITELIST_BY_ADDRESS
        .keys(deps.storage, start, None, Order::Ascending)
        .map(String::from_utf8)
        .take(limit)
        .collect();

    Ok(AllWhitelisted {
        accounts: accounts?,
    })
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