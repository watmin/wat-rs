# Arc 148 Slice 1 — Sonnet Brief — AUDIT (no code changes)

**Drafted 2026-05-03.** Substrate-informed: orchestrator crawled
`src/check.rs:3285-3360` (handler dispatch sites — 7
`infer_polymorphic_*` handlers identified), `src/check.rs:6567+`
(handler bodies — type acceptance logic), `src/runtime.rs:2593-2631`
(9 user-facing op runtime arms), `src/runtime.rs:4424` (eval_eq),
`src/runtime.rs:4603` (eval_compare), `src/runtime.rs:4677`
(eval_poly_arith), `src/runtime.rs:15605-15614` (freeze pipeline
dispatch site).

FM 9 baseline confirmed pre-spawn (2026-05-03):
- `wat_arc146_dispatch_mechanism` 7/7
- `wat_arc144_lookup_form` 9/9
- `wat_arc144_special_forms` 9/9
- `wat_arc144_hardcoded_primitives` 17/17
- `wat_arc143_define_alias` 3/3

**Goal:** produce `AUDIT-SLICE-1.md` enumerating the existing
surfaces of all 7 `infer_polymorphic_*` handlers in `src/check.rs`
+ their runtime counterparts. NO code changes. NO test changes.
Pure documentation deliverable that informs implementation slices
2-3 (numeric arithmetic + numeric comparison migration) and the
parallel-track work on Categories B (time-arith) + C (holon-pair).

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/148-arithmetic-comparison-correction/DESIGN.md`**
   — full arc design. The naming convention (Type-as-namespace +
   comma-mixed), three-layer arithmetic, substrate-primitive +
   selective-mixed-arms comparison, Category A universal delegation,
   min-2 arity, decision history, gaze trail.
2. **`docs/COMPACTION-AMNESIA-RECOVERY.md`** — the discipline.
   Especially § FM 4 (read disk, don't ask), § FM 9 (re-run baselines
   pre-spawn — already done), § FM 10 (entity-kind discipline).
3. **`src/dispatch.rs`** (445 LOC) — arc 146 Dispatch entity. Read
   the doc comment at lines 1-30 to understand fixed-arity contract;
   read `pub struct Dispatch` + `pub struct DispatchArm` to know the
   shape that the eventual implementation slices will use.
4. **`docs/arc/2026/05/146-container-method-correction/DESIGN.md`** —
   the precedent this arc extends. Same Dispatch entity; same per-Type
   impls registered as substrate primitives; same retirement of old
   polymorphic handlers.
5. **`src/check.rs:3285-3360`** — the dispatch site routing
   user-facing op keywords to the 7 polymorphic_* handlers. Source of
   truth for which user-facing ops each handler serves.
6. **`src/runtime.rs:2593-2631`** — the runtime dispatch site. Source
   of truth for which `eval_*` handles each op at runtime.

## Goal — AUDIT-SLICE-1.md content

Produce a single Markdown file `AUDIT-SLICE-1.md` in this arc's
directory. Contents organized as 7 handler sections + a category
summary + an open-questions section.

### Per-handler section (7 of these)

For EACH of the 7 polymorphic_* handlers in `src/check.rs`:

```
## Handler — `infer_polymorphic_<name>` (check.rs:<line>)

### User-facing ops served

(Enumerate every op keyword that dispatches to this handler. Source:
the `match` arm in check.rs around line 3290. Example for arith:
`:wat::core::+`, `:wat::core::-`, `:wat::core::*`, `:wat::core::/`.)

### Argument acceptance

(Read the handler body. For each parameter position: which Types are
accepted? What's the unification logic? What's the result type?
Quote 1-3 lines from the handler body if it clarifies.)

### Mixed-type signatures (if applicable)

(Some handlers explicitly list cross-type combinations. Enumerate
them. For arith: numeric promotion (i64, f64) → f64. For time-arith:
the (Instant, Instant) → Duration etc. signatures. For holon-pair:
the Holon-or-Vector cross-acceptance.)

### Runtime impl

(Which `eval_*` function in `src/runtime.rs` handles each op? The
runtime dispatch site at `runtime.rs:2593-2631` is the source of
truth.)

### Arc 148 categorization

(Per DESIGN: which category does this handler belong to?
- IMMEDIATE numeric (slices 2-3) — arithmetic + comparison
- DEFERRED Category B (time-arith)
- DEFERRED Category C (holon-pair algebra)
- UNIVERSAL (handled by Category A's same-type delegation rule —
  applies to comparison handler's non-numeric branch)
)
```

### Category summary

After the 7 handler sections, a summary section that maps
arc 148's three architectural categories to the audit's findings:

```
## Category mapping

### Category — Numeric arithmetic (arc 148 slice 2)
- Handler: `infer_polymorphic_arith`
- Surface: 4 ops × 8 entities = 32 names per DESIGN
- Per-Type leaves: i64::+ / f64::+ + i64::+,2 / f64::+,2 + +,i64-f64 / +,f64-i64 (per op)

### Category — Numeric comparison (arc 148 slice 3)
- Handler: `infer_polymorphic_compare` (numeric branch)
- Surface: 6 ops × 3 entities = 18 names per DESIGN
- No per-Type same-type leaves; substrate primitive uses Rust trait dispatch

### Category A — Non-numeric eq/ord (NOT deferred; served by slice 3's substrate primitive)
- Handler: `infer_polymorphic_compare` (non-numeric branch)
- Universal same-type delegation handles String, Time, Bytes, Vector, Tuple, Option, Result, Holon, etc.
- Ord allowlist (slice 3 substrate decision): which Types support :<>=
- Equality is universal (any PartialEq Type)

### Category B — Time arithmetic (parallel user track)
- Handler: `infer_polymorphic_time_arith`
- 2 ops × 3 signatures (Instant ± Duration patterns)

### Category C — Holon-pair algebra (parallel user track)
- 4 handlers, 5 user-facing ops
- (Enumerate exact ops + signatures from each handler body)
```

### Open questions

A section listing UNKNOWNS the audit surfaces that the
DESIGN doesn't cover. Examples (not exhaustive — sonnet may find
more):

- Does the substrate have a "PartialOrd-allowlisted Type" registry
  already (referenced or implied by `is_numeric` / `unify` /
  `eval_compare`)? If not, slice 3 will need to build one.
- Are any per-Type comparison leaves (e.g., `:wat::core::i64::<`)
  already wired in `register_builtins` or `wat/core.wat`? If so,
  arc 148 needs to reconcile (extend, retire, or leave alone).
- Does `:wat::core::not=` route to its own runtime fn or is it
  computed as `(:wat::core::not (:= a b))`? Affects slice 3 scope.
- Are any of the holon-pair handlers (Category C) actually in
  use today by the holon module's wat code, or are they vestigial?
  Affects deferred-track work prioritization.
- The `infer_polymorphic_time_arith` handler at check.rs:6698
  declares Instant + Duration arithmetic but the LHS-must-be-Instant
  rule asymmetric — is `(Duration + Instant)` legal anywhere? Audit
  the runtime to confirm.

Surface anything that would cause the implementation slices (2-3)
to discover the substrate doesn't match what DESIGN assumes.

## What this slice does NOT do

- NO new files in `src/`, `wat/`, `wat-tests/`, `tests/`
- NO modifications to existing source files
- NO new tests
- NO retirement of any handler
- NO migration of any user-facing op
- NO substrate primitives registered

The deliverable is one new Markdown file: `AUDIT-SLICE-1.md`.

## STOP at first red

If during the audit you discover that DESIGN.md's architectural
assumptions are factually wrong (e.g., the `infer_polymorphic_compare`
handler doesn't actually accept the Types DESIGN assumes; the
runtime dispatch path differs significantly from what DESIGN sketches;
the existing convention CONVENTIONS.md cites isn't actually
implemented), STOP and report. Do NOT attempt to "fix" the DESIGN
in your audit doc — surface the discrepancy in the open-questions
section so the orchestrator can reconcile.

## Source-of-truth files

These are the files you read to produce the audit. DO NOT consult
external memory or assume — every claim in your audit doc must cite
a file:line reference.

- `src/check.rs:3285-3360` — handler dispatch (which op → which handler)
- `src/check.rs:6567+` — handler bodies (acceptance logic)
- `src/runtime.rs:2593-2631` — runtime op dispatch
- `src/runtime.rs:4424+` — runtime impl bodies (eval_eq, eval_compare, eval_poly_arith)
- `src/runtime.rs:15605-15614` — freeze pipeline dispatch
- `src/dispatch.rs` — the Dispatch entity arc 148 will use

## Honest deltas

If you discover something that CHANGES the slice 2 or slice 3 plan,
surface it in the open-questions section. Examples:

- Existing per-Type leaves already wired that aren't documented
- Runtime op routing that DESIGN's sketch gets wrong
- A type the substrate accepts that DESIGN doesn't account for
- A coupling between handlers DESIGN treats as independent

These are signals; they're valuable. Surface them.

## Report format

After you ship the AUDIT, write a ~200-word report covering:

1. Total handlers audited (should be 7)
2. Total user-facing ops enumerated across all handlers
3. Whether DESIGN.md's architectural assumptions hold
4. Any open questions that affect slice 2 or 3 planning
5. Predicted complexity of slices 2-3 based on the audit (rough
   sense — does the migration look like arc 146's slice 2 length
   migration, or is there structural complexity DESIGN underestimated?)

Time-box: 60 min wall-clock. Pure-audit slice; ~30-50 min predicted.
