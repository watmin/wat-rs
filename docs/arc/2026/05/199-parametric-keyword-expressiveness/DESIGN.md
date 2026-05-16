# Arc 199 — Parametric-keyword expressiveness in defmacro

**Direction:** vend an expand-time mechanism that lets a `:wat::core::defmacro` construct or splice into a parametric type keyword (`:Receiver<I>`, `:Sender<O>`, `:Vector<T>`, etc.) so the user can pass JUST the type arg (`I`) without spelling out the full wrapper at every call site.

**Status:** DESIGN. Not yet open for implementation. Implementation gated on a four-questions pass over the candidate mechanisms.

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
