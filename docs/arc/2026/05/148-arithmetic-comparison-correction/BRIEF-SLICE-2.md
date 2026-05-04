# Arc 148 Slice 2 — Sonnet Brief — Rename per-Type arithmetic leaves to `,2`

**Drafted 2026-05-03.** Foundation slice. Substrate-informed:
orchestrator's slice 1 audit (`AUDIT-SLICE-1.md`) found 8 existing
per-Type arithmetic Rust primitives at bare names; this slice
renames them to add `,2` suffix so slice 4 can place variadic wat
wrappers at the freed bare names.

FM 9 baseline confirmed pre-spawn (2026-05-03 16:11 UTC):
- `wat_arc146_dispatch_mechanism` 7/7
- `wat_arc144_lookup_form` 9/9
- `wat_arc144_special_forms` 9/9
- `wat_arc144_hardcoded_primitives` 17/17
- `wat_arc143_define_alias` 3/3

**Goal:** rename 8 substrate primitives, sweep every call site,
verify green. NO new entities. NO architectural change beyond the
rename. NO `eval_*` body changes (just the registration name +
TypeScheme registration name + freeze pipeline list entry +
caller updates).

**Working directory:** `/home/watmin/work/holon/wat-rs/`

## Required pre-reads (in order)

1. **`docs/arc/2026/05/148-arithmetic-comparison-correction/DESIGN.md`**
   — slice plan section + slice 2 entry. Architecture context.
2. **`docs/arc/2026/05/148-arithmetic-comparison-correction/AUDIT-SLICE-1.md`**
   — Open Question 2 + the "Already wired" subsections. Source of
   truth for which names exist where.
3. **`docs/COMPACTION-AMNESIA-RECOVERY.md`** — discipline. Esp. § 12
   (foundation work; eliminate failure domains, don't bridge them).
4. **`src/runtime.rs:2514-2529`** — `eval_i64_arith` registration arms
   (the names that get renamed for i64).
5. **`src/runtime.rs:2552-2561`** — `eval_f64_arith` registration arms
   (same for f64).
6. **`src/check.rs:8718-8750`** — TypeScheme registrations for the
   per-Type i64 and f64 arithmetic leaves.
7. **`src/runtime.rs:15605-15641`** — freeze pipeline pure-redex list
   (must include the `,2`-suffixed names; old names removed).

## What ships

### 8 substrate primitive renames

| Old name | New name |
|---|---|
| `:wat::core::i64::+` | `:wat::core::i64::+,2` |
| `:wat::core::i64::-` | `:wat::core::i64::-,2` |
| `:wat::core::i64::*` | `:wat::core::i64::*,2` |
| `:wat::core::i64::/` | `:wat::core::i64::/,2` |
| `:wat::core::f64::+` | `:wat::core::f64::+,2` |
| `:wat::core::f64::-` | `:wat::core::f64::-,2` |
| `:wat::core::f64::*` | `:wat::core::f64::*,2` |
| `:wat::core::f64::/` | `:wat::core::f64::/,2` |

For each:
- Update the `env.register(...)` call in `src/runtime.rs` to use
  the new name (eval body unchanged)
- Update the matching `TypeScheme` registration in `src/check.rs`
  to use the new name
- Update the freeze pipeline pure-redex inclusion list at
  `src/runtime.rs:15605-15641` to use the new name

### Mass call-site sweep

Find every call site using one of the 8 old names and update to
the `,2` form. Likely locations:

- `tests/wat_*.rs` — embedded wat strings
- `wat-tests/**/*.wat` — wat test files
- `examples/**/*.wat` — example programs
- `crates/wat-lru/wat-tests/**/*.wat` — sub-crate tests
- `wat/holon/*.wat` — algebra idiom files
- `wat/std/*.wat` — stdlib files

Use grep to find all occurrences:
```bash
grep -rn ":wat::core::i64::[+\-*/]\b\|:wat::core::f64::[+\-*/]\b" \
  src/ tests/ wat/ wat-tests/ examples/ crates/
```

Update each occurrence.

**Boundary:** the lab repo (`holon-lab-trading/`) is OUT OF SCOPE
for this slice. Lab code consumes wat-rs as a downstream — if lab
breaks, that's a separate consumer-update arc post-148-close.

### What does NOT change

- Comparison per-Type leaves (`:wat::core::i64::<`, etc.) — NOT
  renamed. They get retired entirely in slice 5.
- `:wat::core::f64::abs`, `:wat::core::f64::max`, `:wat::core::f64::min`
  — NOT renamed. They have no polymorphic counterpart; no variadic
  wrapper at the bare name is planned.
- `eval_*` function bodies — unchanged. Only registration names.

## What this slice does NOT do

- NO new substrate primitives registered (it's a rename).
- NO new wat files.
- NO new TypeSchemes (renamed, not added).
- NO retirement of any handler (`infer_polymorphic_arith` stays;
  slice 4 retires it).
- NO new tests for new behavior (existing tests + sweep updates only).

## STOP at first red

If you discover a call site you can't update mechanically (e.g.,
the call is constructed dynamically from a string variable), STOP
and report the location. Do NOT attempt to invent a workaround.
The orchestrator will reconcile.

If a test FAILS after the sweep, STOP. The rename should be purely
mechanical — a failing test means either (a) a call site was missed,
or (b) the substrate has hidden coupling at one of the names that
the audit missed. Report which test, what the failure says, and what
your investigation found.

## Source-of-truth files

- `src/runtime.rs:2514-2529` — i64 arith registration arms
- `src/runtime.rs:2552-2561` — f64 arith registration arms
- `src/check.rs:8718-8750` — TypeScheme registrations
- `src/runtime.rs:15605-15641` — freeze pipeline pure-redex list

Use `grep -rn` aggressively. Every claim ships with a file:line
citation.

## Honest deltas

If you find call sites in places the brief didn't anticipate,
surface as honest delta. Examples:
- A `holon-rs` crate consumer
- A `wat-macros/` reference
- A `Cargo.toml` doc-test

These are signals; surface them.

## Report format

After shipping:

1. Total renames performed (should be 8)
2. Total call sites updated (count)
3. Files touched (list)
4. Test results: list which tests changed and confirm all green
5. Workspace failure profile (per FM 9: should be unchanged from
   pre-slice baseline plus the documented `CacheService.wat` noise)
6. Any honest deltas surfaced

Time-box: 60 min wall-clock. Predicted Mode A 30-45 min — the work
is mechanical but spans multiple files.

## What this unlocks

Slice 4 (numeric arithmetic migration) can place variadic wat
function wrappers at the freed bare names (`:wat::core::i64::+`,
etc. become available for the new `(:wat::core::i64::+ 1 2 3 4 5)
=> i64 15` UX).
