# DESIGN — STONE: the fence refuses what it cannot prove, and the enum gets its name

> **Builder, 2026-09-06:** *"we just worked on enums being properly support... so `enum::name`
> restores what the last stone took away"* · then, on the four questions: ***"C+A has been
> reasoned"*.**

Two halves, and **neither is honest alone**: the `:then` fence stops admitting a form it cannot prove
total (**C**), and the callers get the op that makes the right path the only path (**A**).

## HOW WE GOT HERE — the third `:then` defect

`[[SCORE-STONE-the-then-block-never-checked-its-operand]]` left three `Capture.value` sites unable to
render a `NodeKind`, so a file named `277-head-kind-census.wat` **stopped censusing kinds**. Chasing
`enum::name` found something cheaper first — and then found why the cheap thing is wrong:

```
a :then CAN carry an inline (:wat::rete::core::match ?k ((…::Aa) "aa") ((…::Bb) "bb"))
  →  BOXES=1 label=bb                                     it works, and needs no new op

drop an arm:
  →  "no arm matched scrutinee of type wat::core::Enum;
      exhaustiveness should be caught at type-check time"  AT RUNTIME
```

## ★ AND THE FENCE IS NOT BROKEN — IT IS BEING ASKED THE WRONG QUESTION

`then-item-fence` (`wat/rete/compile.wat:742-766`) already checks all four axes, totality included.
It let the partial match through because **`total?`'s own contract is head-level**:

> *"whether every **HEAD** in `expr`'s transitive walk is defined on all its inputs"*
> — `src/intrinsic/rete.rs:191`

`:wat::core::match` is a total HEAD. **Exhaustiveness is a property of THIS match's arms.** A
head-level axis cannot see a form-level property, and no amount of fixing `total?` changes that
without breaking its documented contract.

⚠ **And the fence demonstrably works** — this is a blind spot, not a dead gate:

```
first  (pure, deterministic, PARTIAL on an empty vector)  →  REFUSED by the fence
cond   with no terminal :else                             →  REFUSED at EXPAND
                                                             "cond: non-exhaustive — needs a
                                                              terminal :else arm"
match  with a missing arm                                 →  ADMITTED, dies at runtime
```

**The census is complete: `match` is the ONLY form-level-partial head the `:then` fence admits.**
`cond` — the obvious second suspect — is guarded by its own macro at expand time.

## C — THE FENCE REFUSES `rete::core::match` IN A `:then`

Derived from the nature of the thing rather than from the incident: **a `:then` admits only what the
fence can prove total, the fence's totality axis is head-level, and a `match`'s partiality is
form-level — therefore the fence cannot vouch for it and must not admit it.**

This is constraint engineering, not a patch: the *cannot* follows from what the axis IS.

## A — `:wat::rete::core::enum::name`

C alone takes away the last road, so A builds the road. It is a **missing sibling**, not a new
category — the rete vocabulary already carries a to-string family with no enum member:

```
:wat::rete::core::bool::to-string   :wat::rete::f64::to-string   :wat::rete::i64::to-string
```

and `core::enum::=` / `core::enum::not=` already namespace enum operations (arc 278 #57).

⚠ **It needs a new CORE intrinsic underneath it.** Measured: `:wat::core::variant` is the
**constructor**; nothing anywhere reads an enum value's variant name. `enum::=` had it easy — it
lowers onto core `=`, whose `values_equal` already compares enums. There is nothing for `name` to
lower onto.

**Returns the variant name, no leading colon** — `NodeKind::List` → `"List"` — matching how the value
already renders (`#wat.grep.NodeKind/List []`).

★ **A visible consequence, stated rather than discovered later:** the restored captures will read
`"List"` / `"Keyword"` — the wat-level VARIANT names — where the pre-enum captures read `"list"` /
`"keyword"`, the `ast-kind` strings. That is a change to what the census prints, and it is the more
honest of the two: it names the thing the fact actually holds.

## WHY BOTH, AND IN THIS ORDER — the four questions, run 2026-09-06

| option | Obvious? | Simple? | Honest? | Good UX? |
|---|---|---|---|---|
| **A alone** — mint the op | YES | YES | **NO** — it exists *because* inlining is unsafe and ships without closing or marking that; the next author still gets a runtime failure | — |
| **B** — type-check the inline match's exhaustiveness | YES | **NO** — the logic is inside `infer_match` (`check.rs:5985`), braided with `MatchShape` and inference. Reaching it is a checker refactor; not reaching it is a SECOND exhaustiveness checker — the class `one_name_grammar` exists for, and which bit the previous stone | YES | — |
| **C alone** — the fence refuses `match` | YES | YES | YES | **NO** — removes the only working conversion and leaves the three sites with no path |
| **D** — status quo | **NO** — a file named `head-kind-census` that cannot census kinds | YES | **NO** — a capability silently removed | — |
| **★ C + A** | YES | YES | YES | YES |

⚠ **B's `Simple: NO` is a READ, not a measurement** — `infer_match` was read and the extraction judged
non-trivial; it was not attempted. It is the one cell that could flip this table, and it is recorded
as a read so a later self can overturn it with a measurement rather than inherit it as settled.
`[[feedback_i_cited_a_rule_instead_of_measuring_whether_it_applied]]`

## THE CONTRACT DECISION

**C's refusal names the axis, not the incident.** The diagnostic must say that a `match`'s
exhaustiveness is form-level and the fence's totality axis is head-level — so the reader learns the
rule, not the symptom. `R29 RVINA ERVDIT`.

## WHAT THIS STONE IS NOT

- **NOT a change to `total?`.** Its head-level contract is correct and documented; widening it to
  peer inside forms is a different, larger question.
- **NOT B.** Named, scored, and left with its refuting measurement unrun rather than pretended-about.
- **NOT `if` / `cond`** — the formatter rules, still ruled and still queued, and next after this.

## FILES

```
wat/rete/compile.wat            then-item-fence refuses match; the diagnostic names the axis
src/intrinsic/record.rs         the new core intrinsic that reads a variant's name
src/rete/vocabulary.rs          the :wat::rete::core::enum::name row, beside enum::=
src/check.rs                    its infer_rete_form arm, the shape enum::= already uses
3 caller sites                  the captures restored
tests/rete/…                    negative controls for BOTH halves
```
