# EXPECTATIONS — the poison tears

**Written BEFORE the strike**, from `5d038f9a9`.

## What this stone is graded on

A green floor is nearly free here — one arm in a `wat-scripts/` file. It is graded on **an equality that
was previously impossible** and on a run that previously could not finish.

| # | what | check | expected |
|---|---|---|---|
| 1 | ⭑⭑ `fires == hits`, both > 0 | circuit at a **firing** `disrupt-bp` | the two counters **equal and positive** on the phase line |
| 2 | ⭑⭑ a firing run COMPLETES | same run | `dup=0` and the expected `distinct`; **not** the FINDING's `arrived=0` stall |
| 3 | the arm no longer raises | read the diff | `Malformed` returns a marker; no `assertion-failed!` on that arm |
| 4 | `tore?` admits it | read the diff | the marker is in the predicate |
| 5 | the comment now matches the predicate | read `:387` | it no longer says only *"lost/closed"* |
| 6 | ⛔ `disrupt-fires` untouched | `git diff` | still counted on `hit?`, at the send |
| 7 | ⛔ `hit?` untouched | `git diff` | the roll is unchanged |
| 8 | the redial's own raise survives | read the diff | *"redial seen failed — peer is dead"* still raises |
| 9 | one arm only | `git diff --stat` | `circuit.wat` only; **no** other placeholder migrated |
| 10 | no stdlib/Rust | `git diff --stat -- src/ wat/` | **EMPTY** |
| 11 | unperturbed happy path | `2000 4 3 8192 true 1000` | `distinct=8000;dup=0` |
| 12 | shipped chaos line | `… 0 0 7 0 0 0 0 0 500` | exit 0 |
| 13 | floor | `./scripts/floor.sh` → **Summary line** | **5237 passed, 0 FAIL**, no `ARM.txt` |
| 14 | clippy | `-D warnings` | exit 0 |
| 15 | ⚠ the firing run's cost | its wall clock vs the zero-rate run | **reported**, not hidden — a redial per fire will show |

## ⭑ Row 1 is the stone, and it is an equality on purpose

`disrupt-hits` has been pinned at **0** across two sessions and two different explanations. An *equality*
with `disrupt-fires` is falsifiable in a way "hits > 0" is not: it says the counter tracks the mechanism
exactly, with no third path quietly tearing connections. **If the pair disagrees, I want the disagreement,
not a widened predicate** (STOP-4).

## ⛔ What I will reject

- **`disrupt-fires` or `hit?` adjusted** to make row 1 come out. Rows 6–7. That would destroy the
  separation that exposed this bug in the first place.
- **A second placeholder migrated** to get a firing run to complete. Row 9 / STOP-3. If another arm trips,
  that is a *finding* — the remaining 60 are the builder's to sequence.
- **`tore?` widened beyond `malformed`** to force the equality. STOP-4.
- **A firing run reported green without its counters.** The phase line, verbatim, or it did not happen.

## Runtime prediction

**20–35 minutes.** One arm, one predicate, one comment; two circuit runs plus floor ~490–520 s. The
firing run may be noticeably slower than the zero-rate one — that is expected and is row 15.

## Trap-doors, ranked

1. ⛔ **Forcing the equality** by touching `fires`/`hit?`/`tore?` — rows 6, 7, STOP-4.
2. ⛔ **Migrating a second arm** to get past a stall — row 9, STOP-3.
3. **`points'` starts recording** — correct and expected; not a regression.
4. **A redial per fire** at a high rate — slower run, row 15.
5. **Reading `Malformed` as "not a tear"** because the service survived. The *service* lives; the
   *connection* is dead (`a-again=Closed`). STOP-1 exists to catch this being wrong.
