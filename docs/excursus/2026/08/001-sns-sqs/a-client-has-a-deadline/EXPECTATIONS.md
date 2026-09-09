# EXPECTATIONS — a client has a deadline

Written **before** the strike.

| # | what | expected |
|---|---|---|
| 1 | ★★ a silent server times out, not hangs | the committed probe, still green: `fast=reply; never=TIMED-OUT` |
| 2 | ★★ the retry reaches a **fresh** peer | shown, not asserted. ⛔ Reusing the discarded peer is a hang wearing a fix's name |
| 3 | ★★ **`seen-dups`, five runs** | **any value is the result** — see below |
| 4 | ★ the invariant | `distinct=8000; dup=0`, five runs |
| 5 | the deadline does not fire in normal operation | if it does it is **too short**; the fix is a longer deadline, never a faster server |
| 6 | every `ServiceEvent` arm named | no wildcard in shipped code |
| 7 | scope | `circuit.wat` only; no `src/`, no `Queue/receive`, no drop |
| 8 | the floor | `5214/5214` |

## ⛔ ROW 3 HAS NO EXPECTED VALUE, DELIBERATELY

A client that times out and retries is **retrying a request the server may already have processed.**
That retry is a duplicate by construction — so this stone can move `seen-dups` **before R2, and with
no fault injected at all.**

- **`seen-dups > 0`** — a real retry produced a real duplicate and the consumer absorbed it. That is
  the thing chased since R69, arriving from ordinary operation rather than from chaos.
- **`seen-dups = 0`** — the deadline never fired, which given row 5 is the *correct* outcome for a
  healthy system. Then R2 is still what moves it.

⛔ **Do not tune the deadline to make row 3 come out.** A deadline chosen to produce duplicates is
row 5 failing.

## RUNTIME PREDICTION

**45–75 minutes.** The shape is a working probe; the work is naming the `ServiceEvent` arms honestly
and threading the fresh peer back into state.

## TRAP-DOOR RISKS

1. **Tier is the raced peer's, not the caller's.** A `thread` timer cannot join a socket select set.
   This cost the probe two attempts.
2. **The timer's message must be `Reply`-typed** so `O` unifies; the laundering fixes `I`.
3. **Threading the fresh peer back into state.** Trap 3 from the `Closed` stone, and it converted a
   death into a hang there too.
4. **`Seen/claim`'s `Lost` and `Closed` arms already redial.** The timeout path joins them; do not
   duplicate the redial logic three ways.
5. **A bounded retry that exhausts must REPORT** — depth, attempts, elapsed. "gave up" alone is the
   empty ARM again.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 2 asserted rather than shown.
- Row 3 reported after tuning the deadline.
- A wildcard left in the shipped `ServiceEvent` match.
- The retry reusing the discarded peer.
- Row 4 from fewer than five runs.
