use elm_ast::builder::{app, func_with_sig, string, tname, var};
use elm_ast::node::Spanned;
use elm_codegen_core::{ElmFieldInfo, ElmTypeInfo, ElmTypeRepr};

use crate::helpers::{pipeline_chain, tqualified};
use crate::name_map::NameMap;

/// Build a decoder function declaration for the given type info.
pub fn build_decoder(
    info: &ElmTypeInfo,
    names: &NameMap,
) -> Spanned<elm_ast::declaration::Declaration> {
    let decoder_name = format!("{}Decoder", lcfirst(info.type_name));

    let seed = app(
        elm_ast::builder::qualified(&["Decode"], "succeed"),
        vec![var(info.type_name)],
    );
    let steps: Vec<_> = info
        .fields
        .iter()
        .map(|field| build_field_decoder_step(field, names))
        .collect();
    let expr = pipeline_chain(seed, steps);

    let sig = tqualified(&["Decode"], "Decoder", vec![tname(info.type_name, vec![])]);

    func_with_sig(&decoder_name, vec![], expr, sig)
}

fn build_field_decoder_step(field: &ElmFieldInfo, names: &NameMap) -> Spanned<elm_ast::expr::Expr> {
    if let Some(custom) = field.custom_decoder {
        return app(var("required"), vec![string(field.rust_name), var(custom)]);
    }

    if field.is_optional {
        let inner_type = match &field.elm_type {
            ElmTypeRepr::Maybe(inner) => inner.as_ref(),
            other => other,
        };
        let inner_decoder = decoder_for_type(inner_type, names);
        let nullable_decoder = app(
            elm_ast::builder::qualified(&["Decode"], "nullable"),
            vec![inner_decoder],
        );

        app(
            var("optional"),
            vec![string(field.rust_name), nullable_decoder, var("Nothing")],
        )
    } else {
        let field_decoder = decoder_for_type(&field.elm_type, names);
        app(
            var("required"),
            vec![string(field.rust_name), field_decoder],
        )
    }
}

fn decoder_for_type(repr: &ElmTypeRepr, names: &NameMap) -> Spanned<elm_ast::expr::Expr> {
    match repr {
        ElmTypeRepr::String | ElmTypeRepr::IsoDate => {
            elm_ast::builder::qualified(&["Decode"], "string")
        }
        ElmTypeRepr::Int => elm_ast::builder::qualified(&["Decode"], "int"),
        ElmTypeRepr::Float => elm_ast::builder::qualified(&["Decode"], "float"),
        ElmTypeRepr::Bool => elm_ast::builder::qualified(&["Decode"], "bool"),
        ElmTypeRepr::Posix => elm_ast::builder::qualified(&["Iso8601"], "decoder"),
        ElmTypeRepr::Maybe(inner) => app(
            elm_ast::builder::qualified(&["Decode"], "nullable"),
            vec![decoder_for_type(inner, names)],
        ),
        ElmTypeRepr::List(inner) => app(
            elm_ast::builder::qualified(&["Decode"], "list"),
            vec![decoder_for_type(inner, names)],
        ),
        ElmTypeRepr::Custom(rust_name) => {
            let elm_name = names.resolve(rust_name);
            var(format!("{}Decoder", lcfirst(elm_name)))
        }
    }
}

/// Lowercase the first character (used for decoder/function naming).
pub fn lcfirst(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_lowercase().to_string() + chars.as_str(),
    }
}
