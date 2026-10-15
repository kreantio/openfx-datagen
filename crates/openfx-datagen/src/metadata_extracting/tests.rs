#![allow(non_snake_case)]

use std::collections::BTreeSet;

use super::*;

fn parse_propdef(
    cname: &str,
    content: &str,
    stringname_to_cname: HashMap<&str, &str>,
) -> Result<PropdefMetadataEntry, Box<Error>> {
    let mut ctx = FactoryContext {
        stringname_to_cname,
        error: Default::default(),
    };

    let raw = match serde_saphyr::from_str::<RawPropdefMetadataEntry>(content) {
        Ok(raw) => raw,
        Err(err) => {
            ctx.error.propdef_errors.push((
                cname.to_owned(),
                PropdefError::PropdefSerdeSaphyrParse { error: err },
            ));
            return Err(Box::new(ctx.error));
        }
    };

    match PropdefMetadataEntry::try_from_raw(&mut ctx, cname, raw) {
        Some(entry) => Ok(entry),
        None => Err(Box::new(ctx.error)),
    }
}

fn parse_propset(
    name: &str,
    content: &str,
    stringname_to_cname: HashMap<&str, &str>,
) -> Result<PropsetMetadataEntry, Box<Error>> {
    let mut ctx = FactoryContext {
        stringname_to_cname,
        error: Default::default(),
    };

    let raw = match serde_saphyr::from_str::<RawPropsetMetadataEntry>(content) {
        Ok(raw) => raw,
        Err(err) => {
            ctx.error.propset_errors.push((
                name.to_owned(),
                PropsetError::PropsetSerdeSaphyrParse { error: err },
            ));
            return Err(Box::new(ctx.error));
        }
    };

    match PropsetMetadataEntry::try_from_raw(&mut ctx, name, raw) {
        Some(entry) => Ok(entry),
        None => Err(Box::new(ctx.error)),
    }
}

fn parse_propsetdef(
    name: &str,
    content: &str,
    stringname_to_cname: HashMap<&str, &str>,
) -> Result<PropsetdefMetadataEntry, Box<Error>> {
    let mut ctx = FactoryContext {
        stringname_to_cname,
        error: Default::default(),
    };

    let raw = match serde_saphyr::from_str::<RawPropsetdefMetadataEntry>(content) {
        Ok(raw) => raw,
        Err(err) => {
            ctx.error.propsetdef_errors.push((
                name.to_owned(),
                PropsetdefError::PropsetdefSerdeSaphyrParse { error: err },
            ));
            return Err(Box::new(ctx.error));
        }
    };

    match PropsetdefMetadataEntry::try_from_raw(&mut ctx, name, raw) {
        Some(entry) => Ok(entry),
        None => Err(Box::new(ctx.error)),
    }
}

fn parse_actiondef(
    cname: &str,
    content: &str,
    stringname_to_cname: HashMap<&str, &str>,
) -> Result<ActiondefMetadataEntry, Box<Error>> {
    let mut ctx = FactoryContext {
        stringname_to_cname,
        error: Default::default(),
    };

    let raw = match serde_saphyr::from_str::<RawActiondefMetadataEntry>(content) {
        Ok(raw) => raw,
        Err(err) => {
            ctx.error.actiondef_errors.push((
                cname.to_owned(),
                ActiondefError::ActiondefSerdeSaphyrParse { error: err },
            ));
            return Err(Box::new(ctx.error));
        }
    };

    match ActiondefMetadataEntry::try_from_raw(&mut ctx, cname, raw) {
        Some(entry) => Ok(entry),
        None => Err(Box::new(ctx.error)),
    }
}

#[test]
fn test_propdef_kOfxImagePropBounds() -> Result<(), Box<Error>> {
    let actual = parse_propdef(
        "kOfxImagePropBounds",
        r#"
    type: int
    dimension: 4"#,
        HashMap::from([]),
    )?;

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::Simple {
            one_of: BTreeSet::from([PropdefTypeSimple::Int]),
        },
        dimension: PropdefDimension::Fixed { size: 4 },
        introduced: Default::default(),
        deprecated: Default::default(),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propdef_kOfxImageEffectPropProjectPixelAspectRatio() -> Result<(), Box<Error>> {
    let actual = parse_propdef(
        "kOfxImageEffectPropProjectPixelAspectRatio",
        r#"
    type: double
    dimension: 1
    cname: kOfxImageEffectPropProjectPixelAspectRatio"#,
        HashMap::from([]),
    )?;

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::Simple {
            one_of: BTreeSet::from([PropdefTypeSimple::Double]),
        },
        dimension: PropdefDimension::Fixed { size: 1 },
        introduced: Default::default(),
        deprecated: Default::default(),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Some(r#"kOfxImageEffectPropProjectPixelAspectRatio"#.to_owned()),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propdef_kOfxParamPropEnabled() -> Result<(), Box<Error>> {
    let actual = parse_propdef(
        "kOfxParamPropEnabled",
        r#"
    type: bool
    dimension: 1
    optional: true"#,
        HashMap::from([]),
    )?;

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::Simple {
            one_of: BTreeSet::from([PropdefTypeSimple::Bool]),
        },
        dimension: PropdefDimension::Fixed { size: 1 },
        introduced: Default::default(),
        deprecated: Default::default(),
        host_optional: Default::default(),
        optional: true,
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propdef_kOfxParamPropChoiceEnum() -> Result<(), Box<Error>> {
    let actual = parse_propdef(
        "kOfxParamPropChoiceEnum",
        r#"
    type: bool
    dimension: 1
    added: "1.5""#,
        HashMap::from([]),
    )?;

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::Simple {
            one_of: BTreeSet::from([PropdefTypeSimple::Bool]),
        },
        dimension: PropdefDimension::Fixed { size: 1 },
        introduced: Some(r#"1.5"#.to_owned()),
        deprecated: Default::default(),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propdef_kOfxParamPropPluginMayWrite() -> Result<(), Box<Error>> {
    let actual = parse_propdef(
        "kOfxParamPropPluginMayWrite",
        r#"
    type: bool
    dimension: 1
    deprecated: "1.4""#,
        HashMap::from([]),
    )?;

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::Simple {
            one_of: BTreeSet::from([PropdefTypeSimple::Bool]),
        },
        dimension: PropdefDimension::Fixed { size: 1 },
        introduced: Default::default(),
        deprecated: Some(r#"1.4"#.to_owned()),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propdef_kOfxImageEffectPropColourManagementAvailableConfigs() -> Result<(), Box<Error>> {
    let actual = parse_propdef(
        "kOfxImageEffectPropColourManagementAvailableConfigs",
        r#"
   type: string
   dimension: N
   introduced: "1.5""#,
        HashMap::from([]),
    )?;

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::Simple {
            one_of: BTreeSet::from([PropdefTypeSimple::String]),
        },
        dimension: PropdefDimension::Dynamic,
        introduced: Some(r#"1.5"#.to_owned()),
        deprecated: Default::default(),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propdef_kOfxPropInstanceData() -> Result<(), Box<Error>> {
    let actual = parse_propdef(
        "kOfxPropInstanceData",
        r#"
    type: pointer
    dimension: 1"#,
        HashMap::from([]),
    )?;

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::Simple {
            one_of: BTreeSet::from([PropdefTypeSimple::Pointer]),
        },
        dimension: PropdefDimension::Fixed { size: 1 },
        introduced: Default::default(),
        deprecated: Default::default(),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propdef_kOfxParamPropDefault() -> Result<(), Box<Error>> {
    let actual = parse_propdef(
        "kOfxParamPropDefault",
        r#"
   type: [int, double, string, pointer]
   dimension: N"#,
        HashMap::from([]),
    )?;

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::Simple {
            one_of: BTreeSet::from([
                PropdefTypeSimple::Int,
                PropdefTypeSimple::Double,
                PropdefTypeSimple::String,
                PropdefTypeSimple::Pointer,
            ]),
        },
        dimension: PropdefDimension::Dynamic,
        introduced: Default::default(),
        deprecated: Default::default(),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propdef_kOfxImageEffectPropColourManagementStyle() -> Result<(), Box<Error>> {
    let actual = parse_propdef(
        "kOfxImageEffectPropColourManagementStyle",
        r#"
   type: enum
   dimension: 1
   values:
     - OfxImageEffectColourManagementNone
     - OfxImageEffectColourManagementBasic
     - OfxImageEffectColourManagementCore
     - OfxImageEffectColourManagementFull
     - OfxImageEffectColourManagementOCIO
   introduced: "1.5""#,
        HashMap::from([
            (
                "OfxImageEffectColourManagementNone",
                "kOfxImageEffectColourManagementNone",
            ),
            (
                "OfxImageEffectColourManagementBasic",
                "kOfxImageEffectColourManagementBasic",
            ),
            (
                "OfxImageEffectColourManagementCore",
                "kOfxImageEffectColourManagementCore",
            ),
            (
                "OfxImageEffectColourManagementFull",
                "kOfxImageEffectColourManagementFull",
            ),
            (
                "OfxImageEffectColourManagementOCIO",
                "kOfxImageEffectColourManagementOCIO",
            ),
        ]),
    )?;

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::StringEnum {
            one_of: vec![
                StringEnumVariant::Defined {
                    cname: "kOfxImageEffectColourManagementNone".to_owned(),
                },
                StringEnumVariant::Defined {
                    cname: "kOfxImageEffectColourManagementBasic".to_owned(),
                },
                StringEnumVariant::Defined {
                    cname: "kOfxImageEffectColourManagementCore".to_owned(),
                },
                StringEnumVariant::Defined {
                    cname: "kOfxImageEffectColourManagementFull".to_owned(),
                },
                StringEnumVariant::Defined {
                    cname: "kOfxImageEffectColourManagementOCIO".to_owned(),
                },
            ],
        },
        dimension: PropdefDimension::Fixed { size: 1 },
        introduced: Some(r#"1.5"#.to_owned()),
        deprecated: Default::default(),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propdef_kOfxImageEffectPropOpenGLRenderSupported() -> Result<(), Box<Error>> {
    let actual = parse_propdef(
        "kOfxImageEffectPropOpenGLRenderSupported",
        r#"
    type: enum
    dimension: 1
    values:
      - "false"
      - "true"
      - needed"#,
        HashMap::from([]),
    )?;

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::StringEnum {
            one_of: vec![
                StringEnumVariant::Literal {
                    value: "false".to_owned(),
                },
                StringEnumVariant::Literal {
                    value: "true".to_owned(),
                },
                StringEnumVariant::Literal {
                    value: "needed".to_owned(),
                },
            ],
        },
        dimension: PropdefDimension::Fixed { size: 1 },
        introduced: Default::default(),
        deprecated: Default::default(),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propset_InteractDescriptor() -> Result<(), Box<Error>> {
    let actual = parse_propset(
        "InteractDescriptor",
        r#"
    write: host
    props:
      - OfxInteractPropHasAlpha
      - OfxInteractPropBitDepth"#,
        HashMap::from([
            ("OfxInteractPropBitDepth", "kOfxInteractPropBitDepth"),
            ("OfxInteractPropHasAlpha", "kOfxInteractPropHasAlpha"),
        ]),
    )?;

    let expected = PropsetMetadataEntry {
        write: WriteSide::Host,
        props: BTreeMap::from([
            ("kOfxInteractPropHasAlpha".to_owned(), PropValue::default()),
            ("kOfxInteractPropBitDepth".to_owned(), PropValue::default()),
        ]),
        props_refs: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propset_ParamsGroup() -> Result<(), Box<Error>> {
    let actual = parse_propset(
        "ParamsGroup",
        r#"
    write: plugin
    props:
      - OfxParamPropGroupOpen
      - ParamsCommon_REF"#,
        HashMap::from([("OfxParamPropGroupOpen", "kOfxParamPropGroupOpen")]),
    )?;

    let expected = PropsetMetadataEntry {
        write: WriteSide::Plugin,
        props: BTreeMap::from([("kOfxParamPropGroupOpen".to_owned(), PropValue::default())]),
        props_refs: BTreeSet::from(["ParamsCommon".to_owned()]),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propset_EffectDescriptor() -> Result<(), Box<Error>> {
    let actual = parse_propset(
        "EffectDescriptor",
        r#"
    write: plugin
    props:
      - OfxPropType
      # …
      - OfxPropVersion | host_optional=true
      # …
      - OfxImageEffectPropCPURenderSupported | host_optional=true
      - OfxPluginPropFilePath | write=host"#,
        HashMap::from([
            ("OfxPropType", "kOfxPropType"),
            ("OfxPropVersion", "kOfxPropVersion"),
            (
                "OfxImageEffectPropCPURenderSupported",
                "kOfxImageEffectPropCPURenderSupported",
            ),
            ("OfxPluginPropFilePath", "kOfxPluginPropFilePath"),
        ]),
    )?;

    let expected = PropsetMetadataEntry {
        write: WriteSide::Plugin,
        props: BTreeMap::from([
            ("kOfxPropType".to_owned(), PropValue::default()),
            (
                "kOfxPropVersion".to_owned(),
                PropValue {
                    host_optional: true,
                    ..Default::default()
                },
            ),
            (
                "kOfxImageEffectPropCPURenderSupported".to_owned(),
                PropValue {
                    host_optional: true,
                    ..Default::default()
                },
            ),
            (
                "kOfxPluginPropFilePath".to_owned(),
                PropValue {
                    write: Some(WriteSide::Host),
                    ..Default::default()
                },
            ),
        ]),
        props_refs: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_propsetdef_ParamsValue() -> Result<(), Box<Error>> {
    let actual = parse_propsetdef(
        "ParamsValue",
        r#"
    - OfxParamPropDefault
    - OfxParamPropAnimates
    - OfxParamPropIsAnimating | write=host
    - OfxParamPropIsAutoKeying | write=host
    # …"#,
        HashMap::from([
            ("OfxParamPropDefault", "kOfxParamPropDefault"),
            ("OfxParamPropAnimates", "kOfxParamPropAnimates"),
            ("OfxParamPropIsAnimating", "kOfxParamPropIsAnimating"),
            ("OfxParamPropIsAutoKeying", "kOfxParamPropIsAutoKeying"),
        ]),
    )?;

    let expected = PropsetdefMetadataEntry {
        props: BTreeMap::from([
            ("kOfxParamPropDefault".to_owned(), PropValue::default()),
            ("kOfxParamPropAnimates".to_owned(), PropValue::default()),
            (
                "kOfxParamPropIsAnimating".to_owned(),
                PropValue {
                    write: Some(WriteSide::Host),
                    ..Default::default()
                },
            ),
            (
                "kOfxParamPropIsAutoKeying".to_owned(),
                PropValue {
                    write: Some(WriteSide::Host),
                    ..Default::default()
                },
            ),
        ]),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_actiondef_kOfxImageEffectActionGetOutputColourspace() -> Result<(), Box<Error>> {
    let actual = parse_actiondef(
        "kOfxImageEffectActionGetOutputColourspace",
        r#"
    inArgs:
      - OfxImageClipPropPreferredColourspaces
    outArgs:
      - OfxImageClipPropColourspace"#,
        HashMap::from([
            (
                "OfxImageClipPropPreferredColourspaces",
                "kOfxImageClipPropPreferredColourspaces",
            ),
            (
                "OfxImageClipPropColourspace",
                "kOfxImageClipPropColourspace",
            ),
        ]),
    )?;

    let expected = ActiondefMetadataEntry {
        in_args: BTreeSet::from(["kOfxImageClipPropPreferredColourspaces".to_owned()]),
        out_args: BTreeSet::from(["kOfxImageClipPropColourspace".to_owned()]),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_actiondef_kOfxActionLoad() -> Result<(), Box<Error>> {
    let actual = parse_actiondef(
        "kOfxActionLoad",
        r#"
    inArgs:
    outArgs:"#,
        HashMap::from([]),
    )?;

    let expected = ActiondefMetadataEntry {
        in_args: BTreeSet::from([]),
        out_args: BTreeSet::from([]),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_actiondef_kOfxActionBeginInstanceChanged() -> Result<(), Box<Error>> {
    let actual = parse_actiondef(
        "kOfxActionBeginInstanceChanged",
        r#"
    inArgs:
      - OfxPropChangeReason
      - OfxImageEffectPropThumbnailRender
    outArgs: []"#,
        HashMap::from([
            ("OfxPropChangeReason", "kOfxPropChangeReason"),
            (
                "OfxImageEffectPropThumbnailRender",
                "kOfxImageEffectPropThumbnailRender",
            ),
        ]),
    )?;

    let expected = ActiondefMetadataEntry {
        in_args: BTreeSet::from([
            "kOfxPropChangeReason".to_owned(),
            "kOfxImageEffectPropThumbnailRender".to_owned(),
        ]),
        out_args: BTreeSet::from([]),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_actiondef_kOfxImageEffectActionGetRegionsOfInterest() -> Result<(), Box<Error>> {
    let actual = parse_actiondef(
        "kOfxImageEffectActionGetRegionsOfInterest",
        r#"
    inArgs:
      - OfxPropTime
      - OfxImageEffectPropRenderScale
      - OfxImageEffectPropRegionOfInterest
      - OfxImageEffectPropThumbnailRender
    outArgs:
    # - OfxImageEffectClipPropRoI_ # with clip name"#,
        HashMap::from([
            ("OfxPropTime", "kOfxPropTime"),
            (
                "OfxImageEffectPropRenderScale",
                "kOfxImageEffectPropRenderScale",
            ),
            (
                "OfxImageEffectPropRegionOfInterest",
                "kOfxImageEffectPropRegionOfInterest",
            ),
            (
                "OfxImageEffectPropThumbnailRender",
                "kOfxImageEffectPropThumbnailRender",
            ),
        ]),
    )?;

    let expected = ActiondefMetadataEntry {
        in_args: BTreeSet::from([
            "kOfxPropTime".to_owned(),
            "kOfxImageEffectPropRenderScale".to_owned(),
            "kOfxImageEffectPropRegionOfInterest".to_owned(),
            "kOfxImageEffectPropThumbnailRender".to_owned(),
        ]),
        out_args: BTreeSet::from([]),
    };

    pretty_assertions::assert_eq!(actual, expected);

    Ok(())
}
