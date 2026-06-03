# SCORE — Arc 247 — Clojure-honest seq-HOF order (fn-first)

**Verdict: REMARKABLE — CLOSED 2026-06-03.** The dialect lie is annihilated: the 5 seq-HOFs are fn-first, matching Clojure. No R2 (mechanical cascade, no novel infra).

## Gates (orchestrator's independent re-run)

| Gate | Result |
|---|---|
| `cargo test --release --test probe_arc247_hof_fn_first` | **5 passed / 0 failed / 0 ignored** |
| `cargo test --release --lib -p wat` | **895 passed / 0 failed / 1 ignored** (unchanged) |
| `cargo build --release --tests --workspace` | clean |

## Structural verification (disk, not self-report)

- **The flip is real:** `eval_vec_map` reads `f = args[0]`, `xs = require_vec(args[1])` — fn-first, collection last. All 5 (`map`/`filter`/`foldl`/`foldr`/`sort-by`) impls + their `check.rs` TypeSchemes reordered.
- **Cascade complete:** the only `(:wat::core::<hof> [` remnant is the probe's *intentional* `mint_map_coll_first_is_gone` error-assertion (line 98). Every real call site flipped. **~65 sites** across `wat/` (18), `wat-tests/` (22), `tests/` (18), runtime inline tests (7).
- **HARD CUT confirmed:** `(map [1 2 3] inc)` is now a check error (`TypeMismatch`: expected `fn T→U` at arg0, got `Vector`). Coll-first is gone, not aliased.
- **git-state:** uncommitted, no sonnet commit, no strays.

## Resolves arc 109 § N.1

This arc is the one arc-109 INVENTORY § N.1 asked for — *"`:wat::core::map` arg order is backwards"* (banked, surfaced at the arc-232.0 probe ~6 weeks ago). N.1's target `(map f xs)` + its "sweep the family" (map/filter/reduce/for-each) are exactly this arc's deliverable. **§ N.1 marked RESOLVED.**

## Sibling surfaced — thread-last `->>`

The flip makes `(map f (filter pred xs))` natural Clojure threading, but only `->` (thread-first) exists, and it isn't ergonomic for fn-first HOFs. **Arc 248 (`->>` thread-last macro) is the banked sibling** — anticipated by the DESIGN, confirmed by the strike. Not built here (scope-honest). A small, well-scoped follow-on.

## Pre-existing (not this arc)

`probe_arc216_stone5b_hashset_native_storage::probe_8_atom_round_trip` fails identically before and after (`contains?` TypeMismatch) — the pre-existing `1 ignored` HashSet debt, unrelated to 247.

## Lineage

Surfaced from arc 237's wind-down (the generative-macro layer wanted a Clojure-honest `map`), but stands on its own as **dialect honesty** — the substrate claims clojure-on-rust; its seq-HOFs now tell the truth. The deepest convergence of the day: a 6-week-old banked lie (N.1), rediscovered by the relentless equality→clause→macro→map descent, annihilated.
