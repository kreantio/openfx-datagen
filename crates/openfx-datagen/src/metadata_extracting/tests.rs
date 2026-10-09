#![allow(non_snake_case)]

use super::*;

fn lines(input: &str) -> Peekable<SignificantLines<'_>> {
    SignificantLines::new(input.lines()).peekable()
}

#[test]
fn test_propdef_kOfxImagePropBounds() {
    let mut error = Error::default();

    let actual = parse_propdef(
        "kOfxImagePropBounds",
        lines(
            r#"type: int
    dimension: 4"#,
        ),
        &mut error,
        &HashMap::from([]),
    );

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

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propdef_kOfxImageEffectPropProjectPixelAspectRatio() {
    let mut error = Error::default();

    let actual = parse_propdef(
        "kOfxImageEffectPropProjectPixelAspectRatio",
        lines(
            r#"type: double
    dimension: 1
    cname: kOfxImageEffectPropProjectPixelAspectRatio"#,
        ),
        &mut error,
        &HashMap::from([]),
    );

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

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propdef_kOfxParamPropEnabled() {
    let mut error = Error::default();

    let actual = parse_propdef(
        "kOfxParamPropEnabled",
        lines(
            r#"type: bool
    dimension: 1
    optional: true"#,
        ),
        &mut error,
        &HashMap::from([]),
    );

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

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propdef_kOfxParamPropChoiceEnum() {
    let mut error = Error::default();

    let actual = parse_propdef(
        "kOfxParamPropChoiceEnum",
        lines(
            r#"type: bool
    dimension: 1
    added: "1.5""#,
        ),
        &mut error,
        &HashMap::from([]),
    );

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::Simple {
            one_of: BTreeSet::from([PropdefTypeSimple::Bool]),
        },
        dimension: PropdefDimension::Fixed { size: 1 },
        introduced: Some(r#""1.5""#.to_owned()),
        deprecated: Default::default(),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propdef_kOfxParamPropPluginMayWrite() {
    let mut error = Error::default();

    let actual = parse_propdef(
        "kOfxParamPropPluginMayWrite",
        lines(
            r#"type: bool
    dimension: 1
    deprecated: "1.4""#,
        ),
        &mut error,
        &HashMap::from([]),
    );

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::Simple {
            one_of: BTreeSet::from([PropdefTypeSimple::Bool]),
        },
        dimension: PropdefDimension::Fixed { size: 1 },
        introduced: Default::default(),
        deprecated: Some(r#""1.4""#.to_owned()),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propdef_kOfxImageEffectPropColourManagementAvailableConfigs() {
    let mut error = Error::default();

    let actual = parse_propdef(
        "kOfxImageEffectPropColourManagementAvailableConfigs",
        lines(
            r#"type: string
   dimension: N
   introduced: "1.5""#,
        ),
        &mut error,
        &HashMap::from([]),
    );

    let expected = PropdefMetadataEntry {
        r#type: PropdefType::Simple {
            one_of: BTreeSet::from([PropdefTypeSimple::String]),
        },
        dimension: PropdefDimension::Dynamic,
        introduced: Some(r#""1.5""#.to_owned()),
        deprecated: Default::default(),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propdef_kOfxPropInstanceData() {
    let mut error = Error::default();

    let actual = parse_propdef(
        "kOfxPropInstanceData",
        lines(
            r#"type: pointer
    dimension: 1"#,
        ),
        &mut error,
        &HashMap::from([]),
    );

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

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propdef_kOfxParamPropDefault() {
    let mut error = Error::default();

    let actual = parse_propdef(
        "kOfxParamPropDefault",
        lines(
            r#"type: [int, double, string, pointer]
   dimension: N"#,
        ),
        &mut error,
        &HashMap::from([]),
    );

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

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propdef_kOfxImageEffectPropColourManagementStyle() {
    let mut error = Error::default();

    let actual = parse_propdef(
        "kOfxImageEffectPropColourManagementStyle",
        lines(
            r#"type: enum
   dimension: 1
   values:
     - OfxImageEffectColourManagementNone
     - OfxImageEffectColourManagementBasic
     - OfxImageEffectColourManagementCore
     - OfxImageEffectColourManagementFull
     - OfxImageEffectColourManagementOCIO
   introduced: "1.5""#,
        ),
        &mut error,
        &HashMap::from([
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
    );

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
        introduced: Some(r#""1.5""#.to_owned()),
        deprecated: Default::default(),
        host_optional: Default::default(),
        optional: Default::default(),
        cname: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propdef_kOfxImageEffectPropOpenGLRenderSupported() {
    let mut error = Error::default();

    let actual = parse_propdef(
        "kOfxImageEffectPropOpenGLRenderSupported",
        lines(
            r#"type: enum
    dimension: 1
    values:
      - "false"
      - "true"
      - needed"#,
        ),
        &mut error,
        &HashMap::from([]),
    );

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

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propset_InteractDescriptor() {
    let mut error = Error::default();

    let actual = parse_propset(
        "InteractDescriptor",
        lines(
            r#"write: host
    props:
      - OfxInteractPropHasAlpha
      - OfxInteractPropBitDepth"#,
        ),
        &mut error,
        &HashMap::from([
            ("OfxInteractPropBitDepth", "kOfxInteractPropBitDepth"),
            ("OfxInteractPropHasAlpha", "kOfxInteractPropHasAlpha"),
        ]),
    );

    let expected = PropsetMetadataEntry {
        write: WriteSide::Host,
        props: BTreeMap::from([
            ("kOfxInteractPropHasAlpha".to_owned(), PropValue::default()),
            ("kOfxInteractPropBitDepth".to_owned(), PropValue::default()),
        ]),
        props_refs: Default::default(),
    };

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propset_ParamsGroup() {
    let mut error = Error::default();

    let actual = parse_propset(
        "ParamsGroup",
        lines(
            r#"write: plugin
    props:
      - OfxParamPropGroupOpen
      - ParamsCommon_REF"#,
        ),
        &mut error,
        &HashMap::from([("OfxParamPropGroupOpen", "kOfxParamPropGroupOpen")]),
    );

    let expected = PropsetMetadataEntry {
        write: WriteSide::Plugin,
        props: BTreeMap::from([("kOfxParamPropGroupOpen".to_owned(), PropValue::default())]),
        props_refs: BTreeSet::from(["ParamsCommon".to_owned()]),
    };

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propset_EffectDescriptor() {
    let mut error = Error::default();

    let actual = parse_propset(
        "EffectDescriptor",
        lines(
            r#"write: plugin
    props:
      - OfxPropType
      # …
      - OfxPropVersion | host_optional=true
      # …
      - OfxImageEffectPropCPURenderSupported | host_optional=true
      - OfxPluginPropFilePath | write=host"#,
        ),
        &mut error,
        &HashMap::from([
            ("OfxPropType", "kOfxPropType"),
            ("OfxPropVersion", "kOfxPropVersion"),
            (
                "OfxImageEffectPropCPURenderSupported",
                "kOfxImageEffectPropCPURenderSupported",
            ),
            ("OfxPluginPropFilePath", "kOfxPluginPropFilePath"),
        ]),
    );

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

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_propsetdef_ParamsValue() {
    let mut error = Error::default();

    let actual = parse_propsetdef(
        "ParamsValue",
        lines(
            r#"- OfxParamPropDefault
    - OfxParamPropAnimates
    - OfxParamPropIsAnimating | write=host
    - OfxParamPropIsAutoKeying | write=host
    # …"#,
        ),
        &mut error,
        &HashMap::from([
            ("OfxParamPropDefault", "kOfxParamPropDefault"),
            ("OfxParamPropAnimates", "kOfxParamPropAnimates"),
            ("OfxParamPropIsAnimating", "kOfxParamPropIsAnimating"),
            ("OfxParamPropIsAutoKeying", "kOfxParamPropIsAutoKeying"),
        ]),
    );

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

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_actiondef_kOfxImageEffectActionGetOutputColourspace() {
    let mut error = Error::default();

    let actual = parse_actiondef(
        "kOfxImageEffectActionGetOutputColourspace",
        lines(
            r#"inArgs:
      - OfxImageClipPropPreferredColourspaces
    outArgs:
      - OfxImageClipPropColourspace"#,
        ),
        &mut error,
        &HashMap::from([
            (
                "OfxImageClipPropPreferredColourspaces",
                "kOfxImageClipPropPreferredColourspaces",
            ),
            (
                "OfxImageClipPropColourspace",
                "kOfxImageClipPropColourspace",
            ),
        ]),
    );

    let expected = ActiondefMetadataEntry {
        in_args: BTreeSet::from(["kOfxImageClipPropPreferredColourspaces".to_owned()]),
        out_args: BTreeSet::from(["kOfxImageClipPropColourspace".to_owned()]),
    };

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_actiondef_kOfxActionLoad() {
    let mut error = Error::default();

    let actual = parse_actiondef(
        "kOfxActionLoad",
        lines(
            r#"inArgs:
    outArgs:"#,
        ),
        &mut error,
        &HashMap::from([]),
    );

    let expected = ActiondefMetadataEntry {
        in_args: BTreeSet::from([]),
        out_args: BTreeSet::from([]),
    };

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_actiondef_kOfxActionBeginInstanceChanged() {
    let mut error = Error::default();

    let actual = parse_actiondef(
        "kOfxActionBeginInstanceChanged",
        lines(
            r#"inArgs:
      - OfxPropChangeReason
      - OfxImageEffectPropThumbnailRender
    outArgs: []"#,
        ),
        &mut error,
        &HashMap::from([
            ("OfxPropChangeReason", "kOfxPropChangeReason"),
            (
                "OfxImageEffectPropThumbnailRender",
                "kOfxImageEffectPropThumbnailRender",
            ),
        ]),
    );

    let expected = ActiondefMetadataEntry {
        in_args: BTreeSet::from([
            "kOfxPropChangeReason".to_owned(),
            "kOfxImageEffectPropThumbnailRender".to_owned(),
        ]),
        out_args: BTreeSet::from([]),
    };

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}

#[test]
fn test_actiondef_kOfxImageEffectActionGetRegionsOfInterest() {
    let mut error = Error::default();

    let actual = parse_actiondef(
        "kOfxImageEffectActionGetRegionsOfInterest",
        lines(
            r#"inArgs:
      - OfxPropTime
      - OfxImageEffectPropRenderScale
      - OfxImageEffectPropRegionOfInterest
      - OfxImageEffectPropThumbnailRender
    outArgs:
    # - OfxImageEffectClipPropRoI_ # with clip name"#,
        ),
        &mut error,
        &HashMap::from([
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
    );

    let expected = ActiondefMetadataEntry {
        in_args: BTreeSet::from([
            "kOfxPropTime".to_owned(),
            "kOfxImageEffectPropRenderScale".to_owned(),
            "kOfxImageEffectPropRegionOfInterest".to_owned(),
            "kOfxImageEffectPropThumbnailRender".to_owned(),
        ]),
        out_args: BTreeSet::from([]),
    };

    pretty_assertions::assert_eq!(actual, Some(expected));

    assert!(error.is_empty());
}
