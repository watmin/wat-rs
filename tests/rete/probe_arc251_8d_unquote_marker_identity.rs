//! Arc 251 stone 251.8d-ii (SEVENTH draw) — **the quasiquote MARKERS are identities, and one of
//! them is the whole of this draw's single-file address.**
//!
//! ## The measurement that found it
//!
//! The seventh brief's address was `wat/rete/compile.wat:795`'s `:then` item fence, on the
//! strength of one named test that was said to pass unconverted and fail with
//! `wat/rete/compile.wat` converted alone. ⛔ **That test does not fail** (all four of
//! `probe_arc278_then_is_an_expansion_boundary` pass on a tree with `compile.wat` converted).
//! What DOES fail is eight tests in `binary(rete)`, and every one of them carries ONE arm:
//!
//! ```text
//!   compile-condition: accumulator expr is not pure — '<non-keyword/symbol head>' is not pure
//!     wat/rete/compile.wat:609
//! ```
//!
//! — the ACCUMULATOR fence (line 609), not the `:then` fence (795), and it fires on the FIRST
//! axis, so `primitive?` never gets to decide anything.
//!
//! ⭐ **Narrowed to ONE TOKEN by hunk-level experiment**, converting the whole file and putting
//! back one spelling at a time (each row a rebuild, `binary(rete)` = 522 tests):
//!
//! | `wat/rete/compile.wat:579-580` | `binary(rete)` |
//! |---|---|
//! | fully converted | ⛔ 514 / **8 failed** |
//! | `` `((~acc-hd) __acc__) `` both markers put back | ✅ **522 / 0** |
//! | `quasiquote` SYMBOL, `unquote` keyword | ✅ **522 / 0** |
//! | `quasiquote` keyword, `unquote` **SYMBOL** | ⛔ 514 / **8 failed** |
//!
//! The `quasiquote` head itself is fine — it reaches `eval_quasiquote` through the special-form
//! dispatcher, which already resolves both spellings. The `unquote` INSIDE the template is read
//! by `runtime.rs::match_qq_head`, which was keyword-only. The unquote never fired, the template
//! kept `(wat.core/unquote acc-hd)` verbatim, and `fence-call`'s head became a LIST — which is
//! what `<non-keyword/symbol head>` is.
//!
//! ## The rows
//!
//! Two tiers, because neither alone is enough.
//!
//! **Tier 1 — the two spellings render IDENTICALLY.** Both sides come from the fixture; the test
//! holds no expected text of its own. ⚠ That is not politeness, it is the repo's own rule: a
//! rendered wat form in a Rust string literal IS inlined wat (`no_inlined_wat_in_tests`,
//! `no_inlined_edn`), and both lints went RED on the first cut of this file for exactly that.
//! The rubric says restructure the CODE rather than rune it, so the ABSOLUTE claims moved into
//! the fixture, in wat, where a form is a form.
//!
//! **Tier 2 — the absolute property, computed in the fixture.**
//!
//! | entry | want | what it pins |
//! |---|---|---|
//! | `kw-unquote-fired` | true | ⛔ control — the keyword marker fires (no `unquote` survives) |
//! | ⭐ `sym-unquote-fired` | true | **THE CURE** — the accumulator fence's exact template |
//! | `kw-splice-fired` / ⭐ `sym-splice-fired` | true | one door covers `unquote-splicing` too |
//! | ⛔ `kw-not-a-marker-stayed-data` | true | **NON-VACUITY** — `unquotex` is one letter from `unquote` and stays DATA |
//! | ⛔ `sym-not-a-marker-stayed-data` | true | ⭐ the cure fires on the MARKER, never on "symbol-headed" |
//!
//! ⛔ The last two rows are the wall, and they are green on the pre-cure binary too — which is
//! what makes them a test of the wall rather than a test of the cure. Tier 1 without tier 2
//! would pass if BOTH spellings stopped firing; tier 2 without tier 1 would pass if they fired
//! differently. The claim needs both.
//!
//! Run: `cargo nextest run --release -E 'test(unquote_marker_identity)'`

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn rendered(entry: &str) -> String {
    match call_beside_value(file!(), entry).expect("the fixture must evaluate") {
        Value::String(s) => (*s).clone(),
        other => panic!("{entry} answered {other:?}, not a String"),
    }
}

fn holds(entry: &str) -> bool {
    match call_beside_value(file!(), entry).expect("the fixture must evaluate") {
        Value::bool(b) => b,
        other => panic!("{entry} answered {other:?}, not a bool"),
    }
}

/// ⭐ THE WHOLE TABLE IN ONE TEST — a cure that fired on every symbol-headed sub-form would pass
/// the marker rows and fail only the near-miss rows, so they are weighed together.
#[test]
fn the_quasiquote_markers_are_identities_and_a_near_miss_is_still_not_one() {
    let mut wrong: Vec<String> = Vec::new();

    // ── tier 1: the two spellings of ONE marker must render identically ─────────────────────
    let pairs: &[(&str, &str, &str)] = &[
        (
            ":user::qq-kw-unquote",
            ":user::qq-sym-unquote",
            "THE CURE: the accumulator fence's own template, both spellings",
        ),
        (
            ":user::qq-kw-splice",
            ":user::qq-sym-splice",
            "THE CURE: one door covers unquote-splicing too",
        ),
    ];
    for (kw, sym, why) in pairs {
        let (a, b) = (rendered(kw), rendered(sym));
        if a != b {
            wrong.push(format!("  {kw} → {a}\n  {sym} → {b}\n      they must be one form — {why}"));
        }
    }

    // ── tier 2: the ABSOLUTE property, so tier 1 cannot pass by both sides being broken ─────
    let props: &[(&str, &str)] = &[
        (":user::kw-unquote-fired", "control: the keyword marker fires"),
        (":user::sym-unquote-fired", "THE CURE: the symbol-spelled marker fires"),
        (":user::kw-splice-fired", "control: the keyword splice fires"),
        (":user::sym-splice-fired", "THE CURE: the symbol-spelled splice fires"),
        (
            ":user::kw-not-a-marker-stayed-data",
            "NON-VACUITY: a near-miss head is DATA, keyword-spelled",
        ),
        (
            ":user::sym-not-a-marker-stayed-data",
            "NON-VACUITY: and symbol-spelled — the cure fires on the MARKER, not on \"a symbol\"",
        ),
    ];
    for (entry, why) in props {
        if !holds(entry) {
            wrong.push(format!("  {entry} → false, want true — {why}"));
        }
    }

    assert!(
        wrong.is_empty(),
        "the quasiquote markers answered wrongly on {} row(s):\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
