# SCORE — the deferred reply (`Outcome::ReplyTo`)

**STRUCK.** Executor: grok, 2026-09-01. Every row re-run by me. **Two weighs — the first red, handled
correctly.**

```
Summary [ 358.529s] 5175 tests run: 5175 passed (4 slow), 15 skipped
FLOOR=0
```

| # | what | my re-run |
|---|---|---|
| 1 | ★ client wakes client | ✅ `client_wakes_client` |
| 2 | ★ **a TIMER wakes a client** | ✅ `timer_wakes_client_on_thread` **and** `..._on_process` |
| 3 | one call wakes several | ✅ `one_call_wakes_several` |
| 4 | a vanished waiter is survivable | ✅ `vanished_waiter_keeps_serving` |
| 5 | no `Peer` reaches an arm | ✅ arm shapes untouched; `Directed` carries `conn-id <- i64` |
| 6 | existing outcomes unmoved | ✅ floor green |
| 7 | internal `Reply` still asserts | ✅ `internal_reply_still_asserts` |
| 8 | no runtime change | ✅ `git diff src/` empty |
| 9 | no existing service edited | ✅ `wat/` diff is `service.wat` only |
| 10 | floor | ✅ 5175/5175, my own run |

**Row 2 landed at both loci**, which is the row this stone lives or dies on. The internal-op
assertion now catches `Reply`/`ReplyAndArm` and lets `ReplyTo` through — a fired timer can name the
client it was armed for. Had that been left as it was, the stone would have compiled, the floor would
have been green, and rows 1 and 3–10 would all still have passed.

## The red first weigh, and why the golden recapture is honest

Four arms went red: `probe_arc278_peers_bijection::*` — EDN goldens pinning the `:peers` diagnostic's
span. **ARM kept, not re-run, all four named.** They were recaptured with `UPDATE_EDN=1`.

A recaptured golden is exactly how a regression hides, so I checked rather than took it: filtering
`:line` out of the golden diff leaves **nothing**. Reason, message, every other field byte-identical;
the spans moved a uniform **+18** (`864→882`, `871→889`, `881→899`) because `Directed` was inserted
above the macro-error site.

★ **Those goldens are pinning a line number, and that is the thing worth noticing.** They went red
for a reason that has nothing to do with what they test — the `:peers` diagnostic's *content* is
unchanged, and only its address moved. A golden that pins a span is a golden that will go red every
time anything is inserted above it in `service.wat`. That is a maintenance tax with no defect behind
it, and it is worth its own look; not chased here.

## The shape that landed

```wat
Directed {conn-id <- i64, reply <- :R}
Outcome::ReplyTo [state <- :S, sends <- (Vector :- [(Directed :- [R])])]
```

The loop resolves `conn-id → peer` from `selectables` and sends. Absent conn-id, `Closed` or `Lost`
keeps serving; `Stopped` returns — the world stopping, per arc 278 #73. Delivery follows the vector's
order.

**Surface vs internal `ReplyTo` differ, and the difference is right.** A surface arm's `ReplyTo` is
wrapped in the current arm's reply variant (same `:R` as `Reply`); an internal arm's is sent as-is,
because a fired timer has no invoking client and therefore no arm-variant to wrap in — so the arm
constructs the wire reply itself. That asymmetry is what lets row 2 drive the generated client method
rather than a hand-rolled one.

## No deltas

The brief needed no correction. Second stone running with none, and both were drawn against
measurements taken first — here, `probe-roundtrip-cost.wat`'s 1 µs / 130 µs / 154 µs, which is what
established that the circuit's cost is round-trips rather than speed and therefore that long polling
was the right target at all.

## Next

The consumer: long polling in `wat-queue` (`receive` with a wait duration; `wait = 0` byte-identical
to today), then the circuit asking for `limit > 1`. The substrate is now in place for both.
