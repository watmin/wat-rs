# SCORE — partial only when it must

**SCORED.** Executor: grok, 2026-09-06. Tree dirty, uncommitted.
Prefix only when `n0 > cap`. Below cap, all-or-nothing.
`queue-receive-calls` fell. `publish` followed.

```
Summary [ 406.505s] 5220 tests run: 5220 passed (6 slow), 22 skipped
```

`.floor/2026-09-06T11-48-54Z/`

## THE RULE

```
room  = max(cap - depth, 0)
take  = n0 > cap  →  room          ;; livelock case: prefix
        n0 <= cap →  n0 if n0 <= room else 0
```

`Accepted [count]` unchanged. `:Full` still gone. `Accepted 0` is still the 429.

## THE PROBE — the case that changed, on purpose

`probe-the-server-manages-its-own-capacity.wat`:

```
fill=6;depth6=6;send8=0;depth-after-8=6;drain5-send8=8;depth-after=9;above-cap=accepted=10;n=10;stored=z0,z1,z2,z3,z4,z5,z6,z7,z8,z9;nsubs4-room6=Accepted(1);nsubs7-pub10=Accepted(9)
```

- cap 10, depth 6, send 8 (`8 <= cap`) → **`Accepted 0`**, depth unchanged. Was `Accepted 4`.
- drain 5 (depth 1, room 9), send 8 → **`Accepted 8`**, depth 9. Was `Accepted 5`.
- cap 10, send 15 (`15 > cap`) → `Accepted 10`, stored `z0..z9` in order.
- `nsubs 7` publish 10 (70 > 64) → `Accepted 9`, alive. The cliff stays gone.

## CIRCUIT ×5

`ps` before: grok 8.0 %, claude 4.5 %, else < 1 %.

Every run: `total=8000;distinct=8000;dup=0`. `publish-calls=200`. `seen-skipped=0`.

```
queue-receive-calls  5274  5287  5353  5288  5319     median  5288
publish             22584 22752 22697 22652 22517     median 22652
full-retries         3778  3781  3784  3801  3764     median  3781
dup                     0     0     0     0     0     median     0
setup               10252 10199 10253 10260 10177     median 10252
stop                 6001  5793  6267  5417  6535     median  6001
```

outbox, 8000 items every run:

```
50-250     7619  7599  7559  7589  7609     median  7599
250-1000    370   390   430   410   380     median   390
max-ms      285   289   294   303   299     median   294
```

| | this stone | prior (fan-once) | pre-regression |
|---|---|---|---|
| receive-calls | **5288** | 11859 | 5396 |
| publish | **22652** | 70244 | 22583 |
| dup | **0** | 2759 | 0 |
| full-retries | **3781** | 7900 | 3770 |
| outbox items | **8000** | 10759 | 8000 |

`receive-calls` fell past the pre-regression 5396. `publish` followed, to within noise of 22583. The operation-count model held. STOP-3 did not fire.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ at or below cap is all-or-nothing | ✅ send 8 at depth 6 → `Accepted 0`, depth 6 |
| 2 | ★★ lands whole once there is room | ✅ drain 5, send 8 → `Accepted 8`, depth 9 |
| 3 | ★★ above cap still takes a prefix | ✅ `nsubs 7` publish 10 → `Accepted 9`, no assertion |
| 4 | ⛔ prefix is still a prefix | ✅ send 15 into cap 10 → `z0..z9` in order |
| 5 | ⛔ no response leaks internals | ✅ `:Full` gone; admission arm is `Accepted [count]` |
| 6 | ⛔ delivery exact | ✅ ×5 `distinct=8000;dup=0` |
| 7 | ⛔ floor | ✅ `5220 passed (6 slow), 22 skipped` |
| 8 | ⛔ blast | ✅ `sqs.wat`, the probe, this SCORE. No `sns-fanout.wat`, no `circuit.wat`, no `wat/`, no `src/` |
| 9 | ⛔ no `:cap` changed | ✅ 10× 64, 2× 2, 2× 1, 1× 32, 7× 1024 |

## REPORTS

| ▪ | now (median) | prior → pre-regression |
|---|---|---|
| a | receive-calls **5288** | 11859 → 5396 |
| b | dup **0** | 2759 → 0 |
| c | publish **22652** | 70244 → 22583 |
| d | full-retries **3781** | 7900 → 3770 |
| e | outbox 8000 items, 50-250=7599 250-1000=390 max 294 | 10759 / 8000 items, max 662 / 209 |
| f | setup 10252 / stop 6001 | 10232 / 9742 |

## STOP TRIGGERS

- **STOP-1** did not fire. `n0 > cap` (can it ever fit) is distinct from `n0 > room` (does it fit now).
- **STOP-2** did not fire. nsubs-7 still returns `Accepted 9`.
- **STOP-3** did not fire. receive-calls fell and publish followed.
- **STOP-4** did not fire. No `:cap` change. No `sns-fanout.wat` / `circuit.wat`.

The topic top-up (`daf4a2f39`) stayed. With whole-batch admission at nsubs 4 it does not fire on the circuit (40 <= 64). `dup=0` without removing it.

## NOT TOUCHED

`sns-fanout.wat`. `circuit.wat`. `wat/`. `src/`. Inbox `:cap`. Depth caching.

Tree uncommitted. Do not commit unless asked.
