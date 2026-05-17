# Arc 202 — `ProcessJoinHoldsStdinSender` walker rule

**Direction:** mirror Gap K (`ProcessJoinBeforeOutputDrain`, arc 170 slice E) for the INPUT direction. Substrate refuses to compile/run any wat program that calls `:wat::kernel::Process/join-result proc` without preceding stdin-Sender disposal in scope. Surface as a `CheckError` at freeze time. No runtime auto-correction.

**Status:** DESIGN. Single-slice scope; no decomposition warranted.

**Originating diagnostic:** the workspace `cargo test --release --workspace --no-fail-fast` run launched 17:33 hung for 35+ min. Investigation via `/proc/<pid>/wchan`:

- `tests/wat_run_sandboxed_ast.rs::ast_entry_prints_hello` test binary (PID 1005409)
- Main thread: `futex_do_wait` (blocked on `Process/join-result proc`)
- Helper thread `ast_entry_print` (TID 1005411): `anon_pipe_read` (drain blocked)
- Spawned child (PID 1005413): `futex_do_wait`
- Parent holds fd 4 (write-end of child stdin pipe 1250789) via `proc` binding — never dropped
- Child's structural `StdInService` (arc 170 slice 1f) blocked on `read(fd 0)` waiting for EOF
- Parent's `Process/join-result proc` blocks waiting for child to exit; child can't exit while StdInService is alive; StdInService can't exit without EOF; EOF never arrives because parent never closed its write-end

**Validation:** this is a true deadlock past every existing substrate guarantee. No party is dead (lifeline FDs unfired). No signal pending (cargo alive). The FD-multiplex Phase 1-3 "you cannot escape shutdown" guarantees apply only when a SHUTDOWN EVENT fires — they don't address all-parties-alive-but-stuck. Verified during 2026-05-16 investigation; user direction: panic on bad forms; "we help all users always — we must panic to prove we can help ourselves."

---

## The expression that enabled this

`wat/test.wat:515-565` `run-hermetic-driver`:

```scheme
(:wat::core::let
  [drain-pair
    (:wat::core::let
      [stdout-r       (:wat::kernel::Process/stdout proc)
       stderr-r       (:wat::kernel::Process/stderr proc)
       stdout-lines   (:wat::kernel::drain-lines stdout-r)
       stderr-lines   (:wat::kernel::drain-lines stderr-r)]
      (:wat::core::Tuple stdout-lines stderr-lines))
   stdout-lines   (:wat::core::first drain-pair)
   stderr-lines   (:wat::core::second drain-pair)
   joined-result  (:wat::kernel::Process/join-result proc)
   ...])
```

The driver follows the inner-scope-drains-before-outer-join discipline for output Receivers (stdout-r + stderr-r) but never extracts `Process/stdin proc`. The proc handle holds the stdin Sender for the entire outer-let scope; join runs while it's still held.

## Why Gap K didn't catch it

`ProcessJoinBeforeOutputDrain` (src/check.rs:192-206) is a co-presence matcher: it pairs `(Process/join-result <p>)` with `(Process/{stdout,stderr,output} <p>)` on the same identifier. The stdin case is the inverse shape — join exists, NO `Process/stdin` exists. Absence isn't a syntactic event the current walker can match against.

The 2026-05-13 INTERSTITIAL realization explicitly flagged this:

> The asymmetric stdin-direction concern flagged today (parent forgets to close stdin IOWriter before join → child stalls on readln → child can't write outputs → join blocks forever) — if it becomes a rule, it should follow the same recursive walker shape so the next wrapper minted over `Process/stdin` is caught without re-engineering.

Flagged. Never minted. Arc 202 closes that loop.

## Four questions on the layer choice (settled 2026-05-16)

**Candidate Y — Layer A (runtime auto-close at `Process/join-result`):**

- Obvious: marginal
- Simple: YES
- Honest: **NO** — silent runtime correction hides the rule; deadlock-shape user code "works" by accident; user never learns. Per V5 doctrine (verbatim): *"i do not accept the 5s fix. i want to know exactly where we are failing - our users must be told they did something illegal."* Same comfort-seeking shape as the rejected wall-clock timeout.
- Good UX: marginal

→ DISQUALIFIED on Honest.

**Candidate X — Layer B (walker rule, freeze-time refusal):**

- Obvious: YES — mirrors Gap K precisely; same walker, same diagnostic style
- Simple: YES — additive arm in `collect_process_calls`; one new CheckError variant
- Honest: YES — refusal at freeze time; substrate VOICES the rule; macro authors (the parties writing this kind of code; user-site call code stays clean) get the diagnostic immediately
- Good UX: YES — file:line + rule name at check; tests don't hang

→ **YES YES YES YES.**

**Settled: Layer B.** Walker rule at freeze time. No runtime auto-correction.

## API shape

New `CheckError` variant:

```rust
/// Arc 202 — Process input-channel held-at-join rule.
/// A `let` form contains:
///   - a call to `:wat::kernel::Process/join-result <p>` (blocks
///     until the forked child exits), AND
///   - NO call to `:wat::kernel::Process/stdin <p>` in any nested
///     inner scope preceding the join (where the Sender would drop
///     before join runs).
///
/// The substrate's child has a structural StdInService (arc 170
/// slice 1f) blocked on fd 0. Without an EOF on the parent's
/// write-end of the child's stdin pipe, the child cannot exit;
/// parent's join blocks forever.
///
/// Inverse direction of ProcessJoinBeforeOutputDrain: that rule
/// catches output-Receiver-held-at-join; this catches input-Sender-
/// held-at-join. Both produce the same deadlock signature; both
/// arise from the same lockstep-discipline violation; both fire
/// at freeze time.
ProcessJoinHoldsStdinSender {
    /// Name of the Process identifier the join-result is called on.
    process_identifier: String,
    /// Source location of the `Process/join-result` call.
    join_span: Span,
    /// Source location where `<process_identifier>` was bound.
    bind_span: Span,
}
```

## Detection mechanism

Two options for sonnet to evaluate inline:

**(α) Extend `collect_process_calls` to include `Process/stdin` in accessors.** Then if `find_process_join_before_drain` ALSO fires for `(join, stdin)` pairs at the same let level (analogous to existing output-direction pairing), the existing machinery extends to cover the SIBLING-binding case. Doesn't cover the run-hermetic-driver case (where stdin is absent entirely from outer let), but catches the worst footgun shape.

**(β) Add a parallel `find_process_join_holds_stdin_sender` that does absence detection.** For each `(Process/join-result <p>)`, walk the let's scope; verify SOME `(Process/stdin <p>)` exists in a nested inner scope ahead of the join. If not, flag.

**Sonnet picks based on simplicity** — likely (β) for the run-hermetic-driver case is the actual need; (α) alone wouldn't catch the absent-stdin-extraction shape.

Note: existing Gap K detection scoping at `src/check.rs:7142-7165` (per-let-form invocation of `find_process_join_before_drain` on a constructed let_scope from bindings_pairs + body_ast) is the precedent — mirror it.

## Wat-side fix (mandatory, in-slice)

`wat/test.wat:524-533` `run-hermetic-driver`'s inner let must extract `stdin-w`:

```scheme
(:wat::core::let
  [stdin-w        (:wat::kernel::Process/stdin proc)   ;; ← ADD
   stdout-r       (:wat::kernel::Process/stdout proc)
   stderr-r       (:wat::kernel::Process/stderr proc)
   stdout-lines   (:wat::kernel::drain-lines stdout-r)
   stderr-lines   (:wat::kernel::drain-lines stderr-r)]
  (:wat::core::Tuple stdout-lines stderr-lines))
```

stdin-w drops alongside stdout-r and stderr-r at inner-let exit; child's StdInService sees EOF; child exits; outer join returns cleanly.

If sonnet finds OTHER substrate-side wat helpers with the same shape (e.g., Stone D1 `run-threads` if applicable to Process<I,O>), fix them too.

## Tests

`tests/wat_arc202_process_join_holds_stdin.rs` (sonnet picks final name):

1. `process_join_without_stdin_extraction_fails_check` — minimal wat program with `(let [proc (spawn-process ...) joined (Process/join-result proc)] ...)` → check returns `ProcessJoinHoldsStdinSender` with both spans correct.
2. `process_join_with_stdin_extraction_in_inner_scope_passes_check` — analogous to the run-hermetic-driver fix shape; check succeeds.
3. `process_join_with_stdin_extraction_in_outer_let_still_fails` — IF the rule distinguishes inner vs outer scope (per option β). Captures the subtle case where naive sibling extraction still deadlocks. Optional based on rule shape.
4. Verify `tests/wat_run_sandboxed_ast.rs::ast_entry_prints_hello` (the originally-hung test) now passes cleanly after run-hermetic-driver fix.

## Scope

Single slice. Mechanical shape:
- Add `CheckError::ProcessJoinHoldsStdinSender` variant + Display impl + Diagnostic impl (~40 LOC mirroring Gap K)
- Extend `collect_process_calls` and/or add `find_process_join_holds_stdin_sender` (~30 LOC)
- Wire detection into `check_let` next to existing Gap K hook (~10 LOC)
- Fix `wat/test.wat:524-533` (~3 LOC)
- New test file (~80 LOC)

~160 LOC total across 3 files.

**Predicted:** 60-90 min sonnet.

## Knock-on / non-blocking dependencies

**Unblocks:**
- The hung `wat_run_sandboxed_ast::ast_entry_prints_hello` test
- Workspace tests stop hanging on similar patterns
- Future Stone D/E bracket combinator macros: walker will catch any forgotten stdin-discipline

**Doesn't affect:**
- Arc 201 slices 5-6 (extract-arg-types + closure paperwork) — orthogonal substrate area
- Arc 170 Stone D2 work in flight on D2-related concerns — different file

**Cleanup that should land alongside this:**
- None pre-emptive; sonnet surfaces if any other wat helper has the same shape as run-hermetic-driver

## Discipline anchors

- `feedback_any_defect_catastrophic` — substrate trust is binary; known defect; fix now
- `feedback_no_known_defect_left_unfixed` — we know how to surface; we do it
- `feedback_attack_foundation_cracks` — the hang IS the crack; fix as forward progress
- `feedback_substrate_imposed_not_followed` — substrate enforces (freeze refusal), not callers
- V5 doctrine (INTERSTITIAL § "Failure engineering applied to V5"): substrate refuses to run on illegal orientations; loud rule over silent correction
- `project_signal_cascade` analogue at the let-scope level: structural impossibility over runtime cleverness

## Open questions for user

None. The Layer A vs Layer B fork was settled inline (Layer B wins YES YES YES YES; Layer A fails Honest). Detection mechanism (α vs β) is sonnet's call per simplicity.
