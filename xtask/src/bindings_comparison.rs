use std::{
    collections::{BTreeMap, BTreeSet, HashMap, hash_map},
    hash::Hash,
    path::Path,
};

use serde::Deserialize as _;

use crate::CompareGeneratedBindings;

#[derive(serde::Deserialize)]
struct Config {
    preprocessing: ConfigPreprocessing,
}

#[derive(serde::Deserialize)]
struct ConfigPreprocessing {
    reference_bindings: ConfigPreprocessingReferenceBindings,
}

#[derive(serde::Deserialize)]
struct ConfigPreprocessingReferenceBindings {
    rename_const: Vec<ConfigPreprocessingRename>,
}

#[derive(serde::Deserialize)]
struct ConfigPreprocessingRename {
    #[serde(deserialize_with = "deserialize_regex")]
    pattern: regress::Regex,
    replacement: String,
}

fn deserialize_regex<'de, D>(deserializer: D) -> Result<regress::Regex, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    let re = regress::Regex::new(&text).map_err(serde::de::Error::custom)?;

    Ok(re)
}

pub fn compare_generated_bindings(
    cmd: CompareGeneratedBindings,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = std::fs::read_to_string(&cmd.config)?;
    let config: Config = toml::from_str(&config)?;

    let mut has_problems = false;

    let our_bindings = load_syn_files_folder(&cmd.openfx_bindgen_bindings_folder)?;
    let our_bindings = OurBindings::from_syn_files(our_bindings);

    our_bindings.report(&mut has_problems);

    let ref_bindings = load_syn_file(&cmd.reference_bindings_file)?;
    let ref_bindings = ReferenceBindings::from_syn_file(&config, ref_bindings);

    ref_bindings.report(&mut has_problems);

    return Err("TODO".into());

    #[allow(unreachable_code)]
    if has_problems {
        Err("Some problems were found. (See logs.)".into())
    } else {
        Ok(())
    }
}

fn load_syn_files_folder(
    folder_path: &Path,
) -> Result<HashMap<String, syn::File>, Box<dyn std::error::Error>> {
    let entries = std::fs::read_dir(folder_path)?;
    let mut files = HashMap::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs")
        {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap()
            .to_string();
        files.insert(file_name, load_syn_file(&path)?);
    }
    Ok(files)
}

fn load_syn_file(file_path: &Path) -> Result<syn::File, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let syn_file = syn::parse_file(&content)?;
    Ok(syn_file)
}

#[derive(Default)]
struct OurBindings {
    consts: HashMap<String, syn::ItemConst>,
    structs: HashMap<String, syn::ItemStruct>,
    types: HashMap<String, syn::ItemType>,

    problems: OurBindingsProblems,
}

#[derive(Default)]
pub struct OurBindingsProblems {
    unaddressed_items: Vec<syn::Item>,
    unexpected_type_names: BTreeSet<String>,
    unexpected_value_names: BTreeSet<String>,
    duplicate_names: BTreeSet<String>,
}

impl OurBindings {
    fn from_syn_files(syn_files: HashMap<String, syn::File>) -> Self {
        let mut every_items: Vec<syn::Item> = vec![];
        for (_, syn_file) in syn_files {
            every_items.extend(syn_file.items);
        }

        let every_items = remove_docs(&every_items);

        let mut ret = Self::default();
        let mut expected_consts: Vec<syn::ItemConst> = vec![];
        let mut expected_structs: Vec<syn::ItemStruct> = vec![];
        let mut expected_types: Vec<syn::ItemType> = vec![];

        for item in every_items {
            match item {
                syn::Item::Const(item_const) => {
                    let name = item_const.ident.to_string();
                    if name.starts_with("kOfx") {
                        expected_consts.push(item_const)
                    } else {
                        ret.problems.unexpected_value_names.insert(name);
                    }
                }
                syn::Item::Struct(item_struct) => {
                    let name = item_struct.ident.to_string();
                    if name.starts_with("Ofx") {
                        expected_structs.push(item_struct)
                    } else {
                        ret.problems.unexpected_type_names.insert(name);
                    }
                }
                syn::Item::Type(item_type) => {
                    let name = item_type.ident.to_string();
                    if name.starts_with("Ofx") {
                        expected_types.push(item_type)
                    } else {
                        ret.problems.unexpected_type_names.insert(name);
                    }
                }
                syn::Item::Use(_item_use) => {}
                _ => ret.problems.unaddressed_items.push(item),
            }
        }

        ret.consts =
            Self::vec_to_hashmap(expected_consts, &mut ret.problems, |c| c.ident.to_string());
        ret.structs =
            Self::vec_to_hashmap(expected_structs, &mut ret.problems, |s| s.ident.to_string());
        ret.types =
            Self::vec_to_hashmap(expected_types, &mut ret.problems, |t| t.ident.to_string());

        ret
    }

    fn vec_to_hashmap<T: Hash + Eq>(
        vec: Vec<T>,
        problems: &mut OurBindingsProblems,
        to_string: impl Fn(&T) -> String,
    ) -> HashMap<String, T> {
        let mut hashmap = HashMap::new();
        for item in vec {
            match hashmap.entry(to_string(&item)) {
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(item);
                }
                hash_map::Entry::Occupied(entry) => {
                    problems.duplicate_names.insert(entry.key().clone());
                }
            }
        }
        hashmap
    }

    fn report(&self, has_problems: &mut bool) {
        tracing::info!(
            "our bindings: Found {} expected `const`s.",
            self.consts.len(),
        );
        tracing::info!(
            "our bindings: Found {} expected `struct`s.",
            self.structs.len(),
        );
        tracing::info!("our bindings: Found {} expected `type`s.", self.types.len(),);

        self.problems.report_problems_if_any(has_problems);
    }
}

impl OurBindingsProblems {
    fn is_empty(&self) -> bool {
        self.unaddressed_items.is_empty()
            && self.unexpected_type_names.is_empty()
            && self.unexpected_value_names.is_empty()
    }

    fn report_problems_if_any(&self, has_problems: &mut bool) {
        if self.is_empty() {
            return;
        }

        *has_problems = true;

        let mut unaddressed_items_names: Vec<String> = vec![];
        let mut unnamed_unaddressed_items: Vec<syn::Item> = vec![];

        for item in &self.unaddressed_items {
            if let Some(name) = get_item_name(item) {
                unaddressed_items_names.push(name);
            } else {
                unnamed_unaddressed_items.push(item.clone());
            }
        }

        if !unaddressed_items_names.is_empty() {
            tracing::error!(
                "our bindings: Unaddressed items with names: {}",
                unaddressed_items_names.join(", ")
            );
        }
        for item in unnamed_unaddressed_items {
            tracing::error!("our bindings: Unaddressed item without a name: {:?}", item);
        }

        if !self.unexpected_type_names.is_empty() {
            tracing::error!(
                "our bindings: Unexpected type names: {}",
                self.unexpected_type_names
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        if !self.unexpected_value_names.is_empty() {
            tracing::error!(
                "our bindings: Unexpected value names: {}",
                self.unexpected_value_names
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
}

#[derive(Default)]
struct ReferenceBindings {
    consts: HashMap<String, syn::ItemConst>,
    structs: HashMap<String, syn::ItemStruct>,
    types: HashMap<String, syn::ItemType>,

    renamed_consts: BTreeMap<String, String>,
    ignored_const_count: usize,
    ignored_struct_count: usize,
    ignored_type_count: usize,
    ignored_other_count: usize,

    problems: ReferenceBindingsProblems,
}

#[derive(Default)]
struct ReferenceBindingsProblems {
    unexpected_const_names: BTreeSet<String>,
    unexpected_struct_names: BTreeSet<String>,
    unexpected_type_names: BTreeSet<String>,
    unexpected_other_names: BTreeSet<String>,
}

impl ReferenceBindings {
    fn from_syn_file(config: &Config, syn_file: syn::File) -> Self {
        let every_item = remove_docs(&syn_file.items);

        let mut ret = ReferenceBindings::default();

        for item in every_item {
            match item {
                syn::Item::Const(mut item_const) => {
                    let mut name = item_const.ident.to_string();
                    if let Some(new_name) = ret.rename_const(config, &name) {
                        name = new_name;
                        item_const.ident = syn::Ident::new(&name, item_const.ident.span());
                    }
                    if name.starts_with("kOfx") {
                        ret.consts.insert(name, item_const);
                    } else if name.starts_with("Ofx") {
                        ret.problems.unexpected_const_names.insert(name);
                    } else {
                        ret.ignored_const_count += 1;
                    }
                }
                syn::Item::Struct(item_struct) => {
                    let name = item_struct.ident.to_string();
                    if name.starts_with("Ofx") {
                        ret.structs.insert(name, item_struct);
                    } else if name.starts_with("kOfx") {
                        ret.problems.unexpected_struct_names.insert(name);
                    } else {
                        ret.ignored_struct_count += 1;
                    }
                }
                syn::Item::Type(item_type) => {
                    let name = item_type.ident.to_string();
                    if name.starts_with("Ofx") {
                        ret.types.insert(name, item_type);
                    } else if name.starts_with("kOfx") {
                        ret.problems.unexpected_type_names.insert(name);
                    } else {
                        ret.ignored_type_count += 1;
                    }
                }
                _ => {
                    if let Some(name) = get_item_name(&item)
                        && (name.starts_with("Ofx") || name.starts_with("kOfx"))
                    {
                        ret.problems.unexpected_other_names.insert(name);
                    } else {
                        ret.ignored_other_count += 1;
                    }
                }
            }
        }

        ret
    }

    fn report(&self, has_problems: &mut bool) {
        tracing::info!(
            "reference bindings: Found {} qualified `const`s. (ignored: {})",
            self.consts.len(),
            self.ignored_const_count
        );
        tracing::info!(
            "reference bindings: Found {} qualified `struct`s. (ignored: {})",
            self.structs.len(),
            self.ignored_struct_count
        );
        tracing::info!(
            "reference bindings: Found {} qualified `type`s. (ignored: {})",
            self.types.len(),
            self.ignored_type_count
        );
        tracing::info!(
            "reference bindings: ignored {} other items.",
            self.ignored_type_count
        );
        tracing::info!(
            "reference bindings: renamed `const`s (from => to): {:?}",
            self.renamed_consts
        );

        self.problems.report(has_problems);
    }

    fn rename_const(&mut self, config: &Config, name: &str) -> Option<String> {
        for item in &config.preprocessing.reference_bindings.rename_const {
            if item.pattern.find(name).is_some() {
                let replaced = item.pattern.replace(name, &item.replacement);
                self.renamed_consts
                    .insert(name.to_string(), replaced.clone());
                return Some(replaced);
            }
        }

        None
    }
}

impl ReferenceBindingsProblems {
    fn is_empty(&self) -> bool {
        self.unexpected_const_names.is_empty()
            && self.unexpected_struct_names.is_empty()
            && self.unexpected_type_names.is_empty()
            && self.unexpected_other_names.is_empty()
    }

    fn report(&self, has_problems: &mut bool) {
        if !self.is_empty() {
            *has_problems = true;
        }

        if !self.unexpected_const_names.is_empty() {
            tracing::error!(
                r#"reference bindings: Unexpected names for `const`s (they should not start with `"Ofx"`): {:?}"#,
                self.unexpected_const_names
            );
        }
        if !self.unexpected_struct_names.is_empty() {
            tracing::error!(
                r#"reference bindings: Unexpected names for `struct`s (they should not start with `"kOfx"`): {:?}"#,
                self.unexpected_struct_names
            );
        }
        if !self.unexpected_type_names.is_empty() {
            tracing::error!(
                r#"reference bindings: Unexpected names for `type`s (they should not start with `"kOfx"`): {:?}"#,
                self.unexpected_type_names
            );
        }
        if !self.unexpected_other_names.is_empty() {
            tracing::error!(
                r#"reference bindings: Unexpected names for other items (they should not start with `"Ofx" or "kOfx"`): {:?}"#,
                self.unexpected_other_names
            );
        }
    }
}

fn remove_docs(syn_file: &[syn::Item]) -> Vec<syn::Item> {
    let mut file = syn::File {
        shebang: None,
        frontmatter: None,
        attrs: vec![],
        items: syn_file.to_vec(),
    };

    struct Visitor;
    impl syn::visit_mut::VisitMut for Visitor {
        fn visit_attributes_mut(&mut self, i: &mut Vec<syn::Attribute>) {
            i.retain(|attr| !attr.path().is_ident("doc"));
        }
    }

    let mut visitor = Visitor;
    syn::visit_mut::visit_file_mut(&mut visitor, &mut file);

    file.items
}

fn get_item_name(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::ForeignMod(_)
        | syn::Item::Impl(_)
        | syn::Item::Use(_)
        | syn::Item::Verbatim(_) => None,
        syn::Item::Const(item_const) => Some(item_const.ident.to_string()),
        syn::Item::Enum(item_enum) => Some(item_enum.ident.to_string()),
        syn::Item::ExternCrate(item_extern_crate) => Some(item_extern_crate.ident.to_string()),
        syn::Item::Fn(item_fn) => Some(item_fn.sig.ident.to_string()),
        syn::Item::Macro(item_macro) => item_macro.ident.as_ref().map(|ident| ident.to_string()),
        syn::Item::Mod(item_mod) => Some(item_mod.ident.to_string()),
        syn::Item::Static(item_static) => Some(item_static.ident.to_string()),
        syn::Item::Struct(item_struct) => Some(item_struct.ident.to_string()),
        syn::Item::Trait(item_trait) => Some(item_trait.ident.to_string()),
        syn::Item::TraitAlias(item_trait_alias) => Some(item_trait_alias.ident.to_string()),
        syn::Item::Type(item_type) => Some(item_type.ident.to_string()),
        syn::Item::Union(item_union) => Some(item_union.ident.to_string()),
        _ => todo!(),
    }
}
