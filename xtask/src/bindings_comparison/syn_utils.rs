pub fn remove_docs(syn_file: &[syn::Item]) -> Vec<syn::Item> {
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

pub fn get_item_name(item: &syn::Item) -> Option<String> {
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

pub fn expr_try_as_ident(expr: &syn::Expr) -> Option<&syn::Ident> {
    if let syn::Expr::Path(path) = expr
        && path.attrs.is_empty()
        && path.qself.is_none()
        && path.path.leading_colon.is_none()
        && path.path.segments.len() == 1
        && let Some(path_segment) = path.path.segments.first()
        && path_segment.arguments.is_empty()
    {
        Some(&path_segment.ident)
    } else {
        None
    }
}

pub fn expr_try_as_literal(expr: &syn::Expr) -> Option<&syn::Lit> {
    if let syn::Expr::Lit(expr_lit) = expr {
        Some(&expr_lit.lit)
    } else {
        None
    }
}
