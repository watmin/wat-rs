# SCORE — the malformed census

**SCORED.** Executor: grok, 2026-09-13, branch `sns-sqs`, HEAD `a6da4c256` (DRAWN). Did not commit.

```
     Summary [ 527.534s] 5239 tests run: 5239 passed (9 slow), 22 skipped
```

`.floor/2026-09-13T05-30-27Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**. Count **5239**.
Clippy: **CLIPPY=0**. `git diff --stat -- src/ wat/` **EMPTY**. `wat-scripts/{queue,topic,fanout}` **EMPTY**. Nothing migrated.

---

## ⭑ THE HEADLINE — 464 raising arms, classified; the filter is a column

```
CONTROL-A circuit.wat:544 raising-Malformed: ABSENT
CONTROL-A sns-fanout.wat:369 raising-Malformed: ABSENT
CONTROL-B wat/service.wat raising-Malformed: PRESENT
STOP-5 unclaimed let/do/if/macro body: ABSENT
form-tree keywords=482; Malformed match-arms=479; raising-arms=464; unclaimed-arms=15
```

Two runs, sorted path list (1753 files), `diff` empty.

| head \ home | wat | wat-scripts/service | scratch-pad | tests | docs | other |
|---|---|---|---|---|---|---|
| **defservice** | 6 | **18** | 3 | 8 | 0 | 4 |
| defn | 15 | 14 | 75 | 263 | 3 | 50 |
| **other** | **5** | 0 | 0 | 0 | 0 | 0 |
| defsurface / deftest | 0 | 0 | 0 | 0 | 0 | 0 |

Not resolved as "service items = 18." The five `other` rows are stdlib `defmacro`/`defclause`/`extend-type` (CONTROL B is inside `defmacro defservice`). Marked, not classified as service items.

All 464 raises are `assertion-failed!`. `raise!`/`panic!` recognised; 0 Malformed arms use them.

---

## Instrument correction

File-level CONTROL A would have failed a correct walker: both files still have other raising placeholders. CONTROL A is **line 544 / line 369** (the migrated poisoned-call arms, now unclaimed `raises=string`). A line-oriented pass flags those lines because a raising `Stopped` shares them.

---

## Gap (487 → 464)

DESIGN 487 (draw, `wat/`+`wat-scripts/`+`tests/`) + 3 docs + 3 this census program (comment + 2 strings) = **493** grep-o on the path list.
Walker keywords **482** (no comments, no string contents). Match-arms **479**. Raising **464**. Unclaimed **15** (migrated strings, probe labels, one pass-through, one recurse).

`wat-tests/` out of the path list (80 tokens). Stated.

---

## WHAT LANDED

- `wat-scripts/census-malformed-raising.wat` — report-only form walk
- `docs/excursus/2026/08/001-sns-sqs/the-malformed-census/FINDING-the-malformed-census.md`
- `docs/excursus/2026/08/001-sns-sqs/the-malformed-census/census-run.txt` — raw stdout

No `src/`, no `wat/`, no lint rule, no arm migrated.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | CONTROL A absent | circuit.wat:**544** ABSENT, sns-fanout.wat:**369** ABSENT (unclaimed `string`) |
| 2 | CONTROL B present | `wat/service.wat:3281` PRESENT; head=`other` (defmacro) |
| 3 | controls stated | printed in the census stdout |
| 4 | every row has head | no blanks; defservice 39 / defn 420 / other 5 |
| 5 | every row has home | no blanks |
| 6 | three raise forms | all three matched; 464 / 0 / 0 |
| 7 | form-walk | `ast->children`; CONTROL A is the proof against line-matching |
| 8 | 487 gap explained | FINDING §2 |
| 9 | nothing migrated | queue/topic/fanout **EMPTY** |
| 10 | no stdlib rule | `src/` `wat/` **EMPTY** |
| 11 | ambiguous marked | five `other` rows named with actual enclosing form; not resolved |
| 12 | run twice identical | `diff` empty |
| 13 | floor 5239 | Summary **5239 passed**, no ARM.txt |
| 14 | clippy | CLIPPY=0 |

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-13

**Graded against my OWN reads and runs**, from `a6da4c256` + the working tree.

```
floor   (my own run)  Summary [ 523.572s] 5239 tests run: 5239 passed, 22 skipped · 0 failure tokens · no ARM.txt
clippy  --release --workspace --all-targets -D warnings → exit 0
tree    ZERO modified files — new only. src/, wat/, wat-scripts/{queue,topic,fanout} diffs all EMPTY.
```

**STRUCK — and then I extended the corpus myself, because my own brief was short by four homes.**

## ⛔⛔ MY BRIEF NAMED FOUR `.wat` HOMES. THERE ARE EIGHT.

The SCORE states, honestly, that `wat-tests/` was out of the path list (80 tokens). I went to check whether
that mattered and found the real number:

```
tests 1017 · wat-scripts 667 · wat-tests 85 · wat 55 · docs 14 · examples 6 · crates 3 · benches 1
```

★★ **"Four homes" is a claim I have made all session** — in this stone's BRIEF, in `the-store-says-what-it-
deleted`'s DESIGN, and **in that codemod's path list.** I checked for a live hole: all four omitted homes have
**0** `DeleteResponse::Success`, so that migration was complete — **by luck, not by construction.**

## ⭑ So I re-ran the census over all eight homes myself

```
paths 1848 (sorted)   all four controls PASS
form-tree keywords=562 ; match-arms=559 ; raising-arms=544 ; unclaimed=15
```

| | grok, 4 homes | mine, 8 homes |
|---|---|---|
| raising arms | 464 | **544** (+80 — exactly the `wat-tests/` delta the SCORE predicted) |
| `deftest` rows | **0** | **64** |
| `defservice` total | 39 | 40 |
| ⭑ **`defservice × wat-scripts/service`** | **18** | **18 — unchanged** |

★★★ **The incompleteness was real and it did NOT move the deliverable.** The builder's filter target is stable
at **18**, and the 64 new rows are `deftest` — a head that answers the question by itself, since a test is
where raising is correct. **The census was incomplete without being misleading**, which is the distinction
worth recording: a short corpus is not automatically a wrong conclusion, but only a measurement can say so.

⚠ **One residual, small and named:** the `home` classifier has no arm for `wat-tests/`, so those rows land in
`home=other`. The **head** axis (`deftest`) rescues them, but a reader filtering on `home` would see
uninformative rows. One classifier arm; the builder's to sequence.

## ★ The instrument correction the strike made, which my brief had wrong

I wrote CONTROL A as *"must NOT flag `circuit.wat`'s and `sns-fanout.wat`'s poisoned-call arms."* Read
file-level, **that control would have FAILED a correct walker** — both files still contain *other* raising
placeholders, which are genuine findings. The strike narrowed it to **line 544 / line 369** and stated why. ⭑
That is a control I specified loosely enough to reject correct work — the same defect class as this session's
two scope rows, caught by the executor rather than by me.

## Rows verified on my own instruments

| # | row | my verdict |
|---|---|---|
| 1 | ⭑⭑ CONTROL A absent | ✅ **my run**, both lines, all eight homes |
| 2 | ⭑⭑ CONTROL B present | ✅ **my run** |
| 3 | controls stated | ✅ printed in the output, with why each must hold |
| 4 | every row carries the head | ✅ `defservice=40 defsurface=0 defn=435 deftest=64 other=5` — no blanks |
| 5 | every row carries the home | ✅ present. ⚠ `wat-tests/` → `other` (residual above) |
| 6 | all three raise forms | ✅ `"assertion-failed!"` `"raise!"` `"panic!"` all named in source; **all 544 are `assertion-failed!`** — the other two recognised and unused |
| 7 | form-walking, not lines | ✅ **0** line-based matches, **6** `ast->children` walks |
| 8 | 487 → findings gap explained | ✅ 493 grep-o → 482 keywords → 479 arms → 464 raising → 15 unclaimed, each step named; and my 8-home re-run reconciles to 562/559/544 |
| 9 | ⛔ nothing migrated | ✅ **zero modified files** |
| 10 | ⛔ no stdlib rule | ✅ `src/` `wat/` empty |
| 11 | ambiguous rows marked | ✅ the five `other` rows are stdlib `defmacro`/`defclause`/`extend-type` — **marked, not resolved** (CONTROL B itself sits inside `defmacro defservice`) |
| 12 | run twice identical | ✅ **my own two runs**, `diff -q` clean — and the sorted path list is why |
| 13 | floor | ✅ my own run, 5239, 0 FAIL |
| 14 | clippy | ✅ my own run, 0 |

## What I'd credit above all

**It reported its own coverage gap rather than letting a complete-looking number stand.** `wat-tests/` being
out of the path list was my omission; the SCORE named it with its token count, which is the only reason I
looked — and looking found that my "four homes" claim had been wrong in three documents for a day.

## What this hands the builder

**The filter is a column.** `defservice × wat-scripts/service = 18` is the service-item population on the
service axis, stable across a 4→8 home corpus extension. `deftest=64` and `scratch-pad` rows are where raising
is correct. The five `other` rows are stdlib macros and need a human. ⛔ **Nothing was migrated and nothing
should be until the builder sequences it** — the two arms already done were *tears mislabelled*, which no rule
can know.
