# WEIGH — STONE 255.9: ACCEPTED. Two forged greens closed, and the brief was wrong three times.

Weighed against `4b6088454` by independent re-run. **Diff: ONE file** (`src/load/loader.rs`,
+167/−10, bulk = doc + 3 tests). `git status --porcelain -- '*.wat'` = **0**. Not pushed.

## Re-derived here

| claim | my result |
|---|---|
| ⭐ the cure works on the REAL sample file | `wat-scripts/fmt/run-all.wat`, copied without siblings: keyword `rc=1 "load: file not found: rules/defn.wat"` · converted **`rc=1`, byte-identical message**. Before the cure I measured the same shape at **rc=0**. |
| ⭐⭐ the SECOND forged green | `(wat.config/set-redef! true)` in a **loaded** file → **`SetterInLoadedFile`**, same as the keyword spelling. **Closed.** |
| ⛔ non-vacuity — the gate did NOT open | `(some.other/load-file! "missing.wat")` is **still not a load form** (falls through to `UnresolvedReferences`). Unrelated symbol forms `rc=0`. |
| test quality | exact `assert_eq!` on the whole message, plus a `panic!` arm that names the discrimination — *"must reach the FETCH, not be declined as 'not a load form'"*. ⭐ **It cannot pass by failing for the wrong reason.** |
| floor | RED **5961/1 failed** disclosed with `ARM.txt`, then **5962/5962 passed** (+3 = the new tests) |
| ⭐ **RECOVERY (fail→clean) = 0** | **the forged green is closed** — measured on the committed list |

## ⭐⭐ THE CENSUS IS THE STONE, AND IT IS PREDICTIVE

The rider replaced my grep with a **pipeline-order** classifier: `normalize_symbol_refs` is **step 7**;
anything at or after it in a code position can never see a symbol head, and **data positions are never
normalized at all**.

⭐ **I verified the mechanism and it explains both defects rather than merely covering them:** load
resolution is **step 3**, the config pass is **step 2** — the only head-dispatching steps *before*
normalization. **Both forged greens were there, and the census found both.** A bounded class with
both members named is worth more than a list of 83 greps.

⚠ **And the rider named its own limit:** every UNREACHABLE row is *"a pipeline-order argument, not a
probe — re-order any pass relative to step 7 and they all go live silently, with no gate on that
ordering."* ⛔ **There is no gate on the ordering this census depends on.** That is a real, disclosed
hole and it should become a stone.

## ⛔⛔ THE BRIEF WAS WRONG THREE TIMES — and the first one is mine twice over

**1. "All 4 load-form files in the 179 sample are CLEAN → CLEAN."** ⛔ **FALSE.** The committed sample
contains **exactly ONE** (`wat-scripts/fmt/run-all.wat`), and it is **BROKEN → CLEAN** — it *is* the
forged green. Count and direction both wrong.

⛔⛔ **I got it wrong because I measured on my own index-built reconstruction — in the very brief where
I wrote "USE THE FILE. NEVER REBUILD BY INDEX."** The rule and its violation are in the same document.

**2. "The class is a form's HEAD."** ⛔ The codemod rewrites **every `::`-keyword leaf, position-
independently**. Slot-1 names, match-arm patterns, rete fact-type keywords and `:wat::verify::`
markers all convert. **Three of the four highest-impact blockers the rider found are not head sites.**

**3. My "~14 within three lines" proximity probe** had **3 false positives** and ⛔ **missed
`scan_for_setter` — the second real defect, four lines from its decline, in the file my own brief
names.** `[[GREP IS NOT A CENSUS]]`, demonstrated on me.

## ⭐ AND IT CAUGHT ME A FOURTH TIME, ON THE SAME FILE

My independent delta returned **19**, not 18. The difference is **my instrument, not the tree**: I
checked originals **in-tree** and converted copies **out of tree**. `run-all.wat` is `rc=0` in-tree and
`rc=1` as a copy, because it needs a sibling — so my asymmetry manufactured one phantom regression.
**The rider checked both sides from copies (160 clean / 18 new) and is correct.**
`[[feedback_diff_like_against_like]]`. ⭐ **One file, `run-all.wat`, caught me twice in one session.**

## ⭐⭐ THE INSTRUMENT FINDING — the RECOVERY column is the gate for this whole class

The delta counts **clean → fail**. A forged green is **fail → clean**: invisible, or worse, read as an
improvement. ⛔ **I had this column in my own script and printed `closed: 0` in the 255.8 weigh without
recognising it as the gate that would have caught a green-forging conversion.** It is one line of a
join that already exists.

⭐ **RECOMMENDATION: RECOVERY becomes a standing, reported column of every delta, and any non-zero
recovery is a STOP until explained.** A file that goes from broken to clean under a pure spelling
change is either a real fix or a disarmed checker, and the arc has now met the second kind.

## Open — needing the builder, not a rider

⛔ **`types/surface.rs:1003`** — after conversion the post-`:-` child is no longer a `Keyword`, so
`collect_message_form_type_refs` silently stops collecting and `collect_user_type_paths` narrows. The
rider **named it and declined to guess** what the pass is load-bearing for. ⭐ **Correct call.**
⚠ I tried to test whether this is the same class as the arc's standing `defsurface :messages` residue
(8 files) and **my probe could not discriminate** — the file I picked already fails. **Hypothesis
UNTESTED; I am not asserting it.**

Also reported, not cured, each with a reason: the `:wat::verify::*` markers (LOUD, and **0 of the 26
corpus load files use a verified-load form** — measured); the two mutation walls (BYPASSED but
BACKSTOPPED — *the refusal survives, the diagnostic does not*; **widening a security wall's input is
the builder's call, and the rider says so**); `runtime.rs`'s session/`run()` paths (**unmeasured, and
declared unmeasured**).

## VERDICT

**ACCEPTED.** Pushing. ⭐ **This stone did what it was sequenced first to do: a conversion can no
longer forge a green, so the next three stones' numbers are trustworthy in a way the last eight
were not.**

Queue unchanged, minus this one: `type_denotation` at the two runtime string-table sites · the
416-test rete remainder · 255.8's wrong-join acceptance · then 8d-ii (fourth draw) · then 8d-iii.
**Add:** the RECOVERY column as a standing gate, and a gate on the step-7 pipeline ordering.
