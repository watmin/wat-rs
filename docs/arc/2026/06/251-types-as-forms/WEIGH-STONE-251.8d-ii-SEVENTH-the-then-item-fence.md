# WEIGH — 251.8d-ii (SEVENTH): cures ACCEPTED, conversion STOP. ⭐ **161 → 112, 0 NEW.**
# ⛔⛔⛔ And my "measured" address came from a probe that COULD NOT RETURN PASS.

Weighed against `c39196dfd` by independent re-run. Tree clean, `wat/` restored, nothing pushed.
Trajectory: **3 747 → 416 → 283 → 161 → 112.**

## ⛔⛔⛔ THE WORST INSTRUMENT ERROR OF THIS ARC, AND IT IS MINE

The brief claimed its address was **measured, not inferred** — *"isolated in ~90 seconds."*
⭐ **The rider ran my own two-row table first. It does not reproduce.** I replicated that here, with
pre-cure `src/` and `wat/rete/compile.wat` converted alone:

```
both_fact_form_shapes_expand_their_values   →   PASS
```

**Then I found why my probe disagreed.** It classified on:

```bash
r=$(cargo nextest run … | grep -cE '^\s+PASS')      # ← matches 0 times, ALWAYS
```

⛔⛔ **nextest's PASS line begins with `^[[32;1m` — an ANSI escape, not whitespace.** Verified here:
`grep -cE '^\s+PASS'` returns **0** against real output.

⛔⛔⛔ **MY PROBE WAS INCAPABLE OF RETURNING "PASS". It would have convicted the FIRST FILE I TESTED,
whatever that file was.** `[[feedback_a_label_that_states_a_conclusion_must_be_computed]]` — a **RED**
from a mis-aimed probe, which is the inverse of the failure this arc already has a memory for.

⛔ **And I knew.** My own working FIXED/NEW extractor opens with `sed 's/\x1b\[[0-9;]*m//g'`, and
`scripts/floor.sh` exists partly to keep an ANSI-stripped log. **I had the fix in my hands and did
not apply it to the one probe I then called measured.**

⭐⭐ **This is strictly worse than corrections #22–#24.** Those were reasoning failures presented as
reasoning. **This was a broken instrument presented as a measurement** — and the fourth consecutive
wrong address. **The lesson is not "check the regex": it is that a probe with an unreachable negative
branch is not a probe, and I must prove a probe CAN say both words before trusting either.**

## ⭐⭐ THE REAL NARROWING — ONE TOKEN, FOUR BINARIES

The method was right; only my matcher was broken. The rider used it properly and got to a **single
token**:

| `compile.wat:579-580` | `binary(rete)` |
|---|---|
| fully converted | ⛔ 514 / **8 failed** |
| both markers keyword | ✅ 522 / 0 |
| `quasiquote` symbol, `unquote` keyword | ✅ 522 / 0 |
| `quasiquote` keyword, **`unquote` symbol** | ⛔ 514 / **8 failed** |

⭐ **The discriminator is `unquote` alone.** `runtime.rs::match_qq_head` read the marker as a
`Keyword` payload only, the unquote never fired, the template kept `(wat.core/unquote acc-hd)`
verbatim, and the fence-call's head became a **LIST**.

⛔ **And the fence was the ACCUMULATOR's (line 609, `is-pure`), not the `:then` item's (795,
`is-rete`).** `primitive?` never decides. **My brief was wrong about the fence, the line, and the
mechanism** — while being right that `compile.wat` was involved, for reasons I had not established.

## ⭐ THE ERROR MESSAGE LIED AND THREE BRIEFS BELIEVED IT

`':wat::core::kwargs-construct' is not a rete primitive` names **a spelling no source file contains.**
When the door declines, `classify_expr` re-spells the Symbol head through `canonical_identity`
*before building the violation*. ⛔ **My "141 occurrences naming `:wat::core::kwargs-construct`"
counted the ERROR's spelling, not the source's** — and 141 was over `ARM.txt`, which repeats
(116 over `clean.log`). **Two counting errors in one line, in a brief warning the rider about exactly
that trap.**

## ⭐ QUESTION ONE ANSWERED — AND MY EXPECTATION WAS THE WRONG ONE

`primitive?` **is** spelling-sensitive, at exactly two places (the declaration-derived construction
door — verb **and** type — and `classify_expr`'s core-structural guard). ⛔ **But my brief called
`(:wat::core::+ 1 2)` returning `false` "one that should have passed." It should NOT.** Law A must
refuse core-spelled computation; `false` is correct in both spellings. ⭐ **My argument-shape
*diagnosis* was right and is what made the re-probe possible — the expectation attached to it was
not.**

## Re-derived here

| | mine |
|---|---|
| converted floor | **112 / 6011** |
| ⭐ **FIXED / NEW** vs the sixth draw | ⭐⭐ **49 / 0 — STRICT SUBSET** |
| the fence class | ⭐ **117 → 0** — GONE |
| landable floor | ⭐ **6011 / 6011 GREEN**, twice — the second on the **committed** tree |
| ledger | **223 → 220**, 4/4 pass |

⭐ **The shape-B anchor was deliberately NOT cured** — `walk_rete_defn_callees` stands, assertion
intact. **Exactly what the brief asked for, and the one hard thing it got right.**

## ⭐ THE 5 NEW — ISOLATED, NOT ASSUMED

All five pass, and it proved the mechanism three ways rather than claiming it: a **two-binary
experiment on the converted tree** (revert its two rete doors, keep `match_qq_head` → **all 5 fail
again**; restore → all 5 pass), `git diff --stat` showing the sixth draw's cure **untouched**, and
Law-A rows passing on the converted tree so **the wall is armed**. ⭐ **That is the "which mechanism
did it" the brief demanded, answered by experiment.**

## ⛔ A LEDGER BLIND SPOT THE LEDGER CANNOT SEE

`match_qq_head`'s comparison is `k == head` with **the literal at the CALL SITE** — ⛔ **so it has no
ledger row and cannot get one.** A revert is caught only by the stone's own tier-C probe.
**255.13 §2.3 does not name this shape.** ⭐ **That is a real gap in the countdown we are using to
schedule the terminal cut, and it should be a stone.**

## Their floor red — captured, and the same class as the stone

2 lint gates, both theirs, not re-run. The better offender:
`const NAMED: &str = "':wat::core::if'"` — ⭐ **`'` is the reader's QUOTE SUGAR**, so that literal
parses as a *form*, not a name. *"This stone is about a name that reads two ways, and my own fixture
wrote a FORM where it meant a NAME."*

## VERDICT

**CURES ACCEPTED, CONVERSION STOP.** clippy 0 (not cached, 12.93 s), census `no STOP-8` 213 = 213,
delta **3 / RECOVERY 0**, 0 live `.wat`. **Twenty-five corrections across twenty-three stones.**

**Eighth draw:** `UnresolvedReferences` — **33 of the 112, now the largest typed class, and unmoved
across two draws (33 → 33).** ⛔ **No rete-fence class remains.**
⛔ **And the address will be isolated with a probe I have first proven can say BOTH words.**
