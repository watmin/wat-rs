# Arc 199 — Parametric-keyword expressiveness in defmacro — **REJECTED 2026-05-16**

> **REJECTED — substrate is already sufficient. No work needed.**
>
> Post-DESIGN-sketch investigation revealed every primitive arc 199 would have minted ALREADY EXISTS:
>
> - `:wat::core::keyword/from-string` (src/check.rs:11931) — String → keyword Value
> - `:wat::core::keyword/to-string` (src/check.rs:11923) — keyword → String
> - `:wat::core::string::concat` (src/check.rs:4653) — variadic String concat
> - **Computed unquote at macro expand time** (arc 143 slice 2, src/macros.rs:1010+) — `~(:keyword/op args...)` in a defmacro template substitutes macro params, runs `crate::runtime::eval` on the substituted expression at expand time, then `value_to_watast` converts the result to a `WatAST` node that lands at the `~(...)` position
> - `value_to_watast` (src/runtime.rs:8815) — `Value::wat__core__keyword(k) → WatAST::Keyword(k)` is the working conversion
>
> **Macro dialect reminder (Clojure-style):**
> - `~` = unquote
> - `~@` = unquote-splicing
> - `,` = whitespace literal (commas are visual separator only, like Clojure)
>
> Some docs (including the arc 143 INSCRIPTION quoted below) use the classical Clojure `,` notation when DESCRIBING quasiquote semantics. The actual wat source uses `~`.
>
> ### Production evidence — arc 143 slice 6's `define-alias` macro
>
> From `wat/runtime.wat:22-29`:
>
> ```scheme
> (:wat::core::defmacro
>   (:wat::runtime::define-alias
>     (alias-name :AST<wat::core::keyword>)
>     (target-name :AST<wat::core::keyword>)
>     -> :AST<wat::core::unit>)
>   `(:wat::core::define
>      ~(:wat::runtime::rename-callable-name ...)
>      (~target-name ~@(:wat::runtime::extract-arg-names ...))))
> ```
>
> This macro has been in production since arc 143 shipped (2026-05). It exercises the EXACT pattern arc 170 D1 needed: an inner expression at the `~(...)` position evaluates at expand time, calls arbitrary substrate primitives, and the resulting Value is converted to a `WatAST` node via `value_to_watast`.
>
> The arc 143 INSCRIPTION at `docs/arc/2026/05/143-define-alias/INSCRIPTION.md:116-123` documents the keyword↔symbol distinction in this flow — confirming both the path works AND the failure modes are well-understood.
>
> ### Originating signal post-mortem
>
> Arc 170 Stone D1's authoring miss: sonnet documented the verbose call form as a "substrate constraint" without reaching for the existing computed-unquote + keyword-construction pattern. The verbose workaround landed in production (commit `d704820`); arc 199 was opened (commit `d6d9cc4`) to fix what looked like a substrate gap.
>
> **The substrate gap doesn't exist.** Stone D1's verbose form is a missed substrate-machinery discovery, not a missing substrate primitive.
>
> ### What this means for arc 170
>
> - D1 gets refactored to the clean call form `(run-threads :I :O factory client-fn)` via the computed-unquote pattern
> - D2 (multi-factory) builds on the cleaner shape — no longer blocked by arc 199 (since arc 199 retires)
> - D3 (panic cascade) follows D2
> - Stone E (`run-processes`) mirrors the cleaned-up D family
>
> ### Lesson captured
>
> Before opening a substrate arc, **investigate existing substrate machinery for the pattern in question.** Arc 199's DESIGN sketch was drafted without first grepping `keyword/from-string` + `computed unquote` + walking the macro expander to see how arc 143 slice 2 wired arbitrary expand-time eval. The four-questions on Candidate 1 vs Candidate 2 vs Candidate 3 spent cycles on a non-problem.
>
> Discipline anchor: `feedback_assertion_demands_evidence` — every assertion the substrate is missing X needs evidence the substrate doesn't have X. Grep + read the relevant primitives BEFORE opening the arc.
>
> See also: `feedback_no_new_types` — the constructor-verb reflex caught at D1 BRIEF-time fired again here at arc 199 DESIGN-time. The fix is even more upstream than no-new-types: don't open new substrate arcs without first proving the existing substrate doesn't already solve it.
>
> ---
>
> **The original DESIGN text below is preserved as the historical artifact of what this arc proposed before evidence rejected it. Inscribed per `feedback_inscription_immutable` — what is inscribed is inscribed; we do not hide our faults, we learn from them.**

---

**Direction (HISTORICAL — REJECTED 2026-05-16):** vend an expand-time mechanism that lets a `:wat::core::defmacro` construct or splice into a parametric type keyword (`:Receiver<I>`, `:Sender<O>`, `:Vector<T>`, etc.) so the user can pass JUST the type arg (`I`) without spelling out the full wrapper at every call site.

**Status:** REJECTED 2026-05-16 — substrate already provides this. Original DESIGN preserved below as historical record.

**Originating signal:** arc 170 Stone D1 (commit `d704820`) shipped the `run-threads` defmacro with a 4-arg form that forces callers to spell out the full `:Receiver<I>` / `:Sender<O>` type keywords. The macro author "knows" those wrappers but cannot construct them at expand time. Same constraint surfaced earlier at `:wat::test::run-hermetic-with-io` (`wat/test.wat:800-815`). Will surface again in Stone E (`run-processes`), in any user-side concurrency macro, in any generic-type wrapper macro.

---

## The substrate constraint (current state, post-D1)

`wat::parser` tokenizes parametric type keywords `<...>` **atomically**. `:Receiver<I>` is a single keyword token at parse time; the `<I>` portion is part of the keyword string, not a list-structured AST child.

Consequence at quasiquote expand time:

```scheme
(:wat::core::defmacro
  (some-macro (the-type :AST<wat::core::nil>) -> :AST<wat::core::nil>)
  `(:wat::core::fn [x <- :Receiver<~the-type>]   ;; ❌ ~ does NOT splice into <>
    -> :wat::core::nil
    x))
```

The `~the-type` doesn't reach inside the `<>` because `:Receiver<...>` is one atomic token to the parser. The macro author can't construct `:Receiver<I>` from `:I` at expand time.

Workaround (D1's path): caller passes the full wrapper:

```scheme
(:wat::core::defmacro
  (some-macro (server-rx-type :AST<...>) -> :AST<...>)
  `(:wat::core::fn [x <- ~server-rx-type] -> :wat::core::nil x))

;; caller writes:
(some-macro :Receiver<i64>)            ;; D1's form — verbose
```

vs. the cleaner form arc 199 would enable:

```scheme
(some-macro :i64)                       ;; arc 199 — pass JUST the type arg
```

## Why it's load-bearing

- `:wat::test::run-hermetic-with-io` documents the constraint at `wat/test.wat:800-815` — known and lived with for many months
- `:wat::kernel::run-threads` D1 hits it (this arc 199's originating signal)
- Stone E `:wat::kernel::run-processes` will hit it
- Stones D2/D3 build on D1 and inherit the verbose call form unless arc 199 lands first
- Any user-side macro wrapping a parametric type hits it

The cost compounds with every new parametric-type macro the substrate or users author.

## Four-questions on candidate fixes (informal, settled later in formal DESIGN pass)

### Candidate 1 — Expand-time AST keyword constructor (`:wat::ast::parametric-keyword`)

Substrate adds a new AST-construction primitive:

```scheme
(:wat::ast::parametric-keyword :head :Receiver :args [:I])
;; → AST node `:Receiver<I>`
```

Macros opt in by calling the constructor in quasiquote:

```scheme
(:wat::core::defmacro
  (some-macro (the-type :AST<...>) -> :AST<...>)
  (:wat::core::let
    [rx-type (:wat::ast::parametric-keyword
               :head :rust::crossbeam_channel::Receiver
               :args [the-type])]
    `(:wat::core::fn [x <- ~rx-type] -> :wat::core::nil x)))
```

- Obvious: marginal — explicit but adds a primitive macros must remember to call
- Simple: YES — one new substrate verb; clear contract; no parser change
- Honest: YES — the construction is visible; reader sees `parametric-keyword :Receiver ...`
- Good UX (macro author): marginal — explicit but verbose
- Good UX (macro caller): YES — caller passes `:I` not `:Receiver<I>`

→ YES with marginal-obvious + marginal-author-UX. Open question: is the verbosity worth the explicitness?

### Candidate 2 — `~` splice inside `<>` at expand time (tokenizer/quasiquote change)

Parser changes (or quasiquote handler changes) so `:Receiver<~the-type>` IS valid at expand time, and `~the-type` is interpolated into the keyword token.

Macros need no opt-in; existing quasiquote magic just works:

```scheme
(:wat::core::defmacro
  (some-macro (the-type :AST<...>) -> :AST<...>)
  `(:wat::core::fn [x <- :Receiver<~the-type>] -> :wat::core::nil x))
```

- Obvious: YES — quasiquote already splices `~` everywhere else; `<>` is "just another nesting"
- Simple: NO — parser must distinguish "atomic keyword token" from "splice-bearing quasiquote keyword token" — context-sensitive parse
- Honest: marginal — magic; reader doesn't see the construction
- Good UX (macro author): YES — no opt-in; existing quasiquote pattern
- Good UX (macro caller): YES — caller passes `:I` not `:Receiver<I>`

→ Loses on Simple. Per `feedback_refuse_easy_solutions`: the cleaner-looking path that requires a parser-context change is the wrong reflex.

### Candidate 3 — Hybrid: keyword-construction verb + sugar reader macro

Same as Candidate 1, plus a reader-time macro that REWRITES `:Receiver<~T>` to `(:wat::ast::parametric-keyword :head :Receiver :args [T])` at expand time.

- Obvious: NO — two mechanisms; users have to know both
- Simple: NO — substrate has primitive + reader macro
- Honest: NO — reader macro hides the primitive
- Good UX: marginal — pleasant front but hidden machinery

→ Disqualified.

## Initial lean (subject to formal DESIGN pass)

**Candidate 1 (expand-time AST keyword constructor).** Wins YES YES YES YES on the substrate-correctness axes; loses only on macro-author verbosity. The verbosity is acceptable because:

- macro authors are a small population (substrate maintainers + a few power users)
- macro callers are the broad population (every line of user code calling a macro)
- the explicit `parametric-keyword` construction REVEALS what's happening — `feedback_verbose_is_honest` applied to macro authoring

Candidate 2 (tokenizer/quasiquote change) loses on Simple; the context-sensitive parse is the failure mode.

## Out of scope for this DESIGN

- Implementation slices (those come post-DESIGN-pass with the user)
- The exact primitive name (`:wat::ast::parametric-keyword` is a working name; final naming via `/gaze`)
- Whether `:wat::core::struct->form` or `:wat::core::quasiquote` already cover part of this surface (verify in DESIGN pass)

## Consumer arc affected

**Arc 170** (`run-threads` D1 shipped with workaround; D2/D3 paused until arc 199 lands):

- D2 (multi-factory heterogeneous expansion): blocked-by arc 199 — the call form should be `(run-threads :I :O factory-A factory-B ... client-fn)` not the verbose D1 form
- D3 (panic cascade): blocked-by D2; transitively blocked-by arc 199
- Stone E (`run-processes`): blocked-by arc 199 — same constraint applies to Process peer
- Stone F (-with-io migration): unaffected (separate consumer sweep)
- Stones G/H: unaffected

Once arc 199 ships, refactor `run-threads` to the cleaner call form; D2/D3/E build on the post-199 shape.

## Status

DESIGN open. Formal pass + four-questions on Candidate 1 + decomposition into slices comes next.

---

## See also

- `wat/test.wat:800-815` — `:wat::test::run-hermetic-with-io` documented the constraint earlier
- `wat/kernel/run_threads.wat` (Stone D1) — the originating signal for arc 199
- `feedback_no_new_types` — STOP signal that prevented papering over the constraint with a per-macro workaround
- `feedback_verbose_is_honest` — the macro-author-verbosity acceptable for the expand-time constructor candidate
