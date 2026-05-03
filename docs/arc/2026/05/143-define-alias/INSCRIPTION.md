# Arc 143 — `:wat::runtime::define-alias` — INSCRIPTION

## Status

Shipped 2026-05-02 → 2026-05-03. Six sonnet sweeps; ~54 min wall-clock
across slices 1, 2, 3, 5b, 6, 7. Slice 8 is this paperwork.

The macro that motivated the arc — `(:wat::runtime::define-alias
:wat::list::reduce :wat::core::foldl)` — ships in `wat/list.wat`. Arc
130's `:wat::core::reduce` blocker (the diagnostic that motivated the
arc) is closed; the next arc 130 link (`:wat::core::Vector/len`)
surfaced cleanly and is arc 130's territory.

The reflection layer ships PARTIAL: it works for substrate primitives
that have TypeScheme registrations and for user defines and macros.
**It does NOT yet work for substrate primitives implemented via
hardcoded `infer_*` handlers** (length, get, conj, contains?, the
container constructors, etc.) — those remain invisible to
`:wat::runtime::signature-of`. **Arc 144** delivers the uniform
reflection foundation that closes that gap; arc 143's slice 6 length
test stays red as a regression canary that arc 144 will turn green.

## What this arc adds

### `:wat::runtime::*` — runtime reflection namespace (NEW)

A new top-level namespace alongside `:wat::core::*`,
`:wat::kernel::*`, `:wat::test::*`, `:wat::holon::*`, etc. Houses
runtime-discovery primitives + the wat-side macros that compose them.

### Substrate primitives — `:wat::runtime::lookup-callable`, `signature-of`, `body-of` (slice 1)

Three substrate-side reflection primitives that walk the
SymbolTable + on-demand CheckEnv to produce HolonAST descriptions of
known callables:

```scheme
(:wat::runtime::lookup-callable :wat::core::foldl)
;; → :Option<:wat::holon::HolonAST> wrapping the synthesized define form
;;   (head + body sentinel for substrate primitives; real body for user defines)

(:wat::runtime::signature-of :wat::core::foldl)
;; → :Option<:wat::holon::HolonAST> wrapping the signature head only
;;   (foldl<T,Acc> (acc :Acc) (vec :Vec<T>) (f :fn(Acc,T)->Acc) -> :Acc)

(:wat::runtime::body-of :user::my-add)
;; → :Option<:wat::holon::HolonAST> wrapping the body for user defines
;;   :None for substrate primitives (Rust-implemented; no wat body)
```

Lookup precedence: SymbolTable.functions first (user defines win
over substrate primitives where the same name exists), then
CheckEnv's TypeScheme registry (substrate primitives), else `:None`.

### Computed unquote in defmacro bodies (slice 2)

The macro expander used to treat defmacro bodies as
quasiquote-templates only — `,X` substituted bound parameters, but
`,(expr)` was not evaluated. Slice 2 added a minimal head-is-Keyword
heuristic: when the unquote argument is a List whose head is a
`WatAST::Keyword`, the expander substitutes bound bindings into the
list and EVALUATES the substrate call at expand-time, then converts
the result back to WatAST via `value_to_watast`. Pure template
substitution preserved when the head is not a callable shape.

```scheme
;; Pre-arc-143: ,X substitutes a bound param.
;; Post-arc-143: ,(:some::primitive a b) evaluates at expand-time.
;; Both shapes coexist; heuristic disambiguates by head.
```

The threading of the macro-expansion environment + symbol table
through `expand_template` / `walk_template` / `unquote_argument` /
`splice_argument` / `expand_macro_call` / `expand_form` /
`expand_once` / `expand_all` touched 5 Rust files (`src/macros.rs`,
`src/runtime.rs`, `src/check.rs`, `src/freeze.rs`, `src/resolve.rs`).

### `:wat::runtime::rename-callable-name` + `extract-arg-names` (slice 3)

Two HolonAST-surgery primitives userland macros need:

```scheme
(:wat::runtime::rename-callable-name
   <signature-bundle :HolonAST>
   <from-name :keyword>
   <to-name :keyword>)
;; → new bundle with the head's base name renamed; type-params + args + return preserved.

(:wat::runtime::extract-arg-names <signature-bundle :HolonAST>)
;; → :Vec<:wat::holon::HolonAST> of bare-symbol arg names (suitable for splicing as call positions).
```

Both pattern-match on `HolonAST::Bundle` per arc 057's polymorphic
schema. Both reuse arc 009's `name_from_keyword_or_lambda` helper to
handle the names-are-values quirk on keyword args.

### `value_to_watast` bridges `Value::holon__HolonAST` (slice 5b)

A 1-line addition to `value_to_watast` in `src/runtime.rs` that lets
the macro splicer accept HolonAST results from slice 3's manipulation
primitives:

```rust
Value::holon__HolonAST(h) => Ok(holon_to_watast(&h)),
```

Plus three latent bug fixes the load-bearing transition surfaced —
all in `src/runtime.rs`:

- `type_scheme_to_signature_ast` (slice 1) emitted param names as
  `WatAST::Keyword` instead of `WatAST::Symbol`. Fixed: param names
  emit as `Symbol` (the only shape `:wat::core::define`'s parser
  accepts in arg-name positions).
- `function_to_signature_ast` (slice 1) had the same Keyword-vs-Symbol
  bug for the user-define synthesis path. Same fix.
- `extract-arg-names` (slice 3) returned arg names as
  `Value::wat__core__keyword`. After value_to_watast conversion that
  becomes `WatAST::Keyword` — a literal, NOT a variable reference.
  The macro splice (`,@(extract-arg-names ...)`) needs Symbol
  references for the call positions to work. Fixed: returns
  `Value::holon__HolonAST(HolonAST::symbol(name))` so the new
  HolonAST arm in value_to_watast routes through `holon_to_watast`,
  which emits `WatAST::Symbol` for bare names.

The slice 5b sweep was scoped to "1-line fix + 1 unit test" and
shipped 4 fixes + 1 unit test. The honest scope expansion was within
file scope (`src/runtime.rs`) and necessary to achieve the brief's
load-bearing test transition. Documented in SCORE-SLICE-5b.md as the
discipline lesson "briefs that identify a fix to unblock a test
should anticipate the test surfacing adjacent latent bugs from prior
slices' code."

### `:wat::runtime::define-alias` macro (slice 6 — wat)

Pure-wat defmacro in NEW `wat/runtime.wat`:

```scheme
(:wat::core::defmacro
  (:wat::runtime::define-alias
    (alias-name :AST<wat::core::keyword>)
    (target-name :AST<wat::core::keyword>)
    -> :AST<wat::core::unit>)
  `(:wat::core::define
     ,(:wat::runtime::rename-callable-name
        (:wat::core::Option/expect -> :wat::holon::HolonAST
          (:wat::runtime::signature-of target-name)
          "define-alias: target name not found in environment")
        target-name
        alias-name)
     (,target-name ,@(:wat::runtime::extract-arg-names
                       (:wat::core::Option/expect -> :wat::holon::HolonAST
                         (:wat::runtime::signature-of target-name)
                         "define-alias: target name not found in environment")))))
```

Composes slices 1 + 3 (signature-of + rename-callable-name +
extract-arg-names) with slice 2's computed unquote to emit a fresh
`:wat::core::define` whose body is a single delegating call to the
target. Failure to look up the target name emits a panic at
expand-time naming the missing name.

### `:wat::list::reduce` + `:wat::list::fold` → `:wat::core::foldl` (slice 7 — application)

The arc's primary deliverable, in NEW `wat/list.wat`. Both `reduce`
and `fold` are opinionated user-facing aliases for the atomic
`:wat::core::foldl` primitive — readers reaching for either name
(Clojure's `reduce`, Haskell's `foldl`, Lisp's `fold`, JS's `reduce`,
Python's `reduce`, Ruby's `inject`) get the same delegating shape:

```scheme
(:wat::runtime::define-alias :wat::list::reduce :wat::core::foldl)
(:wat::runtime::define-alias :wat::list::fold   :wat::core::foldl)
```

`foldl` and `foldr` remain the atomic forms; `reduce` and `fold`
are the helper names.

Plus the arc 130 substrate consumers swap `:wat::core::reduce` for
`:wat::list::reduce` at:

- `crates/wat-lru/wat-tests/lru/CacheService.wat:213`
- `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat:251`

After this slice the arc 130 RELAND v1 stepping stone fails on a
DIFFERENT primitive (`:wat::core::Vector/len`), confirming the
`:reduce` gap is closed and the cascade has progressed to its next
link.

The clean separation between `wat/runtime.wat` (the macro itself —
runtime-discovery construct) and `wat/list.wat` (the application —
semantic-domain placement) sets the precedent for future
`:wat::list::*` accumulations as arc 109's namespace migration
continues.

## What this arc does NOT add

- **Reflection over hardcoded `infer_*` primitives** — `:wat::core::length`,
  `:wat::core::get`, `:wat::core::conj`, the container constructors
  (Vector / Tuple / HashMap / HashSet), `:wat::core::contains?`,
  `:wat::core::empty?`, `:wat::core::keys`, `:wat::core::values`,
  `:wat::core::dissoc`, `:wat::core::assoc`, `:wat::core::concat`,
  `:wat::core::string::concat`. These are invisible to
  `:wat::runtime::signature-of`. **Arc 144** ships their TypeScheme
  registrations alongside the unified `Binding` reflection layer.
- **Reflection over special forms** — `:wat::core::if`, `cond`,
  `match`, `let*`, `lambda`, `define`, `defmacro`, `try`,
  `option/expect`, `result/expect`, `quote`, `quasiquote`, etc. Also
  arc 144's territory (slice 2 of arc 144 ships the special-form
  registry).
- **Aliasing user defines at expand-time** — defmacro expansion runs
  at step 4 (before user defines register at step 6). The macro can
  alias substrate primitives but not user defines under the current
  load-order. Out of scope for arc 143; future arc if the bias
  surfaces.
- **`(help X)` REPL consumer** — the data is queryable; a help-form
  consumer is future REPL work (arc 144 ships the prerequisite
  uniformity that lets `(help :if)` "just work" once `help` is
  written).
- **Macro aliasing** — `(:define-alias :my-macro :their-macro)` is
  mechanically possible (defmacro-of-defmacro) but not in this arc.

## The substrate-as-teacher cascade

This arc is the worked example of the cascade discipline:

```
Arc 130 RELAND v1 stepping stone fails: "unknown function: :wat::core::reduce"
  ↓ (the diagnostic motivates arc 143)
Arc 143 slice 1: substrate query primitives (lookup-callable + signature-of + body-of)
  ↓ (slice 1 surfaces the names-are-values quirk + CheckEnv-on-demand)
Arc 143 slice 2: computed unquote in defmacro bodies
  ↓ (slice 2 surfaces freeze.rs + resolve.rs as additional threading sites)
Arc 143 slice 3: HolonAST manipulation primitives
  ↓ (slice 3 confirms TypeScheme rendering matches parser expectations)
Arc 143 slice 6: defmacro emit attempt
  ↓ (slice 6 STOPS at first red — surfaces Gap 1: value_to_watast/HolonAST
      + Gap 2: length not in TypeScheme registry)
Arc 143 slice 5b: value_to_watast HolonAST arm (closes Gap 1)
  ↓ (slice 5b's load-bearing test transition exposes 3 latent slice 1+3 bugs;
      sonnet ships them in the same sweep within file scope)
Arc 143 slice 7: apply (:wat::list::reduce :wat::core::foldl)
  ↓ (arc 130 stepping stone now fails on Vector/len — cascade progressed)
Arc 144 (NEW): uniform reflection foundation closes Gap 2 + extends to special forms
  ↓ (arc 144 elevation: "nothing is special — (help :if) just works")
```

Each STOP-at-first-red surfaced a precise, file:line-attributed
diagnostic. The discipline accelerated failure rather than
absorbing it; each gap got named, scoped, and slice-tracked rather
than hidden behind a workaround.

## The four questions

**Obvious?** Yes for the deliverable verbs:
`:wat::runtime::define-alias` reads as "define an alias"; the
introspection trio (`lookup-callable`, `signature-of`, `body-of`)
all name what they do. The wat-runtime namespace name signals
"reflection on the running wat machine" without ambiguity.

**Simple?** Yes per slice. Each substrate primitive is ~50-150 LOC
of Rust + an arg-validation pattern that mirrors existing eval_*
sites. The macro itself is ~15 LOC of wat. The composition is
load-bearing but mechanical.

**Honest?** Yes — the partial reflection coverage is documented up-
front (this section + slice 6 SCORE + arc 144 DESIGN). The
primitive-body sentinel `(:wat::core::__internal/primitive <name>)`
declares "this is a Rust-implemented primitive, no wat body" rather
than emitting a fake body. The slice 6 length test stays red as a
honest known-defect canary handed off to arc 144.

**Good UX?** Yes for the immediate consumer (the substrate's own
`:wat::list::reduce` alias). Future macro work that needs the
substrate's data — sweep generators, doc extractors, alias
batches — composes the same primitives without hitting a "this only
works for primitive X" wall (within the TypeScheme-registered
subset; arc 144 generalizes).

## Cross-references

- `docs/arc/2026/05/143-define-alias/DESIGN.md` — the design with
  Q1-Q7 findings + 7-slice plan
- `docs/arc/2026/05/143-define-alias/SCORE-SLICE-{1,2,3,5b,6,7}.md`
  — per-slice scorecards + calibration records
- `docs/arc/2026/05/144-uniform-reflection-foundation/DESIGN.md` —
  the follow-on arc that ships the unified reflection foundation
  and closes the slice 6 length test
- `docs/COMPACTION-AMNESIA-RECOVERY.md` — the protocol forged
  mid-arc when slice 6's first attempt cost the rhythm
- `docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md`
  — the arc whose reduce gap motivated this arc
- `docs/arc/2026/04/091-batch-as-protocol/INSCRIPTION.md` (slice 8)
  — the quasiquote + struct→form precedent
- `docs/arc/2026/04/057-holon-ast-polymorphism/INSCRIPTION.md` —
  HolonAST as the reflection AST representation
- `docs/arc/2026/04/037-dim-router/INSCRIPTION.md` — `statement-length`
  introspection precedent
- `docs/arc/2026/05/138-checkerror-spans/INSCRIPTION.md` — the span
  discipline this arc's emit sites honor
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/058-031-defmacro/PROPOSAL.md`
  — defmacro definition
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/058-032-typed-macros/PROPOSAL.md`
  — typed macros (every defmacro param :AST<T>)
- `wat/runtime.wat` — the new top-level wat file housing the
  `:wat::runtime::*` macros
- `wat/list.wat` — the new top-level wat file housing list-domain
  aliases (currently `:wat::list::reduce` and `:wat::list::fold`,
  both → `:wat::core::foldl`)
- `src/runtime.rs:6090+` — `lookup_callable` helper (slice 1)
- `src/runtime.rs:5878+` — `value_to_watast` (slice 5b's HolonAST arm)
- `src/macros.rs:608+` — `expand_template` (slice 2's threading)

## What this arc unblocks

- **Arc 130 slice 1 RELAND v2** — the `:reduce` blocker is closed;
  the next stepping stone (`:wat::core::Vector/len`) is arc 130's
  next link.
- **Arc 144** — uniform reflection foundation can build on slice 1's
  `lookup_callable` + slice 3's manipulation primitives; arc 144
  generalizes both.
- **Future macro work that needs substrate introspection** — sweep
  generators, doc extractors, alias batches, structural validators
  — the substrate side of the data is queryable for the
  TypeScheme-registered subset today; arc 144 closes the gap.
- **Arc 109 v1 closure** — one of two remaining blockers (the other
  being arc 144).
