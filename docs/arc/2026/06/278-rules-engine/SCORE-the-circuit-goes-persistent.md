# SCORE — the circuit goes persistent

**STRUCK.** Executor: grok, 2026-09-02. Container swap only; cursor not taken.

```
Summary [ 355.214s] 5183 tests run: 5183 passed (4 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-02T08-43-17Z/`

## What landed

`wat-scripts/topic/sns-fanout.wat` outbox and `wat-scripts/fanout/circuit.wat`
`:fanout::worker` outcomes are `:wat::core::PersistentVector`. `-deliver`'s
rebuild uses `:wat::vector::conj` / `:wat::vector::get`; both `get`s are
`Option/expect`ed with located messages. Held-worker untouched. `sqs.wat`,
`wat/`, `src/` empty.

Strategy B from `probe-outbox-strategies.wat`. Not C.

## Per-delivery (row 8) — the cubic term is gone

`per-delivery = drain_ms / (N × M)`, same basis as 4.9 → 7.5 → 9.2 ms before.

| N×M×J | drain ms | N×M | µs/delivery | was |
|---|---|---|---|---|
| 500×4×3 | 9611 | 2000 | **4.81 ms** | 4.9 |
| 1000×4×3 | 19622 | 4000 | **4.91 ms** | 7.5 |
| 2000×4×3 | 41429 | 8000 | **5.18 ms** | 9.2 |

Flat across a 4× range of N. The slope is gone.

## Drain (row 2) — the isolated 27 s transferred

| N | drain before | drain after | Δ |
|---|---|---|---|
| 500 | 9793 | 9611 | −0.2 s (noise at this size) |
| 1000 | 29994 | 19622 | **−10.4 s** |
| 2000 | 73461 | 41429 | **−32.0 s** |

Isolated A→B at n=2000 was 30.7 s → 3.7 s (−27 s of a 73.4 s drain, expecting
~46 s). Circuit drain landed at **41.4 s**. The isolated gain transferred; it
was not overlapping I/O. STOP-4 does not fire.

## The circuit at weight

```
queue-receive-calls=8044
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1
setup=8401;publish=805;drain=41429;stop=4496;ticks=827
WALL_SEC=55.743
```

Receive calls 8044 against 8052 — the park and wakeup are untouched.

## EXPECTATIONS

| # | what | this strike |
|---|---|---|
| 1 | ★ `total=8000; distinct=8000; dup=0` | ✅ |
| 2 | ★ drain reported | ✅ **41.4 s** against 73.5 s |
| 3 | ★ both accumulators moved | ✅ topic `outbox` and worker `outcomes` are PersistentVector; held-worker still Vector |
| 4 | `Option` faced | ✅ both `vector::get` sites `Option/expect` with located messages |
| 5 | queue untouched | ✅ `git diff sqs.wat` empty |
| 6 | no substrate | ✅ `git diff wat/ src/` empty |
| 7 | held-worker untouched | ✅ 0 held-worker lines in the circuit diff |
| 8 | scale matrix, per-delivery flat | ✅ 4.81 / 4.91 / 5.18 ms against 4.9 / 7.5 / 9.2 |
| 9 | receive calls | ✅ 8044 against 8052 |
| 10 | wall time | **55.7 s** against 87.3 s — reported |
| 11 | floor | ✅ 5183/5183, `FLOOR=0`, `.floor/2026-09-02T08-43-17Z/` |

## Cursor still cut

C remains 1000× better than B in isolation and still buys only the leftover
rebuild (~3.7 s isolated) against a drain that is now 41 s. Cap-as-count-minus-head
and a compaction point are still not this stone.

## Next

The corpus migration in `main` is what stops this class coming back. This
fixture measured what that migration is worth: **32 s of a 73 s drain**, and
a per-delivery cost that no longer grows with N.
