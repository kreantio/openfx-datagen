use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    num::NonZero,
};

use crate::{
    metadata_extracting::{
        ActiondefError, Error, PropValueError, PropdefError, PropsetError, PropsetdefError,
        types_raw::{
            RawActiondefMetadataEntry, RawPropdefMetadataEntry, RawPropdefType,
            RawPropsetMetadataEntry, RawPropsetdefMetadataEntry,
        },
    },
    utils::write_schema_json_pretty,
};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub propdef_map: BTreeMap<String, PropdefMetadataEntry>,
    pub propset_map: BTreeMap<String, PropsetMetadataEntry>,
    pub propsetdef_map: BTreeMap<String, PropsetdefMetadataEntry>,
    pub actiondef_map: BTreeMap<String, ActiondefMetadataEntry>,
}

impl Metadata {
    pub fn write_schema_json_pretty<W: std::io::Write>(
        writer: W,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        write_schema_json_pretty::<_, Self>(writer)
    }
}

pub(crate) struct FactoryContext<'a> {
    pub stringname_to_cname: HashMap<&'a str, &'a str>,
    pub error: Error,
}

#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
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

impl PropdefMetadataEntry {
    pub(crate) fn try_from_raw(
        ctx: &mut FactoryContext<'_>,
        cname: &str,
        raw: RawPropdefMetadataEntry,
    ) -> Option<Self> {
        macro_rules! err_return {
            ($err:expr) => {
                ctx.error.propdef_errors.push((cname.to_owned(), $err));
                return None;
            };
        }

        let r#type =
            match PropdefType::try_from_raw(&raw.r#type, &raw.values, &ctx.stringname_to_cname) {
                Ok(r#type) => r#type,
                Err(err) => {
                    err_return!(err);
                }
            };
        let Some(dimension) = PropdefDimension::try_from_str(&raw.dimension) else {
            err_return!(PropdefError::PropdefUnexpectedFieldValue {
                field_name: "dimension".to_owned(),
                value: raw.dimension.clone(),
            });
        };

        if raw.introduced.is_some() && raw.added.is_some() {
            err_return!(PropdefError::PropdefDuplicateField {
                field_name: "added".to_owned(),
            });
        }

        Some(Self {
            r#type,
            dimension,
            introduced: raw.introduced.or(raw.added),
            deprecated: raw.deprecated,
            host_optional: raw.host_optional,
            optional: raw.optional,
            cname: raw.cname,
        })
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PropsetMetadataEntry {
    pub write: WriteSide,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub props: BTreeMap<String, PropValue>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub props_refs: BTreeSet<String>,
}

impl PropsetMetadataEntry {
    pub(crate) fn try_from_raw(
        ctx: &mut FactoryContext<'_>,
        name: &str,
        raw: RawPropsetMetadataEntry,
    ) -> Option<Self> {
        macro_rules! err_continue {
            ($err:expr) => {
                ctx.error.propset_errors.push((name.to_owned(), $err));
                continue;
            };
        }

        let mut props: BTreeMap<String, PropValue> = BTreeMap::new();
        let mut props_refs: BTreeSet<String> = BTreeSet::new();

        for raw_prop in raw.props {
            let (stringname, opts) = parse_prop_item(&raw_prop);

            if let Some(props_ref_name) = stringname.strip_suffix("_REF") {
                if !opts.is_empty() {
                    err_continue!(PropsetError::PropsetPropsRefWithOptions {
                        props_ref_cname: props_ref_name.to_owned(),
                    });
                }

                props_refs.insert(props_ref_name.to_owned());
                continue;
            }

            let Some(&prop_cname) = ctx.stringname_to_cname.get(stringname) else {
                err_continue!(PropsetError::PropsetUndefinedProp {
                    stringname: stringname.to_owned(),
                });
            };

            let value = match PropValue::try_from_options(opts) {
                Ok(value) => value,
                Err(errors) => {
                    for error in errors {
                        err_continue!(PropsetError::PropsetPropValueError {
                            prop_cname: prop_cname.to_owned(),
                            error,
                        });
                    }
                    continue;
                }
            };

            match props.entry(prop_cname.to_owned()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                std::collections::btree_map::Entry::Occupied(_) => {
                    if prop_cname == "kOfxImageEffectPluginRenderThreadSafety" {
                        // TODO: report this to the upstream.
                        tracing::warn!(
                            "parse_propset: Don't forget to report this to the upstream!!!: {prop_cname}"
                        );
                        continue;
                    } else {
                        err_continue!(PropsetError::PropsetPropDuplicate {
                            prop_cname: prop_cname.to_owned(),
                        });
                    }
                }
            }
        }

        Some(Self {
            write: raw.write,
            props,
            props_refs,
        })
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PropsetdefMetadataEntry {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub props: BTreeMap<String, PropValue>,
}

impl PropsetdefMetadataEntry {
    pub(crate) fn try_from_raw(
        ctx: &mut FactoryContext<'_>,
        name: &str,
        raw: RawPropsetdefMetadataEntry,
    ) -> Option<Self> {
        macro_rules! err_continue {
            ($err:expr) => {
                ctx.error.propsetdef_errors.push((name.to_owned(), $err));
                continue;
            };
        }

        let mut props: BTreeMap<String, PropValue> = BTreeMap::new();

        for raw_prop in raw.0 {
            let (stringname, opts) = parse_prop_item(&raw_prop);

            let Some(&prop_cname) = ctx.stringname_to_cname.get(stringname) else {
                err_continue!(PropsetdefError::PropsetdefUndefinedProp {
                    stringname: stringname.to_owned(),
                });
            };

            let value = match PropValue::try_from_options(opts) {
                Ok(value) => value,
                Err(errors) => {
                    for error in errors {
                        err_continue!(PropsetdefError::PropsetdefPropValueError {
                            prop_cname: prop_cname.to_owned(),
                            error,
                        });
                    }
                    continue;
                }
            };

            match props.entry(prop_cname.to_owned()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                std::collections::btree_map::Entry::Occupied(_) => {
                    err_continue!(PropsetdefError::PropsetdefPropDuplicate {
                        prop_cname: prop_cname.to_owned(),
                    });
                }
            }
        }

        Some(Self { props })
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ActiondefMetadataEntry {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub in_args: BTreeMap<String, PropValue>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub out_args: BTreeMap<String, PropValue>,
}

impl ActiondefMetadataEntry {
    pub(crate) fn try_from_raw(
        ctx: &mut FactoryContext<'_>,
        cname: &str,
        raw: RawActiondefMetadataEntry,
    ) -> Option<Self> {
        macro_rules! err_continue {
            ($err:expr) => {
                ctx.error.actiondef_errors.push((cname.to_owned(), $err));
                continue;
            };
        }

        let mut in_args: BTreeMap<String, PropValue> = BTreeMap::new();
        let mut out_args: BTreeMap<String, PropValue> = BTreeMap::new();

        for (is_in, args, raw_args) in [
            (true, &mut in_args, raw.in_args),
            (false, &mut out_args, raw.out_args),
        ] {
            for raw_arg in raw_args {
                let (stringname, opts) = parse_prop_item(&raw_arg);

                let Some(&prop_cname) = ctx.stringname_to_cname.get(stringname) else {
                    if is_in {
                        err_continue!(ActiondefError::ActiondefUndefinedInArg {
                            stringname: stringname.to_owned(),
                        });
                    } else {
                        err_continue!(ActiondefError::ActiondefUndefinedOutArg {
                            stringname: stringname.to_owned(),
                        });
                    }
                };

                let value = match PropValue::try_from_options(opts) {
                    Ok(value) => value,
                    Err(errors) => {
                        for error in errors {
                            err_continue!(ActiondefError::ActiondefPropValueError {
                                is_in_arg: is_in,
                                prop_cname: prop_cname.to_owned(),
                                error,
                            });
                        }
                        continue;
                    }
                };

                args.insert(prop_cname.to_owned(), value);
            }
        }

        Some(Self { in_args, out_args })
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "$type")]
pub enum PropdefType {
    Simple { one_of: BTreeSet<PropdefTypeSimple> },
    StringEnum { one_of: Vec<StringEnumVariant> },
}

impl PropdefType {
    fn try_from_raw(
        raw_types: &[RawPropdefType],
        raw_values: &[String],
        stringname_to_cname: &HashMap<&str, &str>,
    ) -> Result<Self, PropdefError> {
        let mut ty = if raw_types.is_empty() {
            return Err(PropdefError::PropdefNoType);
        } else if raw_types.len() == 1 {
            match &raw_types[0] {
                RawPropdefType::Int => PropdefType::new_one(PropdefTypeSimple::Int),
                RawPropdefType::Double => PropdefType::new_one(PropdefTypeSimple::Double),
                RawPropdefType::Bool => PropdefType::new_one(PropdefTypeSimple::Bool),
                RawPropdefType::String => PropdefType::new_one(PropdefTypeSimple::String),
                RawPropdefType::Pointer => PropdefType::new_one(PropdefTypeSimple::Pointer),
                RawPropdefType::Enum => PropdefType::StringEnum {
                    one_of: Default::default(),
                },
            }
        } else {
            let mut one_of: BTreeSet<PropdefTypeSimple> = BTreeSet::new();
            for raw_type in raw_types {
                one_of.insert(match raw_type {
                    RawPropdefType::Int => PropdefTypeSimple::Int,
                    RawPropdefType::Double => PropdefTypeSimple::Double,
                    RawPropdefType::Bool => PropdefTypeSimple::Bool,
                    RawPropdefType::String => PropdefTypeSimple::String,
                    RawPropdefType::Pointer => PropdefTypeSimple::Pointer,
                    RawPropdefType::Enum => {
                        return Err(PropdefError::PropdefEnumAmongOtherTypes);
                    }
                });
            }
            PropdefType::Simple { one_of }
        };

        if let PropdefType::StringEnum { one_of } = &mut ty {
            if raw_values.is_empty() {
                return Err(PropdefError::PropdefEnumWithoutValuesField);
            }

            for stringname in raw_values {
                let variant = if let Some(&cname) = stringname_to_cname.get(stringname.as_str()) {
                    StringEnumVariant::Defined {
                        cname: cname.to_owned(),
                    }
                } else {
                    if stringname
                        .find(|c: char| c.is_whitespace() || c == '|')
                        .is_some()
                    {
                        // must be a new syntax.
                        return Err(PropdefError::PropdefUnexpectedFieldValue {
                            field_name: "values".to_owned(),
                            value: stringname.to_owned(),
                        });
                    }
                    if stringname.starts_with("Ofx") || stringname.starts_with("kOfx") {
                        tracing::warn!("parse_propdef: Slipped through?: {stringname}");
                    }
                    StringEnumVariant::Literal {
                        value: stringname.to_owned(),
                    }
                };
                one_of.push(variant);
            }
        } else if !raw_values.is_empty() {
            return Err(PropdefError::PropdefNonEnumWithValuesField);
        }

        Ok(ty)
    }
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
#[serde(rename_all = "snake_case")]
pub enum PropdefTypeSimple {
    Int,
    Double,
    Bool,
    String,
    Pointer,
}

#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
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

#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "$type")]
pub enum PropdefDimension {
    Fixed { size: NonZero<usize> },
    Dynamic,
}

impl PropdefDimension {
    fn try_from_str(dimension: &str) -> Option<Self> {
        if dimension == "N" {
            Some(Self::Dynamic)
        } else {
            match NonZero::new(dimension.parse().ok()?) {
                None => Some(Self::Dynamic),
                Some(size) => Some(Self::Fixed { size }),
            }
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum WriteSide {
    All,
    Host,
    Plugin,
}

impl WriteSide {
    pub fn try_from(value: &str) -> Option<Self> {
        match value {
            "all" => Some(Self::All),
            "host" => Some(Self::Host),
            "plugin" => Some(Self::Plugin),
            _ => None,
        }
    }
}

#[derive(
    Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PropValue {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub host_optional: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write: Option<WriteSide>,
}

impl PropValue {
    pub fn try_from_options(
        opts: HashMap<&str, Option<&str>>,
    ) -> Result<PropValue, Vec<PropValueError>> {
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
                    if opt_value != Some("true") {
                        errors.push(PropValueError::PropValueUnexpectedOptionValue {
                            option_name: "host_optional".to_owned(),
                            option_value: opt_value.map(|s| s.to_owned()),
                        });
                        continue;
                    }
                    value.host_optional = true;
                }
                // `optional` expands to `host_optional=true`
                "optional" => {
                    if value.host_optional {
                        errors.push(PropValueError::PropValueDuplicateOption {
                            option_name: "optional".to_owned(),
                        });
                        continue;
                    }
                    if !opt_value.is_none() {
                        errors.push(PropValueError::PropValueUnexpectedOptionValue {
                            option_name: "optional".to_owned(),
                            option_value: opt_value.map(|s| s.to_owned()),
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
                    let Some(opt_value) = opt_value else {
                        errors.push(PropValueError::PropValueUnexpectedOptionValue {
                            option_name: "write".to_owned(),
                            option_value: None,
                        });
                        continue;
                    };
                    value.write = match WriteSide::try_from(opt_value) {
                        Some(write_side) => Some(write_side),
                        None => {
                            errors.push(PropValueError::PropValueUnexpectedOptionValue {
                                option_name: "write".to_owned(),
                                option_value: Some(opt_value.to_owned()),
                            });
                            continue;
                        }
                    }
                }
                _ => {
                    errors.push(PropValueError::PropValueUnexpectedOption {
                        option_name: opt_name.to_owned(),
                        option_value: opt_value.map(|s| s.to_owned()),
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

fn parse_prop_item(prop_item: &str) -> (&str, HashMap<&str, Option<&str>>) {
    let (stringname, rest) = prop_item.split_once("|").unwrap_or((prop_item, ""));
    let stringname = stringname.trim();
    if rest.is_empty() {
        return (stringname, HashMap::new());
    }

    let mut opts: HashMap<&str, Option<&str>> = HashMap::new();

    for opt_str in rest.split(",") {
        let opt_str = opt_str.trim();
        if opt_str.is_empty() {
            continue;
        }
        let Some((key, value)) = opt_str.split_once("=") else {
            opts.insert(opt_str.trim(), None);
            continue;
        };
        opts.insert(key.trim(), Some(value.trim()));
    }

    (stringname, opts)
}
