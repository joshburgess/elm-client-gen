use elm_ast::builder::tname;
use elm_ast::node::Spanned;
use elm_ast::type_annotation::TypeAnnotation;
use elm_codegen_core::{ElmFieldInfo, ElmTypeInfo, ElmTypeRepr};

use crate::helpers::{tqualified, trecord};
use crate::name_map::NameMap;

fn build_type_annotation(repr: &ElmTypeRepr, names: &NameMap) -> Spanned<TypeAnnotation> {
    match repr {
        ElmTypeRepr::String => tname("String", vec![]),
        ElmTypeRepr::Int => tname("Int", vec![]),
        ElmTypeRepr::Float => tname("Float", vec![]),
        ElmTypeRepr::Bool => tname("Bool", vec![]),
        ElmTypeRepr::Posix => tqualified(&["Time"], "Posix", vec![]),
        ElmTypeRepr::IsoDate => tname("String", vec![]),
        ElmTypeRepr::Maybe(inner) => tname("Maybe", vec![build_type_annotation(inner, names)]),
        ElmTypeRepr::List(inner) => tname("List", vec![build_type_annotation(inner, names)]),
        ElmTypeRepr::Custom(rust_name) => {
            let elm_name = names.resolve(rust_name);
            tname(elm_name, vec![])
        }
    }
}

/// Build a record type alias declaration from `ElmTypeInfo`.
pub fn build_type_alias(
    info: &ElmTypeInfo,
    names: &NameMap,
) -> Spanned<elm_ast::declaration::Declaration> {
    let fields: Vec<(&str, Spanned<TypeAnnotation>)> = info
        .fields
        .iter()
        .map(|f| (f.elm_name, build_type_annotation(&f.elm_type, names)))
        .collect();

    elm_ast::builder::type_alias(info.type_name, Vec::<String>::new(), trecord(fields))
}

/// Returns true if any field reaches a `Time.Posix`.
pub fn uses_posix(fields: &[ElmFieldInfo]) -> bool {
    fields.iter().any(|f| field_uses_posix(&f.elm_type))
}

fn field_uses_posix(repr: &ElmTypeRepr) -> bool {
    match repr {
        ElmTypeRepr::Posix => true,
        ElmTypeRepr::Maybe(inner) | ElmTypeRepr::List(inner) => field_uses_posix(inner),
        _ => false,
    }
}
