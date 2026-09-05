# SCORE — every client call has a deadline

**STRUCK.** Executor: grok, 2026-09-05. Tree safe, uncommitted.
`wat/service.wat` (one defn, after the macro) + `wat-scripts/fanout/circuit.wat`.

```
Summary [ 363.905s] 5214 tests run: 5214 passed (4 slow), 19 skipped
```

## THE HELPER

`:wat::service::call-by-deadline :- [I O]` sits immediately after `send-keep-serving?` (`service.wat:3123`). Nothing above `:896` moved. Diff is append-only at `:3118`. Goldens green.

Returns `(Tuple (Option O) i64)`: **0 = answer, 1 = lost/closed, 2 = deadline.** `inert` is never read. The extra `i64` is the check-loop's discriminator (retry only on 2); BRIEF sketched `Option O` alone, which would have retried Lost and failed STOP-2.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ dropped `mark` no longer hangs | ✅ tiny ×6 all **terminate in ~8 s**. No `drained-never`. Drop aimed at `mark`. |
| 2 | ★★ `check` behaviour-identical | ✅ rate-0 `total=8000; distinct=8000; dup=0; seen-recorded=8000` ×5. Tiny `total=100; distinct=100; dup=0; seen-recorded=100` ×6 |
| 3 | rate-0 unchanged | ✅ as row 2. `seen-skipped` 3–10 (before 6–11) |
| 4 | tiny unchanged | ✅ **6/6** complete (before 5/6). `seen-skipped` 13–22 — mark-drop retries, reported |
| 5 | the floor | ✅ **5214/5214, 19 skipped** |
| 6 | one new stdlib form | ✅ append after the macro |
| 7 | goldens undisturbed | ✅ no `peers_bijection` failure |
| 8 | timings | publish **45883 46337 46532 46167 46134** (before 45790–46126). Drain 179–222. Stop 5694–6462 |

## ROW 1 — the drop moved

Chaos now hides **`mark`** replies (writes the receipt, then `None`). `check` is a pure read again. Worker `call-by-deadline`s `mark` at 200 ms, redials, retries twice more, then acks anyway. Before: `drained-never` ~160 s. After: every run finishes.

## FOUR SITES

| call | deadline | `:wait` |
|---|---|---|
| `Seen/check` | 200 ms (lifted `once`) | — |
| `Seen/mark` | 200 ms, 3 retries | — |
| `Queue/ack` | 200 ms, 3 retries | — |
| `Queue/receive` | **1000 ms** | **UpTo 250 ms kept** |

Receive's client deadline is above the server long-poll. STOP-3 did not fire.

## NOT TOUCHED

`claim deadline exhausted` (0/6 this tiny set; was 1/6). Redelivery fixture. Send-path scans. Generated methods still exist undeadlined — rung 3, out of scope.

---

# ORCHESTRATOR GRADING — claude, 2026-09-05

**STRUCK, and committed.** Every row re-run by me on the executor's tree.

| # | my result | agrees |
|---|---|---|
| 1 | **6/6 terminate**, drop aimed at `mark`. No `drained-never`, no `claim deadline exhausted` | ✅ |
| 2/4 | tiny **6/6**: `total=100; distinct=100; dup=0; seen-recorded=100`; `seen-skipped` 13–18 | ✅ |
| 3 | rate-0 ×5: `total=8000; distinct=8000; dup=0; seen-recorded=8000` | ✅ |
| 5 | `Summary [ 359.199s] 5214 passed, 19 skipped` — `.floor/2026-09-05T07-55-54Z/` | ✅ |
| 6 | `wat/service.wat` diff is **one hunk, `@@ -3118,0 +3119,50 @@`** — append-only, nothing above `:896` | ✅ |
| 7 | no `peers_bijection` failure | ✅ |
| 8 | publish `45547–46716` vs before `45790–46126` — overlapping, noise | ✅ |

★ **Row 1 is the refutation row and it passed.** The drop is aimed at `mark` — the exact
configuration that hung a worker ~160 s and that the previous strike backed away from. **First
time the tiny cell has gone 6/6**; it was 5/6, then 3/6, then 1/6.

★★ **And `circuit.wat` NET-SHRANK while gaining three deadlines**: `+79 / -122`, **−43 lines**.
The 40-line inline dance collapsed into four calls. That is the DESIGN's argument — *the wrong
path was the easy path* — showing up as a line count.

## THE DEVIATION WAS RIGHT

The BRIEF sketched `-> (Option :- [:O])`. Grok returned `(Tuple (Option O) i64)` with **0 =
answer, 1 = lost/closed, 2 = deadline**, because collapsing `Lost` into the same `None` as a
timeout would have made `check` retry on a dead peer and failed STOP-2. My sketch was wrong and
the executor caught it against the existing `check` behaviour. Recorded so the sketch is not
copied later.

## ⚠ FINDING — the new stdlib form carries one fact twice, and a caller already ignores half

`(Option :- [O], i64)` is **conventionally** consistent — code 0 ⟺ `Some` — but nothing enforces
it. `(None, 0)` and `(Some x, 2)` are both writable, and a caller may consult either half.

**One of the four sites already consults only the Option.** `circuit.wat:401`:

```wat
(:wat::core::match (:wat::core::first recv-got)   ;; the code is never read
```

The other three all branch on `code`. ⚠ **This is not a defect today** — the receive path's
fallback arm redials unconditionally (`:535-542`), so `Lost` and `deadline` both reach a correct
(if redundant) reconnect. But it is the *shape* permitting the mistake, and the discriminator was
added precisely because collapsing those two cases was wrong.

★ **This is now stdlib.** Every future service client will copy this form, so the moment to
tighten it is before there is a second copy. Rung 3 is a three-arm enum —
`Answered [reply <- :O] | PeerLost | DeadlineFired` — under which **the reply cannot be obtained
without learning why it arrived**, and the inconsistent pairs have no form.

That is the follow-up, and it is small while there are four call sites.

## NOT TOUCHED, STILL OPEN

`claim deadline exhausted` (0/6 this set, was 1/6 — within noise, not a fix). The redelivery
fixture that kept its name and lost its meaning. The send-path double scan. Generated client
methods still callable with no deadline — the rung-3 census the DESIGN named and cut.
