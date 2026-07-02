// compile-fail fixture: #[derive(ToEdn)] on an enum with a tuple variant.
//
// Arc 296 Strike 1 — tuple variants are not supported.
// A tuple variant triggers the "does not support tuple variants" compile_error!.

use wat_to_edn_derive::ToEdn;

#[derive(ToEdn)]
pub enum BadKind {
    Named { field: String },
    Tuple(String),
}

fn main() {}
