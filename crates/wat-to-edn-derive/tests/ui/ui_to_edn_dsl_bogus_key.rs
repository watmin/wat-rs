// compile-fail fixture: #[to_edn(bogus = "x")] — unknown directive.
//
// Arc 296 Strike 2a — unknown `#[to_edn(...)]` directives must emit a
// compile_error! naming the allowed set of directives.

use wat_to_edn_derive::ToEdn;

#[derive(ToEdn)]
pub enum BadBogus {
    Foo {
        #[to_edn(bogus = "x")]
        name: String,
    },
}

fn main() {}
