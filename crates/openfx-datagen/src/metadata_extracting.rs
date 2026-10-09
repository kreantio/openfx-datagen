use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    iter::Peekable,
};

use crate::{
    parsing::{BindingsUnprocessed, DefineValue, RootItem, RootItemWithCommentAbove},
    utils::SignificantLines,
};

pub use crate::metadata_extracting::{errors::*, types::*};

mod errors;
mod types;

#[cfg(test)]
mod tests;

pub fn extract_metadata(
    input: &BTreeMap<String, BindingsUnprocessed>,
) -> Result<Metadata, Box<Error>> {
    let stringname_to_cname = extract_stringname_to_cname(input);
    let comment_iter = ExtractedCommentIterator::new(input);

    let mut error = Error::default();
    let mut metadata = Metadata::default();

    for ExtractedComment { cname, content } in comment_iter {
        let Some(section) = extract_metadata_section(&content, &mut error, cname.as_deref()) else {
            continue;
        };
        match section {
            MetadataSection::Propdef(lines) => {
                let Some(cname) = cname else {
                    error.propdef_not_on_item_count += 1;
                    continue;
                };

                if let Some(entry) =
                    parse_propdef(&cname, lines.peekable(), &mut error, &stringname_to_cname)
                {
                    metadata.propdef_map.insert(cname.to_string(), entry);
                }
            }
            MetadataSection::Propset(name, lines) => {
                if let Some(cname) = cname {
                    error.propset_errors.push((
                        name.to_string(),
                        PropsetError::PropsetOnItem {
                            item_cname: cname.to_owned(),
                        },
                    ));
                    continue;
                }

                if let Some(entry) =
                    parse_propset(name, lines.peekable(), &mut error, &stringname_to_cname)
                {
                    metadata.propset_map.insert(name.to_string(), entry);
                }
            }
            MetadataSection::Propsetdef(name, lines) => {
                if let Some(cname) = cname {
                    error.propsetdef_errors.push((
                        name.to_string(),
                        PropsetdefError::PropsetdefOnItem {
                            item_cname: cname.to_owned(),
                        },
                    ));
                    continue;
                }

                if let Some(entry) =
                    parse_propsetdef(name, lines.peekable(), &mut error, &stringname_to_cname)
                {
                    metadata.propsetdef_map.insert(name.to_string(), entry);
                }
            }
            MetadataSection::Actiondef(lines) => {
                let Some(cname) = cname else {
                    error.actiondef_not_on_item_count += 1;
                    continue;
                };

                if let Some(entry) =
                    parse_actiondef(&cname, lines.peekable(), &mut error, &stringname_to_cname)
                {
                    metadata.actiondef_map.insert(cname.to_string(), entry);
                }
            }
        }
    }

    if !error.is_empty() {
        return Err(Box::new(error));
    }

    Ok(metadata)
}

fn parse_propdef(
    cname: &str,
    mut lines: Peekable<SignificantLines>,
    error: &mut Error,
    stringname_to_cname: &HashMap<&str, &str>,
) -> Option<PropdefMetadataEntry> {
    macro_rules! err_continue {
        ($err:expr) => {
            error.propdef_errors.push((cname.to_owned(), $err));
            continue;
        };
    }
    macro_rules! err_return {
        ($err:expr) => {
            error.propdef_errors.push((cname.to_owned(), $err));
            return None;
        };
    }

    let mut r#type: Option<PropdefType> = None;
    let mut values: Vec<StringEnumVariant> = vec![];
    let mut dimension: Option<PropdefDimension> = None;
    let mut introduced: Option<String> = None;
    let mut deprecated: Option<String> = None;
    let mut host_optional: bool = false;
    let mut optional: bool = false;
    let mut prop_cname: Option<String> = None;

    while let Some(line) = lines.next() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        } else if let Some(value) = line.strip_prefix("type: ") {
            if r#type.is_some() {
                err_continue!(PropdefError::PropdefDuplicateField {
                    field_name: "type".to_owned(),
                });
            }

            let value = value.trim();
            r#type = match value {
                "int" => Some(PropdefType::new_one(PropdefTypeSimple::Int)),
                "double" => Some(PropdefType::new_one(PropdefTypeSimple::Double)),
                "bool" => Some(PropdefType::new_one(PropdefTypeSimple::Bool)),
                "string" => Some(PropdefType::new_one(PropdefTypeSimple::String)),
                "pointer" => Some(PropdefType::new_one(PropdefTypeSimple::Pointer)),
                "enum" => Some(PropdefType::StringEnum {
                    one_of: Default::default(),
                }),
                _ if let Some(value) = value.strip_prefix("[")
                    && let Some(value) = value.strip_suffix("]") =>
                {
                    let mut one_of: BTreeSet<PropdefTypeSimple> = BTreeSet::new();
                    for ty in value.split(',') {
                        let ty = ty.trim();
                        let ty = match ty {
                            "int" => PropdefTypeSimple::Int,
                            "double" => PropdefTypeSimple::Double,
                            "bool" => PropdefTypeSimple::Bool,
                            "string" => PropdefTypeSimple::String,
                            "pointer" => PropdefTypeSimple::Pointer,
                            _ => {
                                err_continue!(PropdefError::PropdefUnexpectedFieldValue {
                                    field_name: "type".to_owned(),
                                    value: ty.to_owned(),
                                });
                            }
                        };
                        one_of.insert(ty);
                    }
                    Some(PropdefType::Simple { one_of })
                }
                _ => {
                    err_continue!(PropdefError::PropdefUnexpectedFieldValue {
                        field_name: "type".to_owned(),
                        value: value.to_owned(),
                    });
                }
            }
        } else if line == "values:" {
            while let Some(next_line) = lines.peek()
                && let Some(stringname) = next_line.trim_start().strip_prefix("- ")
            {
                lines.next();
                let stringname = stringname.trim();
                let variant = if let Some(stringname) = stringname.strip_prefix("\"") {
                    let Some(stringname) = stringname.strip_suffix("\"") else {
                        todo!()
                    };
                    if stringname
                        .find(|c: char| !c.is_ascii_alphanumeric() && !c.is_whitespace())
                        .is_some()
                    {
                        // Being conservative here because we don't yet know how
                        // escaping would work.
                        err_continue!(PropdefError::PropdefUnexpectedFieldValue {
                            field_name: "values".to_owned(),
                            value: stringname.to_owned(),
                        });
                    }
                    StringEnumVariant::Literal {
                        value: stringname.to_owned(),
                    }
                } else if let Some(&cname) = stringname_to_cname.get(stringname) {
                    StringEnumVariant::Defined {
                        cname: cname.to_owned(),
                    }
                } else {
                    if stringname
                        .find(|c: char| c.is_whitespace() || c == '|')
                        .is_some()
                    {
                        // must be a new syntax.
                        err_continue!(PropdefError::PropdefUnexpectedFieldValue {
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
                values.push(variant);
            }
        } else if let Some(value) = line.strip_prefix("dimension: ") {
            if dimension.is_some() {
                err_continue!(PropdefError::PropdefUnexpectedFieldValue {
                    field_name: "dimension".to_owned(),
                    value: value.to_owned(),
                });
            }

            let value = value.trim();
            if value == "N" {
                dimension = Some(PropdefDimension::Dynamic);
            } else if let Ok(n) = value.parse::<usize>() {
                dimension = Some(PropdefDimension::Fixed { size: n });
            } else {
                err_continue!(PropdefError::PropdefUnexpectedFieldValue {
                    field_name: "dimension".to_owned(),
                    value: value.to_owned(),
                });
            }
        } else if let Some(value) = line.strip_prefix("introduced: ") {
            if introduced.is_some() {
                err_continue!(PropdefError::PropdefUnexpectedFieldValue {
                    field_name: "introduced".to_owned(),
                    value: value.to_owned(),
                });
            }

            let value = value.trim();
            introduced = Some(value.to_owned());
        } else if let Some(value) = line.strip_prefix("added: ") {
            tracing::warn!(
                "parse_propdef: {cname}:found `added: `, will treat it as `introduced: `."
            );
            if introduced.is_some() {
                err_continue!(PropdefError::PropdefUnexpectedFieldValue {
                    field_name: "introduced".to_owned(),
                    value: value.to_owned(),
                });
            }

            let value = value.trim();
            introduced = Some(value.to_owned());
        } else if let Some(value) = line.strip_prefix("deprecated: ") {
            if deprecated.is_some() {
                err_continue!(PropdefError::PropdefUnexpectedFieldValue {
                    field_name: "deprecated".to_owned(),
                    value: value.to_owned(),
                });
            }

            let value = value.trim();
            deprecated = Some(value.to_owned());
        } else if line == "hostOptional: true" {
            if host_optional {
                err_continue!(PropdefError::PropdefUnexpectedFieldValue {
                    field_name: "hostOptional".to_owned(),
                    value: "true".to_owned(),
                });
            }
            host_optional = true;
        } else if line == "optional: true" {
            if optional {
                err_continue!(PropdefError::PropdefUnexpectedFieldValue {
                    field_name: "optional".to_owned(),
                    value: "true".to_owned(),
                });
            }
            optional = true;
        } else if let Some(value) = line.strip_prefix("cname: ") {
            tracing::warn!("parse_propdef: {cname}: found redundant `cname`: {value}");

            if prop_cname.is_some() {
                err_continue!(PropdefError::PropdefUnexpectedFieldValue {
                    field_name: "cname".to_owned(),
                    value: value.to_owned(),
                });
            }

            let value = value.trim();
            prop_cname = Some(value.to_owned());
        } else {
            tracing::warn!(
                "parse_propdef: {cname}: unrecognized line that will be ignored: {line}"
            );
        }
    }

    if r#type.is_none() || dimension.is_none() {
        err_return!(PropdefError::PropdefIncomplete {
            missing_type: r#type.is_none(),
            missing_dimension: dimension.is_none(),
        });
    }
    let mut r#type = r#type.unwrap();
    let dimension = dimension.unwrap();

    match &mut r#type {
        PropdefType::Simple { .. } => {
            if !values.is_empty() {
                err_return!(PropdefError::PropdefNonEnumWithValuesField);
            }
        }
        PropdefType::StringEnum { one_of } => {
            if values.is_empty() {
                err_return!(PropdefError::PropdefEnumWithoutValuesField);
            }
            debug_assert!(one_of.is_empty());
            *one_of = values;
        }
    }

    Some(PropdefMetadataEntry {
        r#type,
        dimension,
        introduced,
        deprecated,
        host_optional,
        optional,
        cname: prop_cname,
    })
}

fn parse_propset(
    name: &str,
    mut lines: Peekable<SignificantLines>,
    error: &mut Error,
    stringname_to_cname: &HashMap<&str, &str>,
) -> Option<PropsetMetadataEntry> {
    macro_rules! err_continue {
        ($err:expr) => {
            error.propset_errors.push((name.to_owned(), $err));
            continue;
        };
    }
    macro_rules! err_return {
        ($err:expr) => {
            error.propset_errors.push((name.to_owned(), $err));
            return None;
        };
    }

    let mut write: Option<WriteSide> = None;
    let mut props: BTreeMap<String, PropValue> = BTreeMap::new();
    let mut props_refs: BTreeSet<String> = BTreeSet::new();

    while let Some(line) = lines.next() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        } else if let Some(value) = line.strip_prefix("write: ") {
            if write.is_some() {
                err_continue!(PropsetError::PropsetDuplicateField {
                    field_name: "write".to_owned(),
                });
            }
            write = match WriteSide::try_from(value) {
                Some(value) => Some(value),
                None => {
                    err_continue!(PropsetError::PropsetUnexpectedFieldValue {
                        field_name: "write".to_owned(),
                        value: value.to_owned(),
                    });
                }
            };
        } else if line == "props:" {
            while let Some(next_line) = lines.peek()
                && let Some(item) = next_line.trim_start().strip_prefix("- ")
            {
                lines.next();
                let item = item.trim();
                let (stringname, opts) = parse_prop_item(item);

                if let Some(props_ref_name) = stringname.strip_suffix("_REF") {
                    if !opts.is_empty() {
                        err_continue!(PropsetError::PropsetPropsRefWithOptions {
                            props_ref_cname: props_ref_name.to_owned(),
                        });
                    }

                    props_refs.insert(props_ref_name.to_owned());
                    continue;
                }

                let Some(&prop_cname) = stringname_to_cname.get(stringname) else {
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
        } else {
            tracing::warn!("parse_propset: {name}: unrecognized line that will be ignored: {line}");
        }
    }

    let Some(write) = write else {
        err_return!(PropsetError::PropsetIncomplete {
            missing_write: true,
        });
    };

    Option::Some(PropsetMetadataEntry {
        write,
        props,
        props_refs,
    })
}

fn parse_propsetdef(
    name: &str,
    lines: Peekable<SignificantLines>,
    error: &mut Error,
    stringname_to_cname: &HashMap<&str, &str>,
) -> Option<PropsetdefMetadataEntry> {
    macro_rules! err_continue {
        ($err:expr) => {
            error.propsetdef_errors.push((name.to_owned(), $err));
            continue;
        };
    }

    let mut props: BTreeMap<String, PropValue> = BTreeMap::new();

    for line in lines {
        let line = line.trim();

        if line.is_empty() {
            continue;
        } else if let Some(item) = line.trim_start().strip_prefix("- ") {
            let item = item.trim();
            let (stringname, opts) = parse_prop_item(item);

            let Some(&prop_cname) = stringname_to_cname.get(stringname) else {
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
        } else {
            tracing::warn!(
                "parse_propsetdef: {name}: unrecognized line that will be ignored: {line}"
            );
        }
    }

    Some(PropsetdefMetadataEntry { props })
}

fn parse_actiondef(
    cname: &str,
    mut lines: Peekable<SignificantLines>,
    error: &mut Error,
    stringname_to_cname: &HashMap<&str, &str>,
) -> Option<ActiondefMetadataEntry> {
    macro_rules! err_continue {
        ($err:expr) => {
            error.actiondef_errors.push((cname.to_owned(), $err));
            continue;
        };
    }

    let mut in_args: BTreeSet<String> = BTreeSet::new();
    let mut out_args: BTreeSet<String> = BTreeSet::new();

    while let Some(line) = lines.next() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        } else if line == "inArgs:" {
            while let Some(next_line) = lines.peek()
                && let Some(item) = next_line.trim_start().strip_prefix("- ")
            {
                lines.next();
                let item = item.trim();
                let (stringname, opts) = parse_prop_item(item);

                let Some(&prop_cname) = stringname_to_cname.get(stringname) else {
                    err_continue!(ActiondefError::ActiondefUndefinedInArg {
                        stringname: stringname.to_owned(),
                    });
                };

                if !opts.is_empty() {
                    err_continue!(ActiondefError::ActiondefInArgWithOptions {
                        in_arg_cname: prop_cname.to_owned(),
                    });
                }

                in_args.insert(prop_cname.to_owned());
            }
        } else if line == "outArgs:" {
            while let Some(next_line) = lines.peek()
                && let Some(item) = next_line.trim_start().strip_prefix("- ")
            {
                lines.next();
                let item = item.trim();
                let (stringname, opts) = parse_prop_item(item);

                let Some(&prop_cname) = stringname_to_cname.get(stringname) else {
                    err_continue!(ActiondefError::ActiondefUndefinedOutArg {
                        stringname: stringname.to_owned(),
                    });
                };

                if !opts.is_empty() {
                    err_continue!(ActiondefError::ActiondefOutArgWithOptions {
                        out_arg_cname: prop_cname.to_owned(),
                    });
                }

                out_args.insert(prop_cname.to_owned());
            }
        } else if line == "outArgs: []" {
            tracing::warn!(
                "parse_actiondef: Remove the `outArgs: []` branch after we update `vendor/openfx` to a commit that no longer includes this any more."
            )
        } else {
            tracing::warn!(
                "parse_actiondef: {cname}: unrecognized line that will be ignored: {line}"
            );
        }
    }

    Some(ActiondefMetadataEntry { in_args, out_args })
}

fn extract_stringname_to_cname(
    input: &BTreeMap<String, BindingsUnprocessed>,
) -> HashMap<&str, &str> {
    let mut stringname_to_cname: HashMap<&str, &str> = HashMap::new();

    for (file_name, bindings) in input {
        if file_name.starts_with("ofx-") {
            // Skip the default colospace header.
            continue;
        }

        for item in &bindings.items {
            let RootItemWithCommentAbove::Item { item, .. } = item else {
                continue;
            };
            let RootItem::Define { name, value, .. } = item else {
                continue;
            };
            // NOTE: `DefineValue::Identifier` is intentionally skipped, because
            // we only want the direct cnames that correspond to the
            // stringnames. (We don't want a stringname to map to multiple
            // cnames.)
            let DefineValue::StringLiteral { value } = value else {
                continue;
            };
            match stringname_to_cname.entry(value.as_str()) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(name.as_str());
                }
                std::collections::hash_map::Entry::Occupied(_) => {
                    todo!(
                        "Handle this if this happens in the future: {}",
                        value.as_str()
                    )
                }
            }
        }
    }

    stringname_to_cname
}

struct ExtractedComment {
    cname: Option<String>,
    content: String,
}

struct ExtractedCommentIterator<'a> {
    bindings_iter: std::collections::btree_map::Values<'a, String, BindingsUnprocessed>,
    item_iter: Option<std::slice::Iter<'a, RootItemWithCommentAbove>>,
}

impl<'a> ExtractedCommentIterator<'a> {
    pub fn new(bindings: &'a BTreeMap<String, BindingsUnprocessed>) -> Self {
        Self {
            bindings_iter: bindings.values(),
            item_iter: None,
        }
    }
}

impl<'a> Iterator for ExtractedCommentIterator<'a> {
    type Item = ExtractedComment;

    /// Author: Chat in VS Code / DeepSeek V4.1 Flash (provider: DeepSeek, thinking: high) | Reviewed-by & Modified-by: Umaĵo
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(item_iter) = self.item_iter.as_mut()
                && let Some(item) = item_iter.next()
            {
                match item {
                    RootItemWithCommentAbove::Item {
                        comment_above,
                        item,
                    } => {
                        // No comment above this item → nothing to yield.
                        let Some(comment) = comment_above else {
                            continue;
                        };
                        let cname = match item.name() {
                            // The official repo's scripts [handle this as a
                            // special case], so we do the same here.
                            // [handle this as a special case]: https://github.com/AcademySoftwareFoundation/openfx/blob/e40728885390ec16276d11e00025de9b4282060c/scripts/ofx_prop_utils.py#L416
                            //
                            // TODO: submit a PR to the official repo to make
                            // this more elegant (e.g. using
                            // `@actiondef CustomParamInterpFunc` to designate
                            // the name).
                            "OfxCustomParamInterpFuncV1" => "CustomParamInterpFunc",
                            cname => cname,
                        };

                        return Some(ExtractedComment {
                            cname: Some(cname.to_string()),
                            content: comment.clone(),
                        });
                    }
                    RootItemWithCommentAbove::StandaloneComment { comment } => {
                        return Some(ExtractedComment {
                            cname: None,
                            content: comment.clone(),
                        });
                    }
                }
            }

            // Current item iterator is absent or exhausted; move to the next
            // binding's items, or end the iterator when bindings run out.
            let bindings = self.bindings_iter.next()?;
            self.item_iter = Some(bindings.items.iter());
        }
    }
}

enum MetadataSection<'a> {
    Propdef(SignificantLines<'a>),
    Propset(&'a str, SignificantLines<'a>),
    Propsetdef(&'a str, SignificantLines<'a>),
    Actiondef(SignificantLines<'a>),
}

fn extract_metadata_section<'a>(
    comment: &'a str,
    err: &mut Error,
    cname: Option<&str>,
) -> Option<MetadataSection<'a>> {
    let mut lines = comment.lines();

    while let Some(line) = lines.next() {
        let Some((word, rest)) = strip_prefix_word(line.trim_start()) else {
            continue;
        };

        match word {
            "@propdef" => {
                let rest = rest.trim();
                if rest.is_empty() {
                    return Some(MetadataSection::Propdef(SignificantLines::new(lines)));
                } else {
                    err.propdef_errors.push((
                        cname.unwrap_or("?").to_owned(),
                        PropdefError::PropdefWithName {
                            name: rest.to_owned(),
                        },
                    ));
                    return None;
                }
            }
            "@propset" => {
                let rest = rest.trim();
                if rest.is_empty() {
                    err.propset_without_name_count += 1;
                    return None;
                } else {
                    return Some(MetadataSection::Propset(rest, SignificantLines::new(lines)));
                }
            }
            "@propsetdef" => {
                let rest = rest.trim();
                if rest.is_empty() {
                    err.propsetdef_without_name_count += 1;
                    return None;
                } else {
                    return Some(MetadataSection::Propsetdef(
                        rest,
                        SignificantLines::new(lines),
                    ));
                }
            }
            "@actiondef" => {
                let rest = rest.trim();
                if rest.is_empty() {
                    return Some(MetadataSection::Actiondef(SignificantLines::new(lines)));
                } else {
                    err.actiondef_errors.push((
                        cname.unwrap_or("?").to_owned(),
                        ActiondefError::ActiondefWithName {
                            name: rest.to_owned(),
                        },
                    ));
                    return None;
                }
            }
            _ => {}
        }
    }

    None
}

/// Author: GitHub Copilot's tab completion | Reviewed-by: Umaĵo
fn strip_prefix_word(s: &str) -> Option<(&str, &str)> {
    let mut parts = s.splitn(2, char::is_whitespace);
    let first = parts.next()?;
    let rest = parts.next().unwrap_or("").trim_start();
    Some((first, rest))
}

fn parse_prop_item(prop_item: &str) -> (&str, HashMap<&str, &str>) {
    let (stringname, rest) = prop_item.split_once("|").unwrap_or((prop_item, ""));
    let stringname = stringname.trim();
    if rest.is_empty() {
        return (stringname, HashMap::new());
    }

    let mut opts: HashMap<&str, &str> = HashMap::new();

    if rest.matches("=").count() > 1 {
        todo!("parse_prop_item: Support multiple options.")
    }

    let (key, value) = rest.split_once("=").unwrap_or((rest, ""));
    opts.insert(key.trim(), value.trim());

    (stringname, opts)
}
