use elm_codegen_core::ElmType;

#[derive(ElmType)]
#[elm(module = "Api.Event")]
pub enum Event {
    Click { x: i32, y: i32 },
    Scroll(f32),
}

fn main() {}
