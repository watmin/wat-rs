//! GREEN gate: the rete surface admits only `reduce`'s TOTAL arity.
//!
//! **The contradiction this closes.** `wat/seq.wat:317-329` defines `:wat::core::reduce` in two
//! clauses: the 3-arity form is literally `(:wat::core::foldl f init coll)`, and the 2-arity form
//! seeds the fold from the first element and RAISES BY NAME on an empty collection. The rete row
//! declares `total: true` — which every row must, by `vocabulary.rs`'s `every_rete_row_is_total`,
//! on the builder's ruling that *"every rete form MUST be total… a jump table over a partial op is
//! not a thing"*. The 2-arity form and that declaration cannot both stand.
//!
//! **Why REFUSE rather than make the row `Fallback`.** The table's own comment already ruled how a
//! partial core op earns a rete surface: not by weakening the wall, but by BUYING totality with a
//! mandatory `:undefined` — which is exactly why partial `i64::/` is `total: true` there. But
//! `Fallback` is a property of the ROW, so taking it would force that ceremony onto the 3-arity
//! form, which is already total and needs nothing. Refusing the partial arity is the narrower
//! reading of the same doctrine, and it keeps rete's surface narrower than core's for the reason it
//! always is — per-type comparators, eager materializers, and now total arities only.
//!
//! **Provenance, and it is the point of the § 4.1 ledger.** The partiality is not new. It was
//! UNREACHABLE until `exec_reduce` landed on 2026-08-28 — before that the row could not execute at
//! all, so it could not raise either. Making the row runnable is what turned a latent false
//! declaration into a live one, and driving it is what showed the declaration was false. No
//! reading of the table could have: the row said `total: true` and every gate agreed.
//!
//! **WHAT EACH ROW PROVES**
//!
//! | row | asserts | gate |
//! |---|---|---|
//! | `the_partial_two_arity_form_is_refused` | 2-arity is refused, and the refusal TEACHES | `expect_err` + names the fix |
//! | `the_total_three_arity_form_still_fires` | the admitted arity is untouched | count == 1 |
//!
//! The second row is not decoration. The two fixtures are byte-identical but for the `init`
//! operand, so without it this probe would pass just as happily against a change that refused
//! `reduce` outright — an over-refusal reads exactly like a correct one when only the refusal is
//! checked.
//!
//! Run: cargo nextest run --release -E 'test(probe_arc278_reduce_arity_totality)'

use wat::freeze::{startup_from_file, FrozenWorld};
use wat::runtime::{apply_function, Value};

const TWO_ARITY: &str = "tests/rete/probe_arc278_reduce_arity_totality_two.wat";
const THREE_ARITY: &str = "tests/rete/probe_arc278_reduce_arity_totality_three.wat";

/// Load a fixture and run `:probe::run`, returning the derived-fact count.
///
/// A refusal can land at EITHER boundary and both are the same answer to "may a user write this":
/// rule validation runs at freeze, while the lowering fence raises at rule-compile time (inside
/// `compile-all`, which is why this one arrives as a panic out of `apply_function`). The caller
/// must not care which — only that the form did not become a live rule.
fn run_fixture(path: &str) -> Result<i64, String> {
    let world: FrozenWorld = match startup_from_file(path) {
        Ok(w) => w,
        Err(e) => return Err(format!("{e:?}")),
    };
    let Some(func) = world.symbols().get(":probe::run").cloned() else {
        return Err("no entry fn :probe::run".to_string());
    };
    let sym = world.symbols();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, wat::rust_caller_span!())
    })) {
        Ok(Ok(Value::i64(n))) => Ok(n),
        Ok(Ok(other)) => Err(format!("expected an i64 count; got {other:?}")),
        Ok(Err(e)) => Err(format!("{e:?}")),
        Err(payload) => {
            if let Some(p) = payload.downcast_ref::<wat::assertion::AssertionPayload>() {
                Err(p.message.clone())
            } else if let Some(s) = payload.downcast_ref::<String>() {
                Err(s.clone())
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                Err((*s).to_string())
            } else {
                Err("panic-opaque".to_string())
            }
        }
    }
}

/// The partial arity is refused, and the refusal TEACHES.
///
/// R29 `RVINA ERVDIT` — the ruin is the lesson. A diagnostic that merely says "no" leaves the
/// author guessing which of the two clauses they wanted, so the message is asserted to name the
/// admitted spelling. Substring rather than an exact match deliberately: the wording belongs to
/// whoever raises it, and only the load-bearing half is pinned here.
#[test]
fn the_partial_two_arity_form_is_refused() {
    let msg = run_fixture(TWO_ARITY)
        .expect_err("the 2-arity `reduce` is partial on an empty collection and must not compile");
    // rune:lint(loose-assert) — the refusal is a MalformedForm blob embedding a Span path; pin the
    // TEACHING half (the admitted spelling), not the whole blob. Unlike the `DefectKind` case in
    // `rete/reachability.rs`, there is no structured field to assert instead: R29 makes the
    // lesson prose ON PURPOSE, and a golden over it would pin another author's wording.
    assert!(
        msg.contains("3-arity"),
        "the refusal must name the admitted spelling so the author knows what to write; got:\n{msg}"
    );
    // rune:lint(loose-assert) — same blob, same reason; this half pins that the diagnostic NAMES
    // the offending op, which is the other thing R29 requires of a refusal.
    assert!(
        msg.contains("reduce"),
        "the refusal must name the offending op; got:\n{msg}"
    );
}

/// ⛔ THE CONTROL — the admitted arity is untouched.
///
/// Without this the probe above passes against a change that refuses `reduce` in every form. An
/// over-refusal and a correct refusal are indistinguishable when only the refusal is measured, and
/// this arc has shipped that mistake before: a termination verifier once refused a legal
/// fn-headed `:then` and the floor priced it at two tests.
#[test]
fn the_total_three_arity_form_still_fires() {
    assert_eq!(
        run_fixture(THREE_ARITY),
        Ok(1),
        "the 3-arity `reduce` is total and must keep firing — the two fixtures differ ONLY in the \
         `init` operand, so if this fails the refusal is too wide"
    );
}
