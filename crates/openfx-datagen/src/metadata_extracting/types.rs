use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::{metadata_extracting::PropValueError, utils::write_schema_json_pretty};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Metadata {
    pub propdef_map: BTreeMap<String, PropdefMetadataEntry>,
    pub propset_map: BTreeMap<String, PropsetMetadataEntry>,
    pub propsetdef_map: BTreeMap<String, PropsetdefMetadataEntry>,
    // pub actiondef_map: BTreeMap<String, ActiondefMetadataEntry>,
    pub actiondef_map: BTreeMap<String, Todo>,
}

impl Metadata {
    pub fn write_schema_json_pretty<W: std::io::Write>(
        writer: W,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        write_schema_json_pretty::<_, Self>(writer)
    }
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
    pub props: BTreeMap<String, PropValue>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub props_refs: BTreeSet<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum WriteSide {
    Host,
    Plugin,
}

impl WriteSide {
    pub fn try_from(value: &str) -> Option<Self> {
        match value {
            "host" => Some(Self::Host),
            "plugin" => Some(Self::Plugin),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct PropValue {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub host_optional: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write: Option<WriteSide>,
}

impl PropValue {
    pub fn try_from_options(opts: HashMap<&str, &str>) -> Result<PropValue, Vec<PropValueError>> {
        let mut value = Self::default();
        let mut errors = vec![];

        for (opt_name, opt_value) in opts {
            match opt_name {
                "host_optional" => {
                    if value.host_optional {
                        errors.push(PropValueError::PropValueDuplicateOption {
                            option_name: "host_optional".to_owned(),
                        });
                        continue;
                    }
                    value.host_optional = true;
                }
                "write" => {
                    if value.write.is_some() {
                        errors.push(PropValueError::PropValueDuplicateOption {
                            option_name: "write".to_owned(),
                        });
                        continue;
                    }
                    value.write = match WriteSide::try_from(opt_value) {
                        Some(write_side) => Some(write_side),
                        None => {
                            errors.push(PropValueError::PropValueUnexpectedOptionValue {
                                option_name: "write".to_owned(),
                                option_value: opt_value.to_owned(),
                            });
                            continue;
                        }
                    }
                }
                _ => {
                    errors.push(PropValueError::PropValueUnexpectedOption {
                        option_name: opt_name.to_owned(),
                        option_value: opt_value.to_owned(),
                    });
                    continue;
                }
            }
        }

        if errors.is_empty() {
            Ok(value)
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct PropsetdefMetadataEntry {
    pub props: BTreeMap<String, PropValue>,
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
