# AMEND — mid-strike. The brief's predicate is INCOMPLETE, not wrong.

> **Builder, after the brief was sent:** *"i think the nuance of the parametric types is the literal
> forms vs a type decl...."*

**My brief said only: arity-3 → atomic.** That is correct for a type DECLARATION and says nothing
about a CONSTRUCTOR, which has the same head, the same `:-`, the same type-args — **plus values.**

```wat
(wat.type/HashMap :- [wat.type/Keyword wat.type/String])      a DECLARATION   3 children
(wat.type/HashMap :- [wat.type/Keyword wat.type/String]       a CONSTRUCTOR   5 children
  :some-kw "some-string")
```

## ⛔ MEASURED — today's output on a constructor is wrong

```
(:wat::core::Vector :-
  [:wat::core::i64]        ⛔ the type-args TORN from the `:-`
  1
  2
  3))
```

R11 rides `:-` (an atom) and breaks `[…]` (a compound). The builder's shape:

```
(:wat::core::Vector :- [:wat::core::i64]
  1
  2
  3)
```

## ✅ THE RULE, WHICH SUBSUMES THE BRIEF'S RATHER THAN REPLACING IT

> **For a form whose child 1 is a symbol/keyword named `:-` and whose child 2 is a vector:**
> - **children 1 and 2 are GLUED to the head line** — the type-args slot never breaks
> - **arity 3** (nothing follows) → the whole form is ONE LINE — *a type declaration*
> - **arity > 3** → children 3+ each get their own line — *a constructor; the values explode*

★ **The brief's arity-3-atomic falls out of this as the special case where there are no values.**
Everything already written for it stands; this adds the constructor half.

## Checked against all three of the builder's examples

```wat
(wat.type/HashMap :- [wat.type/Keyword wat.type/String]     type-args glued ✓
  :some-kw "some-string")                                   values explode ✓

(wat.type/Vector :- [wat.type/i64]                          glued ✓
  1  2  3)                                                  each own line ✓

(wat.type/Tuple :- [... (wat.type/HashSet :- [String])]     the NESTED type inside the type-args
  42                                                        is arity-3 -> atomic ✓
  69.69
  (wat.type/HashSet :- [String]                             a nested CONSTRUCTOR: glued + explode ✓
    "one"  "two"  "three"))
```

⚠ **Builder's own note on these:** *"this is likely one-liner-able later... exploded for now."*
**Nothing collapses.** Compression remains a later, separate ruling.

## THE COUNTEREXAMPLES STILL BIND

Unchanged from the DESIGN, and the new rule must still refuse them — **the arity check no longer
does that work alone**, so the KIND checks now carry it:

```
(:wat::core::fn :- [~@binder-names-ch] ~params -> ~ret …)   wat/core.wat:1349
```

⛔ **A generic `fn` has child 1 = `:-` and child 2 = a vector — it MATCHES the glue rule.** Gluing
`:- [T]` to `fn`'s head line is in fact CORRECT (it is the param-spec, and the builder ruled
`(wat.core/defn user/some-fn :- [K V]` on one line) — but its remaining children are **`[params]`,
`->`, `:RetType`, body**, which must lay out by `fn`'s OWN rules, not as exploded "values".

> **This is the one case to get right, and row 4 of EXPECTATIONS already gates it:** a generic `fn`
> must still render its arg-spec, ret-spec and body normally. If the glue rule alone cannot
> distinguish a constructor's values from a `fn`'s remaining slots, **STOP and report that** — the
> `Slot` mechanism from the previous stone already knows `fn`'s grammar, and is the honest place for
> the distinction.

## ACCEPTANCE — replacing rows 2-4 of EXPECTATIONS

```
2  a type DECLARATION is one line          [xs <- (:wat::core::Vector :- [:wat::core::i64])]
2b a CONSTRUCTOR glues its type-args and explodes its values
3  a NESTED type inside type-args stays inline
4  a generic `fn` still lays out by fn's rules — ret-spec on its own line, body on its own line
```

Rows 1 and 5-13 are unchanged.
