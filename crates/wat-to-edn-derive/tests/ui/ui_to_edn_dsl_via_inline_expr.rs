// compile-fail fixture: #[to_edn(via = xs.join(", "))] is an inline expression.
//
// Arc 296 Strike 2a — the `via` value must be a bare path (ident or `a::b::c`).
// A method call like `xs.join(", ")` is parsed as path `xs` + trailing
// `.join(", ")`. The derive parser detects the trailing tokens and emits a
// compile_error! naming the constraint.

use wat_to_edn_derive::ToEdn;

#[derive(ToEdn)]
pub enum BadVia {
    Foo {
        #[to_edn(via = xs.join(", "))]
        xs: Vec<String>,
    },
}

fn main() {}
