use elm_codegen_core::ElmType;

#[derive(ElmType)]
#[elm(module = "Api.Pair")]
pub struct Pair(String, i32);

fn main() {}
