# EXPECTATIONS — STONE C

| # | what | expected |
|---|---|---|
| 1 | `put` on an opaque key, direct and generic | a wat `TypeMismatch` naming `:wat::cache::Lru/put`; no `panicked at`, no `RUST_BACKTRACE` |
| 2 | `get` on an opaque key, direct and generic | a miss (`None`); exit 0 |
| 3 | the raise's `:location` | the user's `.wat` (STOP-2 if a `.rs`) |
| 4 | no call site moved | `git diff` touches no `.wat` outside the new probe fixtures |
| 5 | mutation per verb | restoring each `panic!` reds exactly its cases |
| 6 | F-083 still pinned | the board's F-083 row green, unchanged |
| 7 | `panic!` count in `cache.rs` | **0** reachable from wat (`NonZeroUsize` `.expect` behind the `new` guard is fine) |
| 8 | the floor | `Summary`: 0 failed |
| 9 | clippy | clean |

Runtime: edit 30-60 min; STOP-1 is the likeliest blocker. Floor ~17 min.
