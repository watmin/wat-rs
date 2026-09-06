# DESIGN — STONE: the sibling table

> **Builder:** *"this is a /very agreeable/ format... its extremely easy to read.... i think we
> should get this handled"*
> ```
> (:m::Member :id 1 :prefix "String" :base "concat"    :style "slash")
> (:m::Member :id 2 :prefix "string" :base "length"    :style "colons")
> (:m::Member :id 3 :prefix "i64"    :base "to-string" :style "colons")
> ```

## ⛔ THIS IS THE FIRST CARVE-OUT FROM THE EXPLODED FORM — measured, not asserted

The exploded ruling turns a 3-row table into 15 lines **today**:

```
(:wat::grep::Node                 ← input was 3 one-line siblings
  :id     1
  :parent 0
  :index  0
  :kind   "symbol")
(:wat::grep::Node
  :id     2
  …
```

> **D1, RULED:** *"it has to"* — the table beats the exploded form. **`rete should be mature enough
> to handle our complex requirements.`**

★ The builder said earlier *"we add compression rules later"*. **This is that, arriving now for one
recognisable shape**, and it is recorded as an exception rather than smuggled in as a refinement.

## WHAT A GROUP IS

> **D2, RULED:** *"identical types?.. different instances of that type?... they need to be expressed
> sequentially to be applicable"*

```
same HEAD                    the type
2 or more instances          "different instances"
ADJACENT siblings            "expressed sequentially"
same KEY SEQUENCE            ← implied, not added: see below
```

⚠ **The key sequence is part of the definition, not an addition to it.** Keyword arguments may be
given in any order — the corpus does it deliberately at `wat-tests/format.wat:16`
(`(:wat::core::format "{a} {b}" :b 5 :a "x")`, reversed to prove order is irrelevant to the callee).
**Columns cannot align across a different key order**, so two same-typed instances with different key
orders are not one table. For a POSITIONAL table the analogue is the same ARITY.

⚠ **Minimum group size is 2**, per *"different instances"*. I had recommended 3. **2 is the ruling
and 2 is what ships** — with the acceptance reporting how many groups form at 2 versus 3, so the
threshold can be raised on evidence rather than on my preference.

> **D3, RULED:** *"feels like its a yes"* — POSITIONAL tables too.
> ```
> (:wat::core::Tuple ":cons::consumer'"          ":cons::consumer")
> (:wat::core::Tuple ":prod::producer'"          ":prod::producer")
> ```

## ⭐ THE HAND-BUILT TABLES HAVE ALREADY DRIFTED — the rule FIXES them

```
:base "concat"····:style          4 pad
:base "+"········ :style          8 pad AND a stray trailing space
```

`rules-corpus-02:176-180`, maintained by hand, is already inconsistent. **The formatter makes it
exact.** This is not only a preservation argument; it is a correction argument.

## THE MECHANISM — the first rule that reasons about NEIGHBOURS

Every rule so far reasons about a form and its own children. This one needs a fact about a form's
SIBLINGS, which nothing carries:

```wat
(:wat::core::defrecord :wat::fmt::TableRow
  [form  <- :wat::core::i64      ;; a member of a table group
   group <- :wat::core::i64])    ;; the group's id — the first member's node id
```

A rule detects the group and asserts one `TableRow` per member; **the emitter computes each column's
width across the group and pads.** As with indent and `AlignPairs`, **no rule names a column** — the
wall from `[[DESIGN-STONE-indent-is-structural]]` stands at 0.

⚠ The emitter must know every member's rendered widths before emitting the first — a group-scoped
pre-pass, where `AlignPairs` needed only a form-scoped one.

## ⚠ WIDTH — my proposal, NOT ruled, and it is stated so it can be overruled

**A table stays a table regardless of line width**; R15 (120) catches an over-long row as a LINT.

The alternative — fall back to exploded when a row exceeds the budget — makes **one row's layout
depend on its NEIGHBOURS' widths**, and would be the first place width drives layout since the
exploded ruling retired that idea. That property is what has let the last five stones compose, so I
am not spending it without a ruling.

## THE ACCEPTANCE

```
1  ★ a 3-row keyword group renders as 3 ALIGNED LINES, not 15
2  ★ a POSITIONAL group (same head, same arity) aligns too
3  ★★ a NON-group is untouched: different heads, non-adjacent, or a different KEY ORDER
4  ★★ a group of 2 forms a table (the ruling) — and the report says how many groups
     would have formed at a threshold of 3
5  ★★★ the drifted real table (`rules-corpus-02:176-180`) comes out EXACT
6  idempotent — padding computed from this pass, never from the input
7  no rule names a column — `grep -c 'col' rules/*.wat` = 0
8  ★★★ the 614 doc examples: unchanged over-120 count (0), and the number of table groups formed
9  ★★ the corpus proxy: of ~354 candidate runs, how many become tables — the real blast radius
```

⛔ **Row 3 is the one that can be satisfied by the defect.** A rule that groups any adjacent
same-head pair passes rows 1, 2 and 4 and destroys `format.wat`'s deliberately-reordered call.
**The key-order test is what row 3 exists to prove.**

## OUT OF SCOPE

- **Level-2 alignment INSIDE a value** (`Location/line` vs `Location/col ` in `grep.wat:284-289`).
  That is alignment inside sibling VALUES of one form, not across sibling FORMS. Still open.
- **A width fallback** — see above; unruled, not implemented.
