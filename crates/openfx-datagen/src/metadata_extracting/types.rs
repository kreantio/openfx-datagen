use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Metadata {
    pub propdef_map: BTreeMap<String, PropdefMetadataEntry>,
    // pub propset_map: BTreeMap<String, PropsetMetadataEntry>,
    // pub propsetdef_map: BTreeMap<String, PropsetdefMetadataEntry>,
    // pub actiondef_map: BTreeMap<String, ActiondefMetadataEntry>,
    pub propset_map: BTreeMap<String, Todo>,
    pub propsetdef_map: BTreeMap<String, Todo>,
    pub actiondef_map: BTreeMap<String, Todo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct PropdefMetadataEntry {
    pub r#type: PropdefType,
    pub dimension: PropdefDimension,
    /// NOTE: There is also an `added: …`. I guess the two are the same?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introduced: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<String>,
    /// e.g., `hostOptional: true`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub host_optional: bool,
    /// TODO: figure out what is the difference between `hostOptional` and
    /// `optional`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub optional: bool,
    /// NOTE: no idea why there is this field. Only
    /// `kOfxImageEffectPropProjectPixelAspectRatio` uses it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cname: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "$type")]
pub enum PropdefType {
    Simple {
        one_of: BTreeSet<PropdefTypeSimple>,
    },
    StringEnum {
        /// ## Note
        ///
        /// - No idea why `"false"` and `"true"` are quoted, but `needed` is
        ///   not.
        one_of: Vec<StringEnumVariant>,
    },
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
pub enum PropdefTypeSimple {
    Int,
    Double,
    Bool,
    String,
    Pointer,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "$type")]
pub enum StringEnumVariant {
    Defined { cname: String },
    Literal { value: String },
}

impl PropdefType {
    pub fn new_one(one: PropdefTypeSimple) -> Self {
        Self::Simple {
            one_of: BTreeSet::from([one]),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "$type")]
pub enum PropdefDimension {
    Fixed { size: usize },
    Dynamic,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct PropsetMetadataEntry {
    pub write: WriteSide,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub props: BTreeMap<String, PropsetPropValue>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub props_refs: BTreeSet<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum WriteSide {
    Host,
    Plugin,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct PropsetPropValue {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub host_optional: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct PropsetdefMetadataEntry {
    pub propes: BTreeMap<String, PropsetdefPropValue>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct PropsetdefPropValue {
    pub write: WriteSide,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ActiondefMetadataEntry {
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub in_args: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub out_args: BTreeSet<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum Todo {
    TODO { content: String },
}
