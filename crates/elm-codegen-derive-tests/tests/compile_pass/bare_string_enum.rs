use elm_codegen_core::ElmType;

#[derive(ElmType)]
#[elm(module = "Api.Status")]
pub enum Status {
    Active,
    Archived,
    Banned,
}

fn main() {
    let _ = <Status as ElmType>::elm_type_info();
}
