use std::{
    collections::{BTreeMap, BTreeSet, HashMap, hash_map},
    hash::Hash,
    path::Path,
};

use crate::{
    CompareGeneratedBindings,
    bindings_comparison::{
        const_comparison::ConstComparisonResult, struct_comparison::StructComparisonResult,
        type_comparison::TypeComparisonResult,
    },
};

mod config {
    use serde::Deserialize as _;

    #[derive(serde::Deserialize)]
    pub struct Config {
        pub preprocessing: ConfigPreprocessing,
        pub exceptions: ConfigExceptions,
    }

    #[derive(serde::Deserialize)]
    pub struct ConfigPreprocessing {
        pub reference_bindings: ConfigPreprocessingReferenceBindings,
    }

    #[derive(serde::Deserialize)]
    pub struct ConfigPreprocessingReferenceBindings {
        pub rename_const: Vec<ConfigPreprocessingRename>,
    }

    #[derive(serde::Deserialize)]
    pub struct ConfigPreprocessingRename {
        #[serde(deserialize_with = "deserialize_regex")]
        pub pattern: regress::Regex,
        pub replacement: String,
    }

    fn deserialize_regex<'de, D>(deserializer: D) -> Result<regress::Regex, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        let re = regress::Regex::new(&text).map_err(serde::de::Error::custom)?;

        Ok(re)
    }

    #[derive(serde::Deserialize)]
    pub struct ConfigExceptions {
        pub our_const_bools_equal_to_u32_in_reference: bool,
    }
}

pub use config::Config;

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

    TypeComparisonResult::compare_types(&our_bindings, &ref_bindings).report(&mut has_problems);
    StructComparisonResult::compare_structs(&our_bindings, &ref_bindings).report(&mut has_problems);
    ConstComparisonResult::compare_consts(&config, &our_bindings, &ref_bindings)
        .report(&mut has_problems);

    if has_problems {
        Err("Some problems were found. (See tracing logs.)".into())
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
        if !self.renamed_consts.is_empty() {
            tracing::info!(
                r#"reference bindings: renamed {} `const`s ("<real_name>": "<seen_as>"): {:?}"#,
                self.renamed_consts.len(),
                self.renamed_consts
            );
        }

        self.problems.report_problems_if_any(has_problems);
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

    fn report_problems_if_any(&self, has_problems: &mut bool) {
        if self.is_empty() {
            return;
        }

        *has_problems = true;

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

struct Difference {
    ours: String,
    reference: String,
}

mod type_comparison {
    use std::collections::{BTreeSet, HashSet};

    use super::{Difference, OurBindings, ReferenceBindings};

    #[derive(Default)]
    pub struct TypeComparisonResult {
        problems: TypeComparisonProblems,

        shared_name_count: usize,
        our_unique_name_count: BTreeSet<String>,
        same_type_count: usize,
    }

    #[derive(Default)]
    pub struct TypeComparisonProblems {
        reference_unique_names: BTreeSet<String>,
        different_types: Vec<(String, Difference)>,
    }

    impl TypeComparisonResult {
        pub fn compare_types(ours: &OurBindings, reference: &ReferenceBindings) -> Self {
            let mut result = TypeComparisonResult::default();

            let our_names: HashSet<_> = ours.types.keys().cloned().collect();
            let reference_names: HashSet<_> = reference.types.keys().cloned().collect();

            let shared_names: HashSet<_> =
                our_names.intersection(&reference_names).cloned().collect();
            result.our_unique_name_count = our_names.difference(&shared_names).cloned().collect();
            let reference_unique_names: BTreeSet<_> =
                reference_names.difference(&shared_names).cloned().collect();
            let shared_names: BTreeSet<_> = shared_names.into_iter().collect();

            result.shared_name_count = shared_names.len();

            if !reference_unique_names.is_empty() {
                tracing::info!(
                    "compare_types: {} `type`s are unique to the reference bindings (which means they are missing in our bindings, which is not OK).",
                    reference_unique_names.len()
                );
                result.problems.reference_unique_names = reference_unique_names;
            }

            for name in shared_names {
                let our_type = &ours.types[&name];
                let ref_type = &reference.types[&name];
                let ours = Self::stringify_type(our_type);
                let reference = Self::stringify_type(ref_type);

                // NOTE: comparing syn types directly might result in false negatives.
                if ours == reference {
                    result.same_type_count += 1;
                } else {
                    result
                        .problems
                        .different_types
                        .push((name.clone(), Difference { ours, reference }));
                }
            }

            result
        }

        pub fn report(&self, has_problems: &mut bool) {
            tracing::info!(
                "compare_types: {} `type`s shares the same name. ({} of them have the same definition.)",
                self.shared_name_count,
                self.same_type_count
            );
            if !self.our_unique_name_count.is_empty() {
                tracing::info!(
                    "compare_types: {} `type`s are unique to our bindings (which is OK). They are: {}",
                    self.our_unique_name_count.len(),
                    self.our_unique_name_count
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            self.problems.report_problems_if_any(has_problems);
        }

        fn stringify_type(ty: &syn::ItemType) -> String {
            prettyplease::unparse(&syn::File {
                shebang: None,
                frontmatter: None,
                attrs: vec![],
                items: vec![syn::Item::Type(ty.clone())],
            })
        }
    }

    impl TypeComparisonProblems {
        fn is_empty(&self) -> bool {
            self.reference_unique_names.is_empty() && self.different_types.is_empty()
        }

        fn report_problems_if_any(&self, has_problems: &mut bool) {
            if self.is_empty() {
                return;
            }

            *has_problems = true;

            if !self.reference_unique_names.is_empty() {
                tracing::error!(
                    "compare_types: `type`s that are missing in our bindings: {}",
                    self.reference_unique_names
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            for (name, diff) in &self.different_types {
                tracing::error!(
                    "compare_types: `type` `{}` differs between our bindings and the reference bindings: \n{}",
                    name,
                    &prettydiff::diff_lines(&diff.reference, &diff.ours,)
                )
            }

            if !self.different_types.is_empty() {
                tracing::error!("compare_types: different `type` definition ")
            }
        }
    }
}

mod struct_comparison {
    use std::collections::{BTreeSet, HashSet};

    use super::{Difference, OurBindings, ReferenceBindings};

    #[derive(Default)]
    pub struct StructComparisonResult {
        problems: StructComparisonProblems,

        shared_name_count: usize,
        our_unique_name_count: BTreeSet<String>,
        same_struct_count: usize,
    }

    #[derive(Default)]
    pub struct StructComparisonProblems {
        reference_unique_names: BTreeSet<String>,
        different_structs: Vec<(String, Difference)>,
    }

    impl StructComparisonResult {
        pub fn compare_structs(ours: &OurBindings, reference: &ReferenceBindings) -> Self {
            let mut result = StructComparisonResult::default();

            let our_names: HashSet<_> = ours.structs.keys().cloned().collect();
            let reference_names: HashSet<_> = reference.structs.keys().cloned().collect();

            let shared_names: HashSet<_> =
                our_names.intersection(&reference_names).cloned().collect();
            result.our_unique_name_count = our_names.difference(&shared_names).cloned().collect();
            let reference_unique_names: BTreeSet<_> =
                reference_names.difference(&shared_names).cloned().collect();
            let shared_names: BTreeSet<_> = shared_names.into_iter().collect();

            result.shared_name_count = shared_names.len();

            if !reference_unique_names.is_empty() {
                tracing::info!(
                    "compare_structs: {} `struct`s are unique to the reference bindings (which means they are missing in our bindings, which is not OK).",
                    reference_unique_names.len()
                );
                result.problems.reference_unique_names = reference_unique_names;
            }

            for name in shared_names {
                let our_struct = &ours.structs[&name];
                let ref_struct = &reference.structs[&name];
                let ours = Self::stringify_struct(our_struct);
                let reference = Self::stringify_struct(ref_struct);

                // NOTE: comparing syn types directly might result in false negatives.
                if ours == reference {
                    result.same_struct_count += 1;
                } else {
                    result
                        .problems
                        .different_structs
                        .push((name.clone(), Difference { ours, reference }));
                }
            }

            result
        }

        pub fn report(&self, has_problems: &mut bool) {
            tracing::info!(
                "compare_structs: {} `struct`s shares the same name. ({} of them have the same definition.)",
                self.shared_name_count,
                self.same_struct_count
            );
            if !self.our_unique_name_count.is_empty() {
                tracing::info!(
                    "compare_structs: {} `struct`s are unique to our bindings (which is OK). They are: {}",
                    self.our_unique_name_count.len(),
                    self.our_unique_name_count
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            self.problems.report_problems_if_any(has_problems);
        }

        fn stringify_struct(s: &syn::ItemStruct) -> String {
            prettyplease::unparse(&syn::File {
                shebang: None,
                frontmatter: None,
                attrs: vec![],
                items: vec![syn::Item::Struct(s.clone())],
            })
        }
    }

    impl StructComparisonProblems {
        fn is_empty(&self) -> bool {
            self.reference_unique_names.is_empty() && self.different_structs.is_empty()
        }

        fn report_problems_if_any(&self, has_problems: &mut bool) {
            if self.is_empty() {
                return;
            }

            *has_problems = true;

            if !self.reference_unique_names.is_empty() {
                tracing::error!(
                    "compare_structs: `struct`s that are missing in our bindings: {}",
                    self.reference_unique_names
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            for (name, diff) in &self.different_structs {
                tracing::error!(
                    "compare_structs: `struct` `{}` differs between our bindings and the reference bindings: \n{}",
                    name,
                    &prettydiff::diff_lines(&diff.reference, &diff.ours,)
                )
            }
        }
    }
}

mod const_comparison {
    use std::collections::{BTreeSet, HashSet};

    use quote::quote;

    use super::{Config, Difference, OurBindings, ReferenceBindings};

    #[derive(Default)]
    pub struct ConstComparisonResult {
        problems: ConstComparisonProblems,

        shared_name_count: usize,
        our_unique_name_count: BTreeSet<String>,
        same_const_count: usize,
        exceptional_const_count: usize,

        exception_our_const_bools_equal_to_u32_in_reference: BTreeSet<String>,
    }

    #[derive(Default)]
    pub struct ConstComparisonProblems {
        reference_unique_names: BTreeSet<String>,
        different_consts: Vec<(String, Difference)>,
    }

    impl ConstComparisonResult {
        pub fn compare_consts(
            config: &Config,
            ours: &OurBindings,
            reference: &ReferenceBindings,
        ) -> Self {
            let mut result = ConstComparisonResult::default();

            let our_names: HashSet<_> = ours.consts.keys().cloned().collect();
            let reference_names: HashSet<_> = reference.consts.keys().cloned().collect();

            let shared_names: HashSet<_> =
                our_names.intersection(&reference_names).cloned().collect();
            result.our_unique_name_count = our_names.difference(&shared_names).cloned().collect();
            let reference_unique_names: BTreeSet<_> =
                reference_names.difference(&shared_names).cloned().collect();
            let shared_names: BTreeSet<_> = shared_names.into_iter().collect();

            result.shared_name_count = shared_names.len();

            if !reference_unique_names.is_empty() {
                tracing::info!(
                    "compare_consts: {} `const`s are unique to the reference bindings (which means they are missing in our bindings, which is not OK).",
                    reference_unique_names.len()
                );
                result.problems.reference_unique_names = reference_unique_names;
            }

            for name in shared_names {
                let our_const = &ours.consts[&name];
                let ref_const = &reference.consts[&name];
                let ours = Self::stringify_const(our_const);
                let reference = Self::stringify_const(ref_const);

                // NOTE: comparing syn types directly might result in false negatives.
                if ours == reference {
                    result.same_const_count += 1;
                    continue;
                }

                if config.exceptions.our_const_bools_equal_to_u32_in_reference {
                    let our_ty = &our_const.ty;
                    if quote! { #our_ty }.to_string() == "bool" {
                        let our_expr = &our_const.expr;

                        let mut our_const = our_const.clone();
                        our_const.ty = syn::parse_str("u32").unwrap();
                        our_const.expr = match quote! { #our_expr }.to_string().as_str() {
                            "false" => syn::parse_str("0").unwrap(),
                            "true" => syn::parse_str("1").unwrap(),
                            _ => our_expr.clone(),
                        };

                        let ours = Self::stringify_const(&our_const);

                        if ours == reference {
                            result.exceptional_const_count += 1;
                            result
                                .exception_our_const_bools_equal_to_u32_in_reference
                                .insert(name.clone());
                            continue;
                        }
                    }
                }

                result
                    .problems
                    .different_consts
                    .push((name.clone(), Difference { ours, reference }));
            }

            result
        }

        pub fn report(&self, has_problems: &mut bool) {
            tracing::info!(
                "compare_consts: {} `const`s shares the same name. ({} of them have the same definition; {} of them are different but allowed exceptionally.)",
                self.shared_name_count,
                self.same_const_count,
                self.exceptional_const_count
            );
            if !self.our_unique_name_count.is_empty() {
                tracing::info!(
                    "compare_consts: {} `const`s are unique to our bindings (which is OK). They are: {}",
                    self.our_unique_name_count.len(),
                    self.our_unique_name_count
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            if !self
                .exception_our_const_bools_equal_to_u32_in_reference
                .is_empty()
            {
                tracing::info!(
                    "compare_consts: {} `const`s that differ from the reference are allowed under the exception `our_const_bools_equal_to_u32_in_reference`. They are: {}",
                    self.exception_our_const_bools_equal_to_u32_in_reference
                        .len(),
                    self.exception_our_const_bools_equal_to_u32_in_reference
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            self.problems.report_problems_if_any(has_problems);
        }

        fn stringify_const(s: &syn::ItemConst) -> String {
            prettyplease::unparse(&syn::File {
                shebang: None,
                frontmatter: None,
                attrs: vec![],
                items: vec![syn::Item::Const(s.clone())],
            })
        }
    }

    impl ConstComparisonProblems {
        fn is_empty(&self) -> bool {
            self.reference_unique_names.is_empty() && self.different_consts.is_empty()
        }

        fn report_problems_if_any(&self, has_problems: &mut bool) {
            if self.is_empty() {
                return;
            }

            *has_problems = true;

            if !self.reference_unique_names.is_empty() {
                tracing::error!(
                    "compare_consts: `const`s that are missing in our bindings: {}",
                    self.reference_unique_names
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            for (name, diff) in &self.different_consts {
                tracing::error!(
                    "compare_consts: `const` `{}` differs between our bindings and the reference bindings: \n{}",
                    name,
                    &prettydiff::diff_lines(&diff.reference, &diff.ours,)
                )
            }
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
