//! Property-based invariants for the builder's transformation
//! passes. These complement the example-based unit tests by
//! generating random inputs and asserting laws the code must hold
//! for every well-formed input.

use elm_client_gen_builder::{
    build_merged_module, group_by_module, DefaultStrategy, MaybeEncoderRef, NameMap, TypeOverrides,
};
use elm_client_gen_core::{
    ElmFieldInfo, ElmTypeInfo, ElmTypeKind, ElmTypeRepr, ElmVariantInfo, ElmVariantPayload,
    EnumRepresentation,
};
use proptest::prelude::*;

// ── Repr generators ─────────────────────────────────────────────────

fn custom_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("BigDecimal".to_string()),
        Just("Untouched".to_string()),
        Just("UserId".to_string()),
        Just("Money".to_string()),
    ]
}

fn repr_strategy() -> impl Strategy<Value = ElmTypeRepr> {
    let leaf = prop_oneof![
        Just(ElmTypeRepr::String),
        Just(ElmTypeRepr::Int),
        Just(ElmTypeRepr::Float),
        Just(ElmTypeRepr::Bool),
        Just(ElmTypeRepr::Posix),
        Just(ElmTypeRepr::IsoDate),
        custom_name().prop_map(ElmTypeRepr::Custom),
    ];
    leaf.prop_recursive(4, 16, 4, |inner| {
        prop_oneof![
            inner.clone().prop_map(|r| ElmTypeRepr::Maybe(Box::new(r))),
            inner.clone().prop_map(|r| ElmTypeRepr::List(Box::new(r))),
            inner.clone().prop_map(|r| ElmTypeRepr::Dict(Box::new(r))),
            prop::collection::vec(inner.clone(), 2..4).prop_map(ElmTypeRepr::Tuple),
            // Type application with a single arg. Head names are drawn
            // from a small fixed pool that doubles as wrapper-module
            // identifiers in the import-dedupe property below.
            (
                prop_oneof![
                    Just("Patch".to_string()),
                    Just("PatchNullable".to_string()),
                    Just("Translated".to_string()),
                ],
                inner
            )
                .prop_map(|(head, arg)| ElmTypeRepr::App {
                    head,
                    args: vec![arg],
                }),
        ]
    })
}

// ── TypeOverrides idempotency ───────────────────────────────────────

fn wrap_in_record(repr: ElmTypeRepr) -> ElmTypeInfo {
    ElmTypeInfo {
        rust_name: "FuzzWrap",
        module_path: vec!["Fuzz"],
        type_name: "FuzzWrap",
        tags: vec![],
        kind: ElmTypeKind::Record {
            fields: vec![ElmFieldInfo {
                rust_name: "value",
                elm_name: "value",
                elm_type: repr,
                is_optional: false,
                custom_decoder: None,
                custom_encoder: None,
                decoder_step: None,
                encoder_pairs: None,
            }],
        },
    }
}

fn wrap_in_enum_newtype(repr: ElmTypeRepr) -> ElmTypeInfo {
    ElmTypeInfo {
        rust_name: "FuzzEnum",
        module_path: vec!["Fuzz"],
        type_name: "FuzzEnum",
        tags: vec![],
        kind: ElmTypeKind::Enum {
            representation: EnumRepresentation::Untagged,
            variants: vec![ElmVariantInfo {
                rust_name: "Only",
                elm_name: "Only",
                json_tag: "Only",
                payload: ElmVariantPayload::Newtype(repr),
            }],
        },
    }
}

fn overrides_fixture() -> TypeOverrides {
    let mut o = TypeOverrides::new();
    o.alias("BigDecimal", ElmTypeRepr::String);
    o.alias("Money", ElmTypeRepr::Float);
    o
}

proptest! {
    #[test]
    fn type_overrides_apply_is_idempotent_on_record(repr in repr_strategy()) {
        let o = overrides_fixture();
        let info = wrap_in_record(repr);
        let once = o.apply(info);
        let twice = o.apply(once.clone());
        prop_assert_eq!(format!("{:?}", once.kind), format!("{:?}", twice.kind));
    }

    #[test]
    fn type_overrides_apply_is_idempotent_on_enum_newtype(repr in repr_strategy()) {
        let o = overrides_fixture();
        let info = wrap_in_enum_newtype(repr);
        let once = o.apply(info);
        let twice = o.apply(once.clone());
        prop_assert_eq!(format!("{:?}", once.kind), format!("{:?}", twice.kind));
    }
}

// ── group_by_module partition law ───────────────────────────────────

fn module_path_strategy() -> impl Strategy<Value = Vec<&'static str>> {
    // Pick from a small fixed pool of &'static str so we actually get
    // repeats and can exercise the grouping logic.
    prop_oneof![
        Just(vec!["Api", "Person"]),
        Just(vec!["Api", "Order"]),
        Just(vec!["Api", "Person"]),
        Just(vec!["Domain"]),
        Just(vec!["Domain", "Inventory"]),
    ]
}

fn typeinfo_strategy() -> impl Strategy<Value = ElmTypeInfo> {
    module_path_strategy().prop_map(|module_path| ElmTypeInfo {
        rust_name: "Fuzz",
        module_path,
        type_name: "Fuzz",
        tags: vec![],
        kind: ElmTypeKind::Record { fields: vec![] },
    })
}

proptest! {
    #[test]
    fn group_by_module_preserves_count_and_partitions_by_path(
        types in prop::collection::vec(typeinfo_strategy(), 0..20),
    ) {
        let groups = group_by_module(&types);
        let total: usize = groups.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, types.len());
        for (key, members) in &groups {
            for m in members {
                let member_path: Vec<&str> = m.module_path.to_vec();
                prop_assert_eq!(&member_path, key);
            }
        }
    }
}

// ── Import dedupe: at most one import line per target module ────────
//
// `build_merged_module` collects custom refs across every emitted type,
// groups them by target module path, and emits one import per group
// (unioning their exposing lists). The property: regardless of how
// many fields reference the same wrapper, the rendered module never
// contains more than one `import Api.Wrapper...` line per wrapper.

const WRAPPER_POOL: &[&str] = &["WrapperA", "WrapperB", "WrapperC"];
const FIELD_NAMES: &[&str] = &["f0", "f1", "f2", "f3", "f4", "f5"];

fn record_with_wrapper_fields(wrapper_indices: &[u8], field_arg: ElmTypeRepr) -> ElmTypeInfo {
    let fields: Vec<ElmFieldInfo> = wrapper_indices
        .iter()
        .enumerate()
        .map(|(i, idx)| ElmFieldInfo {
            rust_name: FIELD_NAMES[i],
            elm_name: FIELD_NAMES[i],
            elm_type: ElmTypeRepr::App {
                head: WRAPPER_POOL[*idx as usize].to_string(),
                args: vec![field_arg.clone()],
            },
            is_optional: false,
            custom_decoder: None,
            custom_encoder: None,
            decoder_step: Some("step"),
            encoder_pairs: Some("stepPair"),
        })
        .collect();
    ElmTypeInfo {
        rust_name: "R",
        module_path: vec!["Api", "R"],
        type_name: "R",
        tags: vec![],
        kind: ElmTypeKind::Record { fields },
    }
}

fn names_for_wrapper_pool(types: &[ElmTypeInfo]) -> NameMap {
    let mut names = NameMap::from_types(types);
    for w in WRAPPER_POOL {
        names.register_with_exposed(
            (*w).to_string(),
            (*w).to_string(),
            vec!["Api".into(), (*w).to_string()],
            vec![(*w).to_string(), "step".into(), "stepPair".into()],
        );
    }
    names
}

proptest! {
    #[test]
    fn build_merged_module_emits_at_most_one_import_per_wrapper_module(
        wrapper_indices in prop::collection::vec(0u8..(WRAPPER_POOL.len() as u8), 0..FIELD_NAMES.len()),
        arg in repr_strategy(),
    ) {
        let info = record_with_wrapper_fields(&wrapper_indices, arg);
        let types = vec![info];
        let names = names_for_wrapper_pool(&types);
        let strategy = DefaultStrategy;
        let maybe = MaybeEncoderRef::new(vec!["Json", "Encode", "Extra"], "maybe");
        let groups = group_by_module(&types);
        let (module_path, group) = groups.into_iter().next().expect("one module group");
        let module = build_merged_module(&module_path, &group, &names, &strategy, &maybe);
        let rendered = elm_ast::pretty_print(&module);

        for w in WRAPPER_POOL {
            let needle = format!("import Api.{w}");
            let count = rendered.matches(needle.as_str()).count();
            prop_assert!(
                count <= 1,
                "wrapper {w} imported {count} times in:\n{rendered}",
            );
            // If at least one field references this wrapper, the import
            // must be emitted exactly once (not zero).
            let referenced = wrapper_indices
                .iter()
                .any(|idx| WRAPPER_POOL[*idx as usize] == *w);
            if referenced {
                prop_assert_eq!(
                    count, 1,
                    "wrapper {} referenced but not imported in:\n{}",
                    w, rendered,
                );
            }
        }
    }
}
