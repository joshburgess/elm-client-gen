//! Build Elm AST modules (records, decoders, encoders, imports) from
//! `elm-codegen-core` type info.
//!
//! Pair this with [`elm_ast::pretty_print`] to render the resulting
//! modules as `elm-format`-style source.

mod decoder;
mod encoder;
mod helpers;
pub mod module_builder;
mod name_map;
mod overrides;
mod strategy;
mod type_builder;

pub use decoder::build_decoder;
pub use encoder::build_encoder;
pub use helpers::{
    case_of, import_as, import_as_exposing, import_exposing, list_multiline, pipeline_chain,
    record_access, tqualified, trecord,
};
pub use module_builder::{build_merged_module, group_by_module, MaybeEncoderRef};
pub use name_map::{NameEntry, NameMap};
pub use overrides::TypeOverrides;
pub use strategy::{BuildStrategy, DefaultStrategy};
pub use type_builder::{build_type_alias, uses_posix};
