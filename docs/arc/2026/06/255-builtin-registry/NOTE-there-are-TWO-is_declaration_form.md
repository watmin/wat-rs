# NOTE — there are TWO `is_declaration_form`, and they agree about one name in fourteen

> **Builder, 2026-09-02:** *"> is_declaration_form — this is a query against the registry?..."*
>
> It should be. **The finding is that we cannot say "it" without asking WHICH ONE**, and my
> `[[DESIGN-STONE-1a-beta-i-the-type-declaration-family]]` treated them as one function and
> attributed callers to the wrong one.

## The measurement

```
src/freeze.rs:1933        pub fn is_declaration_form(head: &str)      -> bool
src/declare/parse.rs:197  pub(crate) fn is_declaration_form(form: &WatAST) -> bool
```

Same name. Different signature. Different home. **Different populations:**

```
freeze::is_declaration_form           9   def · defalias · defenum · defmacro · defstruct
   (its own `matches!`)                   defsurface · newtype · structtype · typealias

declare::is_declaration_form          6   def · defclause · derive · extend-type
   (→ is_declaration_head                 config::set-redef! · config::set-eval-redef!
      → DECLARATION_HEADS)
```

★★★ **Intersection: `{:wat::core::def}`. Union: 14.** Two functions named *"is this a declaration
form"* that agree about **one name out of fourteen**.

## Which callers belong to which — I had this wrong

| call site | which fn | my DESIGN said |
|---|---|---|
| `closure_extract.rs:2578` | **freeze**'s | freeze's ✅ |
| `runtime.rs:10486` | **declare**'s (imported at `runtime.rs:45`) | freeze's ⛔ |
| `declare/parse.rs:211` | **declare**'s (self-recursive) | freeze's ⛔ |

**`freeze::is_declaration_form` has exactly ONE caller** — `split_body_prelude`, deciding which
leading forms of a fn body's `do`-prefix get lifted into the closure prologue.
`[[feedback_a_census_without_attribution_is_not_a_census]]` — I counted the name, not the function.

## ⛔ And its own doc is stale in BOTH directions

`src/freeze.rs:1917-1932` and `src/closure_extract.rs:2537` both describe the population as
*"def/define/defmacro/define-dispatch/defstruct/enum/newtype/typealias"*:

- **names three that are HARD CUT and absent from the `matches!`** — `define` (Stone 241.16),
  `define-dispatch` (Stone 241.13 → `defclause`), `enum` (Stone 241.9 → `defenum`)
- **omits three that ARE in it** — `defalias`, `structtype`, `defsurface`

And the doc lists a second caller, `check::validate_def_position_with_wrapper`, as *"(Gap I-B, future
slice)"*. That fn exists (`check.rs:682`) and **does not call this predicate.** A deferral written
into a doc comment, describing a caller that never arrived.

★ `runtime.rs:816` records the origin: *"Arc 109 Stone 2 — `is_declaration_form` moved to
`src/declare/parse.rs` (the declare home…)"*. **A move that never deleted the original.** The
`freeze.rs` copy is the survivor, and the two have drifted apart for four months with nothing able to
notice — the same shape as `is_mutation_head` vs `is_mutation_form`, one level worse because these
two share a NAME.

## What this changes for the campaign

1. **The equality must name its function.** `is_declaration_form(h) ≡ the row names a Declare impl`
   was drawn against `freeze`'s 9 — correct as far as it goes, and it kills a 9-name hand-list with
   **one** consumer. Smaller than the design implied, and the design must say so.
2. **`declare`'s 6-name population is a SEPARATE kill** with more consumers, and it is not a
   Declare-impl query: `derive`/`extend-type`/`defclause` are not freeze-time type declarations, and
   the two `config::set-*!` are not declarations at all.
3. ★★★ **The homonym is itself a defect** — worse than a hand-list, because a reader (or an
   orchestrator drawing a stone) cannot tell from a call site which population answered. Renaming or
   deleting one is a prerequisite the campaign did not know it had.

## ⬜ What this NOTE does NOT decide

Whether `freeze`'s copy should be renamed, deleted into `declare`'s, or kept and re-pointed at the
registry. That is a fork, and this arc argues forks against the four questions in the main chat
rather than settling them at the end of a measurement.

★ What it establishes: **the answer to "is it a query against the registry?" is "which one?" — and
that is the whole finding.**
