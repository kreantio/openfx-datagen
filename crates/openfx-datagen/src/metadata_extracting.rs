use std::collections::{BTreeMap, HashMap, HashSet};

use crate::parsing::{BindingsUnprocessed, DefineValue, RootItem, RootItemWithCommentAbove};

#[derive(Debug, snafu::Snafu, Default)]
pub struct Error {
    propdef_not_on_item_count: usize,
    propdef_with_name: HashSet<String>,

    propset_on_item: HashSet<String>,
    propset_without_name_count: usize,

    propsetdef_on_item: HashSet<String>,
    propsetdef_without_name_count: usize,

    actiondef_not_on_item_count: usize,
    actiondef_with_name: HashSet<String>,
}

impl Error {
    pub fn is_empty(&self) -> bool {
        self.propdef_not_on_item_count == 0
            && self.propdef_with_name.is_empty()
            && self.propset_on_item.is_empty()
            && self.propset_without_name_count == 0
            && self.propsetdef_on_item.is_empty()
            && self.propsetdef_without_name_count == 0
            && self.actiondef_not_on_item_count == 0
            && self.actiondef_with_name.is_empty()
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Metadata {
    propdef_map: BTreeMap<String, Todo>,
    propset_map: BTreeMap<String, Todo>,
    propsetdef_map: BTreeMap<String, Todo>,
    actiondef_map: BTreeMap<String, Todo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum Todo {
    TODO { content: String },
}

pub fn extract_metadata(input: &BTreeMap<String, BindingsUnprocessed>) -> Result<Metadata, Error> {
    let _stringname_to_cname = extract_stringname_to_cname(input);
    let comment_iter = ExtractedCommentIterator::new(input);

    let mut error = Error::default();
    let mut metadata = Metadata::default();

    for ExtractedComment { cname, content } in comment_iter {
        let Some(section) = extract_metadata_section(&content, &mut error) else {
            continue;
        };
        match section {
            MetadataSection::Propdef(lines) => {
                let Some(cname) = cname else {
                    error.propdef_not_on_item_count += 1;
                    continue;
                };
                metadata.propdef_map.insert(
                    cname.to_string(),
                    Todo::TODO {
                        content: lines.collect::<Vec<_>>().join("\n"),
                    },
                );
            }
            MetadataSection::Propset(name, lines) => {
                if let Some(cname) = cname {
                    error.propset_on_item.insert(cname.to_string());
                    continue;
                }
                metadata.propset_map.insert(
                    name.to_string(),
                    Todo::TODO {
                        content: lines.collect::<Vec<_>>().join("\n"),
                    },
                );
            }
            MetadataSection::Propsetdef(name, lines) => {
                if let Some(cname) = cname {
                    error.propsetdef_on_item.insert(cname.to_string());
                    continue;
                }
                metadata.propsetdef_map.insert(
                    name.to_string(),
                    Todo::TODO {
                        content: lines.collect::<Vec<_>>().join("\n"),
                    },
                );
            }
            MetadataSection::Actiondef(lines) => {
                let Some(cname) = cname else {
                    error.actiondef_not_on_item_count += 1;
                    continue;
                };
                metadata.actiondef_map.insert(
                    cname.to_string(),
                    Todo::TODO {
                        content: lines.collect::<Vec<_>>().join("\n"),
                    },
                );
            }
        }
    }

    if !error.is_empty() {
        return Err(error);
    }

    Ok(metadata)
}

pub fn extract_stringname_to_cname(
    input: &BTreeMap<String, BindingsUnprocessed>,
) -> HashMap<&str, &str> {
    let mut stringname_to_cname: HashMap<&str, &str> = HashMap::new();

    for bindings in input.values() {
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
            match stringname_to_cname.entry(name.as_str()) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(value.as_str());
                }
                std::collections::hash_map::Entry::Occupied(_) => {
                    todo!("Handle this if this happens in the future.")
                }
            }
        }
    }

    stringname_to_cname
}

pub struct ExtractedComment {
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
    Propdef(std::str::Lines<'a>),
    Propset(&'a str, std::str::Lines<'a>),
    Propsetdef(&'a str, std::str::Lines<'a>),
    Actiondef(std::str::Lines<'a>),
}

fn extract_metadata_section<'a>(comment: &'a str, err: &mut Error) -> Option<MetadataSection<'a>> {
    let mut lines = comment.lines();

    while let Some(line) = lines.next() {
        let Some((word, rest)) = strip_prefix_word(line.trim_start()) else {
            continue;
        };

        match word {
            "@propdef" => {
                let rest = rest.trim();
                if rest.is_empty() {
                    return Some(MetadataSection::Propdef(lines));
                } else {
                    err.propdef_with_name.insert(rest.to_string());
                    return None;
                }
            }
            "@propset" => {
                let rest = rest.trim();
                if rest.is_empty() {
                    err.propset_without_name_count += 1;
                    return None;
                } else {
                    return Some(MetadataSection::Propset(rest, lines));
                }
            }
            "@propsetdef" => {
                let rest = rest.trim();
                if rest.is_empty() {
                    err.propsetdef_without_name_count += 1;
                    return None;
                } else {
                    return Some(MetadataSection::Propsetdef(rest, lines));
                }
            }
            "@actiondef" => {
                let rest = rest.trim();
                if rest.is_empty() {
                    return Some(MetadataSection::Actiondef(lines));
                } else {
                    err.actiondef_with_name.insert(rest.to_string());
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
