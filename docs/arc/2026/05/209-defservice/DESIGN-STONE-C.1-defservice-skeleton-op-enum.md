# DESIGN-STONE C.1 — defservice skeleton + the op enum (the foundation)

> Stone C = mint `:wat::service::defservice` (a PURE-WAT defmacro in `wat/service.wat`) that
> generates the op enum + the `poll'`/`select'` dispatch loop + the client wrappers from a flat
> `:state` + `:ops` surface. It's the boss; **C.1 is the first sub-stone: the macro skeleton +
> the OP ENUM only.** C.2 adds the dispatch loop; C.3 the client wrappers + start fn; Stone D the
> counter proof. Spec: `DESIGN-REGROUNDED-2026-06-12.md`.

## Surface — settled 2026-06-13/14 (four-questions, builder-ratified)

**Option A — each op is a self-contained List with the body INLINE; op-heads are KEYWORDS.**
The whole service is one form; the macro reads the inline surface directly (zero handler-defn
reflection). The regrounded spec's "signatures reflected from separate handler defns" line is
RETIRED (that was option C; cut).

```clojure
(:wat::service::defservice :my::counter
  :state :wat::core::i64
  :ops
  [(:Get [s <- :State]
         -> (:wat::core::Tuple :State :wat::core::i64)
     (:wat::core::Tuple s s))

   (:Increment [s <- :State n <- :wat::core::i64]
               -> (:wat::core::Tuple :State :wat::core::i64)
     (:wat::core::let [s' (:wat::core::i64::+ s n)]
       (:wat::core::Tuple s' s')))])
```

Four-questions on the surface: A is Obvious (one form, each op reads like a `defn`/`cond`-clause)
+ Simple (one source of truth, one WatAST walk, zero reflection) + Honest (types+bodies all
visible inline). Keyword op-heads (`:Get`, not `Get`) win Obvious (every wat name is a keyword —
`defn`/`defenum`/`defstruct`) + Simple (`(first clause)` IS the variant keyword, no conversion)
and flip to bare symbols for free at the arc-251 cutover. Option C (reflect separate defns) fails
Simple (N+1 forms + re-grows `signature-of-fn`); B (flat juxtaposition, no per-op parens) fails
Obvious/Simple once bodies are inline (no delimiter) → degenerates to C.

## ✅ SHIPPED 2026-06-14 — delta: bare `defenum`, not a `do`-wrapper

The DESIGN sketched the emission as `` `(:wat::core::do (defenum …))``. That FAILS for C.1: the
freeze pipeline's `splice_type_decls` (types.rs:1502-1520) registers each type-decl child of a
`do` and does NOT re-push it, so a `do` containing *only* a `defenum` collapses to
`(:wat::core::do)` → fails "do form requires at least one form" (check.rs:7419). C.1 therefore
emits a **bare `defenum`** (a valid top-level form). The `do`-wrapper returns at C.2/C.3 — once
the loop + client wrappers are siblings, the `do` is non-empty (the `defenum` still splices to top
level, the rest stay in the `do`). Grounded on disk during the weigh; the algorithm below shows
the shipped form.

## C.1 deliverable — the op enum ONLY

`(defservice <fqdn> :state <T> :ops [<op-clauses>])` expands (C.1) to a `do`-block whose only
generated form (for now) is the **op enum** `<fqdn>::Op` — one variant per op, each carrying the
op's **client args** (the handler args MINUS the leading state-self `s <- :State`). The above
emits:

```clojure
(:wat::core::do
  (:wat::core::defenum :my::counter::Op
    :Get
    :Increment [n <- :wat::core::i64]))
```

`:Get` handler args `[s <- :State]` → only the self-arg → **bare (fieldless) variant**.
`:Increment` handler args `[s <- :State n <- :i64]` → drop `s` → variant field `[n <- :i64]`.
The `->`/RetType/body are consumed by C.2 (the loop), not C.1 — C.1 reads but ignores them.

## Feasibility — GROUNDED + DE-RISKED on disk (2026-06-14)

Three premises, all proven by probe (NOT assumed — FM-2-bis):

1. **A defmacro CAN emit a `do` of top-level forms** via `` `(:wat::core::do …)``. `Record.wat`
   proves it (defrecord emits a do-block; a top-level `do` registers each child). C.1 emits a
   `do` of one `defenum`; C.2/C.3 add the loop + wrappers as siblings in the same `do`.

2. **The WatAST AST-walk tooling is now available IN a defmacro** — ⚠️ it was NOT, and FM-2-bis
   caught it. The arc-249 total-pure macro fence (`is_pure_total`, `src/macros/eval.rs`) is
   DEFAULT-DENY; the arc-251.5a bridge ops (`ast->children`/`with-children`/`ast-kind`/… ) were
   never admitted. **Stone C.1-pre (`4718c897`) added the 11 pure-total bridge ops to the fence.**
   The original DESIGN's "walk `:ops` with `ast->children` like `fix.wat`" was FALSE in macro
   context — `fix.wat`'s walkers are RUNTIME `defn`s, not defmacros. Probe
   `probe_arc209_c1_defmacro_ast_walk` proves the composition now works.

3. **THE MACRO CONTRACT (the trap the foundation probe surfaced):** a node-walking defmacro must
   use the **PROGRAM-BODY path** — top-level a regular form (`let`/`if`/`do`), params referenced
   as `wat__WatAST` node-values, output built with **NESTED** quasiquote. A **top-level**
   quasiquote `` `~(…)`` routes to the quasiquote path, which EVALUATES the arg — handing
   `ast->children` an evaluated Vector *value* instead of the *node* (the exact RED the probe hit).
   **`cond` is the canonical shape** (top-level `if`, params walked as data, nested `` `(…)``).

**NO `from-wat`/`Bundle/children` (the older holon-reflection crutch). NO `signature-of-fn`** —
`:ops` carries the signatures inline; defservice reads the surface directly.

## The ONE contract decision (pinned)

- **`:ops` grammar:** a Vector of op-clauses, each a **List** `(:OpName [s <- :State …client-args]
  -> RetType body)`. State-as-self: the FIRST handler arg is `s <- :State`; the rest are the
  client args → the enum variant fields. Op-heads are **keywords** (`:Get`).
- **Enum name:** `<fqdn>::Op` — built `(keyword/from-string (string::concat (keyword/to-string
  fqdn) "::Op"))` (the `keyword/of` pattern; `keyword/to-string` drops the colon, `keyword/from-
  string` re-adds it). Variants are the op-head keywords verbatim.
- **`:State` placeholder:** literal sugar for the declared `:state <T>`. C.1 doesn't resolve it
  (it strips the first arg triple); C.2/handlers do.
- **FQDN doctrine** (per `Record.wat`): the user supplies `<fqdn>`; the macro never auto-namespaces.

## The algorithm (WatAST-native, program-body path — model: `cond` + `keyword/of` + the foundation probe)

```clojure
(:wat::core::defmacro :wat::service::defservice
  [fqdn      <- :wat::WatAST     ;; :my::counter
   _state-kw <- :wat::WatAST     ;; the literal :state marker (ignored)
   state-ty  <- :wat::WatAST     ;; :wat::core::i64  (C.2 uses; C.1 ignores)
   _ops-kw   <- :wat::WatAST     ;; the literal :ops marker (ignored)
   ops       <- :wat::WatAST]    ;; the [ (:Get …) (:Increment …) ] vector NODE
  -> :wat::WatAST
  ;; PROGRAM-BODY path: top-level `let`, params are node-values, nested quasiquote at the end.
  (:wat::core::let
    [enum-name (:wat::core::keyword/from-string
                 (:wat::core::string::concat (:wat::core::keyword/to-string fqdn) "::Op"))
     clauses   (:wat::core::ast->children ops)            ;; list of op-List nodes
     variants  (:wat::core::foldl <op->tokens> (:wat::core::Vector :wat::WatAST) clauses)]
    `(:wat::core::defenum ~enum-name ~@variants)))   ;; SHIPPED: bare defenum, NOT a do-wrapper
```

`<op->tokens>` folds each op-clause into the flat variant-token vector (avoids a separate
flatten/`concat`):

```clojure
(:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST> clause <- :wat::WatAST]
  -> :wat::core::Vector<wat::WatAST>
  (:wat::core::let
    [ch      (:wat::core::ast->children clause)
     opkw    (:wat::core::Option/expect -> :wat::WatAST (:wat::core::first ch)
               "defservice: op-clause has no head")
     argvec  (:wat::core::Option/expect -> :wat::WatAST
               (:wat::core::first (:wat::core::drop ch 1)) "defservice: op-clause has no arg-vec")
     fieldch (:wat::core::drop (:wat::core::ast->children argvec) 3)]   ;; drop the `s <- :State` triple
    (:wat::core::if (:wat::core::empty? fieldch)
      (:wat::core::conj acc opkw)                                        ;; bare variant
      (:wat::core::conj (:wat::core::conj acc opkw)
                        (:wat::core::with-children argvec fieldch)))))   ;; :OpName [fields]
```

Every head used is fence-whitelisted (post C.1-pre): `ast->children`, `with-children`, `first`,
`drop`, `empty?`, `conj`, `foldl`, `keyword/from-string`, `keyword/to-string`, `string::concat`,
`Vector`, `Option/expect`, `if`, `let`, `macro-error`. The nested `` `(:wat::core::do …)``
quasiquote splices `~enum-name` (a keyword node) + `~@variants` (the flat token vector).

## Files touched

- NEW `wat/service.wat` — the `defservice` defmacro.
- `src/stdlib.rs` — ONE `WatSource` entry registering `wat/service.wat` (place after `core.wat`/
  `fix.wat`; **order is not load-bearing** — `register_stdlib_defmacros` walks the whole
  concatenated stdlib in one pre-expansion pass, comment at stdlib.rs:231-236). STOP-1 dissolved.
- The probe is on disk (`tests/probe_arc209_c1_defservice_op_enum.rs`, RED-verified at HEAD).
- NO other Rust change (the fence prereq already shipped at `4718c897`).

## The gate (probe — RED at HEAD on exactly the gap)

`tests/probe_arc209_c1_defservice_op_enum.rs` — `defservice`s the counter (surface A) then
constructs `(:my::counter::Op::Increment 5)` + matches both variants (bare `:Get`, payload
`:Increment` extracts `n`). RED at HEAD = `UnresolvedReference :my::counter::Op::Increment` (the
enum doesn't exist). GREEN once C.1 emits it.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1 (DISSOLVED):** `wat/service.wat` registration — proven trivial + order-independent.
2. **STOP-2 (DISSOLVED):** the WatAST walk in a defmacro — proven by `probe_arc209_c1_defmacro_ast_walk`
   after the C.1-pre fence stone. If a NEW gap appears (a head still `RefusedInMacro`), report it
   — do NOT fall back to `from-wat`/holon reflection.
3. **STOP-3:** the program-body contract breaks (a value reaches `ast->children` instead of a
   node) — you used a top-level quasiquote; restructure to `cond`'s shape (top-level `let`/`if`,
   nested quasiquote). Report if it cannot be expressed that way.

## Out of scope (C.2+ / rejected here)

The dispatch loop (C.2 — `poll'`/`select'` over the op enum; `CacheService.wat` `loop-step` +
the c0b1b/c0b3aii probes are the model), the client wrappers + start fn (C.3), handler-body
emission, per-op identity policy, process tier, the proof (Stone D). C.1 is ONLY: parse the
surface + emit a correct op enum.
