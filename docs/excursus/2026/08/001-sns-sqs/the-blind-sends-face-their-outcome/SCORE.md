# SCORE — the blind sends face their outcome (piece A)

**Struck 2026-09-15.** Piece **A** of `a-send-cannot-say-it-is-blocked/DESIGN.md` §decomposition — the
half of that DESIGN that scores **four YES** and needs no ruling. Builder: *"strike A"*.

## The floor, verbatim

```
    Summary [ 544.986s] 5251 tests run: 5251 passed (9 slow), 22 skipped
```

`./scripts/floor.sh`, `.floor/2026-09-16T02-36-29Z/`, exit 0, **no `ARM.txt`**. 545.0 s sits inside the
540.4–561.5 s same-code band for this box. Clippy `--workspace --all-targets`: **0**.
`cargo nextest run --release --no-run`: clean.

## ⚠ THE SITE LIST CHANGED ON READING THE CODE — 4 sites became 3, and my instrument was wrong

The DESIGN says *"4 blind live sites"*, from a balanced-reader census that labelled
`wat-scripts/queue/sqs.wat:2059` and `:2064` **NOT-IN-A-MATCH**, glossed as *"discard the SendOutcome
entirely, so a blocked send there is invisible by construction."* **False.** Both sends are arguments to
`:user::send-ok!` (`sqs.wat:2047`), which **does** match the outcome. The reader saw no match because the
`send` is an ARGUMENT to a helper, not the subject of one.

⭑ **The blindness was real but RELOCATED**: one wildcard inside `send-ok!`, covering **both** call sites,
collapsing `Closed`/`Lost`/`Stopped` into a single message that cannot say which happened
(`[[feedback_a_fallback_that_collapses_failures_reports_nothing]]` — *can two different worlds print this
line?* Three could). So A is **3 edit sites**, one of which fixes two call sites.

## What landed

| site | was | now |
|---|---|---|
| `wat/service.wat` `call-by-deadline` | `(_ …)` holding the whole 37-line select body | body lifted to `:wat::service::race-reply`; **four arms named**, `Sent`/`Closed`/`Lost` all still race |
| `wat-scripts/queue/sqs.wat` `send-ok!` | `(_ "send-ok: not Sent")` | four arms; `Closed`/`Stopped` name themselves, `Lost` carries its cause via `LociDiedError/message` |
| `wat-scripts/fanout/circuit.wat:3836` | `(_ "send-failed")` | `send-closed` / `send-stopped` / `send-lost`, deliberately distinct from the select arms' `closed`/`lost` |

**No `src/` change.** The `race-reply` extraction exists because a 37-line body cannot be repeated in
three arms; its header carries, verbatim, the reason the wildcard was behaviourally CORRECT — the death
notice rides the recv channel, so aborting on a send-side failure would flatten a `Severed` peer into
`Disconnected` without ever reading it.

## ⭐ THE LOUDNESS PROPERTY IS PROVEN, NOT ASSERTED

A's whole purpose is that the **next** variant added to `SendOutcome` becomes a checker error at these
sites instead of being silently absorbed. Driven: a copy of `circuit.wat` with **one arm removed**
(`Stopped`), run on the current runtime —

```
non-exhaustive: enum :wat::kernel::SendOutcome missing arm(s) for variant(s): Stopped (or include `_` wildcard)
```

— and the unmodified file produces **zero** such errors (positive control). The control copy was made
inside `wat-scripts/fanout/` (circuit.wat loads `../topic/sns-fanout.wat`, so a `/tmp` copy dies on the
relative import — a first attempt did exactly that) and **removed immediately**; `git status` after shows
only the three intended edits.

## Behaviour is IDENTICAL

`deadline-redial-is-fresh` runs FIRST in circuit's `main`, so site 3 is on the happy path. Both markers
diff byte-identical against the pre-change run:

```
timeout=yes;discarded=yes;redial=Connected;retry-on=fresh
distinct=8000;dup=0
```

`sqs.wat` is driven by `tests/services/probe_queue_long_poll.rs` and `probe_queue_depth.rs` — in the
floor. `call-by-deadline` is on every generated client method's path — also the floor.

## ⚠ MY DETECTOR READS COMMENTS — stated so the next reader does not repeat it

The post-fix verification first reported `circuit.wat:3819` as STILL wildcarded. The only `(_` left in
that form is the literal `(_ "send-failed")` **inside the comment explaining its removal**. Two further
rows (`service.wat:3311`, `:3369`) were nested matches over a DIFFERENT enum, attributed to the enclosing
form. Comment-stripped and depth-aware (a wildcard ARM is `(_` at depth 1 of the match form), the true
state is:

```
`(match (send …))` forms corpus-wide   198
with a top-level wildcard ARM            1   ← wat-scripts/scratch-pad/probe-client-deadline-via-select.wat:92
```

**Zero live blind send sites remain.** (198 ≠ the census's 215 send *call sites*: the difference is sends
that are arguments to a helper rather than the subject of a match — `send-ok!` being the example that
started this section. Different instruments, both honest, neither contradicting the other.)

## Found and NOT fixed — reported, in scope for a later stone

**`wat/service.wat:3311` and `:3369`** — `grant`'s and `revoke`'s generated bodies match their `Status`
recv and end in a `(_ …)` wildcard before returning `GateOutcome::Applied`/`GaveUp`. **Same blindness
class, opposite primitive.** A is send-only, so they stand. They are adjacent to OPEN item 2, the
unsolicited-frame class, which already names those four methods' one-level drain.

## What A does NOT do

It adds **no bound**. A blocked `send` still blocks, still cannot be reported, and `SendOutcome` still
has no variant for it — that is pieces C+D, which the four questions say are ungradeable until A lands,
and which A has now made gradeable. A only ensures the variant, if it is ever added, **cannot be
ignored at a live site**.
