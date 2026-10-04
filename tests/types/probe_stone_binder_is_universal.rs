//! STONE-the-binder-must-be-universal (arc 109) — the call-site `:- […]` binder must work on
//! ALL ten root-level substrate `eval-…!` forms, not just the two that carry `type_params`.
//!
//! Baseline (measured before the fix, `72c0334c4`): `(:wat::eval-ast! :- [:wat::core::i64] …)`
//! failed at runtime with `"(:wat::eval-ast! <ast-value>) takes exactly 1 argument; got 3"` even
//! though the call type-checked clean — the dispatch cluster's ten thin arms
//! (`src/runtime.rs`, the `":wat::eval-…!" => eval_form_…(args, …)` arms) handed the RAW args
//! (still carrying `:-` and the `[…]` vector) straight to each helper, and each helper counts
//! `args.len()` for itself. Fixed by peeling the binder ONCE, at the dispatch cluster, before
//! the ten-way match — never inside the helpers.
//!
//! Four rows (`BRIEF-STONE-the-binder-must-be-universal.md`):
//! 1. generic form + non-empty binder — the load-bearing row, the one that failed before.
//! 2. non-generic form + EMPTY binder — must behave exactly like no binder at all.
//! 3. no binder at all — no-regression control for row 2.
//! 4. a second (non-generic) form repeating row 2's shape — proves the fix is structural
//!    (applies to every form reached through the dispatch cluster), not a one-armed patch.
//!
//! Wat source lives in the co-located fixture: probe_stone_binder_is_universal.wat (slurped
//! via startup_beside(file!()), the `wat_eval_result.wat` idiom — no inlined wat forms here).

use wat::runtime::{apply_function, Value};

fn run(world: &wat::freeze::FrozenWorld, fn_name: &str) -> Value {
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name:?} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute should run (a MalformedForm/arity refusal would panic HERE, at eval, \
                 not at startup — this is where the pre-fix defect fired)")
}

/// Row 1 — generic form (`:wat::eval-ast!`), NON-empty binder (`:- [:wat::core::i64]`).
/// Load-bearing: this is the exact shape that failed pre-fix with "takes exactly 1
/// argument; got 3". Must evaluate and return `Ok(Value::i64(42))`.
#[test]
fn row1_generic_form_with_binder_evaluates_and_returns_typed_result() {
    let world = wat::freeze::startup_beside(file!()).expect("startup");
    match run(&world, ":t::row1_generic_binder") {
        Value::Result(r) => match &*r {
            Ok(Value::i64(42)) => { /* T bound to i64 at the call site; runtime value matches */ }
            other => panic!("expected Ok(Value::i64(42)); got {other:?}"),
        },
        other => panic!("expected Value::Result; got {other:?}"),
    }
}

/// Rows 2 + 3 — non-generic form (`:wat::eval-edn!`, `type_params: vec![]`), empty binder
/// vs. no binder. `:- []` peels to `Some(&[])`, never `None` (arc 109's own ruling: absent
/// ≡ the empty binder). Both calls must produce byte-identical results.
#[test]
fn row2_and_row3_empty_binder_is_identical_to_no_binder() {
    let world = wat::freeze::startup_beside(file!()).expect("startup");
    let with_binder = run(&world, ":t::row2_empty_binder");
    let without_binder = run(&world, ":t::row3_no_binder");
    // Value has no PartialEq (rune:sequi — see probe_runtime_error_one_door.rs for the same
    // idiom); compare via Debug formatting, which is structural for this value shape.
    assert_eq!(
        format!("{with_binder:?}"),
        format!("{without_binder:?}"),
        "an empty `:- []` binder must be indistinguishable from no binder at all"
    );
    // Non-vacuity: both sides must actually be the Ok row (the edn parse of "42"), not two
    // errors that happen to format identically.
    match &with_binder {
        Value::Result(r) => assert!(
            (**r).is_ok(),
            "expected Ok(_); got {with_binder:?}"
        ),
        other => panic!("expected Value::Result; got {other:?}"),
    }
}

/// Row 4 — repeat the row-2 shape (non-generic + empty binder ≡ absent) on a SECOND form:
/// `:wat::eval-digest-string!`, one of the five forms whose arity message lives in a shared
/// helper (`eval_form_digest_shared`) rather than inline in the per-verb function body, so a
/// literal-message grep does not surface it. Proves the fix is structural — every form
/// reached through the dispatch cluster is covered, not only the ones easy to find by
/// grepping the error text.
/// The `(class, message)` of a `Value::Result(Err(Record(Failure)))`'s `error` field —
/// excursus 003 strike E: `Failure` also carries `frames` (the RAISING call's own
/// activation and `:location`), which legitimately DIFFER between row4's two wrapper
/// functions (`:t::row4_empty_binder_second_form` vs `:t::row4_no_binder_second_form` are
/// two different functions at two different source lines, so the raise's own frame and
/// derived `:location` differ by construction — that is correct, not a regression). The
/// binder-equivalence this test actually claims is about the verification OUTCOME, so the
/// comparison below is scoped to the error's class and message, not the whole Failure.
fn error_class_and_message(v: &Value) -> (String, String) {
    match v {
        Value::Result(r) => match &**r {
            Err(Value::Aggregate(failure)) if failure.class.as_ref() == "wat::kernel::Failure" => {
                match failure.fields.first() {
                    Some(Value::Aggregate(error)) => {
                        let message = match error.fields.first() {
                            Some(Value::String(s)) => (**s).clone(),
                            other => panic!("error.message not String; got {:?}", other),
                        };
                        (error.class.to_string(), message)
                    }
                    other => panic!("Failure.error not Aggregate; got {:?}", other),
                }
            }
            other => panic!("expected Err(Record(Failure)); got {:?}", other),
        },
        other => panic!("expected Value::Result; got {:?}", other),
    }
}

#[test]
fn row4_second_form_empty_binder_is_also_identical_to_no_binder() {
    let world = wat::freeze::startup_beside(file!()).expect("startup");
    let with_binder = run(&world, ":t::row4_empty_binder_second_form");
    let without_binder = run(&world, ":t::row4_no_binder_second_form");
    assert_eq!(
        error_class_and_message(&with_binder),
        error_class_and_message(&without_binder),
        "an empty `:- []` binder on eval-digest-string! must be indistinguishable from no binder \
         (compared by the error's class + message — `:location`/`:frames` legitimately differ \
         between the two wrapper functions' own call sites)"
    );
    // Non-vacuity: the fixture's hash is deliberately wrong, so both sides must be the SAME
    // verification-failed Err, not two things that happen to Debug-format the same nil.
    match &with_binder {
        // `Value::Result` is wat's OWN `:wat::core::Result` value (Ok/Err over `Value`, not a
        // Rust `Result<_, StartupError>`) — `eval-digest-string!` catches its `RuntimeError` and
        // carries it as the real `#wat.kernel/Failure {:error :frames :frames-elided}` Value
        // (excursus 003 strike E: `runtime_error_failure`, retiring the flattened
        // `#wat.core/EvalError {:kind :message}`). `Failure/error`'s own declared record CLASS
        // is now the stable discriminant (never a hand-maintained `kind` string); a bare
        // `is_err()` here is satisfied by ANY Failure, not just the deliberate hash mismatch
        // this test claims.
        Value::Result(r) => match &**r {
            Err(Value::Aggregate(failure))
                if failure.class.as_ref() == "wat::kernel::Failure"
                    && matches!(
                        failure.fields.first(),
                        Some(Value::Aggregate(error))
                            if error.class.as_ref() == "wat::runtime::EvalVerificationFailed"
                    ) => {}
            other => panic!(
                "expected Err(Failure{{error: EvalVerificationFailed, ..}}) (deliberate hash \
                 mismatch); got {:?}",
                other
            ),
        },
        other => panic!("expected Value::Result; got {other:?}"),
    }
}
