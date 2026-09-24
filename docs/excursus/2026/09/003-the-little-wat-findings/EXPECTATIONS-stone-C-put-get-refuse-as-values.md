# EXPECTATIONS — STONE C (redrawn)

| # | what | expected |
|---|---|---|
| 1 | `put` on an opaque key, direct and generic | an `Err` value naming `:wat::cache::Lru/put`; no `panicked at`, no `RUST_BACKTRACE` |
| 2 | `get` on an opaque key, direct and generic | a miss; exit 0 |
| 3 | `panic!` in `cache.rs` reachable from wat | **0** |
| 4 | the 31 callers | moved by a recorded, replay-fixtured codemod; re-run = 0 changes |
| 5 | mutation per verb | restoring each `panic!` reds exactly its cases |
| 6 | F-083 still pinned | board row green, `(0, 0, "0")` |
| 7 | `HolographicLru` suite | green |
| 8 | the floor | `Summary`: 0 failed |
| 9 | clippy | clean |

Runtime: 60-90 min edit + codemod; floor ~17 min.
