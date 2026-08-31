# SCORE — excursus 002 stone 2: the tail escape

**STRUCK.** Executor: grok, 2026-08-31. Every row re-run by me, never read from the report.

```
Summary [ 311.949s] 5135 tests run: 5135 passed (3 slow), 15 skipped
FLOOR=0
```

5135 = 5132 + `probe_ex002_tail_escape` + the drift gate + one more. **All twelve rows pass.**

| # | what | result |
|---|---|---|
| 1 | tail escape rejected | ✅ `HandleTailEscape`, names `:red::tail-escape` |
| 2 | the binding twin compiles | ✅ `:red::held` never named — condition 2 is honoured |
| 3 | a BUILTIN tail head does not fire | ✅ `:red::builtin-head` never named — the subtle one |
| 4 | stone 1 still holds | ✅ `probe_ex002_creation_escape` passes |
| 5 | census, RUN not grepped | ✅ **1698 files checked; exactly one rejection — the acceptance criterion.** Zero live-code hits |
| 6 | the bisect instrument still RUNS | ✅ still prints its discrimination table |
| 7 | ONE tail table, or a drift gate | ✅ **both** — see below |
| 8 | tail semantics unchanged | ✅ dispatch moved to `TailForm`, same seven forms |
| 9 | runes on instruments only | ✅ none on the acceptance criterion |
| 10 | floor | ✅ 5135/5135, FLOOR=0, my own run |
| 11 | the error teaches | ✅ names function, service, created-at span, tail-call span |
| 12 | self-scheduling fixture untouched | ✅ empty diff |

## Row 7 came back stronger than the brief demanded

The brief asked for one shared list **or** a drift gate. The strike shipped **both**:

- `src/tail.rs` — the single table (`TailForm` + `tail_form(head)`), read by `runtime.rs:4417`'s
  dispatch and `check.rs:1997`. The dispatch matches the enum exhaustively, so a new variant is a
  compile error: that is the compile-time half.
- `tail::tests::eval_tail_calls_exactly_the_seven_tail_evaluators` — **parses `eval_tail`'s body out
  of `runtime.rs` and asserts exactly seven `eval_*_tail` calls**, skipping comments. Its own note:
  *"Parse the dispatch, do not trust a comment."* That catches the case the exhaustive match cannot
  — a call added or removed without the enum changing.

This was the row I called "the whole risk of this stone" because its failure mode is invisible.
It came back with two independent guards.

## Deltas, both of which corrected ME

- **My row-9 check was wrong, not the strike.** I ran `grep -c 'rune:' red-tail-escape.wat`, got 1,
  and had a contradiction with the report's "no rune on that file". The hit is *prose forbidding a
  rune*. I matched the token instead of the form — the same distinction `ignore_reason_justified`
  documents at length for `#[ignore]` (a mention is not an attribute). The report was accurate.
- **`:sched::hold-as-param` is not this rule, and my DESIGN said it was.** The strike's reasoning is
  correct: the handle arrives there as a *parameter*, so the let does not create it. I listed it as
  an expected rejection from memory of the bisect's output rather than from the rule I had just
  written.

## ★ THE FINDING — this defect was diagnosed five weeks ago and dismissed

The strike had to rune three scratch-pad probes I did not know existed. They are dated **2026-07-25**,
commit `753b1b9c2`:

```
278 24p curare (compact): the DoS is CLOSED (the day's real work);
the 'TCO bug' was NOT one — apparatus escalated a contrived form
```

`probe-arc278-tco-drops-caller-env.wat`'s header carries the complete, correct mechanism — five
weeks before I re-derived it by four-way bisect:

> *"GROUNDED VERDICT (2026-07-25): it is not `let`. It is the TAIL-CALL TRAMPOLINE. … every binding
> of the calling frame is dropped BEFORE the tail callee runs. For a pure value that is invisible.
> For a value holding a LIVE RESOURCE (RAII drop) it is a reap: the resource dies, the callee runs
> against a corpse, **and the failure returns through a legitimate-looking outcome variant carrying
> a false story**. It is GENERAL, not service-specific."*

And the builder's invariant, recorded in its companion: *"c must not get reaped until the let is
completely resolved — it's still bound in that scope."*

**The timeline is the lesson:**

| | |
|---|---|
| 2026-07-21 | self-scheduling fixture born; strike STOPs on a real timer gap |
| 2026-07-21/23 | two real substrate bugs found and fixed |
| **2026-07-25** | **this defect probed, diagnosed exactly — and judged "NOT one… a contrived form"** |
| then | self-scheduling still red → blamed on a `remove-at` idx-shift → `#[ignore]`, 38 days |
| 2026-08-30 | re-derived by bisect, from scratch |

The dismissal turned on the word **contrived** — and the demonstration *was* contrived, so the
mechanism was judged unreal. But contrived-demonstration and unreal-mechanism are different claims,
and the second does not follow. A real program was hitting it **four days earlier**, and that
program's failure was already being blamed on something else.

★ This is the strongest argument for the wall that exists. The defect was known, correctly
understood, written down, and closed as not-worth-fixing because no real program seemed to hit one.
The runtime notice (`Severed`) cannot fix that — it is measured racy, and it only speaks after the
program has run. A compile-time wall does not race and does not wait.

## Not done, and named rather than deferred

**A third escape shape exists and neither stone catches it.** `:sched::hold-as-param` passes a
freshly-`/start`ed handle as a *temporary argument*; the callee takes it as a param and tail-escapes
a peer of it. The bisect measures it severing (`C-param-tail=-11`), and the census confirms the wall
is silent on it. The creating scope is the *caller's argument evaluation*, which neither stone-1's
"scope that creates" nor stone-2's "let that creates" reaches. That is a stone 3 candidate, and it
is written here rather than left to be re-derived.
