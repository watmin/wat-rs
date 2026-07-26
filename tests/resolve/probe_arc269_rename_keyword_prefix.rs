//! Arc 269 vehicle — `:wat::fix::rename-keyword-prefix`: a reusable, comment-faithful
//! keyword-PREFIX rename rule on the fix-text engine.
//!
//! Contract: `(:wat::fix::rename-keyword-prefix old-prefix new-prefix src) -> migrated-src` — for
//! every keyword leaf whose name STARTS WITH `old-prefix`, splice the prefix → `new-prefix`
//! (the suffix, incl. `/accessor` and `::Variant`, is preserved); comments + formatting survive
//! byte-identical (rides `fix-text-apply`'s right-to-left span splice).
//!
//! RED at HEAD: `:wat::fix::rename-keyword-prefix` does not exist → UnknownCallee. GREEN once the
//! rule ships on the fix-text engine.
//!
//! Run: cargo test --release -p wat --test probe_arc269_rename_keyword_prefix

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn rename_keyword_prefix_swaps_prefix_comment_faithful() {
    // just-eval (rubric): `:user::go` lives in the co-located fixture.
    let got = match call_beside_value(file!(), ":user::go")
        .unwrap_or_else(|e| panic!("go raised: {e:?}"))
    {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String, got {other:?}"),
    };
    assert_eq!(
        got,
        include_str!("probe_arc269_rename_keyword_prefix__swap-prefix-comment-faithful.wat"),
        "rename-keyword-prefix golden mismatch; both accessor prefixes must be swapped, \
         old prefix must be gone, comment must survive byte-identical"
    );
}
