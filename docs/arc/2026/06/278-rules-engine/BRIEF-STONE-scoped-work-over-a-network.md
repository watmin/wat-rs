# BRIEF — STONE: scoped work over a compiled network

Promote three forms from a proven prototype into `:wat::rete::`. Design:
`DESIGN-STONE-scoped-work-over-a-network.md` — **read it first**.

## Your two sources, and they are both already correct

1. **`wat-scripts/scratch-pad/wat-grep-with-network-shape.wat`** — the working prototype. It runs
   (`3 / 0 / 3`) and `--check`s clean. The three forms are there as `:user::with-network`,
   `:user::with-overlay`, `:user::Overlay`. **Copy them; do not redesign them.** The prototype's
   header comments carry WHY each shape is what it is — bring the load-bearing ones with you.
2. **`wat-scripts/intueri/rete-scoped-work-naming.wat.intueri`** — the naming ruling, with the
   rejected candidates and the promise each broke. The names are settled: `with-network`,
   `with-overlay`, `Overlay`, and the body params `base` and `overlay`.

## Read in order

1. **`wat/rete.wat` `;; ─── the session ───` (~:154)** — the typealias goes beside `AlphaMemory` /
   `BetaMemory` / `ProductionMemory`, which are the same naming move on the same kind of type.
2. **`wat/rete.wat:55`** — `:wat::rete::Query`, the defrecord `with-network`'s second param names.
3. **`wat/io.wat:37-45`** — `with-open-file`. The family precedent: a plain defn taking a body-fn,
   acquiring its own resource and releasing after. Match this shape exactly.
4. **`src/rete/kernel/tests.rs:3043`** — `intern_release_one_session_leaves_the_other`. The idiom
   row 3 copies: `rete_arm_leases(id)` / `rete_arm_lookup(id)`.

## The three acceptance rows

**Row 1 — N units, ONE build.** `with-overlay` over N distinct fact sets increments `ARM_BUILDS`
exactly once. rete already gates the mechanism; you are asserting the composition.

**Row 2 — the base is untouched.** After N units, the compiled base still answers its own query with
zero results. (The prototype's `0` in `3 / 0 / 3`.)

**Row 3 — THE LEASE IS RELEASED. This must be a RUST test.**
Inside the body: `assert_eq!(rete_arm_leases(id), Some(1))`.
After `with-network` returns: `assert!(rete_arm_lookup(id).is_none())`.

⛔ **Row 3 is the whole point.** The prototype's first draft leaked a lease — it called
`arm-session` on an already-armed session, took lease 2, released to 1, and left `compile-all`'s
lease held forever. **It passed rows 1 and 2 green.** Leases are invisible from wat, so a wat-only
test cannot see this class at all. If you find yourself writing row 3 in wat, stop — you have lost
the only row that discriminates.

## Blast radius

`wat/rete.wat` (one typealias, two defns) and one new Rust test. **No changes to `compile-all`,
`insert-all`, `fire-rules`, `arm-session`, `release-session`, or anything in `src/rete/`** beyond
the test file.

## STOP triggers — rejection criteria. Ship nothing on the row; report it.

1. **A stdlib `.wat` edit is invisible until you rebuild** (`include_str!` at RUST-compile time). If
   your new verbs come back `UnknownFunction`, that is the rebuild, not a defect. Rebuild, then
   re-read. If they are still unknown after a clean `cargo build --release`, STOP and report.
2. **`Query` does not type-check in the signature** — e.g. `compile-all` wants something else in
   arg 2. Report its actual registered signature verbatim; do not widen the parameter to make it fit.
3. **Row 1 shows more than one build.** That contradicts a gated rete invariant. Capture
   `ARM_BUILDS` before/after verbatim and STOP — it is a finding about the intern, not about your code.
4. **Row 3 cannot see the lease** — `rete_arm_leases` unreachable from your test's module. Report the
   exact compiler error. Do NOT substitute a wat-level assertion; that is the thing that cannot work.

## Method

`cargo build --release` first (the stdlib is frozen in). Then
`cargo nextest run --release -E 'test(scoped_work)'` and run the prototype through
`target/release/wat` to confirm it still gives `3 / 0 / 3`. Report those numbers. Everything in the
FOREGROUND, blocked on completion. Do NOT run the full floor or clippy — the orchestrator runs those.

Do not commit, push, stash, or amend. Leave the git index empty. You may not spawn sub-agents.

## Report

The three rows with actual results; `git diff --stat`; confirmation that nothing in `src/rete/`
changed except the test file; and the exact `ARM_BUILDS` delta and lease readings you observed.
