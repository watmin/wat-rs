//! Arc 170 slice 3 Gap H — probes for closure-extraction prelude-lift.
//!
//! These probes confirm that `extract_closure` lifts leading
//! `define`/`struct`/`enum` forms from a fn body's `do`-prefix INTO the
//! closure's prologue, so that the child's `startup_from_forms` registers
//! them (step 6) before the body is evaluated. Without the lift, the child
//! exits non-zero because `eval_do_tail` encounters `define` at expression
//! position and returns `DefineInExpressionPosition`.
//!
//! ## Why this matters
//!
//! Gap G (commit `021884a`) blocked Path E of `deftest-hermetic` because
//! prelude `define` forms inside a fn body's `do` cannot be evaluated at
//! child runtime. Gap H resolves that by lifting them UPSTREAM (before eval
//! ever sees them), preserving the single mental model "define = top-level
//! registration."
//!
//! ## Probe structure
//!
//! Each probe:
//!   1. Loads its co-suffixed fixture file via startup_from_file.
//!   2. Evaluates `(:my::launch)` in the frozen world.
//!   3. Forks the child, waits for it to exit, asserts exit code 0.
//!
//! Before Gap H: all probes fail (child exits non-zero, `DefineInExpressionPosition`).
//! After Gap H: all probes pass (lifted forms registered via prologue startup).
//!
//! ## The 5 probes
//!
//! 1. `define` in fn body do-prefix lifts to prologue
//! 2. `struct` in fn body do-prefix lifts to prologue
//! 3. `enum` in fn body do-prefix lifts to prologue
//! 4. mixed prelude (struct + enum + define) all lift in order
//! 5. prefix-termination semantics: only LEADING prelude forms lift
//!
//! Wat source: tests/function/probe_closure_body_prelude_lift_tN.wat (one per probe).

use wat::freeze::startup_from_file;
use wat::runtime::Value;

// ─── helpers ────────────────────────────────────────────────────────────────

fn freeze_ok(fixture: &str) -> wat::freeze::FrozenWorld {
    match startup_from_file(fixture) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Apply `(:my::launch)` in the frozen world and return the i64 the child
/// computed from its top-level declarations and sent back over the peer wire.
///
/// Arc 278 IPC de-prime — the old form field-poked the concrete `Process`
/// struct (`fields[2]` stderr, `fields[3]` handle → exit code), an
/// observation model the opaque `Process'` peer has no analog for. The peer
/// model observes the same thing more strongly: the child `println`s the
/// value it derived from the declaration under test, and the parent reads it
/// back via `recv'`. A registration failure now surfaces as a `Lost` cause
/// carrying the child's real reason, not a bare non-zero exit code.
fn run_launch(world: &wat::freeze::FrozenWorld) -> i64 {
    let launcher = world.symbols().get(":my::launch").expect("launch defined");
    let result = wat::runtime::apply_function(
        launcher.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect(":my::launch runs (spawn-program' + recv')");
    match result {
        Value::i64(n) => n,
        other => panic!("expected i64 from launch; got {:?}", other),
    }
}

// ─── Probe 1 — define in fn body do-prefix lifts to prologue ─────────────────

/// A `defn` form at the head of a fn body's `do` (via spawn-process forms)
/// lives at program top-level; the child's `startup_from_forms` registers it at
/// step 6. The body then calls the declared helper via let-binding.
///
/// Stone 241.12 — migrated from `:wat::core::define` to `:wat::core::defn`.
#[test]
fn probe_define_in_fn_body_do_prefix_lifts_to_prologue() {
    // Arc 170 slice 6 — under the new spawn-process program shape, the
    // prelude declarations sit at the program's TOP LEVEL alongside
    // :user::main. The "lift" mechanism that pre-slice-6 moved
    // declarations from the fn body's do-prefix to the closure prologue
    // is retired; the natural shape replaces it (declarations live at
    // their natural top-level position from the start).
    let world = freeze_ok("tests/function/probe_closure_body_prelude_lift_t1.wat");
    assert_eq!(
        run_launch(&world),
        42,
        "child should compute 42 via the top-level :h::helper (defn in do-prefix registered)"
    );
}

// ─── Probe 2 — struct in fn body do-prefix lifts to prologue ─────────────────

/// A `struct` declaration at the head of a fn body's `do` lifts into the
/// prologue.
#[test]
fn probe_struct_in_fn_body_do_prefix_lifts_to_prologue() {
    // Arc 170 slice 6 — struct sits at program top-level via spawn-process's
    // program shape (no lift required; the natural shape supersedes it).
    let world = freeze_ok("tests/function/probe_closure_body_prelude_lift_t2.wat");
    assert_eq!(
        run_launch(&world),
        7,
        "child should compute 3+4 via the top-level :h::LocalPoint (struct registered, accessors resolve)"
    );
}

// ─── Probe 3 — enum in fn body do-prefix lifts to prologue ───────────────────

/// An `enum` declaration at the head of a fn body's `do` lifts into the
/// prologue.
#[test]
fn probe_enum_in_fn_body_do_prefix_lifts_to_prologue() {
    // Arc 170 slice 6 — enum at program top-level.
    let world = freeze_ok("tests/function/probe_closure_body_prelude_lift_t3.wat");
    assert_eq!(
        run_launch(&world),
        1,
        "child should match :h::LocalDir::North → 1 (enum registered, variants construct + match)"
    );
}

// ─── Probe 4 — mixed prelude (struct + enum + define) all lift in order ──────

/// A mixed prelude — struct, then enum, then define — at the head of a fn
/// body's `do`. All three lift into the prologue in order.
#[test]
fn probe_mixed_prelude_lift() {
    // Arc 170 slice 6 — mixed prelude (struct + enum + define) all live
    // at program top-level via the new spawn-process program shape.
    let world = freeze_ok("tests/function/probe_closure_body_prelude_lift_t4.wat");
    assert_eq!(
        run_launch(&world),
        100,
        "child should compute 99 (struct via factory) + 1 (enum match) — all three declarations registered in order"
    );
}

// ─── Probe 5 — prefix-termination semantics ──────────────────────────────────

/// Only LEADING prelude forms lift into the prologue.
#[test]
fn probe_prelude_prefix_terminates_at_first_expression() {
    // Arc 170 slice 6 — the prefix-termination semantics retire under
    // the new substrate: declarations sit at program top-level naturally
    // and there is no "prefix" concept.
    let world = freeze_ok("tests/function/probe_closure_body_prelude_lift_t5.wat");
    assert_eq!(
        run_launch(&world),
        7,
        "child should compute 7 via the top-level :h::counted-helper (prefix-terminating defn registered)"
    );
}
