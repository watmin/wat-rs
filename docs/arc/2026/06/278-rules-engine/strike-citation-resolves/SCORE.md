# SCORE — F1 rows 1+2, weighed against the orchestrator's own re-run

> Re-run here at `2c7200802`.

| # | pre-value | after |
|---|---|---|
| 1 | comments-included universe → **0 of 732**, self-vouching | ✅ **driven by me**: an invented name in a comment REDs, named with file:line |
| 2 | 33 unresolved | ✅ **33** — same set, though see thin spot A on how |
| 3 | *"Tests are `tests.rs`"*, file absent | ✅ fixed, and its class caught — the stale cluster was **27**, not 1 |
| 4 | `no_stale_path_in_doc` needs `/` | ✅ covered by a **sibling** gate, with the reason stated (STOP-3 respected) |
| 5 | two citations my strikes rotted | ✅ **declared, not renamed** — both sentences are *about* the old name |
| 6 | — | ⚠ **deviation, ruled: accepted** — see below |
| 7 | — | ✅ non-vacuity declared, and driven three ways |
| 8 | — | ✅ **every `src/` change is a comment** — 0 non-comment added lines, checked mechanically |
| 9 | lint 153/153 | ✅ **173/173** |
| 10 | floor 5267/5267 | ✅ `Summary [ 389.804s] 5287 tests run: 5287 passed (1 slow), 21 skipped`, zero FAIL rows |
| 11 | clippy rc=0 | ✅ rc=0 |

**Mutations 1 and 2, driven together by me** — one comment carrying both an invented name and a
test-only name:

```
🔥 1 name(s) cited in a comment under src/rete resolve to NOTHING …
   src/rete/alpha_tree.rs:2  `zzz_orchestrator_invented_name`
```

`spec_equals_native_on_every_where_family`, in the same line, is **not flagged**. The gate
discriminates; it is not merely strict.

## ⛔ Where MY brief was thin

- **A. ★★ MY OWN TRAP EXAMPLE WAS A ROTTED CITATION.** Trap 2 offered
  `axis_variant_names_round_trip` as the model of a legitimate test-only name. It resolves
  **nowhere** — the fn is `axis_variant_names_round_trip_through_one_door`. A rider trusting it would
  have widened the universe until a real finding disappeared: **exactly the STOP-1 failure the same
  trap warned against, seeded by that trap's own example.** The rider found the genuine controls
  (`spec_equals_native_on_every_where_family`, `alpha_class_lookup_…`) and made them **live
  assertions** rather than sentences. Promoted to memory.
- **B. ★ The gate's own file is in its own universe, and both universe controls RED-ed naming their
  author.** `NoMatchingArm` and `SiftRulesResponse` resolved against the gate's **own error text** on
  the first run. Without the `SELF` exclusion, a future hand silences any red by adding the offending
  name to a failure message. My ⛔ warned that prose must not vouch for prose; **this is that defect
  one level up**, and it is the thing the brief most needed and did not have.
- **C. The agreement on 33 rests on a shape rule I never stated.** Any bare identifier gives **47** —
  the extra 14 are seven git SHAs and seven Latin session names (`partire`, `probare`, `solvere`…).
  Requiring an `_` or an interior capital reproduces 33 exactly. Stated in the gate as a **boundary**:
  a one-word `fn` cited in backticks is unchecked.
- **D. My universe was short in two directions** — `wat/` (where `SiftRulesResponse` actually lives)
  and **file stems** (this tree cites gates and probe worlds by bare module name). Both are
  widenings; neither rescues a defect, and each is pinned by a control proving it decides something
  no other half can.
- **E. The obvious filename rule misses the finding that motivated it.** `tests.rs` still exists at
  `src/macros/tests.rs`, so basename-existence alone leaves `kernel/mod.rs:4` **green**.
  Ancestor-relative alone reports 55 with **31 false**. Only the conjunction is right — and the real
  cluster was `validate.rs`×9 and `expr_ir.rs`×6, from the same `partire` split that broke `tests.rs`.

## The deviation I was asked to weigh — ruled: ACCEPTED

Row 6 required three **named vocabularies** (clippy lints, memory slugs, `_`-prefixed). The rider
built none, and excluded by **spelling** instead: `clippy::needless_borrow`, `*_pass`,
`[[feedback_…]]` — forms the tree already uses, which I verified at `src/function/parse.rs:48`.

Its argument is the ladder and it is right: **a list keeps exempting a name after it stops being
noise; a spelling rule makes the correct form the only passing form, and the failure text teaches
it.** That is a rung above what I asked for.

## A design consequence, stated rather than discovered later

**This gate must produce more runes than its `wat-scripts` sibling, by construction.** 13 of the 33
name something whose absence *is* the point. The sibling escapes this by exempting comments; here
comments **are** the subject, so no reword both keeps the name and resolves. The rune is the only
branch available.

## The rider's own mistake, self-reported

It piped the first full-lint run through `tail -25` and lost the failing arm — the exact
truncating-pager failure `CLAUDE.md` names. The red was **its own**: a `"[[feedback_x_y]]"` literal
tripping `no_inlined_edn`. It re-captured whole before touching anything and fixed it by
restructuring the literal, which that lint's rubric demands over a rune — and noted that the arm was
recoverable only because the lint is a pure function of the tree, *"which is luck, not method."*

## Arms not driven, named

Citation hollow-rune at corpus level (proven by three classifier unit tests); filename hollow-rune;
filename non-vacuity; subject-walk non-vacuity; `the_gate_does_not_attest_its_own_text` as a mutation
— though **its exact mechanism fired for real** before the exclusion existed, which is stronger
evidence than a mutation.
