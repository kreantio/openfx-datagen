use std::collections::HashSet;

use serde::Deserialize as _;

#[derive(serde::Deserialize)]
pub struct Config {
    pub preprocessing: ConfigPreprocessing,
    pub rules: ConfigRules,
}

#[derive(serde::Deserialize)]
pub struct ConfigPreprocessing {
    pub reference_bindings: ConfigPreprocessingReferenceBindings,
}

#[derive(serde::Deserialize)]
pub struct ConfigPreprocessingReferenceBindings {
    pub rename_const: Vec<ConfigPreprocessingRename>,
}

#[derive(serde::Deserialize)]
pub struct ConfigPreprocessingRename {
    #[serde(deserialize_with = "deserialize_regex")]
    pub pattern: regress::Regex,
    pub replacement: String,
}

fn deserialize_regex<'de, D>(deserializer: D) -> Result<regress::Regex, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    let re = regress::Regex::new(&text).map_err(serde::de::Error::custom)?;

    Ok(re)
}

#[derive(serde::Deserialize)]
pub struct ConfigRules {
    pub our_const_bools_equal_to_u32_in_reference: bool,
    pub try_resolve_our_const_value_idents: bool,
    pub accept_if_const_assert_pass: HashSet<String>,
    pub accept_if_const_assert_as_i128_pass: HashSet<String>,
    pub accept_if_exact: Vec<ConfigRulesAcceptIfExact>,
}

#[derive(serde::Deserialize)]
pub struct ConfigRulesAcceptIfExact {
    pub name: String,
    pub ours: String,
    pub reference: String,
}
