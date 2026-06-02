# Arc 237 — PAUSE-CONTEXT (2026-05-27 night)

**Status:** PAUSED at 237.8b — **RE-PARKED 2026-06-02.** Arc 241 (function-signature parser unification) **closed** (`5d2e3db1`), so the `&` rest-binder is ready (Gate 1 green since 241.5 `639b4862`). But the first attempt to resume 237.8b surfaced a NEW failure domain — the substrate **synthesizes a nil VALUE as the `:wat::core::nil` TYPE keyword**, which arc 242's doctrine correctly rejects — so the probe fails on the nil heresy, NOT on `&`. Spawned **arc 244 (nil-literal-canonicalization)** to annihilate it. 237 now awaits arc 244's closure.

Chain: **237 ⇠ 241 (closed) ⇠ 244 (open).** See `docs/arc/2026/06/244-nil-literal-canonicalization/DESIGN.md` + the locked repro `tests/probe_nil_return_value_position_bug.rs`. The "How to resume" steps below hold once 244 closes (both `&` and the nil-value form will then be sound).

**Original pause note (arc 241 gate) preserved below as historical record.**

## Why paused

237.8b's FM-2-bis probe (`tests/probe_arc237_8b_defclause_arithmetic.rs` Gate 1) surfaced a substrate gap: `defclause`'s argspec parser doesn't support `&` rest-binders (the 3+-ary fold clauses in the recipe need them). The user's response surfaced the deeper finding: **the substrate has FOUR copies of the argspec-parser logic** (fn × 3 variants + defclause × 1) — adding `&` to defclause's parser alone would entrench the duplication. The right move per wat philosophy (one canonical path; remove options) is to consolidate FIRST + extend ONCE.

Spawned **arc 241 — function-signature parser unification** to handle the consolidation. Per spawn-block winding (`feedback_spawn_block_winding`): arc 237 cannot close until arc 241 closes; 237.8b unblocks when 241.5 extends the canonical parser with `&` support; chain resumes from there.

## What's shipped in arc 237

| stone | scope | commit | runtime |
|---|---|---|---|
| 237.1 | `:wat::core::typeunion` substrate primitive | `d40eb4a3` | ~11 min |
| 237.2 | `:wat::core::defclause` foundation | `bdd9eb6c` | — |
| 237.3 | `:guard` + `:ensure` clause-keywords | `ee5e892c` | — |
| 237.4 | rich `:NoMatchingClause` + `:PostconditionFailed` | `5f7bb6e5` | — |
| 237.5 | `:wat::core::conforms?` | `5d667123` | — |
| 237.5.fix | one wildcard-free `Value::declared_type_name` authority | `990542a9` | — |
| 237.6 | auto-mint `is-<Name>?` as named convenience | `3ae844cb` | — |
| (S-A/A1/B/C) | records-doctrine stones (subtype hierarchy, recordtype, defrecord, base/holonic split) | various | — |
| 237.7a | `length` ∀T intrinsic (Tier A recipe-prover) | `8100d9d2` | — |
| 237.7b-i | `empty?` Tier A | `e401c183` | — |
| 237.7b-ii | `contains?` Tier B (HashMap K-not-V trap) | `fef2c8d9` | — |
| 237.7b-iii | `conj` Tier B type-preserving | `2d3259ae` | — |
| 237.7b-iv | `get` Tier B Option-wrapped (INDEX trap) | `fad1c1c6` | 9.5 min |
| 237.7c | `assoc` Tier B umbrella-Path (records-doctrine slice) | `a9961421` | 14.85 min |
| 237.8a | arithmetic + comparison HARD CUT under THE DECISION (widest-contagion deleted) | `154ca713` | ~25 min |

## What's PAUSED in 237 (awaits arc 241)

| stone | scope | gate to resume |
|---|---|---|
| **237.8b** | recipe-lock + numeric grid (arithmetic + ordering for i64/f64 via wat-defclause) — DESIGN + probe COMMITTED `49e2e13b` | 241.5 lands; Gate 1 of probe flips green |
| 237.8c | full primitive equality grid + composite recursive equality (bundled per Q2-C) | after 237.8b |
| 237.8d | DispatchRegistry HARD CUT (mechanical; 0-tenant after 8a + 8b) | after 237.8c |
| 237.9 | INSCRIPTION + memory mint (`feedback_per_type_binary_primitives` doctrine) | closure |

## Key state to preserve across the pause

1. **All decisions locked through 8b's last design pass** (per dialogue 2026-05-27 night):
   - Per-Type primitives ALWAYS 2-ary; drop `'2` suffix on rename
   - `::` separator stays (`/` is reserved for instance-methods; `//` collision on division seals it)
   - Lisp arity rules per clause-set (`+`/`*` 0-ary identity; `-`/`/` 0-ary error via `:NoMatchingClause`; 1-ary identity-on-left; 2-ary direct; 3+-ary fold)
   - `=`/`not=` get the defclause treatment too (per dialogue Q-equality four-questions; user pushed me off "stay polymorphic-structural"; per-Type primitives mint; arc 238 impl refactors INTO them)
   - HARD CUT discipline throughout — no shims, no aliases
   - `!=` → `not=` reconcile (rename `:i64::!=` to `:i64::not=`)

2. **Probe state intact**: `tests/probe_arc237_8b_defclause_arithmetic.rs` is committed at `49e2e13b`. Gates 2/3/4a GREEN (defclause CAN dispatch by arg-Type; 0-ary literal works; i64 ordering aliases correct). Gate 1 `#[ignore]`'d with annotation pointing to 241. Mint-confirmers `#[ignore]`'d until 8b's substrate work lands.

3. **Surface intel from the dig** (preserved in DESIGN-STONE-237.8b.md STATUS section):
   - Per-Type i64 ordering aliases EXIST (237.3): `:i64::=`, `:i64::>`, `:i64::<`, `:i64::>=`, `:i64::!=` — `:i64::<=` MISSING from the set
   - Entire f64 ordering family does NOT exist — 6 mints needed in 8b
   - Per-Type variadic wat fns at `wat/core.wat:104-132` (8 fns) absorb into defclauses; consumer-sweep ~10-16 sites, most look binary

4. **Pre-spawn cadence on 237.8b is fully wound** through DESIGN + probe. The BRIEF + EXPECTATIONS are drafted only mentally — when arc 241 closes, the next move is: re-run the probe (Gate 1 should flip green automatically); update the DESIGN STATUS section to reflect the unblock; draft BRIEF + EXPECTATIONS; baseline re-run; spawn Shadowdancer.

## How to resume

1. Read this PAUSE-CONTEXT
2. Read `docs/arc/2026/05/241-function-signature-unification/INSCRIPTION.md` (arc 241 closure)
3. Re-run `cargo test --release --test probe_arc237_8b_defclause_arithmetic` — Gate 1 should now PASS (no longer `#[ignore]`'d, or simply un-ignored)
4. Update `DESIGN-STONE-237.8b.md` STATUS section: Gate 1 now GREEN; strategy NO LONGER blocked
5. Draft `BRIEF-STONE-237.8b.md` + `EXPECTATIONS-STONE-237.8b.md` per the recipe (locked decisions above)
6. Baseline re-run (lib 834+/0; test-build 0 errors)
7. Spawn Shadowdancer on 237.8b — the recipe-lock + numeric grid
8. Continue per the established cadence: 237.8c (equality bundle) → 237.8d (registry HARD CUT) → 237.9 (INSCRIPTION)

## Cross-references

- `docs/arc/2026/05/241-function-signature-unification/DESIGN.md` — the spawned arc that unblocks 8b
- `tests/probe_arc237_8b_defclause_arithmetic.rs` — Gate 1 surfaces the gap; un-ignore is the post-241.5 contract
- `DESIGN-STONE-237.8b.md` — STATUS section documents the discovery + 241 strategy
- `feedback_spawn_block_winding` — the discipline keeping this pause honest
- `/home/watmin/work/holon/scratch/FAILURE-ENGINEERING.md` — drives 241's class-elimination shape
