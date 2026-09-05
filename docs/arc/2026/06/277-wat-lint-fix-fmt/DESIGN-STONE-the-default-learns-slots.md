# DESIGN — STONE: the default learns slots from the registry

> **Builder, non-negotiable:** *"ret-spec is a single line... i will not accept otherwise... we
> express what we must in the rules to make this hold true."*
> And on the mechanism: *"absolutely this"* — **wat reading wat's own declared grammar.**

## THE DEFECT

R11 splits a ret-spec because `:wat::core::fn` **has a grammar and no rule**:

```
    (:wat::core::fn
      [acc <- :wat::core::i64 x <- :wat::core::i64]
      ->
      :wat::core::i64          ⛔ must be ONE line
```

## THE RULE, and it is already measured

`[[NOTE-the-registry-already-knows-the-slots]]` — four measurements, all asked of the substrate:

```
572 rows · 36 carry a grammar · all 36 parse with read-string · UNREADABLE=0

GRAMMAR OF :wat.core/fn
  idx 2  symbol   ->          idx 3  keyword  :RetType      ← ADJACENT
```

> **If a head's grammar has `->` at index i, then child i+1 of any form with that head is GLUED to
> child i — it never starts a line.**

```wat
(:wat::core::defrecord :wat::fmt::Slot
  [head  <- :wat::core::String    ;; the head, in SOURCE spelling
   glued <- :wat::core::i64])     ;; the child index that must NOT start a line
```

## ⭐ THE HEAD SPELLING COMES FROM THE GRAMMAR, NOT FROM `Row/name`

`Row/name` renders `:wat.core/fn` — the DOT form. The corpus, and `grep.wat`'s `Named` fact, carry
`:wat::core::fn` — the COLON form. Joining on `Row/name` would silently match nothing.

**The grammar string already contains the head in source spelling:**

```
(:wat::core::fn [<param> <- :T ...] -> :RetType <body>+)
 ^^^^^^^^^^^^^^ child 0 of the parsed grammar
```

So the head is read from the PARSED GRAMMAR's child 0, and the name-form problem does not arise.
★ A join that silently matches nothing is the worst failure available here — it looks exactly like
a form having no slots.

## WHERE `Slot` LIVES — `wat/fmt.wat`, not `wat/grep.wat`

`grep.wat`'s stated contract is *"the fact base wat-grep inserts, **per file**"*. `Slot` is neither
per-file nor a property of source — it is derived from the REGISTRY, once per run, and only the
formatter wants it. Putting it in `grep.wat` would widen a shipped contract for one consumer.

## ⛔ THE REFUSAL — a grammar the rule cannot trust must yield NO slot

Grammar index maps to form child index only while every pre-arrow part is exactly one child.
`fn`'s is (`[<param> …]` is one vector). **A grammar with a VARIADIC before the arrow would break the
correspondence**, and none of the 36 has one — *but that was read off a list, not computed.*

> **If any child before the arrow is variadic (`...`, or a token ending `+`/`*`), emit NO `Slot` for
> that head.** Silence is correct; a wrong index is a mangled ret-spec on every use of that form.

★ Same discipline as the walls already standing: **refuse rather than guess.**

## THE ACCEPTANCE

```
1  ★ the ret-spec is ONE LINE — `foldl-bare.wat`'s inner `fn` renders `-> :wat::core::i64` together
2  every fixture stays idempotent
3  `let` is unaffected — its grammar has no arrow, so it gets no Slot, and nothing changes
4  the refusal FIRES — a synthetic grammar with a variadic before the arrow yields no Slot
5  the three walls still stand
```

Row 1 is the builder's non-negotiable. Row 4 is the refusal shown firing, not asserted.

## ⚠ WHAT THIS DOES NOT FIX — measured, and named rather than discovered later

**Type applications still split:**

```
  [m <- (:wat::core::HashMap :-
          [:wat::core::i64 :wat::core::i64])
```

`:wat::core::HashMap` is **not among the 36** — it carries no `@syntax`, so this stone cannot reach
it. The damage is real and remains.

⭐ **And there is a cheaper rule that would cover BOTH, which the builder should see before the next
stone:** `->` and `:-` are LANGUAGE tokens, not policy. A purely lexical rule — *"a `->` or `:-`
child glues to the child after it"* — needs no registry at all and fixes type applications too.

**It is not this stone**, because (b) is what was ruled and because the registry-derived version
proves the grammars are usable, which is worth having on its own. But if the lexical rule is right,
this stone's mechanism becomes the *general* case and the lexical one its specialisation. **That is
the builder's call, and it is better made after seeing this work than before.**

## OUT OF SCOPE

- **Type applications** — above, with the cause measured.
- **`defclause`'s nested arrow** (`:name [-> :T] …`) — inside a vector, so a top-level scan finds no
  slot. Whether that is right for `defclause` is unasked.
- **`defn`/`defrecord`/`defstruct`/`deftest`** — no registry row at all; they have, or will have,
  hand-written rules. `defn`'s ret-spec is already correct because R1 exists.
