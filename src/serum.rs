use std::convert::identity;

use bytemuck::{try_from_bytes, PodCastError};
use safe_transmute::transmute_to_bytes;
use serum_dex::state::{OpenOrders, ACCOUNT_HEAD_PADDING, ACCOUNT_TAIL_PADDING};
use solana_account_decoder::UiAccountEncoding;
#[cfg(not(feature = "program"))]
use solana_client::{
    client_error::Result as ClientResult,
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_program::{
    pubkey,
    pubkey::{Pubkey, PUBKEY_BYTES},
};

const PROGRAM_ID: Pubkey = pubkey!("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin");
const HEAD_PADDING: usize = ACCOUNT_HEAD_PADDING.len();
const TAIL_PADDING: usize = ACCOUNT_TAIL_PADDING.len();

pub struct OpenOrdersAccount {
    pub market: Pubkey,
    pub open_orders: Pubkey,
}

/// Fetch all open orders accounts owned by `owner`.
///
/// Note that `owner` might own multiple open orders accounts per market.
#[cfg(not(feature = "program"))]
pub fn fetch_open_orders_accounts(
    rpc_client: &RpcClient,
    owner: &Pubkey,
) -> ClientResult<impl Iterator<Item = OpenOrdersAccount>> {
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![
            RpcFilterType::Memcmp(Memcmp {
                offset: HEAD_PADDING + memoffset::offset_of!(OpenOrders, owner),
                bytes: MemcmpEncodedBytes::Base58(bs58::encode(owner).into_string()),
                encoding: None,
            }),
            RpcFilterType::DataSize(
                (HEAD_PADDING + memoffset::span_of!(OpenOrders, ..).len() + TAIL_PADDING) as u64,
            ),
        ]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: None,
            min_context_slot: None,
        },
        with_context: None,
    };
    let accounts = rpc_client.get_program_accounts_with_config(&PROGRAM_ID, config)?;
    Ok(accounts.into_iter().map(|(pubkey, account)| {
        let data = strip_header(&account.data[..]);
        let mut bytes = [0u8; PUBKEY_BYTES];

        let offset = memoffset::offset_of!(OpenOrders, market);
        bytes.copy_from_slice(&data[offset..offset + PUBKEY_BYTES]);

        let market = Pubkey::new(&bytes);
        OpenOrdersAccount {
            market,
            open_orders: pubkey,
        }
    }))
}

/// Pubkeys are serialized as `[u64; 4]` in Serum's accounts. This function
/// transmute them into `Pubkey`.
pub fn transmute_pubkey(bytes: [u64; 4]) -> Pubkey {
    Pubkey::new(transmute_to_bytes(&identity(bytes)))
}

/// Decode the amount of tokens held in an open orders account
/// Returns (native_coin_free, native_coin_total, native_pc_free, native_pc_total)
pub fn decode_open_orders_reserve(data: &[u8]) -> Result<(u64, u64, u64, u64), PodCastError> {
    // strip padding
    let data = strip_header(data);
    let &OpenOrders {
        native_coin_free,
        native_coin_total,
        native_pc_free,
        native_pc_total,
        ..
    } = try_from_bytes::<OpenOrders>(data)?;

    Ok((
        native_coin_free,
        native_coin_total,
        native_pc_free,
        native_pc_total,
    ))
}

fn strip_header<'a>(data: &'a [u8]) -> &'a [u8] {
    &data[ACCOUNT_HEAD_PADDING.len()..data.len() - ACCOUNT_TAIL_PADDING.len()]
}
