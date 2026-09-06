# DESIGN — STONE: the definition forms, and the aligned `<-`

> **Builder:** *"defrecord (and defstruct, enum too?...) needs correction..."*
> ```
> (:wat::core::defrecord :probe::StepPayloadExampleTemp
>   [celsius <- :wat::core::i64
>    some    <- :wat::core::f64
>    else    <- :wat::core::String])
> ```

## TWO DEFECTS, BOTH MEASURED

### 1 — `defrecord` has no rule, so the name falls off the head line

```
(:wat::core::defrecord
  :fix::Rec [celsius <- :wat::core::i64 some <- :wat::core::f64 else2 <- :wat::core::String])
```

R11's leading-atom rule owns it: the field VECTOR is compound, so nothing rides — including the
NAME, which is an atom and should ride exactly as `defn`'s does. `defstruct` and `defenum` are the
same shape and the same gap.

### 2 — ★ `defn`'s arg-spec does NOT align its `<-`, and never has

```
[self <- :wat::core::i64
 work-fn <- :wat::core::String        ⛔ the arrows do not line up
 c <- :wat::core::f64]
```

The builder ruled this shape long ago — his own `defn` sketch has *"comments are aligned"* and
`wat/bracket.wat:32` shows `self    <-` / `work-fn <-` **hand-aligned in the live stdlib**. It has
been listed as unbuildable since the first style table, as one of three consumers of a missing
alignment capability.

⭐ **That capability now exists.** `AlignPairs` landed with the kwargs stone. **This stone spends it
on the second consumer** — leaving only aligned trailing comments (R8), which needs the comment fact.

## THE RULE

> **`defrecord` / `defstruct` / `defenum` take `defn`'s treatment: the NAME rides the head line, the
> field vector gets its own line, one field per line, and the `<-` column is ALIGNED.**
> **And `defn`'s own arg-spec gains the same alignment.**

## ⚠ THE ONE MECHANICAL DIFFERENCE — a field is a TRIPLE, not a pair

`AlignPairs` pads child 0 so child 1 lines up: a stride of **2**.

```
[celsius <- :wat::core::i64          celsius · <- · i64   =  THREE children
 some    <- :wat::core::f64
```

A field breaks every **3** children and pads the NAME so the `<-` lines up. **The existing fact
cannot express a stride**, so either it grows one or a sibling fact carries it:

```wat
(:wat::core::defrecord :wat::fmt::AlignStride
  [form   <- :wat::core::i64
   stride <- :wat::core::i64])   ;; 2 = key/value, 3 = name/binder/type
```

★ **`AlignPairs` becomes the stride-2 case.** ⚠ **Whether to widen `AlignPairs` or add a sibling
fact is the strike's call** — but the alignment must stay the EMITTER's computation either way. No
rule names a column; that wall stands at 0.

## THE ACCEPTANCE

```
1  ★ defrecord: the NAME rides, fields one per line, `<-` ALIGNED
2  ★ defstruct and defenum: the same
3  ★★★ defn's arg-spec `<-` is ALIGNED — the long-standing gap, with UNEVEN names
4  ★★ a defenum VARIANT that carries no field vector is not broken by the rule
5  idempotent throughout
6  no rule names a column — grep -c 'col' rules/*.wat = 0
7  ★ the sibling table and the all-atoms inline rule still win where they applied before
8  ★★ the 614 doc examples: over-120 still 0, and the two `defrecord`s in the
   step-payload example render with their names on the head line
```

⛔ **Row 3 is the one that is not new work but old debt.** A strike could satisfy rows 1-2 with a
`defrecord` rule alone and never touch `defn`. **The `<-` alignment is the point**; the definition
forms are what made it finally worth building.

⚠ **Row 4 matters because `defenum`'s shape is irregular:**
`(:wat::core::defenum :Name :wat::enum::Pure :V1 :V2 [f <- :T] …)` — bare variants and
field-carrying variants mix in one child list. A rule that assumes "child 2 is a field vector"
breaks every enum in the corpus.

## OUT OF SCOPE

- **R8 / aligned trailing comments** — the third alignment consumer; needs a comment fact.
- **The `:examples` fat arrow** — `[[NOTE-the-examples-container-wants-a-fat-arrow]]`, deferred by
  the builder.
- **The emitter's comment-indent defects** — corpus-only.
