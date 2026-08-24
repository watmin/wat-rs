# Wat syntax cheatsheet

Single-page reference for writing wat. The substrate teaches you
when you break a rule (every constraint here surfaces as a parse
or type-check error with a concrete fix path); this doc is the
table you check BEFORE writing so the iteration loop is shorter.

For the deep tutorial / mental model see `USER-GUIDE.md`. For
naming + namespacing rules see `CONVENTIONS.md`. For the
concurrency architecture see `ZERO-MUTEX.md`. This cheatsheet is
*how to spell things*; those docs are *what the things mean*.

---

## 1. Colon rule

**The `:` is the symbol-quote marker.** Keywords are symbol-quoted
paths. The leading `:` quotes the symbol path that follows — one
colon, at the start, for that one symbol. A parametric type is NOT
spelled by splicing more text after that colon: `:-` is the ONE
parameterization operator, and it takes a real nested FORM.

```wat
(:wat::core::Vector :- [:wat::core::i64])            ;; reference — in parens
(:wat::core::HashMap :- [K V])                       ;; reference, bare generic params
:wat::bracket::runner-loop :- [I O]                  ;; BINDER — siblings, no parens
(:wat::core::Vector :- [:wat::core::i64] 1 2 3)      ;; constructor — reference PLUS values
```

Because `:- [...]` is a real vector of separate AST nodes — not one
continuous lexical token — nesting falls out for free: each inner
type argument is its OWN keyword, with its OWN leading colon, exactly
like any other symbol-quoted path:

```wat
(:wat::core::Option :- [(:wat::core::Vector :- [T])])
```

— no different from nesting any other wat form. That is precisely
what the OLD `<...>` spelling could not do: `Vec<wat::core::String>`
was ONE lexical token, so typing a `:` inside it opened a SECOND
symbol-quote nested inside the first — illegal (arc 115's rule).
Under `:-` there is no such trap: parens and brackets nest the way
they always do everywhere else in wat.

**Operational form of the rule today:** ONE colon per keyword-path
token, at its start; a nested type is a nested FORM (`:- [...]`),
never nested text spliced after one colon.

| Illegal (doesn't lex) | Canonical |
|---|---|
| `:Vec<wat::core::i64>` | `(:wat::core::Vector :- [:wat::core::i64])` |
| `:Result<Option<i64>,wat::kernel::ThreadDiedError>` | `(:wat::core::Result :- [(:wat::core::Option :- [:wat::core::i64]) :wat::kernel::ThreadDiedError])` |
| `:fn(i64)->bool` | `[:wat::core::i64 :-> :wat::core::bool]` |
| `mk<S,R> [args] -> ret` (a declaration) | `mk :- [S R] [args] -> ret` |

> ⚠ **Historical.** Arc 115 (`docs/arc/2026/04/115-no-inner-colon-in-parametric-args/`)
> forbade a symbol-quote nested inside `<...>`'s single lexical token. Arc 109 ③
> removed `<...>` types outright — 543 files, 710 → 0 — so that specific trap no
> longer exists to fall into; `:-`'s real nesting structurally can't reproduce it.

> **LLM note** — every LLM (sonnet flights, orchestrator instances,
> anyone cloning this repo) initially defaults to spelling a
> parametric type by splicing `<...>` after the name. That reflex is
> wrong here; `:-` is the ONE parameterization operator, full stop —
> `(Head :- [A B])` for a reference, `Head :- [A B]` for a binder,
> `(Head :- [A B] v1 v2)` for a constructor. The substrate's lex
> error names this rule directly and shows the fix; when in doubt,
> write the form and trust the type-checker to teach.

## 2. Whitespace rule

> ⚠ **Historical.** This section used to forbid whitespace inside
> `<...>`, `:(...)`, `:fn(...)`, or `:[...]` — those were single
> lexical tokens (a name with text spliced after it), so the lexer
> rejected whitespace inside the unclosed bracket. Arc 109 ③ retired
> all four spellings; the constraint retired with them.

`:- [...]` is an ORDINARY vector, not a spliced token — whitespace
between its elements is required exactly the way it's required
between the elements of any other wat vector (`[1 2 3]`, an arg
list, a `let` binding), and a comma is accepted as EDN whitespace
between elements the same as everywhere else:

```wat
(:wat::kernel::Peer :- [S R])       ;; canonical, space-separated
(:wat::kernel::Peer :- [S, R])      ;; comma is EDN whitespace — also legal
```

There is no bracket-form left in wat where whitespace is illegal.

## 3. FQDN namespace rule

Substrate-provided types use their full path. No bare aliases
like `(:Sender :- [T])` or `(:Receiver :- [T])` — those are not
registered.

| Illegal / unregistered | Canonical |
|---|---|
| `(:Sender :- [T])` | `(:rust::crossbeam_channel::Sender :- [T])` |
| `(:Receiver :- [T])` | `(:rust::crossbeam_channel::Receiver :- [T])` |
| `:i64` | `:wat::core::i64` (in user code post-arc-109/1c) |
| `:String` | `:wat::core::String` (same) |
| `:wat::core::unit` | `:wat::core::nil` (arc 153 — same type, new name) |

Type aliases CAN be defined in user code (`:wat::core::typealias`)
but are not auto-registered for substrate types. See arc 109's
J-PIPELINE.md for the FQDN sweep.

### `:wat::core::nil` — the singleton (arc 153)

`:wat::core::nil` is wat's name for the unit type — the type
with one inhabitant, the role Rust spells `()`. Same name in
both positions:

- **Type position.** `(:my::probe -> :wat::core::nil)` declares
  "this function returns nothing meaningful." The empty-tuple
  spelling `:()` is bare and retires per arc 109 slice 1d; the
  legacy FQDN `:wat::core::unit` retired arc 153.
- **Value position.** `:wat::core::nil` evaluates to the nil
  singleton. The empty-list literal `()` continues to evaluate
  to the same singleton (transitional spelling kept for
  cross-form ergonomics).

The triplet `nil` / `Some(t)` / `None` reads cleanly and stays
orthogonal — `:wat::core::nil` is the unit type (singleton),
`:wat::core::None` is `(:wat::core::Option :- [T])`'s absence variant,
`:wat::core::Some(t)` is the presence variant. The type system
enforces the split. No "null pointer exception" semantics; no
sentinel-value lies.

### `:wat::program::Env` — wat-level program environment (arc 214 Slice 4)

`:wat::program::Env` is a registered typealias for `(:wat::core::HashMap :- [:wat::core::keyword :wat::holon::HolonAST])`.
It is the second positional argument to `spawn-program'` — the map of configuration
that a spawned program sees as its startup environment.

```wat
;; Pass a program env at spawn time:
(:wat::kernel::spawn-program' :thread {} my-program)
(:wat::kernel::spawn-program' :thread {:config-key (:wat::holon::Atom "value")} my-program)

;; A function accepting a program env:
(:wat::core::define (:user::run-with-env (env :wat::program::Env) -> :wat::core::nil)
  :wat::core::nil)
```

**Namespace separation (arc 214 Slice 4 forward-correction Q4):**

| Name | Alias for | Scope |
|---|---|---|
| `:wat::program::Env` | `(:wat::core::HashMap :- [:wat::core::keyword :wat::holon::HolonAST])` | wat-level program config (Slice 4) |
| `:wat::process::Env` | `(:wat::core::HashMap :- [:wat::core::String :wat::core::String])` | OS-level process env vars (`$HOME`, `$PATH`, …) — separate concern; out of scope Slice 4 |

The two namespaces are orthogonal: program env carries wat-typed config; process env mirrors the OS contract (`getenv`/`setenv`). Callers reach for the right namespace based on what they are talking about.

**Accessor trio (arc 214 Slice 4 Stone 4.2):**

| Verb | Args | Returns | Miss / wrong-type |
|---|---|---|---|
| `:wat::program::Env/get` | `env key -> :T` | `(:wat::core::Option :- [T])` | `None` |
| `:wat::program::Env/expect-get` | `env key -> :T` | `T` | panic with KeyError diagnostic |
| `:wat::program::Env/get-default` | `env key default -> :T` | `T` | `default` |

The `-> :T` annotation sits at TAIL position (after env + key args). The verb looks up `key` in `env`, extracts the stored `HolonAST` leaf to the declared type T, and returns Some(v) / v / default on hit or None / panic / default on miss or type-mismatch.

```wat
;; /get — (:wat::core::Option :- [T]) on miss
(:wat::program::Env/get env :port -> :wat::core::i64)     ;; → (:wat::core::Option :- [:wat::core::i64])

;; /expect-get — T directly; panics if missing or wrong type
(:wat::program::Env/expect-get env :port -> :wat::core::i64)  ;; → i64

;; /get-default — T; returns default on miss or wrong type
(:wat::program::Env/get-default env :port 8080 -> :wat::core::i64)  ;; → i64

;; Typical startup pattern:
(:wat::core::define (:user::run (env :wat::program::Env) -> :wat::core::nil)
  (:wat::core::let
    [port (:wat::program::Env/get-default env :port 8080 -> :wat::core::i64)]
    ;; ... use port
    :wat::core::nil))
```

**Dig trio — multi-step path accessors (arc 214 Slice 4 Stone 4.3):**

| Verb | Args | Returns | Miss / wrong-type |
|---|---|---|---|
| `:wat::program::Env/dig` | `env path -> :T` | `(:wat::core::Option :- [T])` | `None` |
| `:wat::program::Env/expect-dig` | `env path -> :T` | `T` | panic with KeyError diagnostic |
| `:wat::program::Env/dig-default` | `env path default -> :T` | `T` | `default` |

`path` is a `(:wat::core::Vector :- [:wat::core::keyword])` — each element is a navigation step (keyword key for HashMap lookup).  The walk starts at `env` and follows each key in sequence.

**STOP-1 (arc 215 atomizable-set limitation, resolved):** `(:wat::core::HashSet :- [T])` is now atomizable (arc 216 Stone 1); `(:wat::core::Vector :- [T])` is now atomizable (arc 216 Stone 2); `(:wat::core::HashMap :- [K V])` is now atomizable (arc 216 Stone 3).  All three collection types support `(:wat::holon::Atom collection)` → HolonAST round-trip.  Multi-step traversal through nested HashMaps is now fully supported at the algebra level.

Single-step paths (`[:key]`) are equivalent to the `/get` trio and always work.

```wat
;; /dig — (:wat::core::Option :- [T]) on miss or early termination
(:wat::program::Env/dig env [:port] -> :wat::core::i64)         ;; → (:wat::core::Option :- [:wat::core::i64])

;; /expect-dig — T directly; panics if path misses
(:wat::program::Env/expect-dig env [:host] -> :wat::core::String)  ;; → String

;; /dig-default — T; returns default on miss
(:wat::program::Env/dig-default env [:port] 8080 -> :wat::core::i64)  ;; → i64
```

### `:wat::core::do` — sequential evaluation (arc 136)

`(:wat::core::do form_1 form_2 ... form_N)` evaluates each form
left-to-right; non-final results are discarded; the FINAL form's
value is returned and its inferred type IS the do form's type.
Clojure-faithful — non-finals' types are unconstrained.

```wat
;; The print-then-return idiom, daily verb of any Lisp:
(:wat::core::do
  (:wat::kernel::println "computing...")
  (:wat::core::i64::+ 1 1))                ;; → :i64

;; Replaces the let-with-((_ :wat::core::unit) ...) crutch:
(:wat::core::do
  (:wat::test::assert-eq v1 e1)
  (:wat::test::assert-eq v2 e2)
  (:wat::test::assert-eq v3 e3))           ;; → :wat::core::nil
```

Empty `(:wat::core::do)` is a parse error. Single-form
`(:wat::core::do x) ≡ x`. Substrate infers from the final form;
recipient unification verifies.

## 4. Comm-call position rule

`:wat::kernel::send` / `recv` / `try-recv` / `select` /
`process-send` / `process-recv` MUST appear ONLY as:

- the scrutinee of `:wat::core::match`, OR
- the value-position of `:wat::core::Result/expect`, OR
- the value-position of `:wat::core::Option/expect`.

Bare let RHS, function-call argument positions, etc. are
illegal. Arc 110 enforces this — silent disconnect must be
handled at every comm site.

```wat
;; Illegal
(received (:wat::kernel::recv rx))

;; Canonical
(:wat::core::match (:wat::kernel::recv rx)
  -> :T
  ((Ok (Some v)) ...)
  ((Ok :None)    ...)
  ((Err died)    ...))
```

## 5. Control-form shapes

| Form | Required shape |
|---|---|
| `:wat::core::if` | `(if cond -> :T then else)` — arc 108 made `-> :T` mandatory |
| `:wat::core::cond` | `(cond -> :T (test-1 result-1) (test-2 result-2) ... (else default))` |
| `:wat::core::let` | `(let ((name expr) ...) body)` — arc 154 + arc 159; sequential semantics; per-binding type inferred from expression (no `:T` annotation; arc 159 dropped the wrapper); destructure shape `((a b) pair)` unchanged |
| `:wat::core::do` | `(do form_1 form_2 ... form_N)` — arc 136; non-finals' types unconstrained, final form's type IS the do's type |
| `:wat::core::match` | `(match scrutinee -> :T (pattern body) ...)` |
| `:wat::core::define` | `(define (:user::name (arg :T) -> :Ret) body)` |
| `:wat::core::fn` | `(fn ((arg :T) -> :Ret) body)` — arc 155; lambda retired (use fn) |
| `:wat::core::def` | `(def :name expr)` — arc 157; top-level value binding; type inferred from expr; redef is an error by default (opt in via `:wat::config::set-redef!`) |

The `-> :T` is the result-type annotation; required on `if`,
`cond`, `match`, `define`, and `fn`. NOT on `def` (arc 157) and
NOT on `let` per-binding slots (arc 159) — type inferred from
the expression (arc 145's paid-for lesson applied to both
top-level and inner-binding positions; substrate's existing
inference + recipient unification suffices).

`:wat::core::def` is legal at top-level position only — file-root
or direct child of a top-level `do` / `let` body (recursive
splice). Conditional positions (`if`/`cond`/`match`/`Result/try`/
`Option/try`), function bodies, and iteration constructs reject
it with `DefNotTopLevel`.

## 6. Special-form arg shapes

Forms that take ASTs (not strings):

| Form | Takes |
|---|---|
| `:wat::kernel::raise!` | `data: HolonAST`. Wrap a string with `(:wat::holon::leaf "msg")`. |
| `:wat::kernel::assertion-failed!` | `(message :String, actual (:wat::core::Option :- [:wat::core::String]), expected (:wat::core::Option :- [:wat::core::String]))` |
| `:wat::core::eval-ast!` | `:wat::WatAST` (the AST datatype itself) |

Forms that take string literals:

- `assertion-failed!`'s message field
- `:wat::kernel::run-sandboxed`'s src
- error-message slots on `result::expect` / `option::expect`

## 7. No-`:Any`, no-new-types

`:Any` is banned in wat source. Heterogeneous storage uses
`std::any::Any` on the Rust side; wat code uses concrete types
or generics.

Wat does NOT mint its own type system. Generic wat types are backed
by real Rust generics — `(:wat::core::Vector :- [:wat::core::String])`
is Rust's `Vec<String>`, `(:wat::core::Vector :- [:wat::holon::HolonAST])`
is Rust's `Vec<HolonAST>`, etc. No `AtomLiteral` enum or `AtomValue`
trait. Rust types ARE wat types.

## 8. Collection constructors (verb-equals-type)

Per arc 109 slice 1f — the verb IS the type. Parametric containers take
leading type-keyword args (one per type parameter) followed by values;
heterogeneous `Tuple` takes positional values only (element types inferred
from each position).

```wat
(:wat::core::Vector :T x0 x1 ...)              ;; (:wat::core::Vector :- [T])          (1 type-keyword)
(:wat::core::HashMap :K :V k0 v0 k1 v1 ...)    ;; (:wat::core::HashMap :- [K V])       (2 type-keywords)
(:wat::core::HashSet :T x0 x1 ...)             ;; (:wat::core::HashSet :- [T])         (mirror of Vector)
(:wat::core::Tuple x0 x1 x2 ...)               ;; (:wat::core::Tuple :- [T0 T1 T2])    (no type-keywords; types inferred per position)
```

Rules:
- For parametric containers (Vector/HashMap/HashSet): type-keyword args come FIRST and are mandatory (arity error if missing); value args come after.
- For HashMap: value-arg count after `:K :V` must be even (alternating key/value pairs).
- For Tuple: every arg is a value; element types are inferred per position; no leading type-keywords (heterogeneous structural type).
- Type keywords accept aliases (`:my::Key` expands at call site; FQDN also works).

Arc 214 P1 retired the old `(:wat::core::HashMap :(K,V) ...)` tuple-keyword
shape; the two-separate-keywords shape `:K :V` mirrors Vector's `:T` exactly.

### Type-inference placeholder: `:wat::type::Infer` (arc 215 stone 1)

`:wat::type::Infer` is a type-placeholder for HM-style type inference.
Appears in type-arg slots of parametric constructor calls; tells check.rs
"infer this type from the values." Analogous to Rust's `_` in type position.

```wat
(:wat::core::HashMap :wat::core::keyword :wat::type::Infer :foo 42 :bar 99)
;; V inferred as :wat::core::i64 from the values 42 and 99

(:wat::core::HashSet :wat::type::Infer 1 2 3)
;; T inferred as :wat::core::i64 from elements 1, 2, 3

(:wat::core::Vector :wat::type::Infer 1 2 3)
;; T inferred as :wat::core::i64; equivalent to [1 2 3]

(:wat::core::HashMap :wat::type::Infer :wat::type::Infer :foo 1 :bar 2)
;; K inferred as :wat::core::keyword; V inferred as :wat::core::i64

(:wat::core::HashMap :wat::type::Infer :wat::type::Infer "a" 1 "b" 2)
;; K inferred as :wat::core::String; V inferred as :wat::core::i64
```

Empty constructor with `Infer` → fresh type variable (resolves on first use).
Mismatch between inferred type and subsequent values → `TypeMismatch` diagnostic.

### Three-literal unification (arc 215 stone 2)

After arc 215 stone 2, all three collection literals share the `:wat::type::Infer`
machinery. Mental model: **first-unit inference + uniform unification**.

```
[...]  desugars at check time  → (:wat::core::Vector :wat::type::Infer ...)
{...}  desugars at parse time  → (:wat::core::HashMap :wat::type::Infer :wat::type::Infer ...)
#{...} desugars at parse time  → (:wat::core::HashSet :wat::type::Infer ...)
```

Type is inferred from the first element/key/value; all subsequent items must unify.
Mixed types → `TypeMismatch` at check time, position-named.

Escape hatch (power-user explicit form): use the verb form with concrete types:
```wat
(:wat::core::Vector :wat::core::i64 1 2 3)        ;; explicit T
(:wat::core::HashMap :wat::core::keyword :wat::core::i64 :a 1 :b 2)  ;; explicit K, V
(:wat::core::HashSet :wat::core::String "a" "b")  ;; explicit T
```

### Map literal syntax (arc 214 P2, arc 215 stone 1+2)

`{...}` is a map literal sugar with unified K and V inference:

```wat
{:k0 v0 :k1 v1 ...}    ;; desugars at parse time to:
                        ;; (:wat::core::HashMap :wat::type::Infer :wat::type::Infer
                        ;;   :k0 v0 :k1 v1 ...)
                        ;; K and V both inferred from first key/value

{}                      ;; empty map literal — (:wat::core::HashMap :- [fresh-K fresh-V])

{:outer {:inner 42}}    ;; nested: outer V inferred as (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
                        ;; values pass through as-is (no Atom auto-wrap)

{1 "v" 2 "w"}           ;; arc 215 stone 2: non-keyword keys accepted
                        ;; K inferred as :wat::core::i64; V as :wat::core::String
{"a" 1 "b" 2}           ;; K inferred as :wat::core::String; V as :wat::core::i64
```

K and V are both `:wat::type::Infer` (arc 215 stone 2 lifted the keyword-key restriction).
All keys must have the same type; all values must have the same type.
Mixed K or mixed V → `TypeMismatch` at check time, position-named.
For explicit K/V types, use the verb form: `(:wat::core::HashMap :K :V ...)`.

### Set literal syntax: `#{...}` (arc 215 stone 1)

`#{...}` is a set literal sugar:

```wat
#{1 2 3}        ;; desugars to: (:wat::core::HashSet :wat::type::Infer 1 2 3)
                ;; T inferred as :wat::core::i64 from element 1

#{}             ;; empty set — (:wat::core::HashSet :- [fresh-T])

#{:a :b :c}     ;; T inferred as :wat::core::keyword
```

Duplicate elements collapse at construction (dedup). All elements must have the same type.
For an explicit T, use the verb form: `(:wat::core::HashSet :T ...)`.

### Atomizable types and `(:wat::holon::Atom T)` (arc 215 + arc 216)

`:wat::holon::Atom` converts a value to a `HolonAST` node. The atomizable set
determines which types are accepted; non-atomizable types fail at check time
(arc 216 atomizable predicate).

**Atomizable set (recursive):**

```
atomizable(T) :=
  T ∈ {i64, f64, bool, String, keyword, HolonAST, WatAST, Uuid}          -- primitives (arc 215 baseline)
  OR T = (HashSet :- [T']) ∧ atomizable(T')                             -- arc 216 Stone 1 (shipped)
  OR T = (Vector :- [T'])  ∧ atomizable(T')                             -- arc 216 Stone 2 (shipped)
  OR T = (HashMap :- [K V]) ∧ atomizable(K) ∧ atomizable(V)             -- arc 216 Stone 3 (shipped)
```

Canonical implementation: `fn is_atomizable(ty: &TypeExpr) -> bool` at `src/check.rs:3623`.
Called from the `:wat::holon::Atom | :wat::holon::leaf` arm in `infer_list`.

**Encoding shape (DESIGN Q2):**

```wat
(:wat::holon::Atom #{1 2 3})
;; → HolonAST::Bundle([I64(1), I64(2), I64(3)])
;; Set-shape: bare atoms, no Bind keys

(:wat::core::atom-value bundle-of-bare-atoms)
;; → (:wat::core::HashSet :- [T]) (reconstructs from Bundle of bare atoms)
;; Round-trip: #{1 2 3} → Atom → atom-value → #{1 2 3}

(:wat::holon::Atom [1 2 3])
;; → HolonAST::Bundle([Bind(I64(0), I64(1)), Bind(I64(1), I64(2)), Bind(I64(2), I64(3))])
;; Array-shape: positional-Bind keys 0..n-1, order preserved

(:wat::core::atom-value bundle-of-positional-binds)
;; → (:wat::core::Vector :- [T]) (reconstructs from Bundle of Bind(I64(i), _) with sequential keys 0..n-1)
;; Round-trip: [1 2 3] → Atom → atom-value → [1 2 3] (order preserved)

(:wat::holon::Atom {:foo 42 :bar 99})
;; → HolonAST::Bundle([Bind(Symbol(foo), I64(42)), Bind(Symbol(bar), I64(99))])
;; Map-shape: arbitrary-K Bind pairs; order non-canonical (HashMap unordered)

(:wat::core::atom-value bundle-of-arbitrary-k-binds)
;; → (:wat::core::HashMap :- [K V]) (reconstructs from Bundle where all children are Bind nodes
;;   and keys are not sequential i64 0..n-1 — non-sequential I64 keys also → HashMap)
;; Round-trip: {:foo 42} → Atom → atom-value → {:foo 42}

;; Empty Bundle disambiguation — consumer-declared type hint:
(:wat::core::atom-value empty-bundle -> :wat::core::HashMap)
;; → empty HashMap (consumer overrides conservative HashSet default)
;; Without the `-> :T` hint, empty Bundle → empty HashSet (conservative default).
```

**Shape discriminator at `atom-value`:**

| Bundle shape | Result |
|---|---|
| Empty (no children) | empty HashSet (conservative default) |
| Empty + `-> :wat::core::HashMap` annotation | empty HashMap (consumer-declared) |
| All children are bare atoms (non-Bind) | HashSet (set-shape, Stone 1) |
| All children are Bind(I64, _) with sequential keys 0..n-1 | Vec (array-shape, Stone 2) |
| All children are Bind; keys non-sequential I64 OR non-I64 K | HashMap (map-shape, Stone 3) |

**Predicate at check time:**

```wat
;; PASSES: (:wat::core::HashSet :- [:wat::core::i64]) is atomizable (i64 is primitive)
(:wat::holon::Atom #{1 2 3})

;; PASSES: (:wat::core::Vector :- [:wat::core::i64]) is atomizable (i64 is primitive)
(:wat::holon::Atom [1 2 3])

;; PASSES: (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64]) — both K and V are primitive
(:wat::holon::Atom {:foo 42 :bar 99})

;; PASSES: nested (:wat::core::Vector :- [(:wat::core::Vector :- [:wat::core::i64])]) — predicate recurses both levels
(:wat::holon::Atom outer-nested-vec)

;; PASSES: nested (:wat::core::HashSet :- [(:wat::core::HashSet :- [:wat::core::i64])]) — predicate recurses
(:wat::holon::Atom outer-nested-set)

;; PASSES: (:wat::core::HashMap :- [:wat::core::keyword (:wat::core::Vector :- [:wat::core::i64])]) — composes Stone 2 + Stone 3
(:wat::holon::Atom {:data [1 2 3]})

;; PASSES: (:wat::core::HashMap :- [:wat::core::keyword (:wat::core::Vector :- [(:wat::core::HashSet :- [:wat::core::i64])])]) — all three collections nested
;; (triple-nested composition; arc 216 Stone 4 composite)
(:wat::holon::Atom complex-nested-map)

;; FAILS at check: a function value is not atomizable
(:wat::holon::Atom my-fn)
;; TypeMismatch: :wat::holon::Atom #1 expects :wat::holon::HolonAST; got :wat::core::Fn(wat::core::i64)->wat::core::i64
;; (the substrate's own diagnostic still renders a Fn type this way — verified live, arc 109 did not touch Fn's DISPLAY form)

;; FAILS at check: (:wat::core::Vector :- [Fn-type]) — non-atomizable element T
;; TypeMismatch: :wat::holon::Atom #1 expects :wat::holon::HolonAST; got (:wat::core::Vector :- [:wat::core::Fn(wat::core::i64)->wat::core::i64])
;; (:wat::holon::Atom vec-of-fns)  -- rejects at check time

;; FAILS at check: (:wat::core::HashMap :- [Fn-type :wat::core::i64]) — non-atomizable K
;; TypeMismatch: :wat::holon::Atom #1 expects :wat::holon::HolonAST; got (:wat::core::HashMap :- [...])
```

Atomizable composition examples:

| Expression | Passes? | Reason |
|---|---|---|
| `(:wat::holon::Atom v)`, `v : (:wat::core::HashMap :- [:wat::core::keyword (:wat::core::Vector :- [(:wat::core::HashSet :- [:wat::core::i64])])])` | YES | all three collections; T = i64 (primitive) |
| `(:wat::holon::Atom v)`, `v : (:wat::core::Vector :- [(:wat::core::HashSet :- [:wat::core::i64])])` | YES | Vector-of-HashSet; T = i64 (primitive) |
| `(:wat::holon::Atom v)`, `v : (:wat::core::HashSet :- [(:wat::core::Vector :- [:wat::core::i64])])` | YES | HashSet-of-Vector; T = i64 (primitive) |
| `(:wat::holon::Atom v)`, `v : (:wat::core::HashMap :- [:wat::core::keyword Fn-type])` | NO | V = Function; not atomizable |
| `(:wat::holon::Atom v)`, `v : (:wat::core::Vector :- [Fn-type])` | NO | T = Function; not atomizable |

Non-atomizable T (e.g., function values, Thread handles, user structs not in the set)
fails at check with `TypeMismatch` naming `:wat::holon::Atom` and the offending type.

Reference: arc 216 DESIGN `docs/arc/2026/05/216-collections-as-holons/DESIGN.md`.
Arc 216 Stones 1/2/3 all shipped. All three collection types fully atomizable.
(HashSet: Stone 1; Vector: Stone 2; HashMap: Stone 3; composite verification: Stone 4.)

#### Hashable types — `impl Hash for Value` (arc 216 Stones 216.5a-d)

**Contract (post-216.5d):** every type admitted by `is_atomizable` is hashable via
`impl Hash for Value` at `src/runtime.rs`. The canonical-key crutch (`fn hashmap_key`)
was deleted in Stone 216.5d — it no longer exists in the substrate.

**The canonical mechanism:**

- `impl Hash for Value` (Stone 216.5a) — mirrors `HolonAST`'s Hash impl. Per-variant
  payload hash with discriminant tagging. `f64` via `to_bits()`. Non-atomizable opaque-
  handle variants (`wat__core__fn`, `Sender`, `Receiver`, etc.) → `unreachable!()`.
- `is_atomizable` at `src/check.rs:3623` — the check-time gate. Static guarantee that
  only hashable values reach `HashSet<Value>` or `HashMap<Value, _>` operations.
- `value_is_hashable` at `src/runtime.rs` — runtime defense-in-depth. Guards 14 opaque-
  handle variants before `HashSet::insert` or `HashMap::insert` so that a user-visible
  `TypeMismatch` is returned instead of hitting the `unreachable!()` panic in `Hash`.
  Called via thin wrappers `value_is_set_hashable` (HashSet sites) and
  `value_is_key_hashable` (HashMap key sites). Separate names are documentation at
  the call site; they share the same predicate body.

**Storage (post-216.5b/c):**

| Type | Native Rust storage |
|---|---|
| `Value::wat__std__HashSet` | `Arc<HashSet<Value>>` — uses `Value: Hash + Eq` directly |
| `Value::wat__std__HashMap` | `Arc<HashMap<Value, Value>>` — uses `Value: Hash + Eq` as key |

**Guarded runtime error:** attempting to use a non-hashable value (opaque handle such
as a function, channel, or thread handle) as a `HashSet` element or `HashMap` key
produces a runtime `TypeMismatch` from the `value_is_hashable` guard, not a panic.

Reference: `src/runtime.rs` — `pub fn value_is_hashable`, `value_is_set_hashable`,
`value_is_key_hashable`. Arc 216 Stones 216.5a (Hash impl), 216.5b (HashSet storage),
216.5c (HashMap storage), 216.5d (hashmap_key deleted).

### Vector literal syntax: `[...]` (arc 167 + arc 215 stone 2)

`[...]` is a vector literal. Since arc 215 stone 2 it routes through the unified
`:wat::type::Infer` machinery at expression position:

```wat
[1 2 3]         ;; (:wat::core::Vector :- [:wat::core::i64]); T inferred as :wat::core::i64 from element 1
[1.5 2.5]       ;; (:wat::core::Vector :- [:wat::core::f64]); T inferred as :wat::core::f64
["a" "b"]       ;; (:wat::core::Vector :- [:wat::core::String])
[true false]    ;; (:wat::core::Vector :- [:wat::core::bool])
[]              ;; empty (:wat::core::Vector :- [fresh-T])
```

At binder position (let/fn/match), `[...]` continues to act as a tuple-destructure
binder (arc 169 / arc 167 binder semantics; unchanged by arc 215 stone 2).

**Position discipline** (arc 214 P2 + arc 215 stone 1+2 + arc 169):

| Position | Form | Routes to |
|---|---|---|
| Expression | `[x y ...]` | vector literal → `infer_list_constructor` (T inferred) |
| Expression | `[]` — empty | empty vector literal (T fresh) |
| Expression | `{k v ...}` — any non-symbol head | map literal → desugared HashMap verb-call (K, V inferred) |
| Expression | `{}` — empty | empty map literal |
| Expression | `#{x y ...}` | set literal → desugared HashSet verb-call (T inferred) |
| Expression | `#{}` — empty | empty set literal |
| Binding LHS in `let` | `[x 1 y 2]` — alternating binder/expr pairs | tuple-destructure binder (arc 169/167) |
| Binding LHS in `let` | `{field1 field2 ...}` — bare-symbol head | struct destructure → `WatAST::StructPattern` (arc 169) |

Content-shape dispatch for `{...}`: parser reads first child's shape.
A bare Symbol head → struct destructure. Anything else (keyword, integer, string, ...) → map literal.
`#{...}` always routes to set literal (the `#` prefix is the discriminator).
`[...]` always routes to vector literal at expression position; binder position handled separately.

## 9. Common verb signatures

| Verb | Returns |
|---|---|
| `:wat::kernel::send sender value` | `(:wat::core::Result :- [:wat::core::nil (:wat::core::Vector :- [:wat::kernel::ThreadDiedError])])` |
| `:wat::kernel::recv receiver` | `(:wat::core::Result :- [(:wat::core::Option :- [T]) (:wat::core::Vector :- [:wat::kernel::ThreadDiedError])])` |
| `:wat::kernel::try-recv receiver` | `(:wat::core::Result :- [(:wat::core::Option :- [T]) (:wat::core::Vector :- [:wat::kernel::ThreadDiedError])])` |
| `:wat::kernel::select [(rx-1 ...) (rx-2 ...)]` | `(:wat::core::Result :- [(:wat::kernel::Chosen :- [T]) (:wat::core::Vector :- [:wat::kernel::ThreadDiedError])])` |
| `:wat::kernel::spawn-thread body` | `(:wat::kernel::Thread :- [I O])` (arc 114) |
| `:wat::kernel::Thread/join-result thr` | `(:wat::core::Result :- [:wat::core::nil (:wat::core::Vector :- [:wat::kernel::ThreadDiedError])])` |
| `:wat::kernel::spawn-program src scope` | `(:wat::core::Result :- [(:wat::kernel::Process :- [I O]) :wat::kernel::StartupError])` |
| `:wat::kernel::Process/join-result proc` | `(:wat::core::Result :- [:wat::core::nil (:wat::core::Vector :- [:wat::kernel::ProcessDiedError])])` |

Arc 113 widened every Err arm to `(:wat::core::Vector :- [*DiedError])` (chain).
Arc 114 retired `:wat::kernel::spawn` / `join` / `join-result`
in favor of `spawn-thread` + `Thread/join-result`.

## 10. Test verbs

Tests use `:wat::test::*`, NOT `:user::*`:

| Verb | Path |
|---|---|
| `assert-eq` | `:wat::test::assert-eq :- [T]` |
| `assert-substring` | `:wat::test::assert-substring` |
| `assert-coincident?` | `:wat::test::assert-coincident?` |
| `deftest` | `:wat::test::deftest` |

See USER-GUIDE.md § 13 "Testing".

## 11. Scope-deadlock rule

Outer scope holds the Thread; inner scope owns every Sender
clone. The compiler refuses programs where a `Channel` /
`Sender` lives at sibling scope to a Thread whose
`Thread/join-result` runs in the same `let`.

```wat
;; Illegal — pair sibling to thr; pair's Sender outlives thr;
;; the worker's recv never sees EOF.
(:wat::core::let
  (((pair (:wat::kernel::Channel :- [:wat::core::i64])) (:wat::kernel::make-bounded-channel :wat::core::i64 1))
   ((thr  (:wat::kernel::Thread :- [:wat::core::nil :wat::core::i64])) (:wat::kernel::spawn-thread ...))
   ...)
  (:wat::kernel::Thread/join-result thr))

;; Canonical — outer holds thr; inner owns pair + Sender;
;; inner returns thr; pair drops at inner-scope exit.
(:wat::core::let
  (((thr (:wat::kernel::Thread :- [:wat::core::nil :wat::core::i64]))
    (:wat::core::let
      (((pair (:wat::kernel::Channel :- [:wat::core::i64])) (:wat::kernel::make-bounded-channel :wat::core::i64 1))
       ((h    (:wat::kernel::Thread :- [:wat::core::nil :wat::core::i64])) (:wat::kernel::spawn-thread ...))
       ...)
      h)))
  (:wat::kernel::Thread/join-result thr))
```

Same rule applies to `Process/join-result`. Arc 117 enforces it
at type-check time. Arc 131 extended it to `(:wat::kernel::HandlePool :- [T])` —
when T (after alias resolution) contains a Sender, a HandlePool
sibling to a Thread with `Thread/join-result` fires the same
diagnostic with `(a HandlePool)` as the offending kind. Arc 133
extended visibility to tuple-destructure bindings
`((pool driver) ...)` so the check sees them uniformly with
typed-name shapes. See `SERVICE-PROGRAMS.md § "The lockstep"`
for the why.

Arc 134 added two structural narrowings to reduce false positives
on canonical `(:wat::kernel::Thread :- [I O])` usage:

- **Origin-trace exemption.** A Sender whose binding RHS is
  `(:wat::kernel::Thread/input <_>)` or `Process/input` extracts
  the parent-side end of an internal pipe owned by the Thread
  struct. The pair-Receiver is the spawned function's `in`
  parameter — lifetime-coupled to the Thread. The rule does NOT
  fire on this shape, even when sibling to `Thread/join-result`.

- **Body-form exemption.** When the Thread's binding RHS is a
  spawn call whose function argument is an inline lambda whose
  body contains no `(:wat::kernel::recv ...)` / `try-recv` /
  `select` call, no recv-loop can exist; no Sender lifetime can
  deadlock the thread. The rule does NOT fire for any sibling
  Sender in that case.

Both narrowings are heuristic — a body that calls a helper
function which recvs, or a lambda body with an unbounded recv-
loop on its input pipe, can still deadlock at runtime; arc
134's narrowings prefer precision over conservative-fire and
accept the runtime hang as the cost. See arc 134's INSCRIPTION
for the full failure-engineering record.

## 12. Channel-pair-deadlock rule

A function call MUST NOT receive both halves of one
`make-bounded-channel` pair as arguments. Holding both ends
in one role deadlocks any recv — the caller's writer keeps
the channel alive even when the receiving thread dies.

```wat
;; Illegal — caller binds both `tx` and `rx` from one pair;
;; the helper-verb call passes both. Recv inside the helper
;; never sees EOF if the worker dies; caller's tx clone
;; keeps the channel open.
(:wat::core::let
  (((pair (:wat::kernel::Channel :- [:wat::core::nil]))
    (:wat::kernel::make-bounded-channel :wat::core::nil 1))
   ((tx (:wat::kernel::Sender :- [:wat::core::nil]))   (:wat::core::first  pair))
   ((rx (:wat::kernel::Receiver :- [:wat::core::nil])) (:wat::core::second pair))
   ...
   ((_ :wat::core::nil) (:my::helper-verb tx rx ...)))
  ...)

;; Canonical — pair-by-index via HandlePool. Each producer
;; pops one Handle holding ONE end of EACH of two distinct
;; channels. The driver gets the corresponding (Rx, AckTx).
;; Distinct pair-anchors → distinct channels → no deadlock.
(:wat::core::let
  (((handle :svc::Handle)                (:wat::kernel::HandlePool::pop pool))
   ((req-tx (:svc::ReqTx :- [T]))            (:wat::core::first  handle))
   ((ack-rx (:svc::AckRx :- [:wat::core::nil])) (:wat::core::second handle))
   ...
   ((_ :wat::core::nil) (:my::helper-verb req-tx ack-rx ...)))
  ...)
```

Arc 126 enforces this at type-check time. The diagnostic names
the pair-anchor binding and points at `ZERO-MUTEX.md § "Routing
acks"` for the canonical fix patterns. Same trace machinery as
arc 117; different rule arm.

## 13. Discovery loop

When you trip a rule:

1. Read the substrate's error message — it includes the rule + a
   concrete fix path (the substrate-as-teacher discipline; see
   `SUBSTRATE-AS-TEACHER.md`).
2. Re-check this cheatsheet for the rule's canonical form.
3. Find the arc that introduced the rule (the error message names
   it; e.g., "arc 115") and read its INSCRIPTION for the why.

The substrate is the most authoritative reference for its own
behavior — this cheatsheet aggregates the rules at a snapshot in
time. When this disagrees with the substrate, the substrate
wins. File a doc bug.

---

## Sources of truth

- **Active rules** — every entry above traces to an arc inscription
  in `docs/arc/2026/04/`. The arc is the authoritative why; this
  doc is the convenient how.
- **Living changelog** — `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  records every shipped change. When a rule changes, the changelog
  records it; this cheatsheet updates from there.
- **The substrate's own error messages** — every rule above is
  enforced at parse / type-check time with a self-describing
  diagnostic. If the diagnostic is unclear, that's a substrate bug
  to file, not a doc-only fix.
