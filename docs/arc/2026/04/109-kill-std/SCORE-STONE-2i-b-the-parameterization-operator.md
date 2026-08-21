# SCORE — arc 109 Stone ②-i-b: `:-`, the parameterization operator

Rider: one flight, ~15.6 min, no STOP fired. Every row below **re-run by the orchestrator's own
hand** against the built binary, not read from the rider's report.

| # | what | result |
|---|---|---|
| 1 | the RED baseline turns green | ✅ `(wat.type/HashMap :- [String i64])` in a param slot checks |
| 2 | the renderer emits the operator | ✅ **all six rows byte-identical to the pre-strike prediction** |
| 3 | the reader takes what the writer emits | ✅ EXIT 0 |
| 4 | ★ the unmarked bracket still reads | ✅ `(Vector [i64] 1 2 3)` → `[1 2 3]` |
| 5 | ★ the angle form still checks | ✅ |
| 6 | ★ the positional form still builds | ✅ `(Vector :i64 1 2 3)` → `[1 2 3]` |
| 7 | the constructor takes the operator | ✅ `(Tuple :- [i64 keyword] 42 :some-keyword)` → `[42 :some-keyword]` |
| 10 | a literal in a TYPE slot walls | ✅ named diagnostic, not the fn-type misreport |
| 11 | the same form in a VALUE slot is legal | ✅ |
| 12 | ★ the EMPTY tuple LITERAL is writable | ✅ `(Tuple :- [])` → `[]` — **was `[[]]` at HEAD** |
| 13 | a 2-tuple of keyword VALUES is writable | ✅ `(Tuple :- [kw kw] :a :b)` → `[:a :b]` — **was unwritable** |
| 15 | a param-spec holding VALUES is rejected | ⛔ **FAILED — and the row was wrong. See below.** |
| — | floor | ✅ **4855/4855, 0 FAIL, 19 skipped, 71.7s** |
| — | clippy `-D warnings` | ✅ 0 |
| — | rustfmt parity | ✅ zero drift added (by CONTENT — see below) |

## Prediction vs actual

Predicted 25–40 min; actual ~15.6. The one-door contract is why: twelve call sites became a
re-point, not twelve judgments.

## ⛔ THREE DEFECTS, AND TWO OF THEM ARE MINE

**1 — the rider's departure had a bug in it (orchestrator-fixed).** It went outside the brief's
blast radius to handle `(Tuple :- [])`, flagged that prominently, and was right to: `eval_tuple_ctor`
rejects zero args as the illegal bare `(Tuple)`. But its branch guarded on `rest.is_empty()` ALONE,
so `(Tuple :- [A B])` with no values would build an empty tuple at runtime while check-time calls it
an arity mismatch — **check-says-no / runtime-says-yes**, the exact class step ①b's Room 3 was found
by. Fixed to `inner.is_empty() && rest.is_empty()`, which also confines the arm to the `:-` spelling
for free (the unmarked arm cannot produce an empty `inner`).
★ The rider's judgment to depart was CORRECT and its flag is what got the bug read. A silent
compliant version would have shipped green.

**2 — my acceptance row 15 asserted a wall that has never existed.** `(Tuple :- [:a :b])` is
accepted. Controls: the unmarked bracket and the angle form accept it too, so it is spelling-
independent and pre-existing — arc 109's own `NOTE-type-annotation-names-unchecked.md` (*"a type
name is validated in CALL position, never in ANNOTATION position"*), which the builder has already
deferred as backlog. I wrote the row the same day the builder stated the RULE, and wrote it as if
the rule were enforced. **A stated rule is not a built wall.**

**3 — my brief's pinned signature was clippy-illegal.** I pinned
`split_type_param_bracket<'a>(args: &'a [WatAST]) -> Option<(&'a …)>`; with one input lifetime it
elides, and `-D warnings` refused it. The rider followed the brief exactly. Orchestrator-fixed.

## The golden pass — the brief said the floor would name them, and it did

My brief named **four** goldens and said outright that the list was my census and the floor was the
real instrument. The floor named **29 tests / 27 files**. Both halves of that prediction held.

Classified against the arm, none dismissed:
- **22** renderer goldens, every one carrying the pre-strike predicted bytes
  (`(wat.type/Vector :- [wat.type/i64])`, `(wat.type/Tuple :- [])`, …).
- **7** `src/` LINE PINS — `runtime.rs:25330→25343` (×5), `check.rs:13594→13630`, `13576→13612`.
  Messages byte-identical; the rider's inserts moved the `rust_caller_span!()` they point at. Each
  new number verified to land on a `rust_caller_span!()` call, not merely to match.

⚠ **My re-golding script corrupted one file and the floor caught it.** The rule
`\((wat\.type/\w+|:wat::core::\w+) \[` matched `(:wat::core::fn [...] -> :T body)` — **prose inside
an error message**, where `[...]` is `fn`'s parameter list, not a param-spec. Reverted; the golden
was correct as written. Then audited every other added line: all are container heads with `:- [` or
`:line` pins, none prose. *Same class as the codemod bug found this session — one rule applied to
two meanings of one surface.* `[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

## rustfmt — the first instrument was invalid and said the opposite

`rustfmt --check` on a bare `git show HEAD:<f> > /tmp/f.rs` copy cannot resolve the module tree, so
it reported `check.rs HEAD=0` — read as "clean," it meant "bailed." Re-measured against a real HEAD
tree (`git archive HEAD | tar -x`): HEAD already carries drift (104 / 524 / 146 regions), and the
region COUNT moved by +3/+1/+1. **Comparing by content, the added set is EMPTY** — our inserts split
existing unformatted regions in two. Zero drift added; the count was not the measure.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

## What this stone did NOT do, affirmatively

- ③'s hard cut. Every old spelling still works, by design and by rows 4–6.
- The deletion of `is_type_bracket_candidate`. It now has exactly one caller — the unmarked arm
  inside `split_type_param_bracket` — so ③ deletes one door, not six call sites.
- The `def*` name-slot binder (`:- [T …]`), the codemod's slot rule, and `List/of` — all filed as
  their own notes, all sequenced behind this stone.

---

## ⛔ AMENDMENT 2026-08-21 — ②-i-b SHIPPED THE OPERATOR FOR THREE CONSTRUCTORS AND NOT THE OTHER THREE. I SCORED IT GREEN.

Found while answering a builder question about `(wat.type/HashSet :- [I] …)` — **the form did not
work**, four commits after this stone was scored ✅.

```
Tuple · PersistentMap · PersistentVector      :-  WORKED     (split_type_param_bracket)
Vector · HashSet · HashMap                    :-  REJECTED   (unwrap_type_param_bracket)
```

The symptom is a confusing one, because the operator is read as a VALUE:

```
(:wat::core::HashSet :- [:wat::core::i64] 1 2)
  → "HashSet: parameter element #1 expects :-; got :wat::core::i64"
```

**The brief asked for it.** `BRIEF-STONE-2i-b`, room 2, verbatim: *"`src/check.rs:11993`
`unwrap_type_param_bracket` — the UNCONDITIONAL splice used by `Vector`/`HashMap`/`HashSet`. …
Adding a leading-`:-` arm here is internal; no call site changes."* The rider's report described
the new door and the three re-pointed call sites and never claimed to have touched
`unwrap_type_param_bracket` — so the omission was visible in its own report.

★ **And my scoring is what let it through.** Look at what I ran: I exercised
`(Tuple :- [])`, `(Tuple :- [kw kw] :a :b)`, `(PersistentVector :- […])`, `(PersistentMap :- […])`
— all three CONDITIONAL heads — and for the unconditional three I ran only
`(Vector [i64] 1 2 3)` and `(Vector :i64 1 2 3)`, which are the OLD spellings, as additive controls.

**Every new-spelling row I wrote landed on the half the rider had changed, and every row on the
other half tested the half that could not break.** The controls were real controls; they just
answered a different question than the one the stone was about, and a full green over that split
reads exactly like a full green over everything.
`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

**Fixed 2026-08-21.** `unwrap_type_param_bracket` now matches on the whole arg slice with two arms
— `[Keyword(":-"), Vector, rest…]` and `[Vector, rest…]` — mirroring `split_type_param_bracket`,
including the deliberate absence of any content sniffing or `!is_empty()` guard on the `:-` arm.
Verified by running all six constructors in the new spelling and all three old spellings:

```
Vector :- […]  → [1 2]        Tuple :- […]            → [1]
HashSet :- […] → #{2 1}       PersistentVector :- […] → #wat.core/PersistentVector [1 2]
HashMap :- […] → {"a" 1}      PersistentMap :- […]    → #wat.core/PersistentMap {1 "a"}
```

Floor 4855/4855, clippy 0. Two `check.rs` line-pin fixtures moved with the insert
(13630→13651, 13612→13633), each re-verified to land on a `rust_caller_span!()`.

⚠ **The lesson for the next scorecard: when a stone touches N members of a family, the acceptance
rows must name all N.** Mine named the ones the rider's diff had touched — which is the set that
cannot expose a missed member.
