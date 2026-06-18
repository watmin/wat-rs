# EXPECTATIONS — Stone 0a: `:wat::core::PersistentMap`

Written BEFORE the strike, so the result cannot move the goalposts. The orchestrator weighs the kill
against its OWN re-run of these.

| # | what | command | expected |
|---|---|---|---|
| 1 | the probe goes GREEN | `cargo test --release -p wat --test probe_arc278_0a_persistent_map -- --include-ignored` | 1 passed / 0 failed (was RED: `UnknownFunction(":wat::core::PersistentMap")`) |
| 2 | lib floor unchanged | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 929 passed / 36 failed (zero new) |
| 3 | deftest floor unchanged | `cargo test --release --test test 2>&1 \| grep "test result"` | 264 / 1 |
| 4 | nursery floor unchanged | `cargo test --release -p wat --test nursery -- --test-threads=1 \| grep "test result"` | ~893 / 4 (±3 fork flake) |
| 5 | deporder gate green | `cargo test --release --test test_stdlib_load_order \| grep result` | 1 / 0 |
| 6 | workspace compiles | `cargo build --release` | clean (warnings ok) |

Runtime prediction: 15–30 min (mechanical mirror; the compiler-driven cascade across ~7 files is the bulk;
the EDN tagged round-trip is the one genuinely new bit).

Trap-doors named:
- **rpds MSRV** (STOP-1): rpds 1.x uses a recent edition; if it won't build, a pinned older rpds is the
  fallback — a version delta, not a design change.
- **EDN tagged round-trip** (STOP-2): the one non-mirror piece. If the tag read path isn't a clean mirror of
  an existing tagged type, it surfaces here.
- **`Value: Hash` for the new variant**: must be order-independent (maps have no order); copying the
  std-HashMap Hash strategy avoids a subtle "equal maps hash differently" bug.
- **`Send + Sync`**: the `Sync` rpds variant (`HashTrieMapSync`) is required — `Value` crosses threads. A
  plain `HashTrieMap` (Rc-backed) would fail the `Send`/`Sync` bounds on `Value` (compile error, caught fast).

Score (filled after the orchestrator's own re-run): _pending strike_.
