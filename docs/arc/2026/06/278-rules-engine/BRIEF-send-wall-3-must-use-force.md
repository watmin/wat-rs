# BRIEF — send' OUTCOME WALL, Phase 3: the MUST-USE FORCE (swallow → unrepresentable)

> **Tier:** sonnet shadowdancer. **Arc:** 278 send'-wall Phase 3 (see `DESIGN-send-outcome-wall.md` STATUS).
> **HEAD:** `8e46ace0` (Phases 1-2 committed — `send'` returns `SendOutcome`, all sites faced, floor green).
> This strike makes a **discarded** `SendOutcome` a **compile error** — so a future `(send' …)` swallow is
> unrepresentable (R57 `unrepresentable > flagged`). This is a NEW checker feature; the floor must STAY green.

## Why (one paragraph)

Phases 1-2 made `send'` return a matchable `SendOutcome` and faced all 183 current sites. But nothing yet
*forces* facing — a new `(do (send' p m) next)` would discard the `SendOutcome` (a swallow) and compile.
There is **no must-use mechanism** in the checker (grounded: `recv'` is always expression-position, never
needed one). Build a targeted one: a `SendOutcome` value in a **discard position** is a located compile
error. The beautiful invariant — a *faced* send' (`(match (send' …) (Sent nil) (Closed nil) ((Lost _c) nil))`)
has type **`nil`**, not `SendOutcome`, so the check fires **only** on a raw unfaced `send'`. Therefore the
floor MUST stay 4207/0 — and that green *is the proof* Phase 2 left zero swallows.

## The design

**(a) A must-use predicate.** Decide the cleaner of two, grounded (count `EnumDef {…}` construction sites
first — if adding a field cascades to many, prefer the set):
- a `must_use: bool` on `EnumDef` (`types.rs:256`), `true` for `SendOutcome`, default `false`; OR
- a hardcoded checker set `const MUST_USE_TYPES: &[&str] = &[":wat::kernel::SendOutcome"];` (simpler, targeted,
  generalizes later). **Recommended for this first must-use** — one type, no struct-field cascade.

**(b) The discard positions to gate** (a value whose result is dropped):
1. **`do` non-last exprs** (`infer_do`, `check.rs:8230`) — the primary case (`(do (send' …) next)`). For each
   non-last item, if its inferred type resolves to a must-use type → push a located `TypeError`.
2. **`let [_ expr]` wildcard-bind** (`let` handling, `check.rs:3893`) — a `_`-bound must-use value is a
   swallow. If the binding target is the wildcard `_` and the expr's type is must-use → the same error.
3. (A bare top-level statement whose value is unused — only if trivially reachable; do NOT over-reach.)

**(c) The error** — located at the discarded expr, naming the type + the remedy:
> *"unhandled `:wat::kernel::SendOutcome` in statement/discard position — a send' outcome must be faced
> (`match` it: `Sent`/`Closed`/`Lost`), not dropped. This is the send'-outcome wall (Phase 3)."*

## STOP triggers

- **STOP-0:** the must-use check turns EXISTING sites RED (the floor drops below 4207/0) — that means Phase 2
  left a swallow OR the check has a false positive. STOP, report which sites + which: if a real swallow, it's
  a Phase-2 miss to face; if a false positive (a faced site whose type is somehow still SendOutcome), the
  check is wrong. Do NOT mass-edit to force green.
- **STOP-1:** marking `must_use` requires cascading a struct field to >~20 `EnumDef` sites — STOP, switch to
  the hardcoded set (report).
- **STOP-2:** the `let [_ …]` wildcard detection isn't cleanly available in the checker's let handling —
  STOP, ship the `do`-non-last gate alone (the primary case) + report; the `let`-`_` gate can be a follow.

## The RED probe (prove the force)

Add a `.wat.bad` fixture: `(:wat::core::do (:wat::kernel::send' p m) nil)` — a raw discarded send' — + a
`.rs` test asserting it is now a **check error** (use `wat::assert_check_error_present!` — the membership
macro from Strike M — matching the must-use error). Before Phase 3 this compiled; after, it's rejected.
Place beside the arc-278 service/check probes.

## Verify (weigh by your own re-run)

1. `cargo build --release` compiles.
2. The RED probe test passes (the discarded send' IS rejected).
3. **Whole floor: `cargo nextest run --release`** — READ the Summary yourself; it MUST be **4207/0** (the
   force adds one probe → possibly 4208; the point is 0 failed). Run it in the FOREGROUND of your turn (do
   NOT background it and end your turn); wait and self-read the Summary. Green = every existing site is
   genuinely faced (the Phase-3 proof of Phase-2's completeness). If ANY existing test flips RED, that's
   STOP-0 — report it, do not paper over it.

## Deliverable

The must-use force (predicate + the `do`-non-last gate, + the `let`-`_` gate if clean) + the RED probe.
Report: (1) which predicate approach + the gate sites; (2) the RED probe result; (3) the floor Summary
(0 failed) read by you after a foreground run; (4) `git diff --stat`; (5) any STOP-0 RED-flips (should be
none). Do NOT commit — leave for the orchestrator to weigh.

## Blast radius

`src/check.rs` (the discard gates) + `src/types.rs` (the must-use bit/set) + one `.rs`/`.wat.bad` RED probe.
NO changes to the 183 faced sites (they stay green by construction). Scratch logs → `/tmp/claude-scout/`.
