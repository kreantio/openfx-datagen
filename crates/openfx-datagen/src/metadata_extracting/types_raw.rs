use crate::metadata_extracting::WriteSide;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPropdefMetadataEntry {
    #[serde(deserialize_with = "deserialize_scalar_or_list_raw_propdef_type")]
    pub r#type: Vec<RawPropdefType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    pub dimension: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introduced: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<String>,
    /// e.g., `hostOptional: true`.
    #[serde(
        default,
        rename = "hostOptional",
        skip_serializing_if = "std::ops::Not::not"
    )]
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawPropsetMetadataEntry {
    pub write: WriteSide,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct RawPropsetdefMetadataEntry(pub Vec<String>);

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawActiondefMetadataEntry {
    #[serde(default, rename = "inArgs", skip_serializing_if = "Vec::is_empty")]
    pub in_args: Vec<String>,
    #[serde(default, rename = "outArgs", skip_serializing_if = "Vec::is_empty")]
    pub out_args: Vec<String>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum RawPropdefType {
    Int,
    Double,
    Bool,
    String,
    Pointer,
    Enum,
}

impl RawPropdefType {
    fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "int" => Some(Self::Int),
            "double" => Some(Self::Double),
            "bool" => Some(Self::Bool),
            "string" => Some(Self::String),
            "pointer" => Some(Self::Pointer),
            "enum" => Some(Self::Enum),
            _ => None,
        }
    }
}

fn deserialize_scalar_or_list_raw_propdef_type<'de, D>(
    deserializer: D,
) -> Result<Vec<RawPropdefType>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Vec<RawPropdefType>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                f,
                "a scalar or list of `openfx_datagen::metadata_extracting::types_raw::RawPropdefType`"
            )
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if let Some(t) = RawPropdefType::try_from_str(v) {
                Ok(vec![t])
            } else {
                Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(v),
                    &self,
                ))
            }
        }
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut ret = vec![];
            while let Some(v) = seq.next_element::<&str>()? {
                if let Some(t) = RawPropdefType::try_from_str(v) {
                    ret.push(t);
                } else {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &self,
                    ));
                }
            }
            Ok(ret)
        }
    }

    deserializer.deserialize_any(Visitor)
}
