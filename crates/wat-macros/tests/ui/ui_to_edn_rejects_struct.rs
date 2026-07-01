// compile-fail fixture: #[derive(ToEdn)] on a struct must be rejected.
//
// Arc 296 Strike 1 — the derive is for kind-enums only.
// A struct input triggers the "supports enums only" compile_error!.

use wat_macros::ToEdn;

#[derive(ToEdn)]
pub struct BadKind {
    field: String,
}

fn main() {}
