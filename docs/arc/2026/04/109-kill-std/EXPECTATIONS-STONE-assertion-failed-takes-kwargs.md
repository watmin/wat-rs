# EXPECTATIONS — `assertion-failed!` takes kwargs

Written BEFORE the strike, on a tree that could not run a probe. **Row 0 is the precondition.**

| # | what | command | expected |
|---|---|---|---|
| 0 | ⛔ a RED probe exists and was verified on a GREEN tree | the SCORE | written as the strike's FIRST act; red for the right reason (kwargs form refused today), then un-ignored |
| 1 | the kwargs form is accepted | the probe | `(assertion-failed! :message "m")` runs |
| 2 | ⛔ the two Nones are GONE, not rewritten | `grep -c 'assertion-failed![^)]*:wat::core::None :wat::core::None'` over the corpus | **0** — and NOT because they became `:actual :None :expected :None` (STOP-4); spot-check the diff |
| 3 | ⛔ the positional form is REFUSED | a fixture in the old 3-positional form | non-zero exit. One convention, not two (STOP-3) |
| 4 | the corpus moved BY CODEMOD | a committed `wat-scripts/fixes/*.wat` + a `/tmp` dry-run diff | exists; no hand-edited `.wat` call site |
| 5 | the None population fell by ~2532 | `grep -c ':wat::core::None'` before/after | a drop of roughly 2532 from 5784. **Report the ACTUAL delta** — the arm reland is also consuming None in arm position, so the two numbers interact and a clean subtraction is not assumed |
| 6 | the floor | `./scripts/floor.sh` unpiped, Summary line | `0 failed` |
| 7 | clippy | `cargo clippy --release --all-targets --workspace` | 0 errors |

## RUNTIME PREDICTION

**60–90 min.** The macro + primed primitive is the arc-294 kwargs shape and is well-trodden; 442
files by codemod is the bulk; the probe is 10 minutes and comes first.

## TRAP DOORS

- **Row 2 is the stone and row 5 cannot substitute for it.** A migration that emits
  `:actual :None :expected :None` satisfies "the positional form is gone" and keeps every one of the
  2532 Nones. The point was never the calling convention alone — it was that a plain failure stops
  carrying two unreadable positional `:None`s.
- **Row 5's arithmetic is NOT clean.** The match-arm reland consumes `:wat::core::None` in ARM
  position at the same time. Report the measured delta, not `5784 - 2532`.
- **The NOTE's line numbers drifted 7 weeks** (`assertion.rs:107`→`:127`, `check.rs:16583`→`:17239`).
  Anything else it cites is equally due for re-derivation.
- **`(assertion-failed! "msg" a e)` with REAL actual/expected exists** — roughly half of the 2670
  calls do not end in two Nones. Those must gain `:actual`/`:expected` keys, not be dropped.
