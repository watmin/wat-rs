# SCORE — the publisher gives up in time, not in tries

**SCORED.** Executor: grok, 2026-09-15, branch `sns-sqs`, HEAD `c6e53bf74` (DRAWN). Did not commit.

⛔ The headline is that **a slow topic no longer kills the publisher.** The give-up is
a named `PublishLadder::Exhausted` carrying **elapsed-ms and ceiling-ms**, never an
attempt count. `recv-by-deadline` is still unmeasured. The worker's ladder at
`:747` was read and left alone.

```
     Summary [ 559.412s] 5251 tests run: 5251 passed (9 slow), 22 skipped
```

`.floor/2026-09-15T05-17-54Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5251**, unchanged. Clippy **0**. NORUN **0**.
Porcelain: `wat-scripts/fanout/circuit.wat` only. `src/` `wat/` `tests/` empty.

---

## ⭑⭑ THE HEADLINE — named exhaustion, no crash

Slow topic (`:n 1 :delay-bp 10000 :delay-ms 11000 :publish-ceiling-ms 15000`):

```
exit=0
pub-exh-elapsed=20003;pub-exh-ceiling=15000;pub-exh-last=TimedOut
```

Today: `exit=2`, `"publisher stats closed" ×3`, `assertion-failed!` *"the peer is
alive and silent"*. After: no `assertion-failed!`, no `publisher stats closed`.
Two 10 s deadlines then the ceiling; `last=TimedOut`. Wall ~28 s, not ~43 min.

Happy path (injector disarmed, ceiling None → 60000):

```
distinct=8000;dup=0
pub-exh-elapsed=0;pub-exh-ceiling=0;pub-exh-last=none
```

Exhausted is **not** produced on a healthy topic.

---

## The payload is TIME (row 3)

```
(:wat::core::defenum :fanout::PublishLadder :wat::enum::Pure
  :Accepted  []
  :Exhausted [elapsed-ms <- :wat::core::i64
              ceiling-ms <- :wat::core::i64
              last       <- :wat::core::String])
```

No attempt count. Both struck rulings: a named variant the match must name
(`exhaustion-is-a-named-variant`), whose payload is time (the spirit of
`a-give-up-has-no-form-for-attempts`).

The named match is on the **parent** after `join-publishers`. The publisher
child cannot see a script-level enum (the same wall `exhaustion-is-a-named-variant`
measured for `SeenRetry`). The child writes `exh-last` onto stats; the parent
matches `PublishLadder` so Exhausted cannot be dropped.

Not `:fanout::Verdict`. `require!` untouched.

---

## Default 60000, gated at 10000 (row 11, STOP-1)

None → **60000 ms**. That is the existing never-accepted wall that already
lived in the publisher fold (`elapsed >= 60000`). Already a time bound, already
greater than one 10 s `Topic/publish` deadline. `(Some ms) < 10000` is refused:
a ceiling shorter than one attempt is a no-op ladder.

Clock is read at the `-run` boundary (`t-run`), not threaded through the nested
Tuple acc (trap-door 4).

---

## STOP-4 — the sibling was checked

`circuit.wat:747` is still `(:wat::core::range 0 256)` and still returns
`:fanout::SeenRetry::Exhausted` rather than raising (`:754`). Not touched.

---

## WHAT LANDED

`wat-scripts/fanout/circuit.wat` only.
- `:fanout::Input/publish-ceiling-ms` (Option; None = 60000).
- Publisher durable + `StatsResponse::Ok` carry `exh-elapsed-ms`,
  `exh-ceiling-ms`, `exh-last`.
- Six raising arms faced: TimedOut/Lost/Stopped/Closed/not-Accepted/Malformed
  continue until elapsed ≥ ceiling, then stop with a named last.
- Phases: `pub-exh-elapsed` `pub-exh-ceiling` `pub-exh-last` (`none` when idle).
- Fill/drain waits skipped when the publisher already exhausted, so a give-up
  does not stall the parent for 30 s on work that will not arrive.

---

## Floor delta

**559.412 s** vs baseline **537.427 s**. Delta **+22.0 s** on this box.
The orchestrator's quiet-box run grades. Count unchanged.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭑⭑ slow topic does not kill the publisher | exit 0; no assertion-failed; no stats-closed |
| 2 | ⭑⭑ named exhaustion on the report | `pub-exh-elapsed=20003;pub-exh-ceiling=15000;pub-exh-last=TimedOut` |
| 3 | ⛔ payload is time | Exhausted[elapsed-ms, ceiling-ms, last]. No tries |
| 4 | ⛔ healthy never exhausts | `pub-exh-last=none` |
| 5 | floor | **5251 passed**, 0 FAIL |
| 6 | ⭑ happy path | `distinct=8000;dup=0` |
| 7 | clippy | 0 |
| 8 | tests compile | NORUN=0 |
| 9 | ⭐ sibling checked | `:747` still returns `SeenRetry::Exhausted`. Untouched |
| 10 | ⛔ Verdict/require! | defs unchanged |
| 11 | ceiling default stated | **60000**, the old never-accepted wall; `< 10000` refused |
| 12 | blast | `circuit.wat` only |
| 13 | scope | this SCORE's headline |
