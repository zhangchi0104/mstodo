use serde::{Deserialize, Deserializer};

pub fn str2u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse::<u64>()
        .map_err(serde::de::Error::custom)
}
