use std::collections::{BTreeMap, HashMap};

use crate::{
    metadata_extracting::types_raw::{
        RawActiondefMetadataEntry, RawPropdefMetadataEntry, RawPropsetMetadataEntry,
        RawPropsetdefMetadataEntry,
    },
    parsing::{BindingsUnprocessed, DefineValue, RootItem, RootItemWithCommentAbove},
    utils::{LineEx, LinesEx},
};

pub use crate::metadata_extracting::{errors::*, types::*};

mod errors;
mod types;
mod types_raw;

#[cfg(test)]
mod tests;

pub fn extract_metadata(
    input: &BTreeMap<String, BindingsUnprocessed>,
) -> Result<Metadata, Box<Error>> {
    let stringname_to_cname = extract_stringname_to_cname(input);
    let comment_iter = ExtractedCommentIterator::new(input);

    let mut metadata = Metadata::default();

    let mut ctx = FactoryContext {
        stringname_to_cname,
        error: Error::default(),
    };

    for ExtractedComment { cname, content } in comment_iter {
        let Some(section) = extract_metadata_section(&content, &mut ctx.error, cname.as_deref())
        else {
            continue;
        };
        match section {
            MetadataSection::Propdef(raw_entry) => {
                let Some(cname) = cname else {
                    ctx.error.propdef_not_on_item_count += 1;
                    continue;
                };

                if let Some(entry) = PropdefMetadataEntry::try_from_raw(&mut ctx, &cname, raw_entry)
                {
                    metadata.propdef_map.insert(cname.to_string(), entry);
                }
            }
            MetadataSection::Propset(name, entry) => {
                if let Some(cname) = cname {
                    ctx.error.propset_errors.push((
                        name.to_string(),
                        PropsetError::PropsetOnItem {
                            item_cname: cname.to_owned(),
                        },
                    ));
                    continue;
                }

                if let Some(entry) = PropsetMetadataEntry::try_from_raw(&mut ctx, name, entry) {
                    metadata.propset_map.insert(name.to_string(), entry);
                }
            }
            MetadataSection::Propsetdef(name, entry) => {
                if let Some(cname) = cname {
                    ctx.error.propsetdef_errors.push((
                        name.to_string(),
                        PropsetdefError::PropsetdefOnItem {
                            item_cname: cname.to_owned(),
                        },
                    ));
                    continue;
                }

                if let Some(entry) = PropsetdefMetadataEntry::try_from_raw(&mut ctx, name, entry) {
                    metadata.propsetdef_map.insert(name.to_string(), entry);
                }
            }
            MetadataSection::Actiondef(entry) => {
                let Some(cname) = cname else {
                    ctx.error.actiondef_not_on_item_count += 1;
                    continue;
                };

                if let Some(entry) = ActiondefMetadataEntry::try_from_raw(&mut ctx, &cname, entry) {
                    metadata.actiondef_map.insert(cname.to_string(), entry);
                }
            }
        }
    }

    if !ctx.error.is_empty() {
        return Err(Box::new(ctx.error));
    }

    Ok(metadata)
}

fn extract_stringname_to_cname(
    input: &BTreeMap<String, BindingsUnprocessed>,
) -> HashMap<&str, &str> {
    let mut stringname_to_cname: HashMap<&str, &str> = HashMap::new();

    for (file_name, bindings) in input {
        if file_name.starts_with("ofx-") {
            // Skip the default colorspace header.
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
    Propdef(RawPropdefMetadataEntry),
    Propset(&'a str, RawPropsetMetadataEntry),
    Propsetdef(&'a str, RawPropsetdefMetadataEntry),
    Actiondef(RawActiondefMetadataEntry),
}

fn extract_metadata_section<'a>(
    comment: &'a str,
    err: &mut Error,
    cname: Option<&str>,
) -> Option<MetadataSection<'a>> {
    let mut lines = LinesEx::new(comment);

    while let Some(LineEx { content: line, .. }) = lines.next() {
        let Some((word, rest)) = strip_prefix_word(line.trim_start()) else {
            continue;
        };

        match word {
            "@propdef" => {
                let rest = rest.trim();
                if rest.is_empty() {
                    let start_offset = lines
                        .next()
                        .map_or(comment.len(), |line_ex| line_ex.start_offset);
                    match serde_saphyr::from_str::<RawPropdefMetadataEntry>(
                        &comment[start_offset..],
                    ) {
                        Ok(entry) => return Some(MetadataSection::Propdef(entry)),
                        Err(parse_err) => {
                            err.propdef_errors.push((
                                cname.unwrap_or("?").to_owned(),
                                PropdefError::PropdefSerdeSaphyrParse { error: parse_err },
                            ));
                            return None;
                        }
                    }
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
                    let start_offset = lines
                        .next()
                        .map_or(comment.len(), |line_ex| line_ex.start_offset);
                    match serde_saphyr::from_str::<RawPropsetMetadataEntry>(
                        &comment[start_offset..],
                    ) {
                        Ok(entry) => return Some(MetadataSection::Propset(rest, entry)),
                        Err(parse_err) => {
                            err.propset_errors.push((
                                cname.unwrap_or("?").to_owned(),
                                PropsetError::PropsetSerdeSaphyrParse { error: parse_err },
                            ));
                            return None;
                        }
                    }
                }
            }
            "@propsetdef" => {
                let rest = rest.trim();
                if rest.is_empty() {
                    err.propsetdef_without_name_count += 1;
                    return None;
                } else {
                    let start_offset = lines
                        .next()
                        .map_or(comment.len(), |line_ex| line_ex.start_offset);
                    match serde_saphyr::from_str::<RawPropsetdefMetadataEntry>(
                        &comment[start_offset..],
                    ) {
                        Ok(entry) => return Some(MetadataSection::Propsetdef(rest, entry)),
                        Err(parse_err) => {
                            err.propsetdef_errors.push((
                                cname.unwrap_or("?").to_owned(),
                                PropsetdefError::PropsetdefSerdeSaphyrParse { error: parse_err },
                            ));
                            return None;
                        }
                    }
                }
            }
            "@actiondef" => {
                let rest = rest.trim();
                if rest.is_empty() {
                    let start_offset = lines
                        .next()
                        .map_or(comment.len(), |line_ex| line_ex.start_offset);
                    match serde_saphyr::from_str::<RawActiondefMetadataEntry>(
                        &comment[start_offset..],
                    ) {
                        Ok(entry) => return Some(MetadataSection::Actiondef(entry)),
                        Err(parse_err) => {
                            err.actiondef_errors.push((
                                cname.unwrap_or("?").to_owned(),
                                ActiondefError::ActiondefSerdeSaphyrParse { error: parse_err },
                            ));
                            return None;
                        }
                    }
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
