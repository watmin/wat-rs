# SCORE — clamp the request, not the handler

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
The `if` binds the request. The handler is spliced once.

```
Summary [ 478.720s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-07T03-31-51Z/`

## THE RESTRUCTURE

`wat/service.wat` `page-clamped`, was:

```
(if (> limit cap)
  (let [req reconstructed] outcome-match)   ;; copy 1
  outcome-match)                            ;; copy 2
```

is:

```
(let [req (if (> limit cap) reconstructed req)]
  outcome-match)                            ;; one copy
```

`~outcome-match` occurs **once** in the emitted quasiquote (`service.wat:2214`).
The expand-time `(= page-field "")` still chooses "no wrap" vs "this let"; those
two branches are not both in the serve arm.

`req-binder` is the match-pattern symbol. The `let` RHS reads the outer
binding; the handler sees the inner one. STOP-1 did not fire.

## WRITE-SIDE — read, not fixed

`:max-entries` on the serve arm wraps `~guarded-arm` **once** (RTE send vs
handler). Client `byte-body` splices `~send-recv-form` twice (peer-wire?
then/else) — that is the client send/recv, not a serve handler. STOP-3
did not fire.

## THE PROBE ×2

Identical both runs:

```
page-n=64;some-cur=yes;all-n=250;ordered=yes;first-n=1;pages=4;send=Accepted(70);recv-n=64;recv-max=64;field=envelopes
```

`receive :limit 1000` is still 64. The let rebinds the request, not a no-op.

## CIRCUIT ×3 at 25 ms (both sites pinned, then restored)

`ps` before: grok 12.9 %, claude 4.4 %. `circuit.wat` restored.

Every run: `total=8000;distinct=8000;dup=0`.

```
                 r1      r2      r3    med    with-dup   before-page
publish       18575   18560   18582  18575       19912        18466
retries         531     529     529    529
receives       4747    4704    4692   4704        ~4700        ~4780
setup         12348   12229   12223  12229
stop           7323    6268    6513   6513
```

Publish **−1337 ms** of 19912, **+109 ms** of 18466. Recovered. STOP-2
did not fire. The unused-arm cost was the +7.8 %.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ handler spliced once | ✅ one `~outcome-match` in the emitted let |
| 2 | ★★ behaviour identical | ✅ probe ×2 byte-identical to today |
| 3 | ⛔ clamp still clamps | ✅ recv-n=64 |
| 4 | ⛔ delivery exact | ✅ ×3 `distinct=8000` |
| 5 | ⛔ the floor | ✅ `5221 passed (7 slow), 22 skipped` |
| 6 | ⛔ blast | ✅ `service.wat` + SCORE. No `src/`, no `query.wat`, no `sqs.wat`, no `circuit.wat` |

## REPORTS

| ▪ | what |
|---|---|
| a | ★★★ publish median **18575** (target 18466, was 19912) |
| b | receives median **4704** |
| c | setup / stop median 12229 / 6513 |
