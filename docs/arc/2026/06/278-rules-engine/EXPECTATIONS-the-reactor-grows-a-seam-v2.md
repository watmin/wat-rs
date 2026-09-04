# EXPECTATIONS — the reactor grows a seam (v2)

Written **before** the strike.

| # | what | expected |
|---|---|---|
| 1 | ★★ the floor, **after** the edit | `5214/5214`. ⛔ Not run before — a green on the unextracted corpus proves nothing |
| 2 | ★★ the five sites are gone | `grep -n 'kernel::send'` → the helper, plus `1828`, `1939`, `1950`, `2006`, `2012`, **each named in the report** |
| 3 | ★ the circuit | `distinct=8000; dup=0`, five runs |
| 4 | the arms unchanged | `Sent`/`Closed` → true, `Stopped` → false, `Lost` → true |
| 5 | the probe still green | `SEAM-EXPRESSES` |
| 6 | scope | `wat/service.wat` only; no `src/`, no codemod, no drop |
| 7 | what R2 needs | stated, not built |

## ⛔ THE ORDERING IS PART OF THE STONE

Row 1 says *after the edit*, and that is not pedantry. V1's EXPECTATIONS called the floor "both
necessary and sufficient for faithfulness" without saying **when** — and the executor had to refuse
a green row that my own scorecard invited. A green before the edit is a scorecard entry for work that
did not happen, which is the most expensive kind because it reads as evidence.

## RUNTIME PREDICTION

**30–60 minutes.** One `defn`, five call sites, one file, no codemod, no stash. If this runs long,
something about the template's binder hygiene is unexpected and that is a finding.

## TRAP-DOOR RISKS

1. **Quasiquote hygiene.** The five sites are inside the macro's template; the helper call must be
   spliced, not captured. The file's 1422 comment lines are largely about exactly this.
2. **The peer expressions differ** — some sites take `peer` from an `Option`, others
   `(second (nth selectables idx))`. Both are `Peer :- [R O]`; if one is not, that is a finding.
3. **`1828` looks like the shape and is not.** `Stopped → true`. Sweeping it in changes behaviour.
4. **No stash needed** — Stone C established it: no new `fix.wat` verb, no chicken-and-egg. If you
   find yourself reaching for the stash dance, stop and ask why.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 1 run before the edit.
- Row 2 without the grep output, or with an exclusion unnamed.
- An excluded site swept in.
- Any arm disposition changed — including "simplifying" `Closed → true`.
- The drop built here.
