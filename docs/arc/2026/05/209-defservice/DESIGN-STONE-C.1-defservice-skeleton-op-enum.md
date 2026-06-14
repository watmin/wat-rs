# DESIGN-STONE C.1 — defservice skeleton + the op enum (the foundation)

> Stone C = mint `:wat::service::defservice` (a PURE-WAT defmacro in `wat/service.wat`) that
> generates the op enum + the `select'`/`poll'` dispatch loop + the client wrappers from a flat
> `:state` + `:ops` surface. It's the boss; **C.1 is the first sub-stone: the macro skeleton +
> the OP ENUM only.** C.2 adds the dispatch loop; C.3 the client wrappers + start fn; Stone D the
> counter proof. Spec: `DESIGN-REGROUNDED-2026-06-12.md`. Models on disk: **`wat/Record.wat`** for the do-block
> STRUCTURE ONLY (a defmacro emitting `` `(:wat::core::do (def…)(def…))``) — but **NOT** its
> AST walk (it uses the OLDER holon-reflection crutch; see feasibility note 2). For the AST walk,
> the model is **`wat/fix.wat`** + **`cond`** (WatAST-native: `ast->children`/`ast-kind`/
> `with-children`/`first`/`rest`/`map`). **`crates/wat-lru/wat/lru/CacheService.wat`** `loop-step`
> is the dispatch-loop model (for C.2).

## Feasibility — GROUNDED 2026-06-13 (both premises verified on disk)

1. **A defmacro CAN emit multiple top-level defns** — via `` `(:wat::core::do (def …) (def …))``.
   `Record.wat:95,192` prove it (defrecord/defstruct emit a `do` of a recordtype + defns; a
   top-level `do` of defns registers them all). defservice emits
   `(do (defenum …) (defn loop …) (defn wrappers …) (defn start …))`.
2. **WatAST-native AST tooling exists** (arc-251.5a homoiconic bridge) — `:wat::core::ast-kind`,
   `:wat::core::ast->children`, `:wat::core::with-children`, + `first`/`rest`/`second`/`List?`/
   `empty?`/`map`/`take`/`drop`/`concat` + quasiquote `~`/`~@` + `:wat::core::macro-error`.
   **THIS is how defservice walks `:ops` — NOT holon reflection.** ⚠️ DO NOT copy `Record.wat`'s
   `from-wat → Bundle/children → Vector/get` pattern: that is the OLDER holon-reflection crutch
   (predates 251.5a; converts WatAST→holon to index-walk). The holon IR is VSA/semantic
   machinery — using it to walk a plain macro arg-vector is the holon-crutch abuse
   ([[feedback_honest_abstraction_decomplect_crutch_open_seam]]). **The model is `cond` +
   `wat/fix.wat`** (both walk WatAST natively). And because `:ops` carries the op signatures
   INLINE, defservice reads the surface directly — it needs NO reflection at all (not
   `from-wat`, not `signature-of-fn`/`extract-arg-types`; those reflect *separate* handler defns
   we don't have). Fully WatAST-native.

## The C.1 deliverable

`(:wat::service::defservice <fqdn> :state <T> :ops [<op-clauses>])` expands (C.1) to a `do`-block
whose only generated form (for now) is the **op enum** `<fqdn>::Op` — one variant per op, each
carrying the op's **client args** (the handler args MINUS the leading state-self `s <- :State`).

Example:
```clojure
(:wat::service::defservice :my::counter
  :state :wat::core::i64
  :ops [(Get       [s <- :State]                -> (:wat::core::Tuple :State :wat::core::i64))
        (Increment [s <- :State n <- :wat::core::i64] -> (:wat::core::Tuple :State :wat::core::i64))])
```
C.1 emits:
```clojure
(:wat::core::do
  (:wat::core::defenum :my::counter::Op
    :Get
    :Increment [n <- :wat::core::i64]))
```
`:Get` → no client args (only `s`) → bare variant. `:Increment` → client arg `n` → `[n <- :i64]`.

## The ONE contract decision (pinned — flag for builder/intueri at draw)

- **`:ops` grammar:** a Vector of op-clauses, each a **List** `(OpName [s <- :State …client-args]
  -> RetType …body)`. State-as-self: the FIRST handler arg is `s <- :State` (the protected state);
  the remaining args are what the client sends (→ the enum variant fields). The `->`/RetType/body
  are consumed by C.2 (the loop), not C.1. **(The regrounded spec wrote `:ops` flat — without
  per-op parens; pinning each op as a List is cleaner to parse + matches the defclause style.
  CONFIRM this grammar with the builder when C.1 is drawn — it is the user-facing surface.)**
- **Enum name:** `<fqdn>::Op` (append `::Op` to the service fqdn). Variants are the op names as
  keywords (`:Get`, `:Increment`). **(→ intueri may refine `Op` vs `Request`/`Msg` at draw.)**
- **FQDN doctrine** (per `Record.wat`): the macro NEVER inserts into `:user::*`/an auto-namespace;
  the user supplies `<fqdn>`; the enum is `<fqdn>::Op`.
- **`:State` placeholder:** `:State` in the handler args is the literal sugar for the declared
  `:state <T>` type. C.1 doesn't need to resolve it (it strips the first arg); C.2/the handlers do.

## The algorithm (WatAST-native — model: `cond` + `wat/fix.wat`, NOT `Record.wat`)

```clojure
(:wat::core::defmacro :wat::service::defservice
  [fqdn  <- :AST<...>
   _state-kw <- :AST<...>  state-ty <- :AST<...>     ;; the :state <T> pair
   _ops-kw   <- :AST<...>  ops      <- :AST<...>]     ;; the :ops [<clauses>] pair
  -> :AST<wat::core::nil>
  `(:wat::core::do
     (:wat::core::defenum ~(<fqdn>::Op keyword — build via keyword/of or string-append on fqdn)
        ~@(<expand-time>: for each op-clause in `ops` (walk via `ast->children`/`map`):
              the clause is a List → `ast->children` = [OpName, arg-vec, ->, ret, body…];
              OpName = (first children) ; arg-vec = (second children), itself a vector →
              `ast->children` = the triples [s,<-,:State, name,<-,type, …] ; DROP the first
              triple (the `s <- :State` self arg) ; emit the variant: `:OpName` then, if client
              args remain, splice the remaining triples as `[…client-args]`))))
```
WatAST-native tooling to use (the `cond`/`fix.wat` way): `(:wat::core::ast->children node)`,
`(:wat::core::ast-kind node)`, `(:wat::core::first …)`/`(:wat::core::second …)`/`(:wat::core::rest …)`,
`(:wat::core::map …)`, `(:wat::core::take …)`/`(:wat::core::drop …)` for chunking the triple
sequence, quasiquote `~`/`~@`, `(:wat::core::macro-error …)` on malformed `:ops`. **NO `from-wat`,
NO `Bundle/children`, NO `signature-of-fn`** — read the surface directly.

## Files touched

NEW `wat/service.wat` (the `defservice` defmacro; registered like other stdlib defmacros — see
`wat/core.wat:26` `register_stdlib_defmacros` + how `wat/*.wat` are concatenated/loaded; confirm
`service.wat` is added to the stdlib load list — likely `src/macros/parse.rs` or the wat-source
manifest). The probe. NO Rust change expected (pure-wat defmacro) — but VERIFY the stdlib load
list includes a new `wat/service.wat` (a likely small Rust/manifest edit to register it).

## The gate (probe — RED at HEAD: `defservice` is an unknown macro)

`tests/…/probe_arc209_c1_defservice_op_enum` (nursery or top-level): a program that `defservice`s
a `:my::counter` with `:ops [(Get [s <- :State] -> …) (Increment [s <- :State n <- :i64] -> …)]`,
then CONSTRUCTS + MATCHES the generated enum: `(:my::counter::Op::Increment 5)` builds, and a
`match` on it extracts `n == 5`; `(:my::counter::Op::Get)` builds. Asserts the op enum exists +
its variants carry the right client args. RED at HEAD (`defservice` unknown macro → expand error).

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** a new `wat/service.wat` can't be registered into the stdlib defmacro load pass —
   STOP, report where the load list lives (the macro won't be found otherwise).
2. **STOP-2:** the WatAST-native walk (`ast->children`/`first`/`take`/`drop`/`map` over `:ops` +
   the arg-vec triples, drop the self arg, splice variant fields) can't express the op-enum
   emission — STOP, report the gap. (Do NOT fall back to `from-wat`/`Bundle/children` — that is
   the holon crutch this design rejects. If the native tooling has a true gap, that gap is its
   own stone, not a reason to reach for holon.)
3. **STOP-3:** the `:ops`-as-List-of-Lists grammar conflicts with how the builder wants the
   surface — STOP (this is the flagged contract decision; confirm before building).

## Out of scope (C.2+ / rejected here)

The dispatch loop (C.2 — `select'`/`poll'` over the op enum, grow/shrink, TCO; CacheService model),
the client wrappers + start fn (C.3), handler-contract validation via `signature-of-fn` reflection
(folds into C.2/C.3 where the handler bodies are emitted), per-op identity policy, process tier,
the proof (Stone D). C.1 is ONLY: the macro parses the surface + emits a correct op enum.
