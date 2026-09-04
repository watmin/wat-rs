# SCORE — the vocabulary stops mumbling

**STRUCK.** Executor: grok, 2026-09-04. Every row re-run by me.

```
Summary [ 370.225s] 5214 tests run: 5214 passed (3 slow), 15 skipped
FLOOR=0        my own run, 0 FAIL/TIMEOUT
```

**5214, not 5213** — the +1 is a new gate, which is the right kind of floor growth.

## Rows — my re-run

| # | row | result |
|---|---|---|
| 1 | ★★ the rewrite is visible | ✅ `accept!` / `accept-stamped` **gone**; `publish-stamped-until-accepted!` and `publish-until-accepted!` in their place |
| 2 | ★★ the bound reports | ✅ `publish_liveness_bound_reports_what_it_saw` **PASS on my run** — a named gate with five assertions. `verdict=never-accepted; depth=2; cap=2; attempts=1; elapsed=0` — all four fields |
| 3 | ★★ backpressure survives | ✅ `distinct=8000; dup=0` **all five**; the bound did not trip. Production wrapper 60 000 ms |
| 4 | the `Lost`-is-ok arm has a WHY | ✅ documented, behaviour unchanged — and it corrected me |
| 5 | `nap-ms` renamed, not consolidated | ✅ six homes, six `await-timer-ms`, still six |
| 6 | `do-` swept | ✅ |
| 7 | codemod recorded + idempotent | ✅ re-run after apply: **0 Matches** |
| 8 | scope | ✅ no S33 merge, no S34 change, no `wat/service.wat`, no `src/` |
| 9 | the floor | ✅ **5214/5214, my run** |

## ★ THE BOUND IS A NAMED GATE, NOT A PRINTED STRING

Row 2 could have been satisfied by a probe printing four numbers. It is instead
`publish_liveness_bound_reports_what_it_saw` on the floor, asserting the report **field by field** —
so a future edit that reduces it to *"gave up"* goes red. Stone D's lesson (*a bound that only says
"timed out" is the empty ARM again*) is now enforced by a test rather than by a SCORE.

And the force-expire is honest: `limit-ms 0` against a cap-2 inbox with no workers. The production
wrapper is **60 000 ms** — a LIVENESS bound, only a hang may trip it, exactly as the taxonomy says.

## ⛔ MY BRIEF WAS WRONG ABOUT `Closed`, AND IT WAS DOCUMENTED RATHER THAN OBEYED

I wrote: *"`Lost` is treated as success, `Stopped`/`Closed` assert."* The disk, `sns-fanout.wat:600`:

```wat
((:wat::kernel::RecvOutcome::Lost _cause) nil)
(:wat::kernel::RecvOutcome::Stopped  (assertion-failed! "topic-worker: start stopped" …))
(:wat::kernel::RecvOutcome::Closed   nil)
```

**`Closed` is `nil`, treated like `Lost`.** The executor read the disk, found the brief wrong, and
**documented the real behaviour instead of flipping the code to match my sentence.** That is the
correct call and it is the second time this campaign a brief's citation was wrong and the executor
went to the source instead.

The answer to row 4 is now on the disk: *start arms the tick and replies Ok; `Lost` is not death,
because the arm runs to completion — a lost reply still means the tick is armed, so proceed. A dead
worker shows up as `unread` (-1) on drain. `Stopped` is the parent shutting down. `Closed` is a clean
EOF without a message, treated like `Lost`.*

## ★ THE DELTA WORTH KEEPING — a Handle killed the first attempt, and naming an outcome found it

The first force-expire died **for a `Handle`, not for the bound**: the third publish sat in the
`let` **body**, so the topic `Handle` was released, and *the service severed because its owner
released the service handle*. Moving the call into the **binding vector** fixed it.

> *"Naming `Lost` (instead of `_` → 'recv failed') is what made that a sentence."*

★ **The campaign's own rule found an unrelated bug.** A collapsed `_` arm would have reported "recv
failed" and sent the executor after the bound — the thing under test — instead of at handle
lifetime. This is the fourth time this session that naming an outcome, or failing to, decided
whether a failure was legible: `q-depth`'s `(Tuple 1 1)`, the wire probe's `-1`, my reply-drop's
`LOST`, and now this.

## The other honest delta

**`accept-stamped` was not in my DESIGN's table — and the inner does not stamp.** My proposed name
would have been a second lie. Renamed `:fanout::publish-until-accepted!`, so the mumble did not
survive by inheriting a wrong rename.

## Census

Finder, before apply: **94 keyword occurrences across 7 files** — occurrences, not lines. My DESIGN
listed *sites*, which is a different count and was never the instrument. Seventh file is
`probe-refused-retry-self-consumes.wat`, which calls `:demo::` names. Re-run after apply: **0
Matches.**

## Still open

- **Stone C** — `Alarm :delay`, `Milliseconds`. The last drawn-but-unstruck naming work.
- **S33** (`receive`/`receive-wait` merge) · **S34** (`await-timer-ms` swallowing outcomes into
  `nil`) · **S15**–**S32** · the arc-109 phantom-form NOTE.
- **3d** — refuted; needs a mechanism that does not exist in userland.
