// compile-fail fixture: #[to_edn(key = 123)] — key value must be a LitStr.
//
// Arc 296 Strike 2a — the `key` value must be a string literal (LitStr).
// An integer literal is rejected by the parser with a clear error message.

use wat_to_edn_derive::ToEdn;

#[derive(ToEdn)]
pub enum BadKey {
    Foo {
        #[to_edn(key = 123)]
        name: String,
    },
}

fn main() {}
