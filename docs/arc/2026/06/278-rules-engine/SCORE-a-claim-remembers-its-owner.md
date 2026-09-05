# SCORE — a claim remembers its owner

**NOT STRUCK. STOP-3.** Executor: grok, 2026-09-05. Tree safe, uncommitted.
`wat-scripts/fanout/circuit.wat` only (`41/27`). Code left on disk as evidence; not reverted.

```
Summary [ 361.370s] 5214 tests run: 5214 passed (3 slow), 19 skipped
```

## ★ THE TINY ROW DID WHAT THE DESIGN CLAIMED

Drop-after `n=50 m=2 j=2`, completing runs:

```
total=100; distinct=100; dup=0; seen-firsts=100     ×3
```

Before: `total ∈ {89,90,90,91,89}`. The stranded First-claimers were taking `DupSelf`, and emitting on it closed the hole **at this size, under drop-after**. Row 2 held: no double-emission on those runs. The inferred path is now observed — for the drop cell.

Mechanism probe, unchanged file: `discriminates=yes`.

## ⛔ STOP-3 — rate-0 `dup` is no longer 0

Same emit rule, rate 0, 8000 messages, five runs, every one:

| run | total | distinct | dup | seen-dups |
|---|---|---|---|---|
| 1 | 8001 | 8000 | **1** | 4 |
| 2 | 8001 | 8000 | **1** | 2 |
| 3 | 8001 | 8000 | **1** | 3 |
| 4 | 8002 | 8000 | **2** | 2 |
| 5 | 8002 | 8000 | **2** | 4 |

`distinct` stays 8000 — the extra outcomes are the same seq twice. BRIEF STOP-3: *if making `DupSelf` emit pushes `dup` above 0 in any run, STOP. Double-emission is a worse defect than the stranding and must not be traded for it.*

Did not patch. Did not emit on `DupOther`. Did not add a second ledger.

## WHAT THE THREE ARMS CANNOT SAY

`DupSelf` means "this owner already holds the seq." That is one fact covering two acts:

1. **Lost First reply** (drop-after, T1 timeout) — nobody has emitted. Must emit. Tiny row 1.
2. **Redelivery after a successful emit** (ack lost, or vis expired while the owner still holds it) — already emitted. Must not emit. Rate-0 STOP-3.

The server returns the answer to (who owns this?), not (have they reported it?). The DESIGN's contract decision — *return the answer, not the data* — is still right for ownership. It is not enough for emit. Emit needs a second bit the ledger does not have.

Consequence 2 predicted rate-0 `seen-dups` → 0, "those dups are T1 retries, `DupSelf` by construction." They fell (7–10 → 2–4) and did **not** vanish. Residual `seen-dups` is `DupOther`: a *different* worker claiming a redelivered seq. Same class as the extra `dup` — redelivery, not self-retry. Consequence 2's reasoning is incomplete, as the DESIGN said it would be if the number did not go to 0.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ stranding closes | ✅ completing runs `total=100`. ⚠ 3/6 did not complete — see row 8 |
| 2 | ⛔ no double-emission | ✅ on completing tinies: `dup=0` and `distinct=total`. ❌ rate-0 (STOP-3) |
| 3 | ★★ rate-0 `seen-dups=0` | ❌ `2 4 3 2 4` |
| 4 | rate-0 invariant | ❌ `dup=1..2`; `total=8001..8002` |
| 5 | `seen-firsts=100` | ✅ on completing tinies |
| 6 | the floor | ✅ **5214/5214, 19 skipped** |
| 7 | mechanism probe | ✅ `discriminates=yes` |
| 8 | `claim deadline exhausted` | **3/6** (before: 1/6). Reported, not repaired |

## ROW 8, as asked

Tiny ×6: runs 1, 4, 6 died at

```
fanout worker: claim deadline exhausted;depth=3;attempts=3;elapsed=601
```

at `circuit.wat:1036` (`:fanout::worker/stop` ← `collect-stop`). Completing runs 2, 3, 5. Out of scope.

## WHAT DID NOT MOVE

- `wat/`, `sqs.wat`, `src/`, nextest config, send-path scans.
- `held-worker` still `:peers [:queue::Queue]`, does not claim.
- Ledger written only on `First`; `DupSelf` is idempotent (two retries stay `DupSelf`).
- `wid` was in the same `fn` as the send. No second file.

## WHAT THE NEXT STONE NEEDS

Ownership is expressible and the drop-after stranding is the `DupSelf` path. Emit cannot be `First | DupSelf` until the ledger can tell "you have not reported" from "you have." That is a new fact, not a new comparison of the same two strings. Do not paper over STOP-3 by counting at the worker — the DESIGN already refused a caller that can forget to compare.
