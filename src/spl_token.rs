use serde::Deserialize;
use serde_repr::Deserialize_repr;
use solana_program::pubkey::Pubkey;
use thiserror::Error;

use crate::deserialize_pubkey;

const TOKEN_LIST_URL: &str = "https://raw.githubusercontent.com/solana-labs/token-list/main/src/tokens/solana.tokenlist.json";

#[derive(Debug, Deserialize)]
pub struct TokenListWrapper {
    tokens: Vec<TokenInfo>,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
pub enum ClusterSlug {
    MainnetBeta = 101,
    Testnet = 102,
    Devnet = 103,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    #[serde(rename = "chainId")]
    pub cluster_slug: ClusterSlug,
    #[serde(deserialize_with = "deserialize_pubkey")]
    pub address: Pubkey,
    pub name: String,
    pub decimals: u32,
    pub symbol: String,
}

#[derive(Debug, Error)]
pub enum TokenListError {
    #[error("network error")]
    Network(#[from] reqwest::Error),
    #[error("malformed API response")]
    Deserialize(#[from] serde_json::Error),
}

pub async fn fetch_token_list() -> Result<Vec<TokenInfo>, TokenListError> {
    let wrapper = reqwest::get(TOKEN_LIST_URL)
        .await?
        .json::<TokenListWrapper>()
        .await?;
    Ok(wrapper.tokens)
}
