//! End-to-end smoke test: derive a struct, run the builder, render with
//! `elm_ast::pretty_print`, and assert the output contains the expected
//! Elm declarations.

use elm_codegen_builder::{
    build_merged_module, group_by_module, DefaultStrategy, MaybeEncoderRef, NameMap,
};
use elm_codegen_core::{registered_types, ElmType, ElmTypeInfo};

#[derive(ElmType)]
#[elm(module = "Api.Person", name = "Person", tags = "entity")]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct PersonApi {
    pub id: String,
    pub full_name: String,
    pub age: i32,
    pub nickname: Option<String>,
    pub tags: Vec<String>,
}

#[derive(ElmType)]
#[elm(module = "Api.Person", name = "PersonFilter", tags = "filter")]
#[allow(dead_code)]
pub struct PersonFilterApi {
    #[elm(skip)]
    pub internal: String,
    pub name_contains: Option<String>,
    #[elm(name = "minAge")]
    pub min_age_years: Option<i32>,
}

#[test]
fn derive_emits_type_info_with_field_metadata() {
    let info = PersonApi::elm_type_info();

    assert_eq!(info.rust_name, "PersonApi");
    assert_eq!(info.module_path, vec!["Api", "Person"]);
    assert_eq!(info.type_name, "Person");
    assert_eq!(info.tags, vec!["entity"]);
    assert!(info.has_tag("entity"));

    assert_eq!(info.fields.len(), 5);
    assert_eq!(info.fields[1].rust_name, "fullName");
    assert_eq!(info.fields[1].elm_name, "fullName");

    let nickname = &info.fields[3];
    assert_eq!(nickname.elm_name, "nickname");
    assert!(nickname.is_optional);
}

#[test]
fn skip_and_rename_attributes_are_honored() {
    let info = PersonFilterApi::elm_type_info();
    let names: Vec<&str> = info.fields.iter().map(|f| f.elm_name).collect();

    assert!(!names.contains(&"internal"));
    assert!(names.contains(&"nameContains"));
    assert!(names.contains(&"minAge"));
}

#[test]
fn registered_types_picks_up_derived_structs() {
    let names: Vec<&str> = registered_types().iter().map(|t| t.rust_name).collect();
    assert!(names.contains(&"PersonApi"));
    assert!(names.contains(&"PersonFilterApi"));
}

#[test]
fn build_merged_module_renders_expected_elm() {
    let types: Vec<ElmTypeInfo> = vec![
        PersonApi::elm_type_info(),
        PersonFilterApi::elm_type_info(),
    ];
    let names = NameMap::from_types(&types);
    let strategy = DefaultStrategy;
    let maybe = MaybeEncoderRef::new(vec!["Json", "Encode", "Extra"], "maybe");

    let groups = group_by_module(&types);
    let (module_path, group) = groups.into_iter().next().expect("one module group");

    let module = build_merged_module(&module_path, &group, &names, &strategy, &maybe);
    let rendered = elm_ast::pretty_print(&module);

    assert!(rendered.contains("module Api.Person exposing"));
    assert!(rendered.contains("type alias Person ="));
    assert!(rendered.contains("type alias PersonFilter ="));
    assert!(rendered.contains("personDecoder"));
    assert!(rendered.contains("encodePerson"));
    assert!(rendered.contains("import Json.Decode as Decode"));
    assert!(rendered.contains("import Json.Encode.Extra exposing (maybe)"));
    assert!(rendered.contains("nickname : Maybe String"));
}
