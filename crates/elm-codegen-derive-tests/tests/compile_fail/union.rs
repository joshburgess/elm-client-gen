use elm_codegen_core::ElmType;

#[derive(ElmType)]
#[elm(module = "Api.Union")]
pub union Bits {
    as_u32: u32,
    as_bytes: [u8; 4],
}

fn main() {}
