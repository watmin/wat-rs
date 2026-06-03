# SCORE — Stone 237.8c — Equality grid (Shape B)

**Verdict: REMARKABLE** — clean, no R2. The equality grid completes via Shape B (per-Type leaves + structural engine); equality is preserved as the *justified* asymmetry.

## Gates (orchestrator's independent re-run)

| Gate | Result |
|---|---|
| `cargo test --release --test probe_arc237_8c_equality_grid` | **8 passed / 0 failed / 0 ignored** |
| `cargo test --release --lib -p wat` | **895 passed / 0 failed / 1 ignored** (unchanged) |
| `cargo build --release --tests --workspace` | clean |

## Structural verification (disk, not self-report)

- **`:wat::core::f64::=` / `:f64::not=` minted** — check.rs:13791-13792 (type-locked to f64, return bool, mirroring the i64 pair), runtime.rs:5679-5680 (→ `eval_eq`/`eval_not_eq`). The f64 equality pair now sits beside the i64 pair; the per-Type equality-alias family is complete.
- **`infer_comparison` → `infer_equality`** — `grep "fn infer_comparison" src/` returns 0; `infer_equality` at check.rs:11111. Body preserved verbatim (arity-2, cross-numeric rejection, same-type-or-subtype-compat, bool return). Renamed to its true remaining role, **not** deleted — equality stays Rust-checked-structural (Shape B).
- **Dead cross-numeric arms deleted** — `values_equal`'s numeric arms are now only `(i64,i64)`/`(u8,u8)`/`(f64,f64)`; the arc-050 `(i64,f64)`/`(f64,i64)` promotion arms are gone (HARD CUT, tombstone comment, no comment-around). The checker rejects mixed-numeric `=` before eval, so they were unreachable.
- **Composite recursion preserved** — `values_equal`'s Vec/List/Tuple/Map/Record arms untouched; the structural engine is intact.
- **git-state** — uncommitted, HEAD still STRIKE-READY (`a52b0d1d`), no sonnet commit, no strays.

## Why no R2 (vs 8b)

8b introduced a novel stdlib-defclause pipeline (where the privileged-parse hack hid). 8c is mechanical: a rename + two match arms added + two deleted + two registrations, all mirroring existing patterns (`:i64::=`). Low risk surface, and the disk read confirms each change is a faithful mirror. The smaller, truer stone Shape B promised.

## The doctrine this stone adds

`feedback_per_type_binary_primitives` (minted at 237.9) now carries **two** faces: the per-Type-defclause recipe (8a/8b, for arithmetic/ordering — operations closed over one Type) AND the structural exception (8c, for equality — universal/recursive/subtype-compatible). **Equality is the justified asymmetry**: the bar for accepting an odd-one-out is *structural necessity*, and equality clears it (it is genuinely a different kind of operation). `feedback_asymmetries_meet_high_bar` gains its second face here — not all asymmetries are defects.

## What remains in arc 237

- **237.8d** — `DispatchRegistry` HARD CUT (0-tenant after 8a/8b/8c).
- **237.9** — INSCRIPTION + the doctrine mint. (And the breadcrumb forward-instruction: 237.9 unblocks the stubbed arc 245.)
