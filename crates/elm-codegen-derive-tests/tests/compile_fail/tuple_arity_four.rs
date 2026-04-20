use elm_codegen_core::ElmType;

#[derive(ElmType)]
#[elm(module = "Api.Quad")]
pub struct Quad {
    pub payload: (i32, i32, i32, i32),
}

fn main() {}
