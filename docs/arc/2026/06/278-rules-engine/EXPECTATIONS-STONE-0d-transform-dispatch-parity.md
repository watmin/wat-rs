# EXPECTATIONS — Stone 0d: transform-op check-side parity

Independent scorecard, fixed BEFORE the strike. The orchestrator re-runs each row itself and reads the diff;
the worker's report is a hypothesis until the disk confirms it.

| # | what | command | expected |
|---|---|---|---|
| 1 | the 8 ops type-check on PersistentVector | `cargo test --release -p wat --test probe_arc278_0d_transform_dispatch_parity -- --include-ignored` | **2/2 GREEN** (parity + guard) |
| 2 | guard still rejects wrong element | (same run, `wrong_element_still_rejected`) | GREEN — parity ≠ permissiveness |
| 3 | 0a/0b/0c probes unchanged | the three `-- --include-ignored` runs | 1/1 each |
| 4 | lib floor unmoved | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 931 passed / 36 failed (the 36 pre-existing) |
| 5 | deftest floor unmoved | `cargo test --release --test test 2>&1 \| grep "test result"` | 264 / 1 (the 1 pre-existing) |
| 6 | load order green | `cargo test --release --test test_stdlib_load_order \| grep result` | 1 / 0 |
| 7 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished, no errors |

## Runtime prediction
~15-30 min. Eight mechanical mirrors of an existing 4-arm precedent (`infer_conj`/`infer_get`/`infer_assoc`),
plus scheme retirement. The concat alias path is the one genuine unknown.

## Trap-doors named
- **concat alias path.** concat checks via `Vector/concat`, not a surface arm — may need a per-Type
  `PersistentVector/concat` scheme instead of a surface infer arm. STOP-1 covers it. Weigh the worker's
  concat solution especially hard: confirm `(concat pv pv)` checks AND `(concat vec pv)` is rejected
  (same-kind-only), matching the runtime.
- **Scheme retirement totality.** Retiring a static scheme means the new arm must reproduce the FULL Vector
  behavior (element-typing, return type). A subtle regression here would drop a lib test → row 4 moves. If row
  4 ≠ 931/36, a Vector path regressed — reject.
- **Over-permissiveness.** Parity must not become "accept anything." Row 2 (guard) is the canary; also spot a
  `(map string-fn pv-of-i64)` rejection in the weigh.
- **Fresh-var noise.** The RED probe showed `Vector<?32>` etc. for reverse/take/drop (unconstrained element) —
  the arm must still unify the element through the op (reverse of `PV<i64>` → `PV<i64>`, not `PV<?>`).

## Weigh (orchestrator, against own re-run + the diff)
1. Re-run rows 1-7 myself; rows 4/5/6 must be EXACTLY the baseline.
2. Read the `collection/infer.rs` diff: 8 arms, each accepting both container heads, type-preserving, total
   over Vector (no behavior lost), teaching TypeMismatch for other shapes.
3. Read the `check.rs` diff: 8 `return`-early arms; the 8 schemes actually removed (not just shadowed).
4. Verify concat: same-kind-only, mixed rejected.
5. Dogfood clippy on the touched homes if warranted; commit SCOPED on green; push.
