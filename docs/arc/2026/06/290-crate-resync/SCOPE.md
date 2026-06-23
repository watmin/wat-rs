# Arc 290 — re-sync the neglected workspace crates to current arcs

**Status:** RE-GROUNDED 2026-06-22 (was mis-scoped). Surfaced by the arc-258
`-> :T` annihilation: the codemod's universe-load (and `cargo test
--no-fail-fast`) exposed that the workspace member crates were never being
exercised by the habitual gate (`cargo test --test test`, main crate only), so
they accumulated drift unseen. The deeper fix is **closing the gate gap** so
they can't silently re-drift.

> ⚠️ **The original SCOPE (a codemod job: `:nil`/`:i64`-as-value, `define`→`defn`,
> `match -> :T`) was WRONG.** Grounding the *actual* failures (not grep counts)
> showed the crates' `.wat` LOADS and type-checks — the failures are partial
> (e.g. wat-lru 7 pass / 5 fail), which is impossible for a checker-rejection at
> load. The real failures are runtime + small type-drift, split into three
> classes below. Lesson: ground the failure SHAPE (run it, read the panic), not
> the SCOPE doc's paraphrase.

## The neglected crates

`crates/wat-lru`, `crates/wat-holon-lru`, `crates/wat-telemetry`,
`crates/wat-sqlite`, `crates/wat-telemetry-sqlite` (+ `examples/with-lru`).
Each has its own `tests/test.rs` wat-loading harness, run only under `cargo
test` (workspace `default-members`), not the main-crate-only filter.

## The three failure classes (grounded — run the tests, read the panics)

### Class B — type-drift (DONE ✅, commit 952798a8)
`:wat::core::first` now returns the **bare element `T`**, not `Option<T>`
(`get`/`nth` are the Option-returning accessors). Three neglected sites still
treated collection-`first` as Option:
- `wat-telemetry` `Service.wat:245` — `(Some -1)` → `-1`
- `wat-telemetry-sqlite` `reader.wat:229,266` — `first` → `(get events 0)` (the
  existing `((Some e) e)(None …)` match then typechecks)

Result: `wat-telemetry` 31/0 (5 arc-170 spawn tests stay ignored),
`wat-telemetry-sqlite` 11/0.

### Class A — the arc-170 hand-rolled service pattern (leaks/hangs) — THE REAL WORK
`wat-lru` (5), `wat-holon-lru` (9), `examples/with-lru` (2). These services are
**hand-rolled on the pre-defservice machinery** —
`:wat::kernel::{make-channel,select,spawn-thread,HandlePool,send,recv}` + manual
`loop-step`/`DriverPair` pair-by-index. This is the **arc-170 deadlock-prone
pattern**: EVERY equivalent test in the MAIN crate is
`(:wat::test::ignore "arc-170 concurrency layer (subprocess spawn /
thread-on-channel) — leaks/hangs; remove before arc 170 closes")`
(`wat-tests/service-template.wat`, `counter-service-thread-N*.wat`, etc.). The
crates run the same pattern **un-ignored**, so they hang past the deftest
time-limit ("test thread leaked").

**Cure (builder's call): migrate to `defservice`** (arc 209/272, "done done" —
the OTP gen_server-style macro; locus `(:wat::spawn::thread|process)` is one
token; client face via `start`/`connect'`/methods). Canonical exemplar:
`wat-tests/service-locus-parity.wat`; macro: `wat/service.wat`. This kills the
last live users of the parked deadlock pattern (a qualified annihilation aligned
with closing arc 170).

**What the tests actually exercise** (so the migration can shed weight):
- `null-reporter` + `null-metrics-cadence` everywhere → the Reporter /
  MetricsCadence / Stats / tick-window **reporting feature is DEAD** (defservice
  handlers are pure `Outcome::Reply/Stop`; a side-effecting reporter doesn't fit
  — and nothing needs it). Drop it.
- `spawn 16 1` → `count=1`, single-client. Multi-client HandlePool isn't
  exercised (defservice gives clients via `connect'` for free anyway).
- **Open design question (probe FIRST):** does `defservice` support a generic
  `<K,V>` service, or only monomorphic? The counter exemplar is monomorphic;
  `wat/service.wat` shows no type-param handling. If monomorphic-only, the
  migration reshapes the crate's public API from generic `<K,V>` to concrete
  (acceptable — the only consumers are these crates' own tests + `with-lru`).
  Write a 10-line defservice-with-type-params probe before briefing the build.

Method per service: disconfirming probe (generics) → rewrite service `.wat` on
`defservice` (drop reporting) → rewrite the test `.wat` to `start`/`connect'`/
methods → weigh green. Delegate the build (>20 lines); orchestrator owns the
probe + weigh (bootstrap forbids blind delegation).

### Class C — the `:time-limit` harness overhead (small, separate)
`wat-sqlite` `arc-123-fast` deterministically (3/3) exceeds its **100ms** budget
on `(:wat::test::assert-eq 42 42)`. Not a service, not drift — the `:time-limit`
wrapper (thread-spawn + `recv_timeout`) overhead now exceeds 100ms cold. The 5s
/ 1m sibling tests pass (generous budgets). Either the harness regressed
(investigate `src/test_runner.rs` time-limit path) or the 100ms budget is now
unrealistically tight. Decide: bump the budget vs. fix the overhead.

## The gate gap (the real extirpation — unchanged)

The drift accumulated because the routine gate didn't load the crates.
**Close it:** make `cargo test --no-fail-fast` (workspace `default-members`) —
or at least a crate-load smoke — part of the standard strike gate, so a
corpus-wide change can't pass while a crate is red. NOT `cargo test --test test`
(main crate only). `--no-fail-fast` is mandatory so the known main-crate lib
36-floor doesn't fail-fast-mask later crate binaries. (Banked:
`feedback_workspace_gate_not_main_crate`.)

## Done = the gate
`cargo test --no-fail-fast` green for `wat-lru` / `wat-holon-lru` /
`wat-telemetry` / `wat-sqlite` / `wat-telemetry-sqlite` / `examples/with-lru`
(modulo the known main-crate lib 36-floor), with the crate homes vigilia-clean.
