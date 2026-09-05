# DESIGN — no client call can hang

**Rung 3.** `RecvOutcome` gains `TimedOut`; the generated client method can no longer wait
forever; a recorded codemod adds the arm across the corpus. Correctness. No perf work.

## ⛔ THIS SUPERSEDES A WRONG DRAFT OF MINE — 2026-09-05

The first draft had the deadline **raise inside the macro**, so no call site would change. I
chose that because the migration looked expensive. **It was the same collapse this arc has spent
the day removing** — a distinct condition hidden from the caller so the author would not have to
touch 643 sites. `service.wat` names the principle it broke: **no-hidden-failures**.

★ The builder's correction: *"our verbosity is our shield… if we are adding 'timed out' to our
exception list, then so be it — that's the point."* It is. A timeout is a real outcome and every
caller should face it.

## WHY — one line, 220 surfaces

`wat/service.wat:2237`, in the body every generated client method expands to:

```wat
~r-sym (:wat::kernel::recv c)          ;; a bare, unbounded receive
```

220 Peer surfaces across 162 files all get it. It is why `Seen/mark` hung a worker ~160 s and why
`check` needed forty hand-rolled lines to avoid it.

## ⛔ WHAT I CLAIMED WAS UNKNOWABLE, AND IS NOT

I said each of 643 timeout arms was "a judgement per site, not a rewrite." **Measured, that is
false.** The bodies of the existing `RecvOutcome::Lost` arms across the corpus:

| Lost arm body | count |
|---|---|
| `assertion-failed!` | **245** (plus its 470 `:wat::core::None` arguments) |
| `nil` / `Tuple` / `connect` (swallow or redial) | ~22 |

★ **The neighbouring arm tells the codemod what to write.** The rule is *mirror the `Lost` arm*,
with a timeout-specific message where it is an assertion. Where mirroring is imperfect it
produces *the same behaviour as a vanished peer* — defensible, and loud wherever it matters.

**Nothing here is unknowable at codemod time. This is one-shottable.**

## ⛔ THE ONE CONTRACT DECISION

**`TimedOut` is a fifth arm on `:wat::kernel::RecvOutcome`, and every caller faces it.**

Not a raise, not folded into `Lost`. `service.wat:2253-2258` records someone un-collapsing
`Stopped` from `Lost` **in this very arc**; re-collapsing a different condition into it one arc
later is that mistake made deliberately.

Deadline default **10 000 ms**, tunable per feature by an optional `:deadline-ms` following
`:max-frame-bytes`'s optional-with-default shape — **never optional-off**
(`service.wat:372-377`).

★ 10 000 ms because **the deadline must fire before the harness kills the process, or the
diagnostic is destroyed.** This arc paid for that twice: a `TIMEOUT [30.015s]` with an empty ARM,
and a `drained-never` needing 64 s inside a 30 s cap.

## SCALE, MEASURED

- **643** `RecvOutcome::Message` arms across **282** `.wat` files
- a match that already has a catch-all `_` needs **nothing**
- raw `(:wat::kernel::recv …)` sites gain an arm that can never fire — **that is correct**: the
  type says it is possible, and an unreachable arm is cheaper than a lying type

## THE TOOLKIT

1. **wat-grep + rete — the finder.** A `match` on `RecvOutcome` **without** a catch-all arm.
   Grep cannot do this: it must see the arm *set* of a form, not a token.
2. **wat-fix — the migration.** Insert `((:wat::kernel::RecvOutcome::TimedOut) <mirror of Lost>)`.
   Idempotent; a match that already has the arm is left byte-untouched.
3. **The floor is the census that matters.** 5215 tests exercise these paths; a missed site fails
   to type-check, and a too-short default reds while naming the surface.

## FILES

`src/` (the `RecvOutcome` variant), `wat/service.wat` (the bounded receive), and the corpus via
`wat-scripts/fixes/add-timedout-arm.wat`.

## OUT OF SCOPE = REJECTED

- **Raising inside the macro.** My first draft; retracted above with the reason.
- **Folding a timeout into `Lost`.**
- **Hand-editing `.wat`.** 282 files is a codemod, full stop.
- All perf work.
