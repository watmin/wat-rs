# SCORE — Stone S-C.1 — rename `Value::wat__Record` → `Value::wat__holon__Record`

**Date:** 2026-05-26
**Status:** COMPLETE — build clean; 827/0 lib baseline; all regression probes green.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| grep "^error"` | 0 errors (pre-existing warnings ceiling) |
| 2 | **Lib baseline 827/0** (LOAD-BEARING) | `cargo test --release --lib -p wat 2>&1 \| tail -5` | `827 passed; 0 failed` |
| 3 | S-A regression | `cargo test --release --test probe_arc237_sA_hierarchy 2>&1 \| tail -5` | `10 passed; 0 failed` |
| 4 | S-B.1 regression | `cargo test --release --test probe_arc237_sB1_recordtype 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 5 | S-B.2 regression | `cargo test --release --test probe_arc237_sB2_defrecord_recordtype 2>&1 \| tail -5` | `5 passed; 0 failed` |
| 6 | S-A1 regression | `cargo test --release --test probe_arc237_sA1_assignable 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 7 | arc234 stone5 regression | `cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 8 | arc227 stone2 regression | `cargo test --release --test probe_arc227_stone2_defrecord 2>&1 \| tail -5` | `35 passed; 0 failed` |
| 9 | src/ only | STOP-2 check | confirmed — zero files outside `src/` touched |

---

## The rename

### Variant identifier

`Value::wat__Record` → `Value::wat__holon__Record` at all match/construct/comment sites across `src/`.

Total sites renamed: **53**

| File | Sites renamed |
|------|---------------|
| `src/runtime.rs` | 45 |
| `src/check.rs` | 1 |
| `src/edn_shim.rs` | 2 |
| `src/closure_extract.rs` | 3 |
| `src/stdlib.rs` | 1 |
| `src/types.rs` | 1 |

### Discriminant tag string

`src/runtime.rs:1116` `"wat__Record".hash(state)` → `"wat__holon__Record".hash(state)`

Comment at ~1113 updated to match: `Discriminant tag "wat__holon__Record" prevents cross-variant collisions.`

---

## Surface-type colon-strings — explicitly untouched

All returned surface-type strings were left unchanged per Rule 3:

- `src/runtime.rs:1269` — `Value::wat__holon__Record { .. } => "wat::Record"` (variant arm renamed; returned string `"wat::Record"` untouched)
- `src/runtime.rs:7544` — `Value::wat__holon__Record { .. } => ":wat::Record"` (variant arm renamed; returned string `":wat::Record"` untouched)
- All `:wat::Record` / `"wat::Record"` colon-form strings in `types.rs`, `check.rs`, `stdlib.rs`, and the remainder of `runtime.rs` — untouched. (These contain `wat::Record` with colons, not `wat__Record` with double-underscores; `replace_all` on `wat__Record` cannot reach them — confirmed by post-edit grep showing zero unintended changes.)

The holonic value's surface type stays `:wat::Record` for now; the `:wat::holon::Record` surface divergence is S-C.3, NOT this stone.

---

## Honest deltas

### Method

The variant definition at `src/runtime.rs:651` was renamed first. `cargo build --release -p wat` was then run — it produced **zero errors**, confirming the compiler found no remaining sites (the `replace_all` sweep was exhaustive before the build attempt). No iterative fix rounds were needed.

### Why 53 sites, not ~70

The BRIEF estimated ~70 sites as an upper bound from the prior stone's calibration note. The actual count is 53. The difference is: the BRIEF counted "match/construct sites" broadly; the actual grep on `wat__Record` (double-underscore, variant identifier) in `src/` yields 53 distinct occurrences across 6 files. The discriminant tag string at runtime.rs:1116 is included in the 53 (it contains `wat__Record` as a literal).

### No cascade rounds

One sweep pass, one build, zero errors. Compiler confirmed the sweep was complete. No iterative correction loops.

### Files outside src/ — zero

`wat/Record.wat` untouched. `holon-rs` untouched. STOP-2 and STOP-5 not triggered.

---

## Working tree on return

```
 M src/check.rs
 M src/closure_extract.rs
 M src/edn_shim.rs
 M src/runtime.rs
 M src/stdlib.rs
 M src/types.rs
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-C1.md
```

holon-rs untouched. DO NOT commit (orchestrator commits).
