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
    /// Treat items in our bindings like
    ///
    /// ```rust,ignore
    /// pub const FOO: bool = true;
    /// ```
    ///
    /// as items like
    ///
    /// ```rust,ignore
    /// pub const FOO: u32 = 1;
    /// ```
    pub our_const_bools_equal_to_u32_in_reference: bool,
    /// Treat items in our bindings like
    ///
    /// ```rust,ignore
    /// pub const FOO: Bar = BAZ;
    /// ```
    ///
    /// as items like
    ///
    /// ```rust,ignore
    /// pub const FOO: Bar = 42; // 42 is the resolved literal value of BAZ.
    /// ```
    pub try_resolve_our_const_value_idents: bool,
    /// For items included here, they are accepted if the following compiles:
    /// ```rust,ignore
    /// const _: () = assert!($reference_value == $our_value);
    /// ```
    pub accept_if_const_assert_pass: HashSet<String>,
    /// For items included here, they are accepted if the following compiles:
    /// ```rust,ignore
    /// const _: () = assert!(
    ///     $reference_value as i128 as $reference_type == $reference_value
    ///         && $our_value as i128 as $our_type == $our_value
    ///         && $reference_value as i128 == $our_value as i128
    /// );
    /// ```
    pub accept_if_const_assert_as_i128_pass: HashSet<String>,
}

// #[derive(serde::Deserialize)]
// pub struct ConfigRulesAcceptIfExact {
//     pub name: String,
//     pub ours: String,
//     pub reference: String,
// }
