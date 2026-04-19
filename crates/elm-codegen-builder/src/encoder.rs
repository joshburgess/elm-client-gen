use elm_ast::builder::{app, func_with_sig, pvar, string, tname, tuple, var};
use elm_ast::node::Spanned;
use elm_codegen_core::{ElmTypeInfo, ElmTypeRepr};

use crate::helpers::{list_multiline, record_access};
use crate::module_builder::MaybeEncoderRef;
use crate::name_map::NameMap;

/// Build an encoder function declaration.
pub fn build_encoder(
    info: &ElmTypeInfo,
    names: &NameMap,
    maybe: &MaybeEncoderRef,
) -> Spanned<elm_ast::declaration::Declaration> {
    let encoder_name = format!("encode{}", info.type_name);
    let param = "value";

    let fields: Vec<Spanned<elm_ast::expr::Expr>> = info
        .fields
        .iter()
        .map(|f| {
            let accessor = record_access(var(param), f.elm_name);
            let encoded = encoder_for_type(&f.elm_type, accessor, names, maybe);
            tuple(vec![string(f.rust_name), encoded])
        })
        .collect();

    let body = app(
        elm_ast::builder::qualified(&["Encode"], "object"),
        vec![list_multiline(fields)],
    );

    let sig = elm_ast::builder::tfunc(tname(info.type_name, vec![]), tname("Value", vec![]));

    func_with_sig(&encoder_name, vec![pvar(param)], body, sig)
}

fn encoder_for_type(
    repr: &ElmTypeRepr,
    accessor: Spanned<elm_ast::expr::Expr>,
    names: &NameMap,
    maybe: &MaybeEncoderRef,
) -> Spanned<elm_ast::expr::Expr> {
    match repr {
        ElmTypeRepr::String | ElmTypeRepr::IsoDate => app(
            elm_ast::builder::qualified(&["Encode"], "string"),
            vec![accessor],
        ),
        ElmTypeRepr::Int => app(
            elm_ast::builder::qualified(&["Encode"], "int"),
            vec![accessor],
        ),
        ElmTypeRepr::Float => app(
            elm_ast::builder::qualified(&["Encode"], "float"),
            vec![accessor],
        ),
        ElmTypeRepr::Bool => app(
            elm_ast::builder::qualified(&["Encode"], "bool"),
            vec![accessor],
        ),
        ElmTypeRepr::Posix => app(
            elm_ast::builder::qualified(&["Iso8601"], "encode"),
            vec![accessor],
        ),
        ElmTypeRepr::Maybe(inner) => {
            let inner_encoder = encoder_fn_for_type(inner, names, maybe);
            app(
                maybe_encoder_var(maybe),
                vec![inner_encoder, accessor],
            )
        }
        ElmTypeRepr::List(inner) => {
            let inner_encoder = encoder_fn_for_type(inner, names, maybe);
            app(
                elm_ast::builder::qualified(&["Encode"], "list"),
                vec![inner_encoder, accessor],
            )
        }
        ElmTypeRepr::Custom(rust_name) => {
            let elm_name = names.resolve(rust_name);
            app(var(format!("encode{}", elm_name)), vec![accessor])
        }
    }
}

fn encoder_fn_for_type(
    repr: &ElmTypeRepr,
    names: &NameMap,
    maybe: &MaybeEncoderRef,
) -> Spanned<elm_ast::expr::Expr> {
    match repr {
        ElmTypeRepr::String | ElmTypeRepr::IsoDate => {
            elm_ast::builder::qualified(&["Encode"], "string")
        }
        ElmTypeRepr::Int => elm_ast::builder::qualified(&["Encode"], "int"),
        ElmTypeRepr::Float => elm_ast::builder::qualified(&["Encode"], "float"),
        ElmTypeRepr::Bool => elm_ast::builder::qualified(&["Encode"], "bool"),
        ElmTypeRepr::Posix => elm_ast::builder::qualified(&["Iso8601"], "encode"),
        ElmTypeRepr::Custom(rust_name) => {
            let elm_name = names.resolve(rust_name);
            var(format!("encode{}", elm_name))
        }
        ElmTypeRepr::Maybe(inner) => app(
            maybe_encoder_var(maybe),
            vec![encoder_fn_for_type(inner, names, maybe)],
        ),
        ElmTypeRepr::List(inner) => app(
            elm_ast::builder::qualified(&["Encode"], "list"),
            vec![encoder_fn_for_type(inner, names, maybe)],
        ),
    }
}

fn maybe_encoder_var(maybe: &MaybeEncoderRef) -> Spanned<elm_ast::expr::Expr> {
    var(maybe.function_name.as_str())
}
