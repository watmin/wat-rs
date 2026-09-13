# EXPECTATIONS — the inert clause is refused

**Written BEFORE the strike**, from `9d4af896f`.

## What this stone is graded on

That an inert declaration becomes **unwritable**, that the corpus's one real use is **deleted rather than
migrated**, and that the latent risk is **filed rather than silently fixed**.

| # | what | check | expected |
|---|---|---|---|
| 1 | ⭑⭑ the refusal fires | a fixture declaring `:deadline-ms` in `:satisfies` mode | `#wat.macro/MalformedTemplate` naming the fqdn **and** `call-by-deadline` |
| 2 | ⭑ `circuit.wat` compiles | `--check` after the deletion | exit **0** |
| 3 | ⭑ the deletion is behaviour-preserving | the **measurement** is the proof (the clause governs nothing); the happy path is the confirmation | `distinct=8000;dup=0` — the publisher still completes its join |
| 4 | ⛔ the caller was NOT migrated | `git diff -- wat-scripts/fanout/circuit.wat` | **one line removed** (`:deadline-ms 120000`); no `call-by-deadline` added |
| 5 | ⭑ the latent risk is FILED | a new FINDING or NOTE | records the deleted comment's claim (~20 s stats join) and that it is **latent, not active** |
| 6 | ⛔ `:ops` mode untouched | read the guard | scoped to `:satisfies`; `:ops` + `:deadline-ms` still compiles, **or** stated as not cheaply testable |
| 7 | ⛔ the clause not deleted from the macro | `git diff -- wat/service.wat` | the guard added; the clause still read for `:ops` |
| 8 | probes' numbers carried | the SCORE | `10000 vs 300` from **both** probes, quoted |
| 9 | message is a sibling | compare to `:896`/`:913`/the third | same family, ends with the remedy |
| 10 | three corpus `--check`s | sqs / circuit / sns-fanout | exit 0 each |
| 11 | floor | Summary line | green; **state the count** (5244 today, may move) |
| 12 | clippy | `-D warnings` | 0 |
| 13 | no `src/` | `git diff --stat -- src/` | **EMPTY** |
| 14 | chaos | `… 0 0 7 0 0 0 0 0 500` | exit 0 |
| 15 | ⛔ exactly THREE sites trip | the SCORE | `circuit.wat:2083` **plus the two scratch-pad probes** (both declare the clause — they are this stone's own evidence and are expected). ⚠ A **fourth** site would mean my census was wrong; name it |

## ⭑ Rows 4 and 5 are the ones that make this honest

Row 4 is the **restraint**: the tempting move is to make the publisher's deadline actually work, and that is a
behaviour change this stone must not smuggle in. Row 5 is the **handoff**: the deleted comment encodes a real
worry (*"default 10000 would TimedOut mid-run"*), and deleting the line without recording the worry would lose
it. ⭑ **Deleting a comment that documents a risk, without filing the risk, is how a latent bug becomes
invisible.**

## ⚠ What I will reject

- **The publisher's caller migrated** (row 4 / STOP-1).
- **`:ops` mode refused** (row 6 / STOP-2) — 0 sites, unmeasured.
- **The clause removed from the macro** (row 7 / STOP-3).
- **The probes' numbers dropped** (row 8 / STOP-4). They are the evidence.
- **A FOURTH site tripping without being named** (row 15). Three are expected — `circuit.wat` and the two
  probes. A fourth would falsify my census and I want it said, not worked around.

## Runtime prediction

**40–70 minutes.** One guard, one deletion, one fixture, one FINDING; the long pole is the floor.

## Trap-doors, ranked

1. ⛔ **Migrating the caller** (row 4) — the tempting, wrong move.
2. ⛔ **Refusing `:ops`** (row 6) — a rule against no evidence.
3. **Losing the probes' numbers** (row 8).
4. **Deleting the comment without filing the risk** (row 5).
5. **Reporting a stale floor count** (row 11).
