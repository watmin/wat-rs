# BRIEF — one canonical `Failure` constructor + reclaim the client-path struct-new mints (Strike A)

> **Tier:** sonnet shadowdancer. **Arc:** 278, item (c) tail (the `VNDE ORTVM` ouroboros). **HEAD:** `25e316f0`.
> **Companion (do NOT touch):** the fixture-cleanup edit to `tests/services/probe_arc278_self_scheduling.wat`
> is uncommitted in the working tree — leave it; it's the instrument that exposed this bug.

## Why (one paragraph)

`:wat::kernel::Failure` is canonically a **Record** (`Nature::Record`, pure EDN — `test_runner.rs:1032`
"arc 293.W.2b — Failure is now Nature::Record"), because a crash cause **crosses the wire** and only a
Record round-trips EDN (`Nature::is_pure()` = *not* Struct; `types.rs:145`). But construction was never
unified — there is **no single `Failure` constructor**, so sites hand-roll it, and several use
`:wat::core::struct-new :wat::kernel::Failure`, which builds the **wrong nature (a Struct)**. The result:
a peer-lost cause that `:wat::kernel::Failure/message` (a **Record** accessor → `Record/field-at`) cannot
read — it raises `TypeMismatch`. It stayed hidden because the only path that reads a client-side
peer-lost cause is the `#[ignore]`'d self-scheduling test. This strike builds the **one** canonical wat
constructor and routes the **two client-method mint sites** through it, so the client-facing peer-lost
cause is a readable Record. (The corpus-wide wall + the remaining sites are Strike B, next — not here.)

## Phase 0 — PROVE the canonical wat constructor form (gated; STOP if it fails)

There is **no precedent** for constructing a builtin record positionally in wat source. Settle it first.
Write a scratch probe `wat-scripts/scratch-pad/probe-failure-record-ctor.wat` that builds a `Failure`
**as a Record** and reads it back:

```clojure
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [f (:wat::kernel::Failure "hello" :wat::core::None
         (:wat::core::Vector :wat::kernel::Frame) :wat::core::None :wat::core::None)]  ;; bare record ctor
    (:wat::kernel::println (:wat::kernel::Failure/message f))))                        ;; must print "hello", no TypeMismatch
```

`./target/release/wat --check wat-scripts/scratch-pad/probe-failure-record-ctor.wat` then run it. Confirm
the bare-name ctor `(:wat::kernel::Failure …)` produces a **Record** (`Failure/message` returns `"hello"`,
no `Record/field-at`/`TypeMismatch`). Ground the exact 5-field order + types from `message_only_failure`
(`runtime.rs:24808`): `[message <- String, location <- Option<Span>, frames <- Vector<Frame>, actual <- Option, expected <- Option]`.

**HARD STOP-0:** if the bare ctor does NOT type-check or does NOT yield a readable Record, STOP and report
exactly what it produced. Do NOT fall back to `struct-new`, do NOT invent a Value-erasure. The foundation
isn't ready and the orchestrator re-plans (likely: expose the Rust `message_only_failure` as a
`:wat::kernel::message-only-failure` intrinsic instead). Delete the scratch probe once the form is settled
(or keep it under scratch-pad/ if it's a useful reference — it is loader-gated so it must stay green).

## Phase 1 — define the ONE canonical constructor

With the form proven, add a single message-only helper (the common reason-free case) to the stdlib.
Ground its home: `Failure`/`Failure/message` live in the kernel layer — co-locate with the recv'/Failure
stdlib (check `wat/spawn.wat` / `wat/kernel/*.wat` / `wat/core.wat` for where builtin-record helpers
belong; pick the one that loads before `service.wat`, which will call it).

```clojure
(:wat::core::defn :wat::kernel::message-only-failure [msg <- :wat::core::String] -> :wat::kernel::Failure
  (:wat::kernel::Failure msg :wat::core::None (:wat::core::Vector :wat::kernel::Frame)
    :wat::core::None :wat::core::None))
```

(The full-field case, if any site needs location/frames/actual/expected, uses the bare ctor directly. This
strike only needs message-only.)

## Phase 2 — reclaim the TWO client-method mint sites

1. **`wat/service.wat:1217`** — inside the defservice client-method macro's `::Lost` arm. Replace the
   `(:wat::core::struct-new :wat::kernel::Failure "service peer lost (reason on the owner's crash channel)"
   :wat::core::None (:wat::core::Vector :wat::kernel::Frame) :wat::core::None :wat::core::None)` form with
   `(:wat::kernel::message-only-failure "service peer lost (reason on the owner's crash channel)")`. It is
   inside a quasiquote template — keep it syntactically valid there.
2. **`src/runtime.rs:5501`** — the Rust codegen that *emits* the identical `struct-new` WatAST for the
   generated client method's `::Lost` arm (`:5500-5511`). Change the emitted form to the bare record ctor
   (`WatAST` head `:wat::kernel::Failure` + the 5 field nodes, dropping the `:wat::core::struct-new` +
   type-keyword head) — OR, cleaner, emit `(:wat::kernel::message-only-failure "…")`. Preserve the exact
   message string and the reason-free scrubbing behavior (that's by-design: client=reason-free, the real
   reason is the owner's). Do NOT change what the arm *does*, only the nature of the value it builds.

**Do NOT** touch the `test.wat` / `hermetic.wat` / `sandbox.wat` struct-new sites — those are Strike B's
to reclaim under the wall. **Do NOT** add the checker wall — that's Strike B.

## STOP triggers

- **STOP-0** (above): the canonical ctor form doesn't produce a readable Record.
- **STOP-1:** converting `service.wat:1217` breaks the macro quasiquote or the defservice tests — STOP,
  report the exact error; do not restructure the macro.
- **STOP-2:** the `runtime.rs:5501` change requires touching the codegen's *structure* (beyond swapping the
  emitted Failure form) — STOP and report; do not refactor the surrounding client-method codegen.

## Verify (weigh by your own re-run)

1. `./target/release/wat --check` clean on every edited `.wat`.
2. The companion fixture-cleanup edit is already in the tree; run the item-c thread test:
   `cargo nextest run --release self_tick_fires_rearms_and_reactor_serves_thread --run-ignored all --no-capture`
   — it will STILL FAIL (the service still dies mid-tick — that's Finding A, not this strike), but the
   failure must now surface the cause as a **readable value** ("service peer lost …") via the `_s` Lost arm,
   **NOT** a `TypeMismatch` / `Record/field-at` panic. That flip is the proof this strike worked.
3. **Whole release floor:** `cargo nextest run --release` — read the Summary line yourself; no new failures
   vs. the ~4189/0 floor (the self_scheduling ×2 stay `#[ignore]`'d/RED-by-design).

## Deliverable

The canonical constructor + the two reclaimed client-path sites, `--check` clean, release floor green
except the by-design ignored self_scheduling tests. Report: (1) the proven ctor form; (2) the before/after
of the item-c failure (TypeMismatch → readable "service peer lost" value); (3) `git diff --stat`. Do NOT
commit — leave it for the orchestrator to weigh.

## Blast radius

One stdlib `.wat` helper (new `defn`) + `wat/service.wat:1217` + `src/runtime.rs:5500-5511` + a scratch
probe. NO checker changes (Strike B). NO test/hermetic/sandbox sites (Strike B). NO changes to the
fixture-cleanup edit already in the tree. Scratch logs → `/tmp/claude-scout/`.
