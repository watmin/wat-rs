# Arc 148 Slice 5 — Sonnet Brief — Numeric comparison cleanup

**Drafted 2026-05-03.** Cleanup slice. Substrate-informed:
orchestrator's slice 1 audit + slice 3 buildout already established
that the polymorphic comparison surface (`:wat::core::=`, `:not=`,
`:<`, `:>`, `:<=`, `:>=`) works universally via `values_compare` /
`values_equal`. Slice 5 retires the redundant per-Type leaves and
the legacy polymorphic-handler anti-pattern.

FM 9 baseline confirmed pre-spawn (post-slice-3):
- `wat_arc146_dispatch_mechanism` 7/7
- `wat_arc144_lookup_form` 9/9
- `wat_arc144_special_forms` 9/9
- `wat_arc144_hardcoded_primitives` 17/17
- `wat_arc143_define_alias` 3/3
- `wat_polymorphic_arithmetic` 20/20
- `wat_arc148_ord_buildout` 46/46

**Goal:** retire 10 per-Type comparison leaves
(`:wat::core::{i64,f64}::{=,<,>,<=,>=}`) + sweep call sites + retire
the `infer_polymorphic_compare` polymorphic-handler anti-pattern.
NO new substrate primitives shipped — the polymorphic Rust functions
(`eval_eq`, `eval_compare`, `eval_not_eq`) already exist at the bare
names + `values_compare`/`values_equal` already do universal
delegation including mixed-numeric promotion.

**Working directory:** `/home/watmin/work/holon/wat-rs/`

## Required pre-reads (in order)

1. **`docs/arc/2026/05/148-arithmetic-comparison-correction/DESIGN.md`**
   — § "Comparison family (6 ops × 1 entity = 6 names)" + § "Slice 5"
   entry + § "The comma-typed-leaf rule (LLM affordance)".
2. **`docs/arc/2026/05/148-arithmetic-comparison-correction/AUDIT-SLICE-1.md`**
   — Open Question 2 (per-Type comparison leaves at bare names) +
   handler section for `infer_polymorphic_compare`.
3. **`docs/arc/2026/05/148-arithmetic-comparison-correction/SCORE-SLICE-3.md`**
   — what slice 3 shipped (`values_compare` extracted; universal
   delegation working including mixed-numeric).
4. **`docs/COMPACTION-AMNESIA-RECOVERY.md`** — § FM 9 (baselines done);
   § 12 (foundation work — eliminate failure domains, don't bridge).
5. **`src/runtime.rs:2607-2620`** — the 10 per-Type comparison leaf
   dispatch arms to retire.
6. **`src/runtime.rs:15716-15731`** — the 10 per-Type comparison leaf
   freeze-pipeline entries (or whatever range they actually occupy
   post-slice-2; sonnet's grep confirms exact lines).
7. **`src/check.rs:9013-9040`** — the 10 per-Type comparison leaf
   TypeScheme registrations (i64 + f64).
8. **`src/check.rs:3292`** — the dispatch site routing comparison ops
   to `infer_polymorphic_compare`.
9. **`src/check.rs:6567`** — `infer_polymorphic_compare` itself.

## What ships

### Substrate retirements

| Site | What to retire | Replacement |
|---|---|---|
| `src/runtime.rs:2607-2613` | 5 i64 comparison dispatch arms | (none — polymorphic `:wat::core::=` etc. handles via existing eval_eq/eval_compare/eval_not_eq) |
| `src/runtime.rs:2614-2620` | 5 f64 comparison dispatch arms | (none — same reason) |
| `src/runtime.rs:15716-15731` | 10 freeze-pipeline pure-redex entries (i64+f64 × 5 ops) | (none — polymorphic bare-name entries already in the list) |
| `src/check.rs:9013-9024` | 5 i64 TypeScheme registrations | (none — slice 5 doesn't need replacements; polymorphic ops handled by check-side function below) |
| `src/check.rs:9027-9040` | 5 f64 TypeScheme registrations | same |

**Total retired: 10 per-Type comparison leaf primitives** (5 ops × 2 types).

### `infer_polymorphic_compare` — the polymorphic-handler anti-pattern

This function (`src/check.rs:6567`) is the check-side handler for
`:wat::core::=`, `:not=`, `:<`, `:>`, `:<=`, `:>=`. Per arc 148 + arc
146's "retire polymorphic-handler anti-pattern" discipline, it
should be eliminated — but **comparison's check-time semantics
genuinely require custom inference** (any-same-type-with-PartialOrd
+ mixed-numeric exception). One arc 146 Dispatch entity can't
express this without an arm-explosion.

**Two paths:**

(a) **Keep the function but RENAME** to drop the
"polymorphic-handler anti-pattern" framing. Rename to e.g.
`infer_comparison` to reflect what it IS (the check-side
signature inference for `:wat::core::<` family) rather than the
anti-pattern label. Simplify if possible (remove the numeric
arms' mixed-type tracking since `values_compare` handles routing
at runtime; check just confirms args are comparable).

(b) **Promote to TypeScheme registration** if the substrate
supports parametric `∀T: Comparable. (T, T) → bool`. Sonnet
audits whether this is feasible; if not, falls back to (a).

**Default to (a)** unless sonnet finds (b) is genuinely simpler.
Surface the choice in the report.

### Call-site sweep

Find every call site using one of the 10 retired per-Type
comparison leaf names and update to the polymorphic bare name:

```bash
grep -rn ':wat::core::i64::=\|:wat::core::i64::<\|:wat::core::i64::>\|:wat::core::i64::<=\|:wat::core::i64::>=' \
  src/ tests/ wat/ wat-tests/ examples/ crates/

grep -rn ':wat::core::f64::=\|:wat::core::f64::<\|:wat::core::f64::>\|:wat::core::f64::<=\|:wat::core::f64::>=' \
  src/ tests/ wat/ wat-tests/ examples/ crates/
```

Each call site updates from per-Type form to polymorphic bare form:
- `(:wat::core::i64::< a b)` → `(:wat::core::< a b)`
- Same for all 10 retired names

Strict type-locking previously available via per-Type comparison
leaves is now achieved via param types in the call site's enclosing
function — the type system enforces at the binding site.

### What does NOT change

- `eval_eq`, `eval_compare`, `eval_not_eq` (`src/runtime.rs:4424+`,
  `:4603+`, `:4464+`) — STAY UNCHANGED at the polymorphic bare names
- `values_compare`, `values_equal` — STAY UNCHANGED (slice 3's work)
- The polymorphic dispatch site at `src/runtime.rs:2593-2600` for
  the 6 polymorphic comparison ops — STAYS UNCHANGED
- NO new substrate primitives REGISTERED
- NO new wat files
- NO arithmetic changes (slice 4 territory)

## What this slice does NOT do

- NO new substrate primitives.
- NO comma-typed comparison leaves shipped (per the comma-typed-leaf
  rule — comparison's `values_compare` handles mixed-numeric
  universally).
- NO modifications to `eval_eq` / `eval_compare` / `eval_not_eq` body.
- NO arc 146 Dispatch entity creation for comparison (would need
  arm-explosion to express universal delegation).
- NO retirement of arithmetic per-Type leaves (slice 4 work; or
  preserved as substrate addressing).

## STOP at first red

If you discover a call site using a retired per-Type comparison
leaf in a way that the polymorphic bare name CANNOT replace
(e.g., relying on the strict type-locking that only the per-Type
TypeScheme provides), STOP and report. Don't improvise a workaround.
The orchestrator decides whether to keep the leaf or refactor the
caller.

If `infer_polymorphic_compare` removal/rename breaks tests in
non-obvious ways (e.g., the check-side signature for the polymorphic
ops becomes unresolvable), STOP and report what you found.

If the freeze pipeline pure-redex list has dependencies on the
per-Type comparison leaf names that aren't obvious from the audit,
surface them.

## Source-of-truth files

- `src/runtime.rs:2593-2620` — polymorphic + per-Type comparison
  dispatch arms
- `src/runtime.rs:15716-15731` — freeze pipeline pure-redex list
  (per-Type comparison entries)
- `src/check.rs:3292` — `infer_polymorphic_compare` dispatch site
- `src/check.rs:6567` — handler body
- `src/check.rs:9013-9040` — per-Type comparison TypeScheme
  registrations

## Honest deltas

If you find:
- A call site that can't trivially convert (suggesting the per-Type
  leaf provided real value beyond cosmetic type-locking)
- A substrate dependency on the per-Type names (e.g., a hardcoded
  string match somewhere in macro-expansion or freeze pipeline that
  the audit missed)
- A test that relies on the per-Type leaf signature differing from
  the polymorphic shape

Surface as honest delta. These are signals worth recording.

## Report format

After shipping:

1. Total per-Type leaves retired (should be 10 = 5 ops × 2 types)
2. Path chosen for `infer_polymorphic_compare` ((a) rename + simplify
   vs (b) promote to TypeScheme) + rationale
3. Total call sites swept (count + files list)
4. Test results: list which tests changed; confirm baselines green
5. Workspace failure profile (per FM 9: should be unchanged from
   pre-slice baseline plus the documented `CacheService.wat` noise)
6. Any honest deltas surfaced

Time-box: 60 min wall-clock. Predicted Mode A 25-40 min — scope
smaller than slice 2 (10 names sweep vs slice 2's 8 names + crypto
regen).

## What this unlocks

Slice 4 (numeric arithmetic migration) and slice 6 (closure) are
the only remaining slices in arc 148. Slice 5's cleanup brings the
substrate's comparison surface to its final shape: 6 polymorphic
bare-name entities; zero per-Type comparison leaves; zero comma-typed
comparison leaves; one cleaned/renamed check-side inference function.

Maximum LLM affordance for comparison achieved.
