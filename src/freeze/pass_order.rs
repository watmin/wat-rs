//! Stone 255.12, instrument debt B — **the startup pipeline's step ORDER, made
//! falsifiable.**
//!
//! ## Why this exists
//!
//! The order of the startup passes is the premise of a whole family of
//! dispositions written in this arc's SCOREs. The load-bearing sentence is:
//!
//! > *anything at or after step 7 (`normalize_symbol_refs`) cannot see a symbol
//! > head, because normalize has already rewritten Symbol → Keyword.*
//!
//! Three consecutive stones (255.9, 255.10, 255.11) reasoned from it, and until
//! now it was enforced by **nothing at all** — the step numbers live in `//`
//! comments in `freeze.rs` and `freeze/env.rs`. Move one call and every
//! "unreachable, it is post-step-7" row in every prior SCORE silently becomes
//! false, with no test going red. 255.9, 255.10 and 255.11 each recommended a
//! gate; this is it, on the third recommendation.
//!
//! ## What it is
//!
//! Each pass announces itself via [`record`]. Under `cfg(test)` the names
//! accumulate in a thread-local, and `freeze`'s own unit test asserts the
//! sequence against `EXPECTED_ORDER`.
//!
//! ## Excursus 003 D5 — the SAME call sites also carry opt-in phase timing
//!
//! Outside `cfg(test)`, `record` used to be a true no-op. It no longer is:
//! [`take_phase_durations`] answers "where does a freeze spend its time,
//! and which phase grows with declared records" (`AUDIT-the-shape-of-an-
//! error.md`'s Strike B worklist, the S2 cost row) by timestamping these
//! SAME announce points, in every build, when `WAT_FREEZE_PHASE_TIMING` is
//! set. The cost when unset is exactly one `OnceLock` read (a relaxed load
//! after first init) per `record` call — no allocation, no lock, no
//! `Instant::now()` — so a normal, unmeasured `wat` run is unaffected to
//! within noise. See `crate::freeze::take_freeze_phase_timings` for the
//! public accessor and exactly which steps this covers (steps 2 through
//! 7.7; step 8's own work and step 9, `FrozenWorld::freeze`'s body, are
//! the caller's own residual — this module has no announce point after 8).
//!
//! ## ⛔ WHAT THIS GATE CANNOT SEE — read before trusting it
//!
//! It pins the ORDER. It does **not** pin the thing that actually went wrong in
//! 255.9 and 255.10, which was the second half of the sentence above, not the
//! first: `normalize_symbol_refs` rewrites **CODE positions only**, so a DATA
//! position (a `quote` body, a quasiquote template, a `match` arm pattern, a
//! `make-rule`'s quoted `:when`/`:then`) still carries raw symbols at step 8 and
//! step 9. Both stones marked a live capability wall "unreachable" while the
//! order was exactly what they believed it was.
//!
//! Gating THAT would need a different instrument: a post-normalize assertion
//! that walks the residue and reports which positions still hold a
//! namespaced `WatAST::Symbol`, pinned as a census with a per-site disposition —
//! roughly the shape of `tests/lint/`'s existing walkers, and a stone of its own.
//! This module deliberately does not pretend to it.

/// The pipeline's pass order, as the steps announce themselves.
///
/// Names are the FOUNDATION.md step number plus the function that runs it, so a
/// diff of this array reads as a pipeline change rather than a string edit.
/// Stdlib-side passes are included where they are a distinct call: they register
/// ahead of the user-side pass of the same number, and that precedence is itself
/// load-bearing (a user form may not shadow a stdlib declaration).
#[cfg(test)]
pub(crate) const EXPECTED_ORDER: &[&str] = &[
    "2-collect-entry-file",
    "3-resolve-loads",
    "3b-extract-rete-defn-names",
    "4-register-stdlib-defmacros",
    "4-register-defmacros",
    "4-expand-all",
    "5-register-stdlib-types",
    "5-register-types",
    "6-register-stdlib-defines",
    "6-register-defines",
    "7-normalize-symbol-refs",
    "7-normalize-stored-function-bodies",
    "7-resolve-references",
    // ⚠ The pipeline calls `normalize_stored_function_bodies` a SECOND time, after
    // `resolve_references` and after step 7.7's extend-type pre-registration — the
    // first pass only saw functions registered before it. The FOUNDATION.md step
    // list does not mention it; this array is derived from what the passes actually
    // announce, so it does. That divergence is itself the argument for the gate.
    "7.7-normalize-stored-function-bodies",
    "8-check-program",
    // Excursus 003 D5 — the ONE new announce point this instrument adds to the pipeline
    // itself (everything else in D5 only changed what the EXISTING points do). Marks the
    // boundary `check_program` ⇒ `FrozenWorld::freeze`'s own body, so a phase-timing
    // reader can tell "check_program's own cost" (the `8-check-program` → `9-freeze`
    // delta) apart from "freeze()'s own body" (the residual after this — see
    // `crate::freeze::take_freeze_phase_timings`'s doc). `FrozenWorld::freeze` is called
    // from exactly one place in this pipeline (`startup_from_forms_post_config`, right
    // after the check-program match block below), so this fires exactly once per
    // successful freeze on this path.
    "9-freeze",
];

#[cfg(test)]
thread_local! {
    static TRACE: std::cell::RefCell<Vec<&'static str>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Excursus 003 D5 — is phase timing on for this process? Read from the environment
/// exactly ONCE (`OnceLock`), so every `record` call after the first pays a single
/// relaxed load, never a syscall. Off by default: a normal `wat` invocation never sets
/// `WAT_FREEZE_PHASE_TIMING`, so this is `false` for the process's whole life.
fn timing_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("WAT_FREEZE_PHASE_TIMING").is_some())
}

thread_local! {
    /// Excursus 003 D5 — per-thread phase-boundary timestamps, populated only when
    /// [`timing_enabled`] is true. A freeze that runs on a spawned thread (a sandboxed
    /// eval, a `:process`/`:thread` peer) carries its OWN trace, never mixed with its
    /// parent's — matching `TRACE` above and `USER_SOURCE_FILES`'s thread-local shape.
    static TIMING: std::cell::RefCell<Vec<(&'static str, std::time::Instant)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Announce that `step` is running. Under `cfg(test)`, also appends to `TRACE` (the
/// order gate). When [`timing_enabled`], also stamps an `Instant` into `TIMING` (the
/// Excursus 003 D5 phase-timing instrument — see the module doc). Neither is state a
/// normal, unmeasured `wat` run pays for beyond the one `OnceLock` read.
///
/// Excursus 003 strike D, item 4 — ALSO names `step` as the current activation
/// (`crate::value::frame::set_activation`), so a `RuntimeError` raised during this
/// phase carries an honest innermost-Rust-frame name instead of a placeholder. A
/// plain overwrite, unconditional — cheap (one `String` allocation per pass, a
/// handful of passes per freeze) and, unlike `TRACE`/`TIMING`, not gated behind
/// `cfg(test)`/`timing_enabled`: every build needs the name, not just a measured or
/// tested one.
///
/// ⛔ CORRECTED, excursus 003 strike D3 (GD2a, the 40-kind activation census) — this
/// comment used to claim `UserMainMissing`/`EvalVerificationFailed` as "the only two
/// live producers before any wat activation exists." Both halves of that claim were
/// false, measured directly rather than assumed:
/// - `UserMainMissing` does not fire HERE at all. It is raised by
///   `invoke_user_main_orchestrated` (`src/freeze.rs`) AFTER `FrozenWorld::freeze`
///   has already returned — its own raise site carries no `ActivationGuard` of its
///   own, so it merely INHERITS whatever this fn last announced (`"9-freeze"`, the
///   final step below), the same plain-overwrite name every other post-freeze raise
///   on this thread would inherit too.
/// - `EvalVerificationFailed`'s real, wat-reachable producer
///   (`:wat::eval-digest-string!`/`:wat::eval-signed-string!`, `src/runtime.rs`) is a
///   RUNTIME keyword dispatch, not a freeze-phase one — its activation is the
///   dispatcher's own guard (writer 1, the literal op spelling), never this fn's name
///   at all.
///
/// What this fn's name DOES genuinely reach, measured via GD2a's standing census
/// (`tests/diagnostics/probe_excursus003_d3_gd2a_census.rs` + sibling `.wat`/`.wat.bad`
/// fixtures): every registration-time kind that raises during step `6-register-defines`
/// (`UnnamespacedName`, `DottedName`, `ReservedPrefix`, `DuplicateDefine` — the extend-type
/// surface-collision shape, at step `7-resolve-references`'s window specifically —
/// and `UnreachableClause`), and every kind raised inside `FrozenWorld::freeze`'s own
/// body at step `9-freeze` (`ReteDefnAxisViolation`, `ReteDefnRecursive`, and — by
/// inheritance, after freeze returns — `UserMainMissing`). No count is stated as
/// exhaustive on purpose, for the same reason this section exists: a stale "only N"
/// claim here is exactly the failure mode this correction is annihilating.
#[inline]
pub(crate) fn record(step: &'static str) {
    #[cfg(test)]
    TRACE.with(|t| t.borrow_mut().push(step));

    crate::value::frame::set_activation(step);

    if timing_enabled() {
        TIMING.with(|t| t.borrow_mut().push((step, std::time::Instant::now())));
    }
}

/// Drain this thread's recorded phase-boundary timestamps into per-phase durations —
/// phase `i`'s duration is the wall-clock time from `record(i)` to `record(i+1)`, i.e.
/// "how long step i's own work took before the NEXT step announced itself". The very
/// LAST announced step's own duration (step 8, `check_program`) is therefore NOT
/// included, nor is anything after it (step 9, `FrozenWorld::freeze`'s body) — neither
/// has a following announce point in this module. A caller recovers both by bracketing
/// its OWN call to `startup_from_source`/`startup_from_forms*` with `Instant::now()`
/// and subtracting the sum of these durations from that outer total; see
/// `examples/freeze_phase_timing.rs`.
///
/// Empty unless [`timing_enabled`]. Clears the thread's trace on every call (including
/// when timing is off, which is always a no-op clear of an already-empty Vec) so a
/// second freeze on the same thread starts from zero.
pub(crate) fn take_phase_durations() -> Vec<(&'static str, std::time::Duration)> {
    TIMING.with(|t| {
        let mut v = t.borrow_mut();
        let out = v
            .windows(2)
            .map(|w| (w[0].0, w[1].1.duration_since(w[0].1)))
            .collect();
        v.clear();
        out
    })
}

#[cfg(test)]
pub(crate) fn reset() {
    TRACE.with(|t| t.borrow_mut().clear());
}

#[cfg(test)]
pub(crate) fn trace() -> Vec<&'static str> {
    TRACE.with(|t| t.borrow().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ The gate. Freeze a minimal program and assert the passes ran in the
    /// declared order.
    ///
    /// ⛔ If this goes red, do NOT re-order [`EXPECTED_ORDER`] to match the code
    /// until you have re-read every "post-step-7 ⇒ unreachable" disposition in
    /// `docs/arc/2026/06/255-builtin-registry/`. That sentence is what this
    /// array exists to protect; a pipeline re-order invalidates it wholesale,
    /// and the whole point of the gate is that the invalidation is LOUD.
    #[test]
    fn the_startup_passes_run_in_the_declared_order() {
        reset();
        let world = crate::freeze::startup_from_source(
            "(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println 41))",
            None,
            std::sync::Arc::new(crate::load::loader::InMemoryLoader::new()),
        );
        assert!(world.is_ok(), "the probe program must freeze: {world:?}");
        let got = trace();
        assert_eq!(
            got,
            EXPECTED_ORDER,
            "\n⛔ the startup pipeline's pass order CHANGED.\n   expected: {EXPECTED_ORDER:?}\n   \
             got:      {got:?}\n   Every \"unreachable because it is post-step-7\" disposition in \
             this arc's SCOREs rests on `7-normalize-symbol-refs` sitting where it sits. Re-read \
             them before touching this array."
        );
    }

    /// Non-vacuity: the array is not trivially satisfied by an empty trace, and
    /// the assertion above is comparing something a mis-ordering could break.
    #[test]
    fn the_order_gate_is_not_vacuous() {
        assert!(
            EXPECTED_ORDER.len() >= 10,
            "the pipeline has more than ten ordered passes; a short array means \
             `record` calls were dropped rather than the pipeline shrinking"
        );
        let n = EXPECTED_ORDER.iter().position(|s| *s == "7-normalize-symbol-refs");
        let c = EXPECTED_ORDER.iter().position(|s| *s == "8-check-program");
        let e = EXPECTED_ORDER.iter().position(|s| *s == "4-expand-all");
        assert!(
            matches!((e, n, c), (Some(e), Some(n), Some(c)) if e < n && n < c),
            "expand (4) must precede normalize (7), which must precede check (8) — \
             the three fixed points every reachability disposition in this arc cites"
        );
    }
}
