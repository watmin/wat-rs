# SCORE — the `:peers` bijection keeps its negative controls

Brief + Expectations: `BRIEF-STONE-…` / `EXPECTATIONS-STONE-the-peers-bijection-keeps-its-negative-controls.md`.
Scored against my own re-run of every load-bearing row.

## Mode A. Every row passed, and the one gap in the scorecard was mine.

| # | row | my result |
|---|---|---|
| 1 | five tests registered | five names under `binary_id(wat::services)`; `build.rs` picked them up, no registration edit |
| 2 | scoped suite green | **133/133**, 2 skipped (128 + 5) |
| 3 | case 3 accepts the `:-` form | PASS |
| 4 | cases 1·2·4·5 reject | PASS |
| 5 | ★★ case 5 names the surface | golden contains `probe::Echo` **twice**; the `contains` assertion runs BEFORE the golden compare |
| 6 | ★★ perturb-to-prove | **both checks, both directions — see below** |
| 7 | `wat/service.wat` untouched | `git diff --stat` **empty** after the rider, and again after my own perturbation |
| 8 | nothing under `wat-scripts/` | 0 |
| 9 | no pre-existing golden rewritten | 0 modified files; all ten paths are new |
| 10 | floor | **4859/4859**, 0 FAIL, 19 skipped |
| 11 | clippy | 0 under `-D warnings` |

Row 10 landed exactly on the predicted arithmetic (4854 + 5 = 4859), so the count needed no
explaining — which is the only reason it is worth writing down.

## Row 6 — and the gap the EXPECTATIONS did not catch

The rider stubbed BIJECTION check 2 ("extra") to succeed unconditionally:

```
RED    5 tests run: 3 passed, 2 failed   — cases 2 and 5, the two that exercise "extra"
GREEN  133/133 after revert; git diff wat/service.wat empty
```

Cases 1, 3 and 4 stayed green, exactly as the mechanism predicts — they run the *other* check.

★ **Which is precisely the hole.** My brief said *"make one bijection check succeed
unconditionally"*, and my EXPECTATIONS row 6 accepted that as sufficient. It is not: with only the
"extra" check stubbed, **cases 1 and 4 were never shown able to fail at all.** Two of the four
negative controls had no red-evidence, and the row that exists to prove the suite can go red would
have signed off on a suite half of which might have been decoration.

I ran the complement myself — stubbed check 1 ("missing") instead:

```
RED    5 tests run: 3 passed, 2 failed   — cases 1 and 4, the two that exercise "missing"
       (cases 2, 3, 5 green — the complement, exactly)
GREEN  5/5 after revert; git diff wat/service.wat empty
```

Together the two perturbations are a **2×2**: each bijection check × each spelling, and every one of
the four negative controls is now proven able to fail **for its own mechanism**, not merely to fail.
Neither run alone establishes that; the pair does.

★ **The rule this adds:** a perturb-to-prove row must name **one perturbation per mechanism the
suite claims to cover**, not "one perturbation". "Can this suite go red?" and "can each test in it go
red for the reason its name gives?" are different questions, and only the second is worth the tests.
Counting mechanisms is the same discipline as `[[feedback_a_totality_claim_is_only_as_good_as_its_sampling]]`
— enumerate MECHANISMS, not shapes — arriving at a scorecard row instead of a probe.

## What I checked myself rather than credited

- `git status` — ten new paths, **zero modified**. `UPDATE_EDN=1` did not re-pretty-print a
  bystander golden this time; the rider scoped its filter as the brief required.
- **Case 5's fixture, read directly.** `grep -c ":peers"` on it returns **1**, which reads like the
  clause is still there. It is a header comment on line 5. The `defservice` genuinely has no `:peers`
  clause, and its ephemeral field genuinely carries the form spelling:
  `echo <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])`. Grep counted prose as
  code, in the one file where that would have invalidated the stone.
  `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`
- The driver's header cited a `STOP-STONE-…` path that does not exist (the brief is `BRIEF-STONE-…`).
  Fixed in this commit — a wrong file reference in a doc comment costs the next reader a search.

## Honest deltas

The rider reported *"nothing surprised me — the BRIEF's destinations were exact, down to the
`wat/service.wat` span line numbers."* That is true and it is the point: every destination had been
run before the brief was written, so the flight was mechanical, 5m52s against a predicted 25-40 min.
**The cost of that accuracy was paid yesterday**, by the stone whose evidence I ordered deleted.

One addition beyond the exemplar, and the rider justified it correctly: case 5 carries a bare
`msg.contains("probe::Echo")` in front of `assert_edn_matches_file!`. The golden alone would prove
data-equality but would not say *which* property is load-bearing, so a future regression would fail
as an opaque diff rather than at a named assertion.

## What this stone closes

`SCORE-STONE-defservice-compares-types-as-data.md` recorded that the bijection's rejection evidence
existed only in prose. It no longer does. Four goldens and a positive control now re-run on every
floor, and the 2×2 above is on disk as the reason to trust them.

---

## ⛔ THE FLOOR WENT RED — and my own wrapper reported it as green

Row 10 above is the SECOND floor. The first one was red, and I nearly did not find out.

**The failure, verbatim from `.floor/…/ARM.txt`:**

```
     Summary [  77.449s] 4859 tests run: 4858 passed, 1 failed, 19 skipped
        FAIL [   0.071s] (  60/4859) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert

    thread 'no_loose_string_assert::tests_carry_no_loose_string_assert' panicked at
    tests/lint/no_loose_string_assert.rs:112:5:

    🔥🔥🔥 LOOSE STRING ASSERTIONS — 1 site(s) assert a value with contains/starts_with/
    ends_with where an exact `assert_eq!` belongs. A loose check passes on reordered fields,
    malformed maps, and appended garbage.
    …
    Drive it to ZERO. Offenders:

    tests/services/probe_arc278_peers_bijection.rs:115
```

**That line is the one my BRIEF ordered written.** *"Its assertion must check that the diagnostic
literally contains `probe::Echo`."* The substrate has a wall against exactly that idiom, the lint was
at **zero** offenders, and this stone made it one — a real regression, not an inherited count.

### The instrument defect that hid it

I launched the floor as `systemd-run … scripts/floor.sh > run.txt 2>&1; echo "FLOOR EXIT=$?"`. The
harness reported **"completed (exit code 0)"** because `run_in_background` reports the LAST command's
exit — and the last command was the `echo`, which cannot fail.

The house rule I was following says *never a piped exit code*. **A `;` does the same thing and does
not look like it does.** And the sharpest part: the echo PRINTED THE TRUE VALUE — `FLOOR EXIT=100`
was in the captured file the whole time, and `floor.sh` printed its own `⛔ RED — exit=100` banner
under a status line that said green. I built the reporting wrapper that destroyed the signal it
existed to report. I caught it only by tailing the log for an unrelated reason.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

⚠ **And note which run caught it: the CENTRAL floor.** The rider's scoped
`binary_id(wat::services)` was 133/133 green, because the lint lives in `wat::lint`. The seam's own
alarm — *A RIDER'S SCOPED RUN IS NOT THE FLOOR* — firing again, in the stone that exists to make
evidence permanent.

### The disposition — the exemption is EARNED, and deleting the line is not the cheap fix

The lint offers `// rune:lint(loose-assert) — <reason>`, used at 136 sites. Two candidates:

**Delete the `contains` and rely on the golden alone.** Obvious YES. Simple YES. **Honest NO** — and
that decides it. The golden is **CAPTURED by `UPDATE_EDN=1`, not authored**: it records whatever the
macro emitted at capture time. Had the structural reader returned an empty surface, the capture would
have recorded THAT message and the test would pass forever, green, asserting nothing about the
property its own name claims. Worse, re-capture is a documented workflow — a future regression plus a
re-capture turns the golden green silently. **That one line is the only assertion in the file
`UPDATE_EDN=1` cannot rewrite.**

**Keep it with the rune and a real reason.** Obvious YES (136 precedents). Simple YES (one comment).
Honest YES — a targeted PRESENCE over a large structured output, which is the mirror of the
exemption's own documented shape, and the golden still does the structure-exact compare beside it.
Good UX YES — a regression fails at a named assertion saying what broke, not as an opaque EDN diff.

Taken, and the reason is written at the site rather than here, because
`[[feedback_an_exemption_is_earned_when_the_alternative_is_worse]]` — earned when the alternative is
worse, unearned when it is merely unfinished. Verified the rune actually satisfies the lint
(`test(no_loose_string_assert)` → 1 passed) rather than assuming its placement was recognized.

★ **The lesson for the brief, not the test: a brief that mandates an IDIOM must check the idiom
against the repo's own walls first.** I specified `contains` because it was the clearest way to state
the property. One `grep -rn "rune:lint(loose-assert)"` would have shown me both that the wall exists
and that the escape hatch is routine. The rider could not have caught it — its scoped run cannot see
`wat::lint`, and the brief told it what to write.
