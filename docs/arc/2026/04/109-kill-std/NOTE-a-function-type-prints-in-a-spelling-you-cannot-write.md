# NOTE — a FUNCTION type still prints in a spelling you cannot write

**Filed 2026-08-23.** Two prose riders reached this independently; I confirmed it by measurement.

The renderer stone (`64a8fa5a0`) made parametric types print in the spelling the reader accepts —
`(:wat::core::Vector :- [:wat::core::i64])`, verified by pasting the printed string back into source
and running it. **Function types were missed.**

```
;; a real TypeMismatch on a function-typed parameter, measured:
:expected ":wat::core::Fn(wat::core::i64)->wat::core::i64"      ← what the diagnostic PRINTS
           [:wat::core::i64 :-> :wat::core::i64]                 ← what you must WRITE
```

The printed form is not merely a different spelling — for more than one argument it **cannot be read
at all**: `Fn(A,B)->C` carries a comma inside a keyword body, which the reader has refused since the
comma strike. So the substrate prints a function type that its own reader would reject.

★ **This is the exact defect the renderer stone was written to end**, surviving in the one type
constructor that stone did not enumerate — and I did not enumerate it because my census was of
`format!("{}<{}>")` sites. A function type is not spelled with angles, so it was invisible to the
instrument. `[[feedback_scope_the_check_from_the_rule_not_the_diff]]`: the rule is *"a type must print
in a spelling the reader accepts"*, and the angle bracket was one instance of violating it.

## What is owed

- The `Fn(...)->...` renderer emits the bracket form `[Arg… :-> Ret]`.
- ⚠ **And then a census scoped from the RULE, not from this instance.** Ask of every type constructor
  the substrate can print: does its rendering parse? The honest instrument is to round-trip — render a
  value of each constructor and feed the text back through the reader — not to grep for a shape.

## Kin

- `DESIGN-STONE-defservice-emits-the-binder.md` — the renderer stone; its measurement and its blind spot.
- `NOTE-the-guides-are-not-executable.md` — the same class in the teaching materials.
