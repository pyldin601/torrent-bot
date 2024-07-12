use serde::de;

pub(crate) fn deserialize_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;

    s.parse::<i64>()
        .map_err(|_| de::Error::custom(format!("Unable to parse as number: {}", s)))
}
