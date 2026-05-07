# Arc 145 — Typed Let Substrate BRIEF (sweep 1a)

**Drafted 2026-05-06.** Sweep 1a of arc 145's typed-let work.
Per DESIGN's revised slice plan: sweep 1a is substrate-only;
sweep 1b is the consumer migration; atomic commit when workspace
= 0-failed (per recovery doc § 7 atomic-commit-across-coordinated-sweeps).

User direction (verbatim, captured in DESIGN):
> *"the ret val of a let statement /must be declared/ .. the
> 'user's choice' is whether or not to use let or let* -- both
> must have a ret val declared.. the let's ret val can be bound
> to something and used later - just like if, match etc"*

## Goal

Make `:wat::core::let` and `:wat::core::let*` REQUIRE `-> :T` at
HEAD position (before the bindings). Untyped form becomes a parse
error with a migration-hint MalformedForm.

## Q1 resolved — HEAD position

Per arc 108 INSCRIPTION + analysis of the convention:
- `if` / `match`: `-> :T` AFTER first arg (dispatch determiner — scrutinee/cond is NOT a value producer)
- `option/result::expect`: `-> :T` at HEAD (before value expr — the value-expr IS the value producer)

For `let` / `let*`: bindings are SETUP (not dispatch determiners,
not value producers); body is the value producer. Closer analogy
to `option/result::expect` — `-> :T` declares the form's contract
BEFORE any args, then bindings, then body.

**Target shape:**

```scheme
(:wat::core::let -> :ResultType
  (((n :Type) expr) ...)
  body)

(:wat::core::let* -> :ResultType
  (((n :Type) expr) ...)
  body)
```

Old form `(:wat::core::let bindings body)` becomes a parse error.

## Substrate evidence (verified pre-brief)

Current substrate state:
- `src/check.rs:5184-5208` — `infer_let` (parallel; no `-> :T`)
- `src/check.rs:5946+` — `infer_let_star` (sequential; no `-> :T`)
- `src/runtime.rs:2402-2403` — `eval_let` + `eval_let_star`
- `src/runtime.rs:1969-1970` — tail-call paths
- `src/runtime.rs:14852+` — `step_let_star` (incremental evaluator)
- `src/special_forms.rs` — arc 144 slice 2's let/let* sketches (need updating)

`infer_match` at `src/check.rs:3801` is the canonical "required
`-> :T` with migration-hint" pattern. Read it before drafting your
implementation. The shape:

```rust
// Pre-inscription shape detection: if args[1] isn't `->`, this
// is the old form. Surface a migration-hint error.
if args.len() >= 2
    && !matches!(&args[1], WatAST::Symbol(s, _) if s.as_str() == "->")
{
    errors.push(CheckError::MalformedForm {
        head: ":wat::core::match".into(),
        reason: "`:wat::core::match` now requires `-> :T` ...".into(),
        span: head_span.clone(),
    });
    return None;
}
```

For let/let*, the migration-hint reason should read:

```
"`:wat::core::let` now requires `-> :T` at HEAD — write
 (:wat::core::let -> :ResultType ((... ) ...) <body>)"
```

(and analogous for `let*`).

## What to do

### Pre-flight crawl (mandatory)

1. **`docs/arc/2026/05/145-typed-let/DESIGN.md`** — read in full,
   especially sections "Target shape", "Slice plan", "Q1 — `-> :T`
   placement", "Q2 — RESOLVED: REQUIRED".
2. **`docs/arc/2026/04/108-typed-expect-special-forms/INSCRIPTION.md`**
   — the `-> :T` precedent for value-bearing special forms.
3. **`src/check.rs` `infer_match`** at line ~3801 — your canonical
   pattern for required `-> :T` + migration-hint.
4. **`src/check.rs` `infer_if`** at line ~5055 — another instance
   of required `-> :T` (different placement: after cond).
5. **`src/check.rs` `infer_let`** + **`infer_let_star`** at lines
   ~5184 + ~5946 — your edit targets.
6. **`src/runtime.rs` `eval_let`** + **`eval_let_star`** + tail-call
   paths + `step_let_star` — runtime mirror your edits to.
7. **`src/special_forms.rs`** — the registry sketches for let /
   let* added by arc 144 slice 2. Update them to reflect required
   `-> :T` at HEAD.

### The substrate edits

#### A — `infer_let` (parallel let) at `src/check.rs:5184-5208`

Add migration-hint detection (args[1] must be `->`); add `-> :T`
parsing at args[2] (Keyword); validate body's inferred type
unifies with declared `:T`; return declared `:T`.

Mirror `infer_match`'s migration-hint shape verbatim where
applicable.

#### B — `infer_let_star` (sequential let*) at `src/check.rs:5946+`

Same shape as A, but for sequential semantics. The `cumulative`
env walks bindings; the result type is declared `:T` (same
unification check as A).

#### C — `eval_let` + `eval_let_star` at runtime layer

The `-> :T` token is a NO-OP at runtime (type-check-time only).
Each eval needs to:
1. Skip the `->` token at args[1]
2. Skip the `:T` keyword at args[2]
3. Process bindings starting at args[3]
4. Evaluate body at args[args.len() - 1] (last arg)

Tail-call paths (`eval_let_tail`, `eval_let_star_tail`) +
incremental-step (`step_let_star`) similarly skip args[1] +
args[2] + process bindings + body the same way.

#### D — `src/special_forms.rs` registry sketches

Update arc 144 slice 2's sketches for `:wat::core::let` and
`:wat::core::let*` to show the required `-> :T` slot at HEAD.

### The substrate tests

Add 6-10 unit tests in a new file
`tests/wat_arc145_typed_let.rs`:

1. Typed parallel `let` with `-> :i64`, body returns 42, asserts type + value
2. Typed sequential `let*` with `-> :String`, body returns "hello", asserts type + value
3. Type mismatch on parallel `let` (`-> :i64` declared but body returns String) — assert TypeMismatch fires
4. Type mismatch on sequential `let*` — same shape
5. Untyped `(:wat::core::let bindings body)` — assert MalformedForm with migration-hint
6. Untyped `(:wat::core::let* bindings body)` — assert MalformedForm with migration-hint
7. Nested typed lets (let body containing another let) — assert types compose
8. Tail-call optimization preserved with typed let (sanity check)
9. Sequential binding visibility preserved with typed let* (sanity check)
10. Reflection round-trip: `lookup-form :wat::core::let` shows the declared `-> :T` slot

## Constraints

- **Substrate-only edits.** ONLY these files:
  - `src/check.rs`
  - `src/runtime.rs`
  - `src/special_forms.rs`
  - NEW `tests/wat_arc145_typed_let.rs`
- NO consumer-side wat file edits (sweep 1b's scope; it ships next).
- NO other test file edits (sweep 1b touches them).
- **The workspace WILL break post-substrate-change** — every
  existing `(:wat::core::let ...)` / `(:wat::core::let* ...)`
  call site fails with MalformedForm migration-hint until sweep
  1b ships. THIS IS EXPECTED. Sweep 1b runs immediately after.
- DO NOT COMMIT. Working tree stays modified; orchestrator
  commits sweep 1a + sweep 1b + 2 SCORE docs atomically when
  workspace = 0-failed.
- STOP at first red, distinguishing: substrate-internal red
  (parse error inside check.rs/runtime.rs/special_forms.rs;
  unexpected) vs consumer red (existing wat call sites failing
  with MalformedForm; expected).
- No grinding.

## Pre-flight verification (test it BEFORE editing)

Run the canonical baseline:
```bash
cargo test --release --workspace 2>&1 | grep -cE "FAILED"
```
Must be 0 (workspace currently clean post-arc-130 + arc-119
closures). If not, surface as substrate-pre-existing failure.

## Verification (after edits)

Run the new test file:
```bash
cargo test --release --test wat_arc145_typed_let 2>&1 | tail -10
```
Expect: all 6-10 tests pass.

Run the workspace:
```bash
cargo test --release --workspace 2>&1 | grep -E "test result:|FAILED" | tail -10
```
Expect: many CONSUMER tests fail with MalformedForm
(the migration-hint substring "now requires `-> :T`" should
appear in the failure messages). NO substrate-internal parse
errors, NO panics, NO unexpected runtime failures.

If anything other than "MalformedForm: migration-hint" surfaces
in the consumer failures, surface it — substrate has an
unexpected gap.

## Out of scope

- Sweep 1b (consumer migration across all wat call sites)
- Slice 2 closure paperwork (INSCRIPTION + 058 row + USER-GUIDE)
- Lab consumers (`holon-lab-trading/`) — separate workspace
- Any optional `-> :T` form (REQUIRED is the resolved Q2 stance)
- Renaming `let*` → `let` (out of scope per DESIGN; both forms stay)

## Reporting (~250 words)

1. **Pre-flight crawl confirmation:** DESIGN, arc 108 INSCRIPTION,
   infer_match, infer_if, infer_let, infer_let_star, eval_let +
   tail/step paths, special_forms.rs all read.

2. **Edit summary:**
   - infer_let — migration-hint detection + `-> :T` parsing + body unification
   - infer_let_star — same shape, sequential semantics preserved
   - eval_let, eval_let_star, tail-call paths, step_let_star — `->` + `:T` skip
   - special_forms.rs — sketches updated to show required `-> :T` at HEAD
   - tests/wat_arc145_typed_let.rs — 6-10 new tests

3. **LOC delta:** before/after across the 4 source files +
   new test file.

4. **Verification:**
   - `cargo test --test wat_arc145_typed_let` — pass count
   - `cargo test --workspace` failure profile —
     MalformedForm-migration-hint shape (✓ expected) or
     unexpected substrate red (✗)

5. **Path:** Mode A clean (substrate ships; consumer failures
   match expected MalformedForm migration-hint shape) / Mode B
   substrate-internal-bug / Mode C unexpected-failure-shape.

6. **Honest deltas:** any HEAD-vs-AFTER placement nuance, any
   tail-call-path edge case, any reflection-trio interaction.

DO NOT write a SCORE doc — orchestrator's work after 1b ships.

## Time-box

60 minutes wall-clock (2× predicted 30-min upper-bound). If
you hit 60 min and aren't done, STOP and report what you have.

## Why this brief matters

The user direction 2026-05-03 evening clarified `-> :T` is REQUIRED
on both forms. The DESIGN's earlier "OPTIONAL forever" framing
was the orchestrator misinterpretation. Sweep 1a substrate
change ships the REQUIRED stance; sweep 1b migrates consumers.
Per arc 109's "no bridges" doctrine + FM 11's no-deferral
discipline, the inconsistency closes cleanly rather than papering
over with optional-forever.

Mode A clean = substrate ships REQUIRED `-> :T` cleanly; consumer
tests fail expectedly with migration-hint MalformedForm; sweep
1b proceeds.

Mode B/C/D = surface the gap; orchestrator adjusts.
