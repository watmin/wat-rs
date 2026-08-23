//! FM 2-bis probe — arc 251 Stone 251.5a-i: the homoiconic `read`.
//!
//! Run: `cargo test --release --test probe_arc251_stone5a_read_string`

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed bool.
fn eval_bool(fn_name: &str) -> Result<bool, String> {
    match call_beside_value(file!(), fn_name).map_err(|e| format!("eval: {e:?}"))? {
        Value::bool(b) => Ok(b),
        other => Err(format!("non-bool: {other:?}")),
    }
}

#[test]
fn contract_01_read_string_returns_walkable_forms() {
    assert_eq!(
        eval_bool(":user::c01"),
        Ok(true),
        "read-string must return a forms-List the macro engine can walk (List? recognizes it)"
    );
}

#[test]
fn contract_02_read_string_reads_the_dirty_surface() {
    // Arc 109 wave 2 "annihilate the angle bracket" — THE PERMISSION IS GONE. This
    // contract's whole subject WAS the angle form: that `read-string` could read a
    // "dirty" pre-251.5 `Vector<...>` spelling the strict EDN reader refused. The
    // lexer wall (this stone) refuses `<` in a name universally — `read-string` shares
    // the same lexer as everything else — so there is no longer any surface `read-
    // string` reads that a stricter reader wouldn't also refuse; the whole
    // "dirty surface" `read-string` existed partly to read is gone. Class 3 (b):
    // re-pointed as a refusal control on the mechanism that actually fires now.
    //
    // The fixture's `ReadOutcome::Malformed` arm calls `(:wat::core::Error/message
    // __cause)` on a `:wat::edn::ForeignRecord` with no `message` surface method, so
    // the lex refusal surfaces as an unrelated `UnknownFunction` crash rather than a
    // clean error — a SEPARATE, already-documented defect (DESIGN-STONE-annihilate-
    // the-angle-bracket.md's sequencing section), out of this stone's boundary.
    // Asserting the crash's mechanism, not a made-up clean message.
    let err = eval_bool(":user::c02").expect_err("angle-bracket source must fail to read");
    assert!( // rune:lint(loose-assert) — targeted substring: the read-string crash's mechanism, not the whole located error's structure
        err.contains("ForeignRecord") && err.contains("message"),
        "expected the read-string/ForeignRecord crash (see comment above); got: {err}"
    );
}
