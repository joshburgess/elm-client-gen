//! `#[elm(type = "Head Arg")]` on a record field should parse as
//! `ElmTypeRepr::App { head: "Head", args: [..] }` and round-trip
//! through `elm_type_info()` without runtime panics.

use elm_client_gen_core::{ElmType, ElmTypeRepr};

#[allow(dead_code)]
pub struct Patch<T>(std::marker::PhantomData<T>);

#[derive(ElmType)]
#[elm(module = "Api.Profile")]
#[serde(rename_all = "camelCase")]
pub struct ProfilePatch {
    #[elm(type = "Patch String")]
    pub display_name: Patch<String>,
}

fn main() {
    let info = <ProfilePatch as ElmType>::elm_type_info();
    let f = info.fields().first().expect("displayName field");
    assert!(matches!(
        &f.elm_type,
        ElmTypeRepr::App { head, args }
            if head == "Patch"
            && args.len() == 1
            && matches!(args[0], ElmTypeRepr::String)
    ));
}
