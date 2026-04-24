//! `#[elm(decoder_step = "fn")]` should land on `ElmFieldInfo::decoder_step`
//! as `Some("fn")` without conflicting with sibling attributes.

use elm_client_gen_core::ElmType;

#[allow(dead_code)]
pub struct Patch<T>(std::marker::PhantomData<T>);

#[derive(ElmType)]
#[elm(module = "Api.Profile")]
#[serde(rename_all = "camelCase")]
pub struct ProfilePatch {
    #[elm(type = "Patch String", decoder_step = "patch")]
    pub display_name: Patch<String>,
}

fn main() {
    let info = <ProfilePatch as ElmType>::elm_type_info();
    let f = info.fields().first().expect("displayName field");
    assert_eq!(f.decoder_step, Some("patch"));
    assert!(f.encoder_pairs.is_none());
}
