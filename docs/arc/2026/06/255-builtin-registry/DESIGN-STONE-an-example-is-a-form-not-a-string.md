# DESIGN — STONE: an example is a FORM, not a string

> **Builder, 2026-08-31:** *"hrm... why do we need to quote the examples... that feels
> ....disappointing.... can the macro not handle them as literal syntax?..."*
>
> Right, and the reason it does not is a shape inherited rather than chosen.

## The defect

```clojure
:examples [["(:wat::string::capitalize \"object\")" "\"Object\""]      ;; what shipped
           …]

:examples [{:expr (:wat::string::capitalize "object") :expected "Object"}]   ;; what it should be
```

```rust
/// The wat form, verbatim — the text left of `#=>` … trimmed.
pub expr: String,
```

That field is **an artifact of the `///` grammar**, where an example genuinely *is* text. The
metadata path had to produce the same struct, so **the data form inherited the text form's
blindness** — a `///` block cannot see a reader; a metadata map sits inside one.

★ And the cost is already on the record: `Record/field-at` shipped `#=> <r's first field's value>`
— prose where a form belonged — and it was caught **late**, by a reflection test, as
`TrailingContent`. As a form it could not have been written down.

## What the change actually buys

- **A malformed example becomes unrepresentable**, not caught downstream.
- **The wat side stops escaping.** No `\"` inside a string inside a map.
- **One consumer gets simpler:** `src/intrinsic/reflect.rs:93` does
  `parse_one_with_file(ex.expr, …)` today — it *re-parses the string into a form*. That line
  disappears; it is handed the form.

## ⚠ THE BLOCKER — and it is real, measured, and not in this stone's gift

**One consumer prints:** `src/intrinsic/reflect.rs:522`, `out.push_str(ex.expr)` — `render-doc`.
A form needs a printer, and the only form→text path in the tree renders **EDN spelling**:

```
write-forms / show-source  →  "(:wat.core/* x 2)"
a user wrote               →   (:wat::core::* x 2)
```

(`crate::edn::bridge::watast_to_edn` + `wat_edn::write`, per `src/rete/validate.rs:370`.)

⛔ **`docs/arc/2026/06/288-structural-pretty-printer/` is a STUB. Never shipped.** So there is no
form→wat-source printer, and `render-doc` would start showing users a spelling they cannot type —
which is the same defect class as the retired-name lint's whole reason for existing.

## ✅ THE FORK IS RULED — B, and my framing of it was wrong

> **Builder, 2026-08-31:** *"we are movign to only edn.... 251 demanded 255.....
> `(wat.core/+ 40 2) #=> 42`"*

⛔ **I called EDN spelling "a spelling a user cannot type". That measured against TODAY, not against
the road** — which is the builder's own ruling of 2026-08-27, on this seam:

```
3  kill `::` in keywords    4  every call head a symbol    5  = EDN/Clojure-compliant syntax
```

`(wat.core/+ 40 2)` is not a foreign spelling. **It is the destination**, and doc output reaching it
early is a preview rather than a regression. Arc 251 parked precisely to let 255 clear the way —
*"we park 251 and 278 on 255's clean up… 255 will force us to organize"*.

★★ **AND THERE IS A STRONGER ARGUMENT FOR FORMS THAN THE ONE THIS DESIGN MADE, which I missed:**

**A stored string must be MIGRATED when steps 3–5 land. A stored form does not.**
`"(:wat::core::+ 40 2)"` is text — every example in the corpus would need a codemod rewrite at the
syntax flip. A form renders in whatever spelling is current, so **examples written today survive the
flip untouched.**

That inverts the fork's cost table: option A does not "wait safely" — it accumulates 400-odd string
examples that a later codemod has to carry across the very migration this arc is clearing the way
for. **The blocker I named was pointing the wrong direction.**

## ~~THE FORK~~ — kept struck, for the record



| | what ships | cost |
|---|---|---|
| **A. wait for arc 288** | nothing now; this stone is drawn and blocked on a printer | `@example` stays text, and every new declaration keeps escaping |
| **B. store the form, render via EDN** | the form is the stored truth; `render-doc` prints EDN spelling | doc output changes to a spelling a user cannot type |
| **C. store the form AND its source text** | both; render from the text, validate from the form | ⛔ two representations of one fact — they drift, which is the defect this arc keeps finding |

★ **C is disqualified on Honest** and I would not build it: a stored string beside a stored form is
exactly the "two copies of one claim" shape that has cost this arc repeatedly today.

★★ **A is the conservative answer and B is the fast one, and the difference is who absorbs the
wrongness:** under A, authors keep writing escaped strings; under B, readers see EDN spelling. Both
are real costs and neither is hidden.

⚠ **What I would NOT do is pick B quietly.** Changing `render-doc`'s output spelling is user-visible,
and it would arrive as a side effect of an internal representation change — the shape of decision
this arc has spent the day removing.

## What is NOT in question

The **direction** is settled: an example should be a form. `wat-doc` already depends on `wat-reader`
and already calls `parse_one_with_file`, so parsing costs nothing new. The only open question is
**what `render-doc` prints in the meantime.**

## Acceptance — once the fork is ruled

| what | expected |
|---|---|
| a malformed example | refused where written, not at a downstream reflection test |
| `reflect.rs:93`'s re-parse | gone — the form arrives as a form |
| the wat side | `{:expr (:wat::string::capitalize "object") :expected "Object"}`, unescaped |
| `render-doc` | per the ruling: unchanged (A) or EDN-spelled (B) |
| floor | 5110/5110, 0 failed |
