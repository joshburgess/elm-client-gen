use elm_codegen_core::ElmType;

#[derive(ElmType)]
#[elm(module = "Api.Bad")]
#[serde(tag = "kind")]
#[serde(untagged)]
pub enum Bad {
    One,
    Two,
}

fn main() {}
