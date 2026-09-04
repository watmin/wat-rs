# EXPECTATIONS — the reactor grows a seam (v3)

Written **before** the strike.

| # | what | expected |
|---|---|---|
| 1 | ★★ **the four bijection goldens stay GREEN** | nothing above line 896 moved. **This is the placement's proof and v2 had no such row** |
| 2 | ★★ **the floor, after the edit** | `5214/5214`. ⛔ Never run before the edit |
| 3 | ★ the five sites are gone | `grep -n 'kernel::send' wat/service.wat` → **15 lines**, each classified: helper, five exclusions, nine never-candidates |
| 4 | ★ the circuit | `distinct=8000; dup=0`, five runs |
| 5 | both probes still green | `FORWARD-REFERENCE-OK` · `SEAM-EXPRESSES` |
| 6 | the arms unchanged | `Sent`/`Closed` → true, `Stopped` → false, `Lost` → true |
| 7 | scope | `wat/service.wat` only — no `src/`, no `.edn`, no codemod, no drop |
| 8 | what R2 needs | stated, not built |

## ⛔ ROWS 1 AND 2 ARE ONE PAIR AND NEITHER COUNTS ALONE

This is what v2 got wrong, and it cost a strike.

- **The floor** answers *did behaviour change?* — 5210 non-golden tests, every one expanding through
  this macro.
- **The goldens** answer *did anything shift?* — four tests that snapshot `service.wat` line numbers
  and go red if any line above 896 moves.

My v2 contract called the floor *"necessary and sufficient."* It is **necessary and not sufficient**:
it conflates behaviour with position, and any insertion above 896 reds it regardless of correctness.
**Having one instrument was having none.**

⛔ **If a golden goes red, do not patch the `.edn`.** That was v2's correct refusal — *"patching
goldens after STOP-4 is the improvisation v1 refused on `:- [R O]`."* A red golden means the helper
landed in the wrong place, which is the row doing its job.

## RUNTIME PREDICTION

**20–45 minutes.** v2's extraction is already correct except for placement; this is a move plus a
re-run. If it runs long, the forward reference is behaving differently inside `defservice` than in
the probe — and that is a finding, not a slog.

## TRAP-DOOR RISKS

1. **The helper must go after the macro's closing paren**, not merely "near the bottom." Anything
   above 896 shifts the goldens.
2. **Quasiquote hygiene** — the call is spliced into the template, not captured.
3. **The peer expressions differ** across the five sites — some from an `Option`, some
   `(second (nth selectables idx))`. Both should be `Peer :- [R O]`; if one is not, that is a finding.
4. **`1828` looks like the shape and is not** (`Stopped → true`).
5. **v2's working tree may still hold the extraction.** Reuse it; move the `defn`.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 1 not reported, or a golden patched to make it pass.
- Row 2 run before the edit.
- Row 3 without the grep output, or with an exclusion unnamed.
- Any arm disposition changed.
- The drop built here.
