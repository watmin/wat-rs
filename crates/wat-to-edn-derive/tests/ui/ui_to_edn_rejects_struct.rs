// compile-fail fixture: #[derive(ToEdn)] on a non-named-field struct must be rejected.
//
// Arc 296 stone A — the struct derive supports NAMED-FIELD structs only.
// A tuple struct (or unit struct) triggers the "supports named-field structs only"
// compile_error!.
//
// NOTE (stone A, STOP-4 finding): the original test in wat-macros used a
// named-field struct `BadKind { field: String }` and expected the error
// "supports enums (kind-enums) only in Strike 1". That error no longer fires —
// named-field structs ARE now supported (S1 test `struct_derive_emits_*`).
// This is a behavior change; the test is updated to test a tuple struct which
// IS still rejected. The .stderr golden is updated accordingly.

use wat_to_edn_derive::ToEdn;

#[derive(ToEdn)]
pub struct BadKind(String);

fn main() {}
