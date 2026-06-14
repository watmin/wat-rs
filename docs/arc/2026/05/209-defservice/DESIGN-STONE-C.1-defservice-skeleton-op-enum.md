# DESIGN-STONE C.1 — defservice skeleton + the op enum (the foundation)

> Stone C = mint `:wat::service::defservice` (a PURE-WAT defmacro in `wat/service.wat`) that
> generates the op enum + the `select'`/`poll'` dispatch loop + the client wrappers from a flat
> `:state` + `:ops` surface. It's the boss; **C.1 is the first sub-stone: the macro skeleton +
> the OP ENUM only.** C.2 adds the dispatch loop; C.3 the client wrappers + start fn; Stone D the
> counter proof. Spec: `DESIGN-REGROUNDED-2026-06-12.md`. Models on disk: **`wat/Record.wat`**
> (`Record::def` — the do-block-of-defns defmacro pattern) + **`crates/wat-lru/wat/lru/CacheService.wat`**
> `loop-step` (the dispatch loop, for C.2).

## Feasibility — GROUNDED 2026-06-13 (both premises verified on disk)

1. **A defmacro CAN emit multiple top-level defns** — via `` `(:wat::core::do (def …) (def …))``.
   `Record.wat:95,192` prove it (defrecord/defstruct emit a `do` of a recordtype + defns; a
   top-level `do` of defns registers them all). defservice emits
   `(do (defenum …) (defn loop …) (defn wrappers …) (defn start …))`.
2. **Reflection exists** — `:wat::runtime::extract-arg-types` / `extract-arg-names` /
   `signature-of-fn` (`src/macros/eval.rs:557`; used live in `wat/kernel/run_threads.wat:135`).
   Plus the holon-AST reflection `Record.wat` uses at expand time: `:wat::holon::from-wat`,
   `:wat::holon::Bundle/children`, `:wat::holon::statement-length`, `:wat::core::map`,
   `:wat::core::Option/expect`, quasiquote `~`/`~@`, `:wat::core::macro-error`.

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

## The algorithm (mirror `Record.wat:92-115`)

```clojure
(:wat::core::defmacro :wat::service::defservice
  [fqdn  <- :AST<...>
   _state-kw <- :AST<...>  state-ty <- :AST<...>     ;; the :state <T> pair
   _ops-kw   <- :AST<...>  ops      <- :AST<...>]     ;; the :ops [<clauses>] pair
  -> :AST<wat::core::nil>
  `(:wat::core::do
     (:wat::core::defenum ~(<fqdn>::Op keyword, built via keyword/of or string-append)
        ~@(<expand-time let>: from-wat ops → Bundle/children → for each op-clause:
              extract OpName (first) + the args-vec (second) ; drop the leading `s <- :State`
              triple ; emit the variant: `:OpName` then, if client args remain, `[...client-args]`))))
```
Reflection details to copy from `Record.wat`: `(:wat::holon::from-wat (:wat::core::quote ops))`,
`(:wat::holon::Bundle/children …)`, `(:wat::holon::statement-length …)`, `(:wat::core::map …)`,
`(:wat::core::Option/expect -> :wat::holon::HolonAST …)`. The variant args are a field-triple
vector exactly like `Record::def`'s `[name <- type …]` parsing — reuse that field-walk, just
DROP the first triple (the `s <- :State` self arg) per op.

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
2. **STOP-2:** the expand-time field-walk (drop the self arg, emit variant fields) can't reuse the
   `Record::def` reflection pattern — STOP, report the gap (it should — same holon-AST tooling).
3. **STOP-3:** the `:ops`-as-List-of-Lists grammar conflicts with how the builder wants the
   surface — STOP (this is the flagged contract decision; confirm before building).

## Out of scope (C.2+ / rejected here)

The dispatch loop (C.2 — `select'`/`poll'` over the op enum, grow/shrink, TCO; CacheService model),
the client wrappers + start fn (C.3), handler-contract validation via `signature-of-fn` reflection
(folds into C.2/C.3 where the handler bodies are emitted), per-op identity policy, process tier,
the proof (Stone D). C.1 is ONLY: the macro parses the surface + emits a correct op enum.
