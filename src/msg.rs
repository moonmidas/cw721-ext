use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, Uint128};
use cw721::Expiration;
use cw20::{Cw20ReceiveMsg};
use cw721_base::{
    msg::{
        ExecuteMsg as CW721ExecuteMsg, InstantiateMsg as CW721InstantiateMsg,
        QueryMsg as CW721QueryMsg,
    },
    MintMsg as CW721MintMsg,
};
use crate::state::Extension;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,

    /// The minter is the only one who can create new NFTs.
    /// This is designed for a base NFT that is controlled by an external program
    /// or contract. You will likely replace this with custom logic in custom NFTs
    pub minter: String,

    // maximum token supply
    // pub token_supply: Option<u64>,
    pub payment_token: String,
    pub price: Uint128,
    pub treasury: String,
    // maximum number of reservations per address
    pub limit_per_address: u64,
    // maximum number of nfts this contract is allowed to mint
    // so owner cannot dilute the supply
    // lootbox info
    pub names: Vec<String>,
    pub origins: Vec<String>,
    pub professions: Vec<String>,
    pub obsessions: Vec<String>,
    pub talents: Vec<String>,
    pub skills: Vec<String>,
    pub alignments: Vec<String>,
    pub num_items: u64,
    // Enable or disable whitelist
    pub whitelist: bool,
    pub whitelist_admin: String,
    
    // General admin
    pub admin: String,
}

impl From<InstantiateMsg> for CW721InstantiateMsg {
    fn from(msg: InstantiateMsg) -> CW721InstantiateMsg {
        CW721InstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            minter: msg.minter,
        }
    }
}

pub type MintMsg = CW721MintMsg<Extension>;

// Extended CW721 ExecuteMsg, added the ability to update, burn, and finalize nft
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]

pub enum ExecuteMsg {


        /// Mint a new NFT, can only be called by the contract minter
        Mint(MintMsg),
    
        // Standard CW721 ExecuteMsg
        /// Transfer is a base message to move a token to another account without triggering actions
        TransferNft {
            recipient: String,
            token_id: String,
        },
        /// Send is a base message to transfer a token to a contract and trigger an action
        /// on the receiving contract.
        SendNft {
            contract: String,
            token_id: String,
            msg: Binary,
        },
        /// Allows operator to transfer / send the token from the owner's account.
        /// If expiration is set, then this allowance has a time/height limit
        Approve {
            spender: String,
            token_id: String,
            expires: Option<Expiration>,
        },
        /// Remove previously granted Approval
        Revoke {
            spender: String,
            token_id: String,
        },
        /// Allows operator to transfer / send any token from the owner's account.
        /// If expiration is set, then this allowance has a time/height limit
        ApproveAll {
            operator: String,
            expires: Option<Expiration>,
        },
        /// Remove previously granted ApproveAll permission
        RevokeAll {
            operator: String,
        },

        // Receive a cw20 token message
        Receive(Cw20ReceiveMsg),

        // Withdraw sales made
        WithdrawSales {
            amount: Uint128,
        },

        // create a reservation, and pay cost
        AddWhitelistAddresses {
            addresses: Vec<String>,
        },

        ToggleWhitelist {
            whitelist: bool,
        },

        SetWhitelistAdmin {
            whitelist_admin: String,
        },

        // Set the admin
        SetAdmin {
            admin: String,
        },

        // Update token
        UpdateAllMetadata {
            token_id: String,
            extension: Extension,
        },

}

impl From<ExecuteMsg> for CW721ExecuteMsg<Extension> {
    fn from(msg: ExecuteMsg) -> CW721ExecuteMsg<Extension> {
        match msg {
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => CW721ExecuteMsg::TransferNft {
                recipient,
                token_id,
            },
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => CW721ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            },
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => CW721ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            },
            ExecuteMsg::Revoke { spender, token_id } => {
                CW721ExecuteMsg::Revoke { spender, token_id }
            }
            ExecuteMsg::ApproveAll { operator, expires } => {
                CW721ExecuteMsg::ApproveAll { operator, expires }
            }
            ExecuteMsg::RevokeAll { operator } => CW721ExecuteMsg::RevokeAll { operator },
            _ => panic!("cannot covert {:?} to CW721ExecuteMsg", msg),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Buy {
        mint_msg: CW721MintMsg<Extension>,
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Standard cw721 queries
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    ApprovedForAll {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    NumTokens {},
    ContractInfo {},
    NftInfo {
        token_id: String,
    },
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    Whitelisted {
        start_after: Option<String>,
        limit: Option<u32>,
    }
}

impl From<QueryMsg> for CW721QueryMsg {
    fn from(msg: QueryMsg) -> CW721QueryMsg {
        match msg {
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => CW721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            QueryMsg::ApprovedForAll {
                owner,
                include_expired,
                start_after,
                limit,
            } => CW721QueryMsg::ApprovedForAll {
                owner,
                include_expired,
                start_after,
                limit,
            },
            QueryMsg::NumTokens {} => CW721QueryMsg::NumTokens {},
            QueryMsg::ContractInfo {} => CW721QueryMsg::ContractInfo {},
            QueryMsg::NftInfo { token_id } => CW721QueryMsg::NftInfo { token_id },
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => CW721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => CW721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllTokens { start_after, limit } => {
                CW721QueryMsg::AllTokens { start_after, limit }
            }
            _ => panic!("cannot covert {:?} to CW721QueryMsg", msg),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct AllWhitelisted {
    pub accounts: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg<T> {
    pub version: String,
    pub config: Option<T>,
}