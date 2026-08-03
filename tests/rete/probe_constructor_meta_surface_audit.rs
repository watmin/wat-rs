//! BRIEF-constructor-meta-audit.md — `constructor_meta`'s two return sites (`src/rete/purity.rs`),
//! audited: `b98cf189` classified the EXPANDED constructor verbs (`aggregate-new` /
//! `kwargs-construct`) `pure ∧ deterministic ∧ total` and named, without touching, that
//! `constructor_meta` still ruled the SURFACE form (a bare `(:T arg…)` / `(:Enum::Variant arg…)`
//! written directly inside a quoted `:then`/`:when` item, never macro-expanded there) `total:
//! false` on undocumented discipline, and derived `pure` from the target's declared purity
//! marker rather than from the act of construction.
//!
//! Three fixtures, three claims:
//!
//!   - `probe_constructor_meta_surface_pure_green.wat` — GREEN: the `pure` flip. A `defstruct`
//!     with only a plain `:wat::core::i64` field, constructed via its bare surface form directly
//!     in a `:then` item, now compiles AND fires — it used to be refused unconditionally on
//!     Pure grounds regardless of what the struct actually held.
//!   - `probe_constructor_meta_surface_total_aggregate.wat` — `total` STAYS false (aggregate
//!     site): a nested surface aggregate-constructor operand compiles clean (pure ∧ det both
//!     hold) and dies at FIRE time with `UnknownFunction` — the generic evaluator has no arm for
//!     a bare aggregate-type keyword outside the `:then` item's own specially-cased top-level
//!     shape.
//!   - `probe_constructor_meta_surface_total_enum.wat` — `total` STAYS false (enum-variant
//!     site), a DIFFERENT failure mode: a wrong-arity nested enum-variant constructor call
//!     compiles clean and dies at FIRE time with a clean, located `ArityMismatch` — no
//!     freeze-time wall (analogous to `validate_and_reorder_then`, which only resolves
//!     `TypeDef::Aggregate` heads) validates a bare `:Enum::Variant` call's arity ahead of time.
//!
//! Run: cargo test --release -p wat --test rete probe_constructor_meta_surface_audit

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

const WORLD_PURE_GREEN: &str = "tests/rete/probe_constructor_meta_surface_pure_green.wat";
const WORLD_TOTAL_AGGREGATE: &str = "tests/rete/probe_constructor_meta_surface_total_aggregate.wat";
const WORLD_TOTAL_ENUM: &str = "tests/rete/probe_constructor_meta_surface_total_enum.wat";

/// Run `:user::run` (zero-arg, declared `-> :wat::core::i64`) and return its result, or an `Err`
/// string for either an ordinary raise OR the fence's `Option/expect` compile-time panic — the
/// same dual capture `probe_construction_headline.rs::run` uses, since a regression on the
/// `pure` fix would surface as a PANIC during `(:wat::rete::compile rules)`, not a clean `Err`.
fn run(world_path: &str) -> Result<Value, String> {
    let world = startup_from_file(world_path).map_err(|e| format!("startup: {e:?}"))?;
    let func = world.symbols().get(":user::run").unwrap_or_else(|| panic!("no entry fn :user::run in {world_path}")).clone();
    let sym = world.symbols();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, wat::rust_caller_span!())
    })) {
        Ok(res) => res.map_err(|e| format!("eval: {e:?}")),
        Err(panic_payload) => {
            if let Some(s) = panic_payload.downcast_ref::<String>() {
                Err(s.clone())
            } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                Err((*s).to_string())
            } else {
                Err("panic-opaque".to_string())
            }
        }
    }
}

/// PURE flip: a `Nature::Struct` constructed via its bare surface form directly in a `:then`
/// item now compiles (no axis-violation panic during `compile-rule`) AND fires, returning the
/// value carried through the struct's only field.
#[test]
fn struct_surface_constructor_now_admitted_pure() {
    let r = run(WORLD_PURE_GREEN);
    assert!(matches!(r, Ok(Value::i64(5))), "expected label=5 via the newly-admitted surface struct constructor; got {r:?}");
}

/// `total` stays false, aggregate site: a nested surface aggregate constructor operand compiles
/// clean and aborts at fire time with `UnknownFunction` — the reachable path that keeps this
/// site `false` rather than defaulted.
#[test]
fn nested_surface_aggregate_constructor_compiles_clean_then_dies_unknown_function() {
    // Startup (parse + `--check`-equivalent + freeze, including `validate_and_reorder_then`)
    // must succeed — the gap is NOT a checker rejection, it is a silent admission.
    let world = startup_from_file(WORLD_TOTAL_AGGREGATE).unwrap_or_else(|e| {
        panic!("expected clean startup (the gap is a fire-time surprise, not a checker rejection); got {e:?}")
    });
    let func = world.symbols().get(":user::run").expect("no entry fn :user::run").clone();
    let sym = world.symbols();
    let err = apply_function(func, vec![], sym, wat::rust_caller_span!())
        .expect_err("a nested bare aggregate-constructor operand must fail at fire time — the surface form has no dispatch for it outside a :then item's own top-level shape");
    let msg = format!("{err:?}");
    // rune:lint(loose-assert) — the Debug rendering embeds an absolute file path (Span),
    // non-deterministic across machines/CI (same reason probe_construction_headline.rs's own
    // arity-check probe uses `contains`, not `assert_eq!`).
    assert!(msg.contains("UnknownFunction"), "must be UnknownFunction, got:\n{msg}");
    // rune:lint(loose-assert) — same span/path reason as the line above.
    assert!(msg.contains(":cg::Inner"), "must name the offending nested constructor head, got:\n{msg}");
}

/// `total` stays false, enum-variant site — a DIFFERENT failure mode than the aggregate site's:
/// a wrong-arity nested enum-variant constructor compiles clean and aborts at fire time with a
/// clean, located `ArityMismatch`.
#[test]
fn nested_surface_enum_variant_constructor_compiles_clean_then_dies_arity_mismatch() {
    let world = startup_from_file(WORLD_TOTAL_ENUM).unwrap_or_else(|e| {
        panic!("expected clean startup (the gap is a fire-time surprise, not a checker rejection); got {e:?}")
    });
    let func = world.symbols().get(":user::run").expect("no entry fn :user::run").clone();
    let sym = world.symbols();
    let err = apply_function(func, vec![], sym, wat::rust_caller_span!())
        .expect_err("a wrong-arity nested enum-variant constructor call must fail at fire time — no freeze-time wall validates a bare :Enum::Variant call's arity");
    let msg = format!("{err:?}");
    // rune:lint(loose-assert) — same span/path reason as the aggregate probe above.
    assert!(msg.contains("ArityMismatch"), "must be an ArityMismatch, got:\n{msg}");
    // rune:lint(loose-assert) — same span/path reason as the line above.
    assert!(msg.contains(":cg::Status::Active"), "must name the offending callee, got:\n{msg}");
    // rune:lint(loose-assert) — same span/path reason as the line above.
    assert!(msg.contains("expected 1") && msg.contains("got 3"), "must name the actual/expected arity, got:\n{msg}");
}
