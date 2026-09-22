# WEIGH — STONE 255.8: ACCEPTED, with a finding that gates 8d-iii

Weighed against `9dbb016e8` by independent re-run, not by reading the SCORE.
Delta **49 → 18** (grok) / **→ 16** (mine, see the sample note). ⭐ **The largest drop of the arc.**

## Re-derived here, not inherited

| claim | my result |
|---|---|
| `reconstruct_call_path` body unchanged | ⭐ **byte-identical** to `0d6da5f19`. The claim is exact. |
| the original defect is cured | `(wat.core/defrecord my.Journal/Req …)` + annotation → **CLEAN**. I had measured `:path ":my::Journal/Req"` on this shape before the stone. |
| the colon spelling still works | `:my::Journal::Req` → **CLEAN** |
| ⛔ the member join is NOT undone | `wat.core.Option/expect` **and** `:wat::core::Option/expect` → **CLEAN** both |
| the new probe | **PASS**, and **non-vacuous**: it declares `my/Journal` as a real type, so the ambiguity is genuinely present |
| floor | **5959 passed / 22 skipped**, exit 0 |
| clippy | `--all-targets --workspace -D warnings` → **0** |
| census | `no STOP-8` |
| `wat/` untouched | `git status --porcelain -- wat/` → **0 lines** |

## ⭐⭐ THE RED FLOOR IS THE BEST ARTIFACT IN THIS STONE

The SCORE discloses a red floor earlier the same day: **66 failed**. I checked, and the doctrine was
followed exactly — `.floor/2026-09-21T23-21-45Z/` carries `ARM.txt`, `clean.log`, `raw.log`, the
whole run, **not re-run to make it go green**.

⭐⭐ **And look at WHAT went red:** `one_variant_separator::only_identifier_rs_spells_the_variant_separator`
and `one_name_grammar::only_identifier_rs_parses_a_name` — **the ONE DOOR lints.** The first attempt
added a second name parser and **the gate caught it**.

⭐ **`tests/lint/` is untouched in the whole diff.** The lints went red, then green, **with the gates
unmodified**. The code was fixed, not the gate. That is the single strongest signal in the stone —
`[[feedback_a_green_test_can_prove_nothing]]`'s converse: a gate that has now failed once is a gate.

The third red was `arc109_two_iii_defclause_return_slot::row3_defclause_still_rejects_non_type_return_slot`
— **a negative control**, and it survives explicitly: `!id.is_reference() && !is_type_param_letter(&te)`
keeps bare `n` rejected. ⭐ **The widening did not eat its own control.**

## ⛔ FINDING — the wrong join is now ACCEPTED for a SYMBOL author

The SCORE's "what a door cannot express" names three registry families. It does **not** state the
user-facing consequence, which I measured:

```
(:wat::core::Option::expect …)   keyword author  → REFUSED ✅
(wat.core.Option.expect …)       symbol author   → ⛔ ACCEPTED
```

`wat.core.Option.expect` has **no slash at all**. `ns_to_wat_path` makes `:wat::core::Option::expect`;
`other_join_spelling` flips it to `Option/expect`; the registry holds that, so it resolves.

⛔ **Today this is contained** — the corpus is keyword-spelled and a keyword author never reaches the
fallback. ⛔⛔ **8d-iii is what un-contains it.** After 2,076 files become symbols, a call head with
the wrong join **silently resolves** where it used to raise `UnresolvedReference`. The campaign's
whole premise is one canonical spelling; this admits two.

**This does not block 8d-ii** (the stdlib is 64 files, converted and proven to load). **It gates
8d-iii**, which is exactly the stone that makes it reachable.

⚠ Related, same function: `resolve/normalize.rs` consults `other_join_spelling` **twice**, at `:708`
and `:719`, under **different authorities** (`name_has_binding` vs `name_is_registered`). Registry-
arbitrated, not a shape guess — but two consults of one question is the shape this arc keeps paying
for. **Name the rule once.**

## ⚠ FINDING — the arc's headline metric is not pinned to a committed list

I reconstructed the sample as *every 12th non-stdlib tracked `.wat`* and got **179 files — the right
count — but 164 originals clean, where the SCORE says 161.** No offset I tried reproduces 161.

⛔ **`104 → 97 → 80 → 77 → 66 → 61 → 49 → 18` is a trend line across samples that are rebuilt by hand
every stone.** The conclusion survives — my own run independently shows **16 new failures against 49**,
so the direction and magnitude are real — but the arc has been comparing slightly different
populations for eight stones. **Commit the list.**

## The one opened regression is REAL — I tried to refute it and failed

`wat/holon/Ngram.wat`: original **CLEAN**, converted **`DuplicateMacro :wat::holon::Ngram`**.

I suspected the stdlib-copy artifact (the trap the 8d-ii redraw already documented) and ran the
control — **an UNCONVERTED copy at the same out-of-tree path: rc=0.** The artifact hypothesis is
refuted by measurement, not argued away. `macro_structurally_equivalent` now compares names by
`canonical_identity`, including a `Keyword`↔`Symbol` arm, but falls through to exact `a == b`
otherwise; Ngram is the residue. **Disclosed, 1 against 31 closed. Accepted as residue.**

## ⚠ The diff is 2,889 lines and its own formatting hid it

`cargo fmt` ran over the touched files (`collection/eval.rs` 1219 violations → 1, `runtime.rs`
351 → 1, `macros/tests.rs` 223 → 1). Formatting **both** sides and re-diffing gives the true figure:
**~2,251 semantic lines across 24 files** — `collection/eval.rs` is **58**, not 958.

⛔ **A reflow of a file you are also changing makes the change unreviewable by default.** It cost me
four instruments to get an honest number. **Reformat in its own commit, or not at all.**

Against a brief that scoped *one door*, ~2,251 semantic lines is a large stone — but the SCORE says
so itself, under **"Fifteenth correction — the brief named the annotation"**, and it is right: the
annotation slot was never the bug. **The brief's own hint was the correct one** — *"if a type
annotation is reaching a function named for CALL paths, the bug may be the ROUTING"* — and the answer
was that it does not reach it at all. ⭐ **Sixteen corrections across thirteen stones.**

## My own error this session

Reporting `wat/holon/Ngram.wat`'s original as FAIL. A `grep -c` returned 0 and **exited 1**,
short-circuiting the `&&` chain, so `wat --check` **never ran** and `|| echo FAIL` fired on the broken
chain. I nearly filed a false accusation against a correct SCORE.
`[[feedback_a_label_that_states_a_conclusion_must_be_computed]]`, and the second time this arc that
an exit code I did not own has spoken for a measurement I did.

## VERDICT

**ACCEPTED.** Pushing. Then:

1. **251.8d-ii re-runs** — its conversion was always proven; the load now works.
2. ⛔ **A stone for the wrong-join acceptance BEFORE 8d-iii.** Pin the registry rows that legitimately
   hold a non-canonical join to a **list**, and refuse every other flip.
3. Commit the 179-file delta list so the ninth measurement means the same as the first.
