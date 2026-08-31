# SCORE — excursus 002 stone 3: a param is an owning binding, downward

**STRUCK.** Executor: grok, 2026-08-31. Every row re-run by me.

```
Summary [ 316.072s] 5136 tests run: 5136 passed (3 slow), 15 skipped
FLOOR=0
```

**All twelve rows pass.** The excursus is complete: all four cells of the invariant are settled.

| # | what | result |
|---|---|---|
| 1 | road 3 rejected | ✅ `HandleTailEscape` names `:red::drive-param` |
| 2 | upward untouched | ✅ `:red::conn` never named — the widening did not leak across directions |
| 3 | binding form compiles | ✅ `:red::held-param` never named |
| 4 | stone 1 holds | ✅ |
| 5 | stone 2 holds | ✅ |
| 6 | census, RUN | ✅ **1699 files; only the three red probes.** Zero live-code hits |
| 7 | bisect still RUNS | ✅ still prints `C-param-tail=-11` |
| 8 | no runtime change | ✅ empty diff |
| 9 | no rune on the criterion | ✅ 0 rune FORMS (1 prose mention — the distinction I got wrong twice) |
| 10 | floor | ✅ 5136/5136, my own run |
| 11 | the error teaches | ✅ and better than asked — see below |
| 12 | the four cells hold | ✅ |

## Row 11 came back better than the row asked for

The brief asked the error to name the param. The strike gave `HandleTailEscape` a
`param: Option<String>` field that **discriminates which stone fired**: `None` = stone 2 (the handle
was created in this scope), `Some("h")` = stone 3 (this frame's parameter owns it). One error kind,
two causes, told apart in the payload rather than by reading the message. That is the shape this
codebase already uses for `RecvOutcome`/`LociDiedError` — a failure is a matchable value, not a
string to parse.

## ★ The conservative trade SHIPPED, because the census said it could

This stone was drawn with a live STOP: it rejects a callee that tail-escapes a peer of a param
handle **even where the caller still holds it** and the program is safe. Row 6 was written as a STOP,
not a cleanup task — *if live code hits that shape, the trade is wrong and the stone does not ship.*

Census: **zero live-code hits across 1699 files.** The only rejections are the three deliberate red
probes. So the trade is not merely defensible in theory, it costs nothing in this corpus today. That
is the difference between a wall that was measured and one that was argued for.

## Delta — my brief named the wrong function, again

STOP-3 said to rune `:sched::hold-as-param`. The rune correctly went on **`:sched::drive-param`**
instead, and the rune's own prose says why: *the wall names the callee.* `hold-as-param` is the
CALLER — it only passes a temporary. The frame that owns the param and tail-escapes is the callee,
so that is where the rejection lands.

## ★ The pattern across this excursus, which is mine and worth keeping

Every stone in excursus 002 shipped, and **every stone also corrected a specification error of
mine**:

| stone | my error | how it was caught |
|---|---|---|
| 1 | rule keyed on the PARAM — would reject every `conn` helper, three in the stdlib | reading the corpus, before the brief shipped |
| 1 | put the must-be-REJECTED probe under `wat-scripts/`, whose gate demands every file PASS | the strike refusing to rune it |
| 2 | DESIGN listed `hold-as-param` as an expected rejection; it is not that rule | the strike |
| 2 | graded a rune by `grep -c 'rune:'` — matched the TOKEN, hit prose | my own re-check |
| 3 | STOP-3 named the caller instead of the callee | the strike |

The common shape: **I wrote rows from memory of a measurement rather than from the rule I had just
written.** The bisect's output said `C-param-tail=-11`, so `hold-as-param` went into the DESIGN as
an expected rejection — without asking which function the rule actually names. That is
`feedback_cite_an_exemplar_do_not_describe_one` wearing a different coat: a described shape decides
things the prose never ruled on, and here the described shape was my own prior measurement.

The counter-practice that worked every time: **run the census before writing the acceptance
criterion.** Stone 3's rule was derived by probing road 4 and road 3 first, and it is the only stone
whose rule needed no correction.
