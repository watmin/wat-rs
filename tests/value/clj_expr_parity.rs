//! THE clj-EXPRESSIVENESS EQUALITY MATRIX — `clojure.core` is the oracle; non-parity is a flaw.
//!
//! For every row in `clj_expr_oracle/corpus.txt` (a faithful `wat.core/…` expression), the wat
//! runtime's evaluated result — rendered as canonical EDN — must equal what `clojure.core` evals
//! + `pr-str`s (baked in `clj_expr_oracle/golden.txt`, so this runs WITHOUT clj). clj's `pr-str` is
//!   TYPE-DISCRIMINATING (1 / 1N / 1.0 / 1/2 print distinctly), so a string-equal render carries
//!   VALUE *and* TYPE parity in one comparison — the whole "clojure on rust" claim, measured.
//!
//! Regenerate the oracle when the corpus grows (needs the `clojure` CLI):
//!   CORPUS=tests/clj_expr_oracle/corpus.txt GOLDEN=tests/clj_expr_oracle/golden.txt \
//!     clojure -M tests/clj_expr_oracle/regen.clj
//!
//! Directional obligation: a `clj:<edn> / wat:<edn>` row is a wat FLAW unless justified-exempt.
//! Grow the corpus until it stops finding divergences (loop-until-dry).

// rune:lint(no-inlined-wat) — the harness's whole job is evaluating an arbitrary `wat.core/…`
// expression PER CORPUS ROW (`clj_expr_oracle/corpus.txt`, 300+ rows, growing) against a
// clj-baked golden — the format!-wrapped expr and the `(:probe::e)` call stub are not a fixed
// wat program that could live in one co-located `.wat`; a genuinely dynamic driver, same shape
// as `probe_rational_C2_arithmetic.rs`'s exemption.
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use wat::edn::render::value_to_edn;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::runtime::Environment;

const GOLDEN: &str = include_str!("../clj_expr_oracle/golden.txt");

/// Eval one faithful `wat.core/…` expression through wat's FULL pipeline (check + eval) and render
/// its result as canonical EDN. Wrapping in a `-> :wat::core::Value` defn makes the harness
/// type-agnostic (Value is the universal subtype-top, arc 278 R7) AND runs the checker, so a
/// check-rejection surfaces as `:ERR` exactly like an eval error.
fn wat_eval_edn(expr: &str) -> String {
    let src = format!("(:wat::core::defn :probe::e [] -> :wat::core::Value {expr})");
    let world = match std::panic::catch_unwind(AssertUnwindSafe(|| {
        startup_from_source(&src, None, Arc::new(wat::load::loader::InMemoryLoader::new()))
    })) {
        Ok(Ok(w)) => w,
        Ok(Err(_)) => return ":ERR".to_string(), // parse / check rejected
        Err(_) => return ":PANIC".to_string(),
    };
    let call = wat::parse_one!("(:probe::e)").expect("parse the probe call");
    match std::panic::catch_unwind(AssertUnwindSafe(|| {
        match eval_in_frozen(&call, &world, &Environment::new()) {
            Ok(tv) => wat_edn::write(&value_to_edn(&tv.value_owned())),
            Err(_) => ":ERR".to_string(),
        }
    })) {
        Ok(s) => s,
        Err(_) => ":PANIC".to_string(),
    }
}

/// The ONLY allowed divergences — each with a load-bearing reason. Anything else must match clj.
fn exemption(expr: &str, clj: &str, _wat: &str) -> Option<&'static str> {
    // clj special forms are not `clojure.core` vars, so the namespace-swap makes the oracle THROW
    // (`if`/`let`/`fn`/`do`/`quote`). Not a wat flaw — a corpus-syntax nuance. These rows should be
    // rewritten bare on both sides; exempted here until the corpus does so.
    if clj == ":THROW" && (expr.contains("wat.core/if") || expr.contains("wat.core/let")) {
        return Some("clj special form is not a clojure.core var; namespace-swap THROWs — corpus nuance, rewrite bare");
    }
    None
}

#[test]
#[ignore = "WIP flaw-tracker (300 equality matrix) — RED by design (real divergences on the board); \
            un-ignore when axis-2 is wired + the fight-list is driven green. Kept #[ignore]'d so it doesn't \
            pollute other strikes' floor=0 gates (e.g. 118.2a)."]
fn wat_expr_matches_clj_oracle() {
    let mut rows = Vec::new();
    let mut fails = Vec::new();
    for line in GOLDEN.lines() {
        if line.is_empty() {
            continue;
        }
        let (clj, expr) = line.split_once('\t').expect("golden row must be RESULT\\tEXPR");
        let wat = wat_eval_edn(expr);
        let ok = wat == clj || exemption(expr, clj, &wat).is_some();
        let mark = if wat == clj {
            "  parity"
        } else if ok {
            "  EXEMPT"
        } else {
            "DIVERGE!"
        };
        rows.push(format!("  {mark}  {expr:40}  clj:{clj:16}  wat:{wat}"));
        if !ok {
            fails.push(format!("  {expr}\tclj:{clj}\twat:{wat}"));
        }
    }
    // The full head-to-head table — printed on failure so the flaws are visible in one read.
    assert!(
        fails.is_empty(),
        "\n\nclj-EXPRESSIVENESS parity — {} divergence(s) of {} rows (each a candidate FLAW):\n\n{}\n\n\
         --- full grid ---\n{}\n\nTriage each: fix wat to match clj, or add a justified exemption.\n",
        fails.len(),
        rows.len(),
        fails.join("\n"),
        rows.join("\n"),
    );
}
