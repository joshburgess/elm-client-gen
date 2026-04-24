//! `#[elm(encoder_pairs = "fn")]` should land on `ElmFieldInfo::encoder_pairs`
//! as `Some("fn")`. Combined with `decoder_step` and a `type = "Head Arg"`
//! override, every 0.3.0 wrapper-codec hook is exercised in one shape.

use elm_client_gen_core::ElmType;

#[allow(dead_code)]
pub struct Patch<T>(std::marker::PhantomData<T>);

#[derive(ElmType)]
#[elm(module = "Api.Profile")]
#[serde(rename_all = "camelCase")]
pub struct ProfilePatch {
    #[elm(
        type = "Patch String",
        decoder_step = "patch",
        encoder_pairs = "patchPair"
    )]
    pub display_name: Patch<String>,
}

fn main() {
    let info = <ProfilePatch as ElmType>::elm_type_info();
    let f = info.fields().first().expect("displayName field");
    assert_eq!(f.decoder_step, Some("patch"));
    assert_eq!(f.encoder_pairs, Some("patchPair"));
}
