//! Arc 251 stone 251.8d-ii (SEVENTH draw) — **the `:then` item fence reads one IDENTITY, and so
//! does the lowering behind it.**
//!
//! ## What the fence actually refused, and it was never a keyword-spelled form
//!
//! The sixth draw's five NEW converted-floor failures all carried one arm:
//!
//! ```text
//!   compile-condition: then expr is not a rete primitive —
//!     ':wat::core::kwargs-construct' is not a rete primitive; a then admits only :wat::rete:: ops
//! ```
//!
//! ⛔ **The keyword spelling in that message is the ERROR's, not the SOURCE's.** When
//! `rete/purity.rs::is_declaration_derived_construction` declines, the walk falls to
//! `classify_expr`'s general list arm, which **already** re-spells a `Symbol` head through
//! `canonical_identity` before it builds the `AxisViolation`. So a `:then` item written
//! `(wat.core/kwargs-construct :T …)` is refused under the name `:wat::core::kwargs-construct`,
//! and every reader of that log — including three briefs — read a symbol-spelled refusal as a
//! keyword-spelled one.
//!
//! `wat/Record.wat:207`, `wat/Record.wat:296` and `wat/core.wat:2078` are the three templates
//! that emit the verb (`` `(:wat::core::kwargs-construct ~_kc-type ~@call-args) ``). Converted,
//! all three emit a `Symbol` head — which is why every `:then` holding a nested constructor
//! joined the fence class at once.
//!
//! ## Two walls, one language fact — and the first cure only moved the error one frame
//!
//! Measured here, on three binaries:
//!
//! | binary | `:user::sym-ctor` |
//! |---|---|
//! | HEAD | ⛔ `then expr is not a rete primitive — ':wat::core::kwargs-construct'` (the FENCE) |
//! | `is_declaration_derived_construction` cured alone | ⛔ `malformed :wat::rete::lower form: call head must be a keyword` (the LOWERING) |
//! | both cured | ✅ derives 1 |
//!
//! The fence and `rete/expr_ir::lower_list` have to agree about what a head IS, or the `:then`
//! surface has two answers to one question. Both now read through `canonical_identity`.
//!
//! ## The rows — ONE test, because the claim is about their JOINT behaviour
//!
//! | entry | want | what it pins |
//! |---|---|---|
//! | `kw-ctor` | 1 | ⛔ control — the keyword spelling may not move |
//! | ⭐ `sym-ctor` | 1 | **THE CURE** — a symbol-spelled `kwargs-construct` VERB |
//! | ⭐ `sym-type-ctor` | 1 | **THE CURE** — a symbol-spelled TYPE in argument 0 (a second read) |
//! | ⭐ `sym-head-and-type` | 1 | both at once, which is what a converted template emits |
//! | ⛔ `kw-computation` | REFUSED | **LAW A HOLDS** — core-spelled computation in a `:then` |
//! | ⛔ `sym-computation` | REFUSED | ⭐ **LAW A HOLDS IN THE OTHER SPELLING TOO**, same located reason |
//!
//! ⛔ **Rows 5 and 6 are the stone.** This is the fourth consecutive cure in the permissive
//! direction and the subject is a WALL: if widening the spelling had widened the POPULATION,
//! `sym-computation` would compile. It does not, and it is refused with the SAME message and the
//! SAME `wat/rete/compile.wat` line as its keyword twin — the wall moved zero millimetres.
//!
//! ⚠ The tightness rows the door itself owns — *the verb alone is not enough; argument 0 must
//! resolve to a declared aggregate, in either spelling* — cannot be written here: an undeclared
//! type in a `:then` operand is caught EARLIER by `rete/validate`'s
//! `RhsOperandTypeMismatch` at freeze, which kills the whole fixture rather than one row
//! (measured). They live beside the door, in `src/rete/purity.rs`'s
//! `mod declaration_derived_identity_tests`.
//!
//! Run: `cargo nextest run --release -E 'test(then_item_identity)'`

use wat::assertion::AssertionPayload;
use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

const FIXTURE: &str = "tests/rete/probe_arc251_8d_then_item_identity.wat";

/// Compile+fire the fixture's zero-arg entry fn. The `:then` fence rejects by PANICKING
/// (`Option/expect` → `panic_any(AssertionPayload)`), so catch the unwind and pull the human
/// message back out — never just "did it reject".
fn outcome(entry: &str) -> Result<i64, String> {
    let world = startup_from_file(FIXTURE)
        .unwrap_or_else(|e| panic!("the fixture must freeze: {e:?}"));
    let func = world
        .symbols()
        .get(entry)
        .unwrap_or_else(|| panic!("no entry fn {entry:?} in {FIXTURE}"))
        .clone();
    let sym = world.symbols();
    let prior = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, wat::rust_caller_span!())
    }));
    std::panic::set_hook(prior);
    match caught {
        Ok(Ok(Value::i64(n))) => Ok(n),
        Ok(Ok(other)) => Err(format!("{entry} answered {other:?}, not an i64")),
        Ok(Err(e)) => Err(format!("{e:?}")),
        Err(payload) => Err(match payload.downcast_ref::<AssertionPayload>() {
            Some(p) => p.message.clone(),
            None => payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_else(|| "panic-opaque".to_string()),
        }),
    }
}

/// ⭐ THE WHOLE TABLE IN ONE TEST — a per-row test would let a cure that simply disarmed the
/// fence pass four of the six. Asserted as a table so a failure prints every wrong row.
#[test]
fn the_then_item_fence_reads_one_identity_and_law_a_still_refuses_both_spellings() {
    // ── tier 1: a legitimate constructor in a `:then` compiles, in every spelling ────────────
    let admits: &[(&str, &str)] = &[
        (":user::kw-ctor", "control: the keyword spelling may not move"),
        (":user::sym-ctor", "THE CURE: a symbol-spelled kwargs-construct VERB"),
        (":user::sym-type-ctor", "THE CURE: a symbol-spelled TYPE in argument 0"),
        (
            ":user::sym-head-and-type",
            "THE CURE: both at once — what a converted Record.wat template emits",
        ),
    ];
    // ── tier 2: LAW A. Core-spelled computation is still refused, in every spelling ──────────
    let refuses: &[(&str, &str)] = &[
        (":user::kw-computation", "LAW A: core-spelled computation, keyword"),
        (
            ":user::sym-computation",
            "LAW A IN THE OTHER SPELLING: the cure widened the spelling, not the population",
        ),
    ];
    const LAW_A: &str = "is not a rete primitive";
    // ⚠ NOT `"':wat::core::if'"` — the `'` is the reader's QUOTE sugar, so that literal parses
    // as `(:wat::core::quote :wat::core::if')`, a list with a keyword head, and
    // `no_inlined_wat_in_tests` correctly convicted it (it did, on this stone's first floor).
    // The bare identity is not a list and pins the same fact: the refusal must NAME the head.
    const NAMED: &str = ":wat::core::if";

    let mut wrong: Vec<String> = Vec::new();

    for (entry, why) in admits {
        match outcome(entry) {
            Ok(1) => {}
            Ok(n) => wrong.push(format!("  {entry} → derived {n}, want 1 — {why}")),
            Err(e) => wrong.push(format!("  {entry} → REFUSED, want 1 derived — {why}\n      {e}")),
        }
    }
    for (entry, why) in refuses {
        match outcome(entry) {
            Ok(n) => wrong.push(format!(
                "  {entry} → derived {n}, WANT A REFUSAL — {why}\n      ⛔ LAW A WAS DISARMED"
            )),
            Err(e) => {
                if !e.contains(LAW_A) {
                    wrong.push(format!("  {entry} → refused, but not on law A — {why}\n      {e}"));
                } else if !e.contains(NAMED) {
                    wrong.push(format!(
                        "  {entry} → refused on law A but did not NAME {NAMED} — {why}\n      {e}"
                    ));
                }
            }
        }
    }

    assert!(
        wrong.is_empty(),
        "the :then item fence answered wrongly on {} row(s):\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
