# EXPECTATIONS — Stone S-C.1 (rename `wat__Record` → `wat__holon__Record`)

Mode A: build clean + baseline 827/0 held + `src/` only. Pure rename, zero behaviour change.

## Scorecard

| # | Row | Verification | Expected |
|---|-----|--------------|----------|
| 1 | Build clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | Lib baseline held | `cargo test --release --lib -p wat 2>&1 \| tail -3` | `827 passed; 0 failed` (1 ignored) |
| 3 | S-A hierarchy | `cargo test --release --test probe_arc237_sA_hierarchy 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 4 | S-B.1 recordtype | `cargo test --release --test probe_arc237_sB1_recordtype 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 5 | S-B.2 defrecord | `cargo test --release --test probe_arc237_sB2_defrecord_recordtype 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 6 | S-A1 assignable | `cargo test --release --test probe_arc237_sA1_assignable 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | records dual-form | `cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 \| tail -3` | pass |
| 8 | defrecord surface | `cargo test --release --test probe_arc227_stone2_defrecord 2>&1 \| tail -3` | pass |
| 9 | files in scope | `git status --short` | `src/*.rs` (runtime/check/stdlib) + SCORE doc ONLY; NO wat/, NO holon-rs, NO test files (unless a test embeds the variant in Rust) |

**Clippy NOT a ceiling concern** per standing direction.

## Independent prediction

**Target band: 15–30 min Mode A. STOP-3: 40 min. STOP-4: 60 min.** Pure rename; the
compiler names every variant site; the only judgment is the 3 rules (variant→rename,
discriminant-tag→rename, surface-strings→keep). Baseline-preserving by construction — if
827 moves, a surface string was wrongly touched or a behaviour changed → STOP.

## Risks / trap-doors

1. **Over-rename the surface type.** The biggest risk: renaming a colon `wat::Record` /
   `:wat::Record` string (the wat-SURFACE base type) would corrupt the type system. KEEP
   all colon-strings; rename only the double-underscore variant identifier (+ the
   discriminant tag at runtime.rs:1116).
2. **Forget the discriminant tag.** runtime.rs:1116 `"wat__Record".hash(...)` MUST become
   `"wat__holon__Record"` — it's the variant's Hash discriminant; leaving it stale would
   collide with the future base variant's tag (S-C.2). (Doesn't break baseline now, but
   it's part of THIS rename — the variant's discriminant follows the variant's name.)
3. **Scope creep into S-C.2/3.** Do NOT mint the base variant or touch the macros. Rename
   only.

## SCORE

`SCORE-STONE-S-C1.md` (NEW). 9-row scorecard + the rename (variant identifier + the
discriminant-tag string; explicit note that surface-type strings were left untouched) +
honest deltas + working tree. Mirror SCORE-STONE-S-A1.
