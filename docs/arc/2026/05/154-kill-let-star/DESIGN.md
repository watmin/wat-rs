# Arc 154 — Kill `:wat::core::let*`; make `:wat::core::let` sequential

**Status:** opened 2026-05-06 evening, after arc 153 (unit→nil) and
arc 136 (do form) closed earlier the same session.

## User direction (verbatim)

> *"ok... a thing i wanted to deal with a while ago.... can we
> kill let\* and just make let be what let\* is... clojure just
> aliases let\* to let - i just want let like clojure has it"*

> *"new arc - let's do it"*

## Goal

Single letform name. `:wat::core::let` becomes the sequential
binding form (current `:wat::core::let*` semantics); `:wat::core::let*`
retires. Wat's letform vocabulary collapses to one keyword,
matching Clojure's user-facing surface (Clojure's `let` is the
sequential primitive; Clojure's `let*` is a substrate-internal
form not part of normal user code).

After arc 154:

```scheme
(:wat::core::let
  (((a :i64) 10)
   ((b :i64) (:wat::core::i64::+ a 5)))   ;; b sees a — sequential
  (:wat::core::i64::+ a b))
```

`:wat::core::let*` is a parse error post-arc-154 with a
substrate-as-teacher migration hint per Pattern 3.

## Provenance + the previous attempt

Task #185 originally read *"arc 109 follow-up: rename `let*` → `let`."*
Surfaced during arc 109's symbol-cleanup sweep. The task was
marked **SUPERSEDED** in arc 145 with the note *"user kept both
forms in arc 145"* — the user's 2026-05-03 evening direction at
the time chose to preserve both forms as a deliberate
parallel-vs-sequential distinction.

Arc 145 then backed out as foundation-correction-non-shipping
when the typed-let UX failure surfaced. With arc 145's typed-let
detour closed and the cleaner foundation insight in hand, the
user reopened the rename today with a different framing:

> *"clojure just aliases let\* to let - i just want let like
> clojure has it."*

The earlier "preserve both forms" stance was based on
parallel-vs-sequential being a meaningful user choice. Empirical
evidence as of 2026-05-06: **zero `:wat::core::let` (parallel)
sites exist in the codebase.** The parallel form was minted but
never reached for. The "user's choice" the earlier stance
preserved was a choice nobody made.

Arc 154 closes the rename cleanly with empirical justification:
nobody uses parallel let; sequential is strictly more permissive
(any site that worked under parallel works under sequential too);
the rename is purely cosmetic at the consumer surface.

## What this arc does

### Substrate

1. **Switch `:wat::core::let` semantics from parallel to
   sequential.** The current `infer_let_star` / `eval_let_star`
   logic moves under the `let` keyword. The current parallel
   `infer_let` / `eval_let` paths retire (zero consumers).

2. **Mint `BareLegacyLetStar` walker** on `:wat::core::let*`
   Path detection per substrate-as-teacher Pattern 3. The
   walker emits a migration hint naming `:wat::core::let` as
   the canonical FQDN. Mirrors arc 109 slice 1d's
   `BareLegacyUnitType` / arc 153's `BareLegacyUnitName`
   precedents.

3. **Tail-call + step paths** mirror — `eval_let_tail` (was
   `eval_let_star_tail`); `step_let` (was `step_let_star`).

4. **Special-form registry** updated — `:wat::core::let`
   sketch reflects the sequential semantics; `:wat::core::let*`
   either retires from registry entirely OR stays as a
   walker-firing entry that surfaces the migration hint at
   reflection time. Slice 1a chooses.

### Consumer migration

Workspace-wide sweep ~827 sites. Substrate-as-teacher
walker-driven (mirrors arc 153 sweep 1b):

- **`wat/*.wat`** stdlib (~141 sites)
- **`crates/*/wat/**/*.wat`** per-crate substrates
- **`wat-tests/**/*.wat`** workspace tests
- **`crates/*/wat-tests/**/*.wat`** per-crate tests
- **`examples/**/*.wat`**
- **Embedded wat in `tests/*.rs` + `src/*.rs`** (~391 sites)

Mechanical 1:1 transform: every `:wat::core::let*` site
becomes `:wat::core::let`. No semantic change at the consumer
(sequential semantics preserved).

### Retirement

Per substrate-as-teacher § "Retire the hint when its window
closes" — same recipe as arc 153 slice 2:

- Drop walker body once sweep is structurally complete
- Retain `CheckError::BareLegacyLetStar` variant + Display as
  orphaned scaffolding per arc 113 precedent
- Drop `:wat::core::let*` registry entry (or strikethrough with
  arc 154 reference if a record-style note serves)

## The four questions

Run on the arc shape 2026-05-06:

1. **Obvious?** YES. Single letform name across the codebase;
   Clojure-familiar; one fewer special form to learn.
2. **Simple?** YES. Sequential is a strict superset of parallel
   in terms of accepted code; renaming the keyword has no
   consumer breakage. Substrate change moves logic between two
   already-implemented paths.
3. **Honest?** YES. Zero parallel-let consumers makes the
   rename's "no breakage" claim empirical, not theoretical.
4. **Good UX?** YES. One letform vocabulary; reads like
   Clojure; nothing for the user (or LLM) to remember about
   "when do I use let vs let*."

## Slice plan

### Slice 1a — substrate

Switch `:wat::core::let` to sequential semantics; mint
`BareLegacyLetStar` walker; retire parallel-let paths;
update tests covering retired-vs-canonical surfaces. Mirror
arc 153 slice 1a's substrate shape.

DO NOT COMMIT (atomic with sweep 1b per recovery doc § 7).

### Slice 1b — consumer sweep

~827 mechanical 1:1 transforms `:wat::core::let*` →
`:wat::core::let` driven by the substrate's diagnostic stream
(BareLegacyLetStar walker fires per offending site).

Atomic commit with slice 1a when workspace = 0 failed.

### Slice 2 — retirement + paperwork

- Retire walker body; retain CheckError variant scaffolding
- Drop `:wat::core::let*` registry entry
- Update arc 154 tests for post-retirement behavior
- INSCRIPTION + 058 row + USER-GUIDE update + WAT-CHEATSHEET
  update + task #185 marked CLOSED (closed by arc 154; no longer
  SUPERSEDED)
- Pre-INSCRIPTION grep mandatory per FM 11
- Orchestrator-side INSCRIPTION synthesis per
  `feedback_paperwork_orchestrator_side.md`

## Cross-references

- **Task #185** — originally "rename let\* → let" follow-up;
  SUPERSEDED in arc 145; reopened with corrected direction
  in this arc; closure marks it CLOSED-by-arc-154
- **Arc 109 slice 1d** — Pattern 3 walker precedent
  (`BareLegacyUnitType` retired `:()` type-position spelling)
- **Arc 153** — closest precedent (same recipe; same
  mechanics; closed earlier the same day)
- **Arc 113** — orphaned scaffolding precedent (variant +
  Display preserved after firing body retires)
- **Arc 145** — the typed-let detour whose backout closed
  the "preserve both forms" stance; arc 154 is the cleaner
  vocabulary correction that arc 145's failure paid for
- **Clojure reference** — `let` is the user-facing sequential
  primitive; `let*` is internal substrate not part of normal
  user surface

## Why this matters

> *"i need a lisp on rust to satisfy what we're building
> towards"*

Two foundation marks landed today (`nil`, `do`); arc 154 lands
the third — single-letform vocabulary matching Clojure's
user-facing surface. Three marks in one session that shape the
substrate's Lisp identity. Arc 109 v1 closure trajectory clearer
by another link.

## Estimated effort

- Slice 1a substrate: ~25-40 min sonnet wall-clock (matches
  arc 153 slice 1a profile)
- Slice 1b consumer sweep: ~80-120 min sonnet wall-clock
  (~827 sites; ~1.8x arc 153 sweep 1b's ~455-site count;
  but mechanical 1:1 is faster per site than arc 153's
  type-position-and-value-position dual sweep)
- Slice 2 retirement + paperwork: ~30 min orchestrator + ~15
  min sonnet (mechanical retirement)
- Total: ~2 hours wall-clock if Mode A clean throughout

## Status notes

- Drafted 2026-05-06 evening
- Slice 1a BRIEF + EXPECTATIONS to be drafted next
- Closure paperwork orchestrator-side per discipline restored
  earlier this session
