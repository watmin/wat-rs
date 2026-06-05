# NOTE (arc 109 vocabulary) — the typed FORM: `:-` annotation + a `:wat::type::` namespace + the dotted clojure-form

**Filed 2026-06-04. A POINTER, not a decision.** Queue marker for the *shape of a typed wat form*
as the clojure-ination completes — three coupled moves the builder raised: (1) swap the param-type
arrow `<-` → `:-`; (2) give types their own `:wat::type::` namespace, separate from the operators
in `:wat::core::`; (3) render the final surface in the **dotted clojure-form** (`wat.type/i64`),
not the colon-keyword form. No four-questions verdict locked — this records the direction, the
core.typed precedent, and the open questions.

## The current form (grounded 2026-06-04)

```
(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ x 2))
```

- **`<-`** = the param-type annotation arrow (`x` *of type* `:wat::core::i64`). Reads like `∈`/`:-`.
- **`->`** = the return-type arrow.
- **Types and operators share one namespace:** `:wat::core::i64` (the *type*) and `:wat::core::+`
  / `:wat::core::i64::+` (the *operators*) both live under `:wat::core::`.
- **Colon-keyword surface:** `:wat::core::i64` — leading `:`, `::` segment separators (a keyword).

## The proposed form (the builder's direction)

```clojure
;; anonymous add-2
(fn
  [x :- wat.type/i64]
  -> wat.type/i64
  (wat.core/+ x 2))
```

Three moves, each a clojure-faithfulness step (the same "embody clojure-on-rust" telos that drove
arc 247 fn-first + arc 249 threading):

1. **`<-` → `:-`** — **core.typed's annotation arrow.** core.typed writes `(fn [a :- t/Int] ...)`
   and `(ann name t/Int)`; `:-` *is* the Clojure way to say "of type." wat's `<-` was a wat-ism;
   `:-` is the dialect-honest spelling. (`->` stays the return arrow — core.typed's
   `[Params -> Return]` is the same; positionally unambiguous from a threading `(-> …)` head, per
   arc 249.)
2. **A dedicated `:wat::type::` namespace** — `:wat::type::{i64,f64,bool,char,String,…}` hold the
   **types**; `:wat::core::` keeps the **operators** over them (`wat.core/+`). This mirrors
   **core.typed's `t/` namespace** (`t/Int`, `t/Str`, `t/Bool`): the type vocabulary is its own
   namespace, distinct from the value/operator vocabulary. Today they're conflated in `:wat::core::`
   (`:wat::core::i64` the type sits beside `:wat::core::+` the operator); the split names the two
   roles separately. "core provides operators for those types" = `wat.core/+` dispatches over
   `wat.type/i64` (ties to the per-Type defclause arithmetic, arc 237.8b).
3. **The dotted clojure-form** — `wat.type/i64`, `wat.core/+` (segments by `.`, name by a single
   `/`) instead of `:wat::core::i64`. This is the EDN-faithful rendering and the **post-keyword**
   surface: it's what a type *looks like* once types stop being `:`-keywords. Directly coupled to
   the keyword-as-type retirement (see [[NOTE-generic-bracket-syntax-edn]]).

## The clojure reference

core.typed (clojure.core.typed):
- Types live under the `t` alias: `t/Int`, `t/Str`, `t/Bool`, `t/Vec`, `t/Map`, … — a dedicated
  **type namespace**, exactly the `:wat::type::` proposal.
- Annotation uses **`:-`**: `(let [a :- t/Int, 1] …)`, `(fn [x :- t/Int] :- t/Int (inc x))`, and
  `(ann v t/Int)`.
- The function-type arrow is `->` inside `[Params -> Return]`.

So the proposed wat form is `(fn [x :- wat.type/i64] -> wat.type/i64 (wat.core/+ x 2))` — a
near-verbatim core.typed annotated-fn, with `wat.type/` for `t/` and `wat.core/` for `clojure.core/`.

## Why coupled — and the sequencing caution

These three moves are facets of one larger change: **what a type IS and how it's spelled** once the
clojure-ination finishes. The dotted form (`wat.type/i64`) only makes sense if types stop being
keywords — which is the keyword-as-type retirement that [[NOTE-generic-bracket-syntax-edn]] argues
dissolves the generic-bracket lexer problem too. So:

- The **`:-` swap** is the most independent (a parser/lexer change to the annotation token;
  a bounded HARD-CUT cascade across every `fn`/`defn`/`defclause` signature + `wat/*.wat` + tests).
  Could ship alone, but is cheap to fold into the larger type-syntax arc.
- The **`:wat::type::` namespace split** moves the type atoms out of `:wat::core::` — a rename
  cascade (every `:wat::core::i64` type-position → `:wat::type::i64`), distinct from operator
  positions. Needs care: the SAME token `:wat::core::i64` is a type in annotation position; the
  split must distinguish type-position from value-position (which connects to the
  `WatAST::Keyword` type/value context-polymorphism named as the `src/ast/` ward-enabler).
- The **dotted form** is the biggest reader-surface change and the most coupled to killing
  keywords-as-types — almost certainly the *last* move, landing with that retirement.

**Recommended (for the deciding arc, not locked here):** treat all three as one *type-form* arc
that lands after (or with) the keyword-as-type retirement, so the surface changes once, not three
times. Don't ship `:-` in isolation if the `wat.type/` + dotted form are coming right behind it —
that's three churns of every signature in the corpus.

## Open questions

- **`:-` vs `<-` cascade size:** every annotated parameter in `wat/*.wat` + `src/` synthesis +
  every test fixture. Bounded + mechanical (substrate-as-teacher), but corpus-wide.
- **Type/value disambiguation for the `type` split:** `:wat::core::i64` in annotation position is a
  type; the same lexeme elsewhere may be a value. Splitting to `:wat::type::i64` requires the
  parser to know position — the `WatAST::Keyword` type/value split (the named `src/ast/` arc).
- **Composite/parametric types:** `Vector<T>`, `Option<T>`, `Result<T,E>`, user records — do they
  move to `:wat::type::` too (`wat.type/Vector`)? And how does the dotted form render generics
  (`wat.type/Vector<wat.type/i64>` vs an EDN bracket form — see [[NOTE-generic-bracket-syntax-edn]])?
- **Colon-keyword → dotted:** is the dotted `wat.type/i64` the *only* form (HARD CUT the
  `:wat::core::i64` keyword), or do both read for a transition? One-canonical-path says one form.

## Refs

- Sibling notes: [[NOTE-generic-bracket-syntax-edn]] (the generic `<...>` facet + keyword-as-type
  retirement), [[NOTE-type-decl-def-prefix-renames]] (the `def<noun>` declarator family).
- The dialect-honesty telos: arc 247 (fn-first) + arc 249 (threading, total-pure macros).
- The type/value context-polymorphism enabler: the `WatAST::Keyword` split (named the `src/ast/`
  ward-enabler in arc 244's close).
- core.typed: `:-` annotation + the `t/` type namespace.
