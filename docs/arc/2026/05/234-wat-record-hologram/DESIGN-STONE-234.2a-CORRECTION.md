# DESIGN — Arc 234 Stone 234.2a — forward-correction (TypeScheme heterogeneous struct_form)

**Status:** ACTIVE (2026-05-24 — forward-correction; SCORE-STONE-234.2a.md stays immutable per `feedback_inscription_immutable`).

**Origin:** Stone 234.2b's probe 5 (heterogeneous fields `[7 "hello" true]`) fired STOP-7 on a substrate TypeScheme gap. The diagnosis was honest: this is NOT a substrate flaw we discovered — this is an **authoring inconsistency** in Stone 234.2a that 234.2b's probe surfaced.

---

## The honest framing — what happened

Three artifacts disagreed about `:wat::Record::of`'s struct_form parameter:

| Artifact | Says |
|---|---|
| **DESIGN.md umbrella line 19** | `struct_form: Arc<Vec<Value>>` — heterogeneous by intent (Values are variant) |
| **`eval_record_of` runtime** | Accepts `Value::Vec` without checking element types — runtime got it right |
| **`:wat::Record::of` TypeScheme in check.rs line 16989** | `Vector<T>` (single type variable) — uniform-T enforced |

The TypeScheme was authored too narrowly. The umbrella DESIGN intent was heterogeneous; the runtime honored that intent; the type-checker contract did not.

### Why it went undetected

Stone 234.2a's probe suite tested:
- Probe 1: single field (`[5.0]`) — uniform
- Probe 3: single field (`[42.0]`) — uniform
- Probe 4: two fields same type (`[3 4]` both i64) — uniform
- Probe 5: positional access on uniform — uniform
- Probe 7: equality construction — single uniform field

**No probe exercised heterogeneous fields.** The TypeScheme's uniform-T constraint never fired. 234.2a's SCORE shipped 6/6 PASS honestly — the test suite never exercised the gap. 234.2b probe 5 is the first heterogeneous test in the chain.

### What this isn't

- NOT a substrate-evolution stone — no new capability shipping
- NOT a "fix arc" — no separate plan; this CORRECTS 234.2a's incomplete authoring
- NOT a 234.2b problem — the 234.2b macro is correct; sonnet's STOP-7 report was honest

### What this is

A forward-correction commit: the 234.2a TypeScheme is amended to match the umbrella DESIGN intent + the runtime's actual behavior. Per `feedback_inscription_immutable`: SCORE-STONE-234.2a.md stays unchanged (historical record of what shipped at the time, including the latent gap); the correction lives in a new artifact + a new commit.

---

## The fix

Change `:wat::Record::of`'s TypeScheme registration in `src/check.rs` so the struct_form parameter accepts heterogeneous Vec values (per-element types are arbitrary; the substrate doesn't unify across them).

### Three approaches considered

**(A) Custom inference handler (RECOMMENDED).** Pattern: `infer_record_of` modeled after `infer_arithmetic` (check.rs line 10907). The primitive `:wat::Record::of` gets a dispatch hook in the inference machinery that:
1. Verifies arg #1 is `:wat::core::keyword`
2. Verifies arg #2 is a Vec-shaped expression (`:wat::core::vec` head OR Vector literal) WITHOUT enforcing element-type uniformity
3. Verifies arg #3 is `:wat::holon::HolonAST`
4. Returns `:wat::Record`

Precedent: `infer_arithmetic` (~lines 10885-10980) for the family of arithmetic primitives. The custom handler pattern is established.

**(B) Variadic individual params (REJECTED).** Change `:wat::Record::of` signature to take fields as rest args: `(class, holon-form, ...fields)`. Cleaner architecturally but breaks the existing runtime contract; the 234.2b macro would need rewriting; downstream effects on Stone 234.2a probes.

**(C) Read fields from holon_form (REJECTED).** Defeats the hologram (struct_form is the Rust-fast path; reading from holon_form requires HolonAST traversal per access).

**Going with (A).**

### Implementation surface (predicted)

- `src/check.rs` — add `infer_record_of` function (mirroring `infer_arithmetic`'s shape); add dispatch entry for `:wat::Record::of` in the primitive-inference dispatcher; remove or amend the existing `env.register(":wat::Record::of", ...)` TypeScheme registration at lines 16993-17001 (whichever path the dispatcher uses — custom handler OR TypeScheme, but not both)
- ~30-50 lines total

### What stays unchanged

- `src/runtime.rs::eval_record_of` — already heterogeneous-accepting; correct since 234.2a
- `src/runtime.rs` Value::wat__Record variant — already `Arc<Vec<Value>>`; correct since 234.1
- Stone 234.2a probe — still 6/6 PASS (it tests only uniform fields; the correction doesn't regress uniform-vec acceptance)
- Stone 234.2b probe — flips from 5/6 PASS to 6/6 PASS (probe 5's heterogeneous case starts passing)
- The 234.2b macro at `wat/Record.wat` — UNCHANGED; correct as authored by sonnet

---

## Locked decisions

### D1 — Approach: custom inference handler

Per Option (A) above. Mirrors `infer_arithmetic` precedent. Minimal blast radius. Sonnet decides the exact dispatch-entry mechanism (custom handler hook OR special-case branch in the primary inference dispatcher).

### D2 — `:wat::Record/field-at` TypeScheme stays unchanged

`:wat::Record/field-at` (lines 17002-17010) is a separate primitive; its TypeScheme `record × i64 → T` works correctly via recipient inference. The correction targets `:wat::Record::of` ONLY.

### D3 — Runtime unchanged

`eval_record_of` accepts heterogeneous Vec already; no Rust change outside `src/check.rs`.

### D4 — Reuse Stone 234.2b probe as load-bearing test

No new probe file authored. `tests/probe_arc234_stone2b_defrecord_macro.rs::probe_5_multi_field_accessors_in_order` is the load-bearing test. Initial state: 1 FAIL. Post-correction state: 0 FAIL (full probe 6/6 PASS).

Probe 5 exists on disk (committed at `676e861`). It's the user-side contract; flipping it to GREEN proves the correction.

### D5 — Atomic commit with Stone 234.2b

The correction commits AS PART of the atomic Stone 234.2b shipment. Single commit message names BOTH:
- Stone 234.2b macro (`wat/Record.wat` + `src/stdlib.rs` entry + SCORE-STONE-234.2b.md)
- Stone 234.2a forward-correction (`src/check.rs` change + SCORE-STONE-234.2a-CORRECTION.md + this DESIGN doc + BRIEF + EXPECTATIONS)

Per `feedback_no_broken_commits`: don't ship broken intermediate states. The macro alone (5/6) is broken; the correction alone (no macro consumer) lacks consumer evidence. Both ship atomically.

### D6 — DESIGN.md umbrella will get a note pointing to this correction

After the atomic commit lands, update `DESIGN.md` (the arc 234 umbrella) with a brief reference to "Stone 234.2a forward-correction at <commit-hash>" so future readers crawling the arc see the correction's existence.

DESIGN.md is a living doc per FM 11 (only INSCRIPTIONs are immutable). This is appropriate forward-correction.

### D7 — SCORE-STONE-234.2a.md stays unchanged

Per `feedback_inscription_immutable`: SCORE docs are historical record. SCORE-STONE-234.2a.md says what shipped at 234.2a's commit time, including the latent gap. We don't revise it; we forward-correct via a new SCORE-STONE-234.2a-CORRECTION.md.

### D8 — Honest commit message

The commit message MUST openly acknowledge: "Stone 234.2a's TypeScheme was authored too narrowly relative to the umbrella DESIGN intent + the runtime's actual behavior; surfaced by Stone 234.2b's probe 5; corrected here." No euphemism. No "we found a substrate gap" framing — we authored an inconsistency; the correction fixes it.

Per user direction 2026-05-24: "we do not shy away from honesty."

---

## Trap-door audit

### T1 — `infer_arithmetic` dispatch-hook mechanism

Where in check.rs does the primitive-inference dispatcher route `:wat::core::+` to `infer_arithmetic`? Sonnet investigates + mirrors the same hook for `:wat::Record::of` → `infer_record_of`. Should be a match arm in the primary inference dispatcher fn.

### T2 — Coexistence: custom handler + existing TypeScheme registration

If the existing `env.register(":wat::Record::of", TypeScheme {...})` at lines 16993-17001 stays AND a custom handler is added, which wins? Sonnet investigates. Likely answer: custom handler takes precedence (special-case BEFORE generic TypeScheme path). Or the TypeScheme registration must be removed in favor of the handler.

### T3 — Vec-shape recognition for struct_form arg

The custom handler must accept arg #2 if it's a `:wat::core::vec` head OR a `[...]` Vector literal. The `[a b c]` parses to `(:wat::core::vec a b c)` per arc 109 slice 1f (verb-equals-type). The handler ACCEPTS the head; it just doesn't enforce uniform-T inference on the args.

### T4 — Empty struct_form ([])

Stone 234.2b probe 6 (zero-field) currently PASSES. The correction must not break the empty-Vec case. The custom handler should treat 0-element struct_form as valid.

### T5 — Single-field uniform struct_form

Stone 234.2b probes 1, 2, 3, 4 (single-field uniform) currently PASS. The correction must not break them. The custom handler accepts any Vec arity (0, 1, N).

### T6 — Stone 234.2a regression guard

`tests/probe_arc234_stone2a_record_primitives.rs` has 6 PASS contracts. After the correction, ALL 6 must continue to PASS. The handler must accept the existing 234.2a probe call shapes (uniform-vec arg #2).

### T7 — `:wat::Record/field-at` polymorphic-T inference

The accessor `:wat::Record/field-at` returns `:T`; recipient inference drives T. The correction does NOT touch this primitive. T6 of the prior Stone 234.2a SCORE confirmed it works; must keep working post-correction.

### T8 — Macros that already use `:wat::Record::of`

The 234.2b macro at `wat/Record.wat` is the primary consumer. The correction allows it to type-check. Any other consumer (none expected; macro is the v1 surface) would also benefit.

---

## What the load-bearing test (234.2b probe 5) demonstrates

`tests/probe_arc234_stone2b_defrecord_macro.rs::probe_5_multi_field_accessors_in_order`:

Defines `:myapp::Triple [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool]` via the 234.2b macro. The macro expansion includes:

```
(:wat::Record::of
  (:wat::core::keyword/from-string "myapp::Triple")
  [a b c]              ; heterogeneous Vec — currently TypeMismatch
  <holon-form>)
```

**Pre-correction:** TypeMismatch `expected :i64, got :String` at vec position #3.
**Post-correction:** clean type-check; runtime constructs `Value::wat__Record` with heterogeneous struct_form.

The probe also calls all three accessors (`:myapp::Triple/a`, `/b`, `/c`) and asserts each returns the correctly-typed value. The accessors use `:wat::Record/field-at` (T7 stays clean).

Result: probe 5 flips from FAIL to PASS. Full 6/6 PASS for Stone 234.2b.

---

## STOP triggers (rejection criteria)

- **STOP-1** — unexpected compile errors not tracing to the check.rs change
- **STOP-2** — lib tests baseline regresses below 827
- **STOP-3** — 40 min elapsed (small change; tight cap)
- **STOP-4** — `holon-rs` touched
- **STOP-5** — Rust changes outside `src/check.rs` (the correction is check.rs ONLY)
- **STOP-6** — scope creep: changes to `eval_record_of` runtime, to Value variant, to other primitives' TypeSchemes, to the 234.2b macro
- **STOP-7** — 234.2b probe 5 does not flip to PASS
- **STOP-8** — Stone 234.2a probe regresses (6/6 PASS must stay)
- **STOP-9** — any prior arc 234 regression guard regresses (234.0, 234.1, 234.1.5)
- **STOP-10** — clippy warnings exceed 54

Per FM 2-bis: each STOP is REJECTION criteria, not permission slot. If hit: report; surface; orchestrator decides next move.

---

## Calibration prediction

**Target runtime:** 20–35 min Mode A
**Upper bound:** 40 min (STOP-3 hard cap)
**Confidence:** high — small focused change; precedent (`infer_arithmetic`) is established; probe is committed; load-bearing test is ONE probe flipping.

**Rationale:**
- check.rs change: ~30-50 lines
- Read precedent (`infer_arithmetic` + its dispatch hook): ~10 min
- Author + compile cycles: ~10-15 min (expected 1-2 rounds)
- SCORE writing: ~5-10 min

**Risks:**
- **T1 dispatch-hook location** — finding the exact insertion point in the primary inference dispatcher might take 5 min of orientation
- **T2 coexistence rule** — whether to remove the existing TypeScheme registration OR rely on dispatcher precedence; sonnet investigates + chooses

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella (line 19: `struct_form: Arc<Vec<Value>>` heterogeneous intent)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a.md` — predecessor SCORE (stays immutable per `feedback_inscription_immutable`)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — sibling SCORE (sonnet authored; ships in same atomic commit)
- `tests/probe_arc234_stone2b_defrecord_macro.rs::probe_5_multi_field_accessors_in_order` — load-bearing test (flips FAIL → PASS)
- `tests/probe_arc234_stone2a_record_primitives.rs` — regression guard (6/6 PASS stays)
- `src/check.rs` line 10885-10980 — `infer_arithmetic` precedent for custom-handler pattern
- `src/check.rs` line 16989-17001 — existing `:wat::Record::of` TypeScheme registration (target of correction)
- `src/runtime.rs::eval_record_of` (line ~14543) — runtime accepts heterogeneous; correct since 234.2a
- `feedback_inscription_immutable.md` — SCORE-STONE-234.2a.md stays unchanged
- `feedback_no_broken_commits.md` — atomic commit with 234.2b (don't ship intermediate broken states)
- `feedback_any_defect_catastrophic.md` — substrate trust is binary; the correction restores binary-green state
