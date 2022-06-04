use std::str::FromStr;

use serde::de::Error;
use serde::{Deserialize, Deserializer};
use solana_sdk::pubkey::Pubkey;

pub mod serum;
pub mod spl_token;

pub fn deserialize_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    Pubkey::from_str(&s).map_err(|_| D::Error::custom(format!("invalid base58 address: {}", s)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct PubkeyContainer {
        #[serde(deserialize_with = "deserialize_pubkey")]
        pubkey: Pubkey,
    }

    #[test]
    fn test_deserialize_pubkey() {
        let input = r#"{ "pubkey": "So11111111111111111111111111111111111111112" }"#;
        let PubkeyContainer { pubkey } = serde_json::from_str(input).unwrap();
        assert_eq!(
            pubkey,
            Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap()
        );

        let malformed_input = r#"{ "pubkey": "Solana1111111111111111111111111111111111112" }"#;
        assert!(serde_json::from_str::<PubkeyContainer>(malformed_input).is_err());
    }
}
