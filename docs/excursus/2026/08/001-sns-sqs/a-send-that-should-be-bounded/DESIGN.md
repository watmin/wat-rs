# DESIGN — a send that should be bounded (the classification)

**Drawn 2026-09-17.** Builder's item **B**, the last of the four-questions decomposition in
`a-send-cannot-say-it-is-blocked/DESIGN.md` §decomposition. **REPORT-ONLY: no `.wat`, no `.rs`.**
Scores four YES and needs no ruling — which is why it can be struck before the mechanism is chosen.

## Why, and what it is for

`a-send-cannot-say-it-is-blocked` proved by measurement that a `send` into a peer whose receiver has
stopped draining **blocks** (≥ 38 s, returning only when `timeout`'s SIGTERM killed the service) and
that **`SendOutcome` has no variant that can report it** — `Sent | Closed | Lost | Stopped`, where the
value finally observed names the peer's *death*, not the block.

The mechanism to fix that (a `send-by-deadline` primitive plus a variant, versus a `try-send` retry
loop, versus a separate outcome enum) is **the builder's ruling and is not this stone**. This stone
makes that ruling cheap by answering the question it depends on: **of the 32 live send sites, how many
actually want a bound?** Paying 215 corpus-wide arms for a variant is a very different proposition if
the answer is 3 than if it is 30.

⭑ Piece **A** (`e03fc7f46`) already landed: every live send site now *faces* its outcome, so this
classification is purely about **intent**, not about blindness.

## The classes

| class | test |
|---|---|
| **BOUND** | a blocked send would hang something that should instead report. The site has a caller who is owed an answer. |
| **PARK** | blocking IS the loop — a worker or serve loop with nothing else to do until the peer drains. |
| **REPLY** | a serve-loop answer to a caller. ⭑ Dropping it is **worse** than blocking; a bound here changes the protocol, not just the timing. |
| **UNKNOWN** | the code does not say, and reading harder will not settle it. Say so — it is a finding, not a failure. |

## The 32, by file (re-measured 2026-09-17, `census-waiter-bounds.wat`)

```
wat/service.wat        17   serve loop + client methods + call-by-deadline's own send
wat/bracket.wat         8   map/reduce runners, collect-loop, map-worker
wat-scripts/…           4   circuit 2 (deadline-redial-is-fresh) · sqs 2 (park-receive!)
wat/spawn.wat           2   ⭑ the SEND halves of the handshake ab419aaa3 bounded on the RECV side
wat/test.wat            1   run-thread
```

⚠ **Start at `wat/spawn.wat:527` and `:597`.** `both-ends-bound-the-same-handshake` bounded the *recv*
half of that handshake and left the *send* half unbounded — bounding a wait and not the ship is half a
fence, and it is the one pair whose classification is close to predetermined.

## Shape to copy — and the defect that shape already found

`a-wait-that-should-be-bounded/FINDING-the-classification.md` is the recv twin: 23 sites, every row
citing a `file:line` **opened and read in context**, three named controls, and a totals table that
adds up. Copy it exactly, including:

- **§0-style instrument disclosure first.** That finding opened by reporting that the BRIEF's own grep
  saw 12 of 23 sites, because a trailing space excluded every site whose peer expression starts on the
  next line. Report what your instrument can and cannot see **before** the table.
- **Controls.** Name 3 sites whose class you can predict before reading, and report whether the
  reading agreed. A classifier that has never been shown to be wrong is not a classifier.
- **Totals that reconcile.** BOUND + PARK + REPLY + UNKNOWN must equal the live total, and that total
  must be **re-derived**, not quoted from this DESIGN.

## ⛔ THE INSTRUMENT TRAP THIS STONE HAS ALREADY BEEN BITTEN BY

Piece A was drawn for **4** blind sites and found **3**. Two of the four — `sqs.wat:2059`/`:2064` —
were labelled `NOT-IN-A-MATCH` by a balanced-paren reader and glossed as *"discard the outcome
entirely"*. **False.** Both are **arguments to `:user::send-ok!`**, which does match. The reader was
honest about what it saw; the gloss was wrong.

★ **So: a send whose outcome is consumed by a HELPER is not an unclassified send.** Follow the value.
The same applies to `send-keep-serving?` (`service.wat:4041`) and anything else that takes a
`SendOutcome` as a parameter.

## Out of scope — REJECTED

- **Choosing the mechanism.** That is the ruling this stone informs, and taking it here would make one
  stone do two jobs (`a-send-cannot-say-it-is-blocked` §THE RULING OWED).
- **Bounding anything.** No `.wat` changes. If a site is obviously wrong, report it; do not fix it.
- **The 184 non-live sites** (216 corpus-wide − 32 live). Tests and probes; say the number and move on.
- **`readln` (161), `accept` (8), `poll` (14 UNKNOWN).** Same invariant, different primitives, each
  needing its own classification.
