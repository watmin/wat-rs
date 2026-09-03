# SCORE — D10, weighed against the orchestrator's own re-run

> **The soundness hole is closed at the top level of a `:then` fact form, and the cure is proven not
> to over-refuse.** Two defects in my brief, one of which would have produced a green, plausible,
> WRONG cure that missed the repro's own subject.

## The scorecard

| # | required | result, MY re-run |
|---|---|---|
| 1 | ★ knowably-wrong `:then` value refused | ✅ `RhsFieldTypeMismatch`, driven on the original repro |
| 2 | ★ a literal refused too | ✅ |
| 3 | ★ not-knowable still compiles | ✅ — **and mutation C proves the alternative breaks the corpus** |
| 4 | the well-typed control still derives | ✅ |
| 5 | the corpus survives | ✅ **1664 `.wat` scanned, ZERO newly-failing**; independently corroborated by a green floor |
| 6 | an RHS type kind exists | ✅ `RhsFieldTypeMismatch`, six fields |
| 7 | `:when` typing untouched | ✅ `resolve_operand_type` unchanged (STOP-3 held) |
| 8 | no engine change | ✅ `git diff --stat -- src/rete/kernel/` empty |
| 9 | floor / lints / clippy | ✅ **`5351 tests run: 5351 passed, 21 skipped`**, 210/210, rc=0 |

The diagnostic, driven by me on the pre-cure repro:

> ``defrule `tr::bad`: `:then` insert of `:tr::Bad` fills field `:n`, declared `:wat::core::i64`
> (rete `i64`), with operand `?s`, whose type is `string` — the same construction written outside a
> rule is a TypeMismatch, and a `:then` value is checked the same way.``

## ⭐ A — STOP-4 ANSWERED WITH A FIELD NO SIBLING KIND CARRIES

`field_rete_type`. `resolve_operand_type` answers in rete **segments**, not declared paths, so the
comparison actually performed is `i64` vs `string` — **not** `:wat::core::i64` vs
`:wat::core::String`. Reporting only the declared path would state a comparison that was never made
(two distinct enums both segment to `enum`); reporting only the segment would hide the `defrecord`
line. Both are carried, so **the message is checkable against the check**. That is the
diagnostic-completeness standard met rather than gestured at.

## ⛔⛔ B — "EVERYTHING NEEDED IS ALREADY IN SCOPE" WAS FALSE, AND IT WAS THE LOAD-BEARING SENTENCE

`resolve_operand_type` needs `binds`. `validate_then_form` did not receive it, and
`collect_rule_bind_types(when_conds, types)` was computed **inside** the `if let Some(when_conds)`
block and dropped before the `:then` loop. Without hoisting it out, every `?var` answers
`UnboundInThisRule` — so the **literal** arm is caught and **the bound-`?var` arm, which is the
repro's own SUBJECT, silently passes.**

**A rider taking my sentence literally ships a cure that goes green on its own scorecard while the
defect that motivated the strike still fires.** This is the single most dangerous line I have written
in this arc.

## ⛔ C — AND MY EXAMPLE OF A NOT-KNOWABLE OPERAND WAS WRONG IN THE DIRECTION THAT REOPENS THE BUG

I listed *"a `?var` bound from a derived fact"* beside `Form`/`Redispatch` heads and type variables as
something that must still compile *because it is not knowable*. **It is fully knowable** — the bind
names a declared field of a registered fact type, and `collect_rule_bind_types` resolves it exactly
as for a seeded fact. A rider trusting that framing would have **skipped the derived-fact case**,
reopening the hole for every cascading rule — most of the corpus.

`[[an-example-in-a-brief-is-a-claim-too]]`, and mine failed in the worst direction.

## ⭐ D — MUTATION C IS THE ROW THAT MATTERED, AND IT WAS EARNED EMPIRICALLY

Making `ComputedNotDerivableHere` a refusal: **the not-knowable probe RED *and it took
`probe_arc278_then_is_an_expansion_boundary`'s four pre-existing tests down with it*** — while every
refusal arm stayed green. That is the named failure-even-if-green demonstrated against the real
corpus, not argued. Two further call-site mutations (A: kwargs only; B: positional only) prove the
check is wired at **both** producers independently.

## ⛔ E — RESIDUAL, DRIVEN BY ME, NOT SUPPOSED

```
:then [(:nh::Outer :i (:nh::Inner :n ?s))]   →   #nh/Outer {:i #nh/Inner {:n "nested-string"}}
```

`walk_nested_constructors` has no `binds`, so a **nested** constructor's fields are untyped. Same
class, same fact set, one level deeper. The rider stopped rather than thread `binds` through a
recursive walker with four other producers — correct, and it is **D11**.

Two further bounds it stated rather than hid: the check is only as sharp as `rete_type_segment_of`
(two distinct enums both segment to `enum`), and `NotComparable` is deliberately passed because a
parametric record's erased field arrives through that same channel — refusing it would be D7's
ground, explicitly cut by the DESIGN.

## ⛔ F — AND A CLASS DEFECT IN HOW THIS REPO WRITES REFUSAL PROBES

Every `.wat.bad` fixture ends `(:user::main [] -> :wat::core::nil nil)` — which is **itself** a
startup failure (`MainSignatureError`). So with the wall mutated away the file **still fails**, for
the wrong reason, and `assert!(!ok)` cannot go red under the very mutation it exists to detect. Only
the `.edn` golden can.

The rider found this when its own mutation dumped `MainSignatureError` where the golden expected
`RhsFieldTypeMismatch`, gave its four fixtures real mains, and **named five-plus existing fixtures
carrying the same blindness**. That is **C18**, and it is bigger than this strike.

## Per-arm status

| arm | status |
|---|---|
| kwargs `:then`, bound `?var` / literal / computed | **proven** — RED under full revert and under the kwargs-only mutation |
| positional `:then` (the second producer) | **proven** — RED under positional-only, GREEN under kwargs-only |
| `ComputedNotDerivableHere` passes | **proven** — mutation C, and it takes 4 corpus tests with it |
| `UnboundInThisRule` passes | **reachable but not driven** — `RhsUnresolvableOperand` already walls the reachable spellings |
| `NotComparable` passes | **deliberate**, documented on the variant — D7's ground |
| `MistypedEnumVariant` | **not reachable from `:then`** — `RhsUnresolvableOperand` catches every keyword in RHS value position first |
| nested constructor values | **NOT COVERED — D11**, driven by me |
