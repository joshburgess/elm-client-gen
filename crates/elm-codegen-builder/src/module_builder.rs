use std::collections::{BTreeMap, BTreeSet};

use elm_ast::builder::spanned;
use elm_ast::declaration::Declaration;
use elm_ast::exposing::{ExposedItem, Exposing};
use elm_ast::file::ElmModule;
use elm_ast::import::Import;
use elm_ast::module_header::ModuleHeader;
use elm_ast::node::Spanned;
use elm_codegen_core::{ElmFieldInfo, ElmTypeInfo, ElmTypeRepr};

use crate::decoder::{build_decoder, lcfirst};
use crate::encoder::build_encoder;
use crate::helpers::{import_as_exposing, import_exposing};
use crate::name_map::NameMap;
use crate::strategy::BuildStrategy;
use crate::type_builder::{build_type_alias, uses_posix};

/// Where to find the `encodeMaybe` helper in the consumer's Elm
/// codebase. The encoder generator emits a call to `function_name`
/// (unqualified), and the import is added automatically.
#[derive(Clone)]
pub struct MaybeEncoderRef {
    pub module_path: Vec<String>,
    pub function_name: String,
}

impl MaybeEncoderRef {
    pub fn new(module_path: Vec<&str>, function_name: &str) -> Self {
        Self {
            module_path: module_path.into_iter().map(String::from).collect(),
            function_name: function_name.to_string(),
        }
    }
}

/// Group `ElmTypeInfo` values by their target module path.
pub fn group_by_module<'a>(
    types: &'a [ElmTypeInfo],
) -> BTreeMap<Vec<&'a str>, Vec<&'a ElmTypeInfo>> {
    let mut groups: BTreeMap<Vec<&'a str>, Vec<&'a ElmTypeInfo>> = BTreeMap::new();
    for info in types {
        let key: Vec<&str> = info.module_path.to_vec();
        groups.entry(key).or_default().push(info);
    }
    groups
}

/// Build a complete Elm module for one group of types that share a
/// module path.
pub fn build_merged_module<S: BuildStrategy>(
    module_path: &[&str],
    types: &[&ElmTypeInfo],
    names: &NameMap,
    strategy: &S,
    maybe: &MaybeEncoderRef,
) -> ElmModule {
    let mut declarations: Vec<Spanned<Declaration>> = Vec::new();
    let mut exposed_items: Vec<Spanned<ExposedItem>> = Vec::new();
    let mut all_fields: Vec<&ElmFieldInfo> = Vec::new();
    let mut needs_encoder = false;

    for info in types {
        all_fields.extend(info.fields.iter());

        declarations.push(build_type_alias(info, names));
        exposed_items.push(spanned(ExposedItem::TypeOrAlias(info.type_name.to_string())));

        if strategy.should_emit_decoder(info) {
            let decoder_name = format!("{}Decoder", lcfirst(info.type_name));
            declarations.push(build_decoder(info, names));
            exposed_items.push(spanned(ExposedItem::Function(decoder_name)));
        }

        if strategy.should_emit_encoder(info) {
            let encoder_name = format!("encode{}", info.type_name);
            declarations.push(build_encoder(info, names, maybe));
            exposed_items.push(spanned(ExposedItem::Function(encoder_name)));
            needs_encoder = true;
        }
    }

    let module_name: Vec<String> = module_path.iter().map(|s| s.to_string()).collect();
    let header = spanned(ModuleHeader::Normal {
        name: spanned(module_name),
        exposing: spanned(Exposing::Explicit {
            items: exposed_items,
            trailing_comments: Vec::new(),
        }),
    });

    let owned_fields: Vec<ElmFieldInfo> = all_fields.into_iter().cloned().collect();
    let imports = build_imports(&owned_fields, needs_encoder, module_path, names, maybe);

    ElmModule {
        header,
        module_documentation: None,
        imports,
        declarations,
        comments: Vec::new(),
    }
}

fn build_imports(
    fields: &[ElmFieldInfo],
    needs_encoder: bool,
    current_module: &[&str],
    names: &NameMap,
    maybe: &MaybeEncoderRef,
) -> Vec<Spanned<Import>> {
    let mut imports = vec![
        import_as_exposing(&["Json", "Decode"], "Decode", vec!["Decoder"]),
        import_exposing(
            &["Json", "Decode", "Pipeline"],
            vec!["required", "optional"],
        ),
    ];

    if needs_encoder {
        imports.push(import_as_exposing(
            &["Json", "Encode"],
            "Encode",
            vec!["Value"],
        ));
    }

    if uses_posix(fields) {
        imports.push(elm_ast::builder::import(vec!["Time"]));
        imports.push(elm_ast::builder::import(vec!["Iso8601"]));
    }

    let has_maybe = fields
        .iter()
        .any(|f| matches!(&f.elm_type, ElmTypeRepr::Maybe(_)));
    if has_maybe && needs_encoder {
        let module_refs: Vec<&str> = maybe.module_path.iter().map(|s| s.as_str()).collect();
        imports.push(import_exposing(
            &module_refs,
            vec![maybe.function_name.as_str()],
        ));
    }

    let current: Vec<String> = current_module.iter().map(|s| s.to_string()).collect();
    let mut custom_refs: BTreeSet<String> = BTreeSet::new();
    for f in fields {
        collect_custom_refs(&f.elm_type, &mut custom_refs);
    }
    for rust_name in &custom_refs {
        let Some(entry) = names.lookup(rust_name) else {
            continue;
        };
        if entry.module_path == current {
            continue;
        }
        let module_refs: Vec<&str> = entry.module_path.iter().map(|s| s.as_str()).collect();
        let decoder = format!("{}Decoder", lcfirst(&entry.elm_name));
        let encoder = format!("encode{}", entry.elm_name);
        let mut exposed = vec![entry.elm_name.as_str(), decoder.as_str()];
        if needs_encoder {
            exposed.push(encoder.as_str());
        }
        imports.push(import_exposing(&module_refs, exposed));
    }

    imports
}

fn collect_custom_refs(repr: &ElmTypeRepr, out: &mut BTreeSet<String>) {
    match repr {
        ElmTypeRepr::Custom(name) => {
            out.insert(name.clone());
        }
        ElmTypeRepr::Maybe(inner) | ElmTypeRepr::List(inner) => collect_custom_refs(inner, out),
        _ => {}
    }
}
