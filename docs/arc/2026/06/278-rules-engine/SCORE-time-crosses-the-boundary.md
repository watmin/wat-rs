# SCORE — time crosses the boundary

**STRUCK.** Executor: grok, 2026-09-03. Every row re-run by me on a quiet box.

```
Summary [ 360.103s] 5213 tests run: 5213 passed (3 slow), 15 skipped
FLOOR=0        .floor/2026-09-03T21-11-38Z/        my own run, 0 FAIL/TIMEOUT lines
```

## ★ ROW 2 WAS THE STONE, AND IT LANDED

```
zero=[MALFORMED:expected=:wat::time::NonZeroDuration;got=Integer];then=[ok:250000000]
```

3/3, process locus, same connection. **Not `LOST`. Not `CLOSED`.** A peer sent a zero wait, got a
typed refusal naming both sides of the mismatch, and the service answered the next call.

That is the thing this stone existed for. Stone A's wall is rung 3 for a literal and **rung 2 for a
computed value** — a runtime panic surfacing as `LociDiedError/Panic`, which at process locus kills
the child. **A zero arriving over the wire is the computed case by definition.** Until now the only
available answer to a remote caller sending zero was a corpse. Now it is `RequestMalformed`.

The floor carries it by name: `time_nonzero_duration_refuses_zero_as_coerce_error_not_panic`.

## Rows — my re-run

| # | row | result |
|---|---|---|
| 1 | ★ all four cells cross | ✅ 3/3 — `immediate=[ok:0]; upto=[ok:250000000]; duration-CONTROL=[ok:1000000000000000]; instant-EXEMPLAR=[ok:1000000000000000]` |
| 2 | ★★ zero refused, service lives | ✅ 3/3, above |
| 3 | negative control — String | ✅ `Invalid [] ":wat::time::Duration" "String"` |
| 4 | negative duration refused | ✅ `Invalid [] ":wat::time::Duration" "Integer"`; and zero-to-`NonZeroDuration` likewise |
| 5 | encode untouched | ✅ **no hunk anywhere in 4100–4200.** `Instant` was right: encoding was never the blocker |
| 6 | blast radius | ✅ `src/edn/render.rs` only, +128/−10 |
| 7 | doc table not left lying | ✅ |
| 8 | the floor | ✅ **5213/5213, my run** |

## ★ THE DELTA THE DESIGN MISSED — three arms were not enough

I wrote *"three arms in one `match`"* and predicted 25–45 minutes. The arms fixed **thread** locus.
**Process** locus then died on a *valid* `UpTo`: record reconstruction took the untyped path, left a
bare `i64` in the field, validation accepted the `Integer` and discarded the coerced value, and the
handler called `nanoseconds` on an i64.

The repair is `decode_declared_field` — typed coerce first, untyped fallback so a wrong-typed body
still reconstructs and the request-malformed wall can still name it. Its own comment states the
reason better than my DESIGN did:

> *"Instant does not need this — the untyped decoder already yields `Value::Instant` from
> `Edn::Inst`. Duration / NonZeroDuration do, because their wire form is a bare Integer."*

★ **That is a genuinely new case, not an oversight in the executor's reading.** `i64→i64`,
`String→String`, `Instant→Instant`: every surface payload type before these round-tripped through
the untyped decoder to *its own variant*. **`Duration` and `NonZeroDuration` are the first whose
wire form untyped-decodes to a DIFFERENT variant.** Nobody hit it because nobody could — no time
type had ever been a surface payload.

And note the shape: **the coerce arms alone went green at thread locus and died at process.** A
third locus asymmetry in this arc, after the duration-0 timer and the `Closed`-vs-`TIMED-OUT`
divergence. Recorded, not chased — **S22**.

The blast-radius prediction held exactly (`render.rs` only); the *content* prediction did not.

## My error, caught by the executor

EXPECTATIONS row 1 specified `duration-CONTROL=[ok:1000000]` and `instant-EXEMPLAR=[ok:1000000]`.
Both are `1000000000000000`.

`:wat::time::at` takes **epoch-seconds** — `time.rs:86`, *"the instant at that epoch-seconds
mark"*, with `@example (epoch-seconds (at 1000000000)) #=> 1000000000`. So
`(at 2000000) − (at 1000000)` is 10⁶ **seconds** = 10¹⁵ ns. **I was off by 10⁹**, having assumed
nanos without reading the intrinsic I was calling.

It cost nothing only because the probe's `verdict` keys on `upto` alone. Had it keyed on all four,
a correct implementation would have reported red against my arithmetic.

## What landed

Three target arms in `edn_to_typed_value_inner` beside the `:wat::core::i64` exemplar, plus
`decode_declared_field` at the three record-reconstruction sites, plus six unit tests named for
their subjects. Encode at `render.rs:4158-4160` untouched. No `.wat`, no codemod, no new `Value`
variants. Floor 5207 → 5213, all six new tests green.

**Stone B can use the wire.**

## Still open

- **Stone B** — `wait <- Queue::Wait` with `:Immediate` / `:UpTo [NonZeroDuration]`. **Unblocked.**
  `sqs.wat:737`'s clamp **stays** (a tick-rate floor and now a panic boundary), and gets the WHY
  comment it never had.
- **Stone C** — `Alarm :delay`, `Milliseconds`, `visible`/`unacked`.
- **Stone D** — the helper vocabulary. Owns the open race; reproducer committed and deterministic.
- **S15** teardown wedge · **S16** SCORE divergence · **S17** `Duration`'s `i64` · **S18** spurious
  `Closed` · **S19** `time::+` · **S20** `wait`'s five senses · **S21** `sqs.wat:11-12`'s
  now-half-true comment · **S22** the thread/process reconstruction asymmetry.
