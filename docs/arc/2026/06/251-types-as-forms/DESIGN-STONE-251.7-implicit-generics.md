# DESIGN — Stone 251.7: implicit generics (Hindley-Milner, bare type-vars)

**Status: SHIPPED 2026-06-10 (`0c95ae2c`). Sonnet-built, orchestrator-weighed against the disk.**
`collect_free_type_vars` (runtime.rs) + union into `raw_type_params` at the 3 fn-registration sites;
the HM pipeline (instantiate/rename/unify) was already correct, unchanged. Probe
`tests/probe_arc251_implicit_generics.rs` 5/5; lib 949/0, types 83/0 isolated (deterministic). The
faithful bare-var-no-suffix generic form now type-checks. NEXT consumers: Arc 256 (generic defclause
— ports this recipe to `ClauseSet`) then 251.5 drops the `<T,U>` suffix corpus-wide.

> This DESIGN is the durable home for the generics MODEL resolved in session 2 (2026-06-10). It
> was living only in the recovery breadcrumb (`project_typed_clojure_parity_pivot.md`), which is
> the wrong lane — the breadcrumb holds *orientation*, not a buildable spec. Captured here so the
> reasoning, the contract decision, and the rooms survive in the arc record. The breadcrumb is
> trimmed to a one-line pointer at this file.

## The move (faithful-Clojure / ADT-native generics)

wat is a faithful-Clojure dialect that is an **ADT language** (nominal tagged sums — ML/Haskell/Rust
lineage), NOT core.typed's set-theoretic type theory. Its generics must therefore be **EDN-valid**
and **ADT-native**. Three candidate spellings, and why only one survives:

| spelling | example | verdict |
|---|---|---|
| Rust angle-brackets | `map<T,U>`, `Stream<T>` | **non-EDN** — `<`/`>` aren't valid in a symbol; same heresy as `::`-keywords. Dies at the 251 hard-cut. |
| core.typed `ann`+`All` | `(ann map (All [T U] …))` | **EDN-valid but rejected** — separate-ascription is a *retrofit* (Clojure fns carry no type info, so core.typed bolts types on after the fact; the same retrofit pressure that forces its anonymous `(U A B)` unions). wat is typed-from-birth → no retrofit pressure → no separate ascription. |
| **implicit (ML / Hindley-Milner)** | bare type-vars in the signature | **the answer.** A free, un-namespaced type variable in a signature is **automatically `∀`-quantified**. No `<>`, no `All`, no `ann`. ML's `'a`, the bedrock ADT mechanism. |

**Parametric polymorphism (`∀T. …`) is 100% ADT-native** — it is ML's `'a`, Haskell's `forall`,
Rust's `<T>`; it is NOT set-theoretic, NOT unions, NOT occurrence typing. Only core.typed's
*separate-ascription spelling* was the theocracy sneaking in. The ADT-compass cut it — the third
core.typed idea rejected this session, after anonymous unions and anonymous tuples. (`ann` for var
annotation may still earn a place someday for a non-fn `def`; it is **off the generics critical
path**.)

## The faithful form

The EDN-compliant wat generic is **bare type-variables in the signature** — nothing declared up
front; the signature *is* the type:

```clojure
;; insert: HashMap<K,V> + key + val -> HashMap<K,V>
(wat.core/defn user/put
  [m :- (wat.type/HashMap K V)  k :- K  v :- V]
  :- (wat.type/HashMap K V)
  (wat.core/assoc m k v))

;; transform every value: HashMap<K,V> + (V -> W) -> HashMap<K,W>
(wat.core/defn user/map-values
  [m :- (wat.type/HashMap K V)  f :- [V :-> W]]
  :- (wat.type/HashMap K W)
  …)
```

`K`, `V`, `W` are bare symbols → the checker generalizes them → the fn is generic. EDN-valid (every
token is a symbol, vector, or list), ADT-native (ML), zero ceremony — one fewer construct than even
Rust (no `<…>` decl).

## Why it is unambiguous — three lexical classes, zero overlap

This pays out the builder's **FQDN-ALWAYS law** (a named type is *always* `wat.type/X`; the resolver
makes bare-name resolution impossible):

| class | example | what it is |
|---|---|---|
| lowercase bare | `m`, `k`, `v`, `f` | value bindings |
| **Uppercase bare** | `K`, `V`, `W` | **type variables** (`∀`, implicit) |
| FQDN | `wat.type/HashMap`, `wat.type/Stream` | named types |

A bare, un-namespaced symbol **in type position** can only be a type variable — names are FQDN by
law, so you cannot accidentally write a bare type name and have it typo-mask as a var. `(wat.type/HashMap K V)`
reads instantly as "a named type applied to two type variables." No sigil needed.

**The LLM-first justification (north-star test):** wat is engineered for an LLM to operate it by
instinct, never having seen it. Uppercase-single-letter-type-var is one of the *most over-represented
conventions in the entire training corpus* (`HashMap<K,V>`, `Map<K,V>`, `(All [K V] …)`, Haskell,
ML). An LLM walks in already knowing it — wat is just declining to fight that instinct. The bare `K`/`V`
is not "a convention we hope is learned"; it is the instinct already present. Spelling (`K` vs a
sigil like ML's `'a`) is an **intueri taste call**, not a type-theory decision, and not blocking —
the lean is bare `K` (Rust/Java/core.typed flavor).

## It IS unification (Robinson's MGU)

The mechanism is **unification** — and not "like" Prolog's, the *same* algorithm. When the checker
reads `user/put`, it sees `K` in three places (the map's key slot, param `k`, the return's key slot)
and `V` in three, and **unifies every occurrence of each variable to one type**. At a call site
`(user/put some-map "name" 42)` it unifies `K := String`, `V := i64`, and checks `some-map` is
`HashMap<String,i64>`. Solving-for-the-variables-by-matching-structure is unification.

| | unifies… | wrapped in… |
|---|---|---|
| Prolog | logic terms (`foo(X,bar) = foo(baz,Y)` → `X=baz, Y=bar`) | SLD-resolution + backtracking |
| Hindley-Milner (ML/Rust/**wat**) | types (`(HashMap K V) = (HashMap String i64)` → `K=String, V=i64`) | constraint solving |

Same core engine — Robinson's unification (1965), find the **most general unifier** of two terms —
pointed at two domains (terms vs types). Same safety valve: the **occurs check** (`T = (List T)` is
an infinite type, rejected). Clara is the *cousin, not the twin* — RETE (forward-chaining rules
network), which binds vars by structural match but is a different algorithm. Prolog + wat are the
literal-unification siblings.

**wat has been doing this all along.** `infer_if` already calls strict `unify` (not `assignable`)
on its branches; the generic `K`/`V` signature doesn't *add* unification — it makes visible at the
surface the thing the checker was always doing. Reaching for generics over a parametric map made the
checker show its Prolog face — a **GREEN reach-tell** (instinct landed on a named great =
on the ridgeline; see `feedback_reach_lands_on_great_is_green.md`).

## The disk (grounded + empirically probed, 2026-06-10)

The HM pipeline is **present AND already works end-to-end** — generic user defns are genuinely
`∀`-checked TODAY, sourced from the `<T,U>` name suffix:

- **Source (parse):** `split_name_and_type_params` (`runtime.rs:2634`) parses `:map<T,U>` →
  `("…map", ["T","U"])`. `try_parse_fn_shape_def` sets `type_params: raw_type_params`
  (`runtime.rs:2081`, comment *"Arc 139 — preserve type_params from the name keyword"*). The
  canonical name (no suffix) is the `sym.functions` key; `canonical_callable_name` (`runtime.rs:2622`)
  strips the suffix at call sites too (symmetric register/lookup).
- **Scheme:** `CheckEnv::from_symbols` (`check/env.rs:125`) runs `derive_scheme_from_function`
  (`check.rs:13678`), which clones `type_params: func.type_params.clone()` (`check.rs:13689`) into
  the `TypeScheme` (`check.rs:79`).
- **Instantiate/unify:** at each call site `instantiate` (`check.rs:13553`) makes a fresh var per
  `type_params` entry and `rename`s `Path(":T")` → that fresh var (`check.rs:13577`); `unify`
  (`check.rs:13149`) / `fresh_var` (`check.rs:13409`) solve them.

**Empirically locked** (`tests/probe_arc251_implicit_generics.rs`, all green at HEAD):
- `F01` — `(defn :pair-first<T> [a <- :T b <- :T] -> :T a)` called `(pair-first 1 "two")` is
  **REJECTED** (`T:=i64` then `b=String` fails to unify). Proves suffix-generics are *really*
  checked, not tolerated.
- `F02` — `(pair-first 1 2)` checks.
- `R03` — the faithful **bare-var-no-suffix** form `(defn :pair-first2 [a <- :T b <- :T] -> :T a)`
  **FAILS** at HEAD: with no suffix, `type_params` is empty, `instantiate` short-circuits, `:T`
  stays a rigid `Path`, and `(pair-first2 1 2)` unifies `i64` vs `Path(":T")` → spurious
  `TypeMismatch`. **This is the RED 251.7 flips.**

**So the premise (corrected from an earlier wrong note):** generics are NOT broken or leniently
tolerated today — they *work*, but only when the genericity is declared via the non-EDN `<T,U>`
name suffix. The faithful bare-var form (no suffix) currently produces a *spurious type error*.
251.7 does NOT build the HM pipeline (it exists and is correct) and does NOT "first-populate"
anything — it **adds a second SOURCE for `type_params`**: auto-generalize the free bare type-vars
out of the signature, so the bare-var form checks identically to the suffix form. That makes the
`<T,U>` suffix redundant → 251.5 drops it.

## The strike (the build, when reached)

The contract decision, pinned to ONE shape and the real site:

> **At fn registration, add signature free-var generalization as a SECOND source for
> `type_params`: UNION the `raw_type_params` already parsed from the `<T,U>` name suffix with the
> free, un-namespaced (bare-`Path`, no `::`/`.`) type-variable symbols collected from the
> signature's param `TypeExpr`s + return `TypeExpr` (first-occurrence order, deduped). Everything
> downstream — `derive_scheme_from_function`, `instantiate`, `rename`, `unify` — is UNCHANGED; it
> already consumes `type_params` correctly (proven: probe F01/F02 green). The build changes only
> the SET of `type_params`: name-suffix vars ∪ free-signature vars, so the bare-var form checks
> identically to the suffix form.**

Sketch (rooms named, grounded by the lair-study):

1. **A free-var collector** over `TypeExpr` — walk a `TypeExpr`, emit each `Path` that is a type
   VARIABLE, recursing into `Parametric.args`, `Fn.{args,ret}`, `Tuple.elems`. First-occurrence
   order, deduped. **The var test (the three-lexical-classes rule, made precise):** strip the
   leading `:`; the `Path` is a type var iff it is **bare** (contains neither `::` nor `.` → not
   FQDN) **AND its first alphabetic char is UPPERCASE** (the `Uppercase-bare = type-var` class —
   `K`, `V`, `T`). This is load-bearing: it EXCLUDES lowercase legacy bare primitives (`:i64`,
   `:bool`, `:f64`, `:nil`) — named types pending FQDN migration — which must NOT be generalized.
   FQDN `Path`s (with `::`/`.`) are named types, skipped. (`Path`s carry the `:`-prefixed spelling
   `":T"` here; strip it as `rename` does at `check.rs:13580`.) **Risk + STOP:** if any defn uses a
   bare *Uppercase* `Path` that is actually a NAMED type (not FQDN — e.g. a legacy bare `:Vector`),
   the collector would wrongly generalize it and the workspace gate would surface a new type error.
   That is the FQDN-always law being violated upstream — STOP and surface it; do NOT special-case it.
2. **Merge into `raw_type_params` at the real site:** `try_parse_fn_shape_def`
   (`runtime.rs:1998`–`2081`) computes `raw_type_params` from `split_name_and_type_params` and
   assigns `type_params: raw_type_params` at `runtime.rs:2081`. The build inserts, just before that
   assignment (both `param_types` + `ret_type` are already parsed by `runtime.rs:2073`):
   `raw_type_params ∪= free_vars(param_types ∪ ret_type)`. **Mirror at the two variadic sites:**
   `try_parse_variadic_def_fn_form` (`runtime.rs:~2118`/`2182`) and
   `try_parse_user_variadic_def_fn_form` (`runtime.rs:~2324`). (NOT `eval_fn`/`function/eval.rs:64`
   — that's the anonymous-`fn`-value path; named defns route through `try_parse_fn_shape_def`.)
3. **No downstream change.** `derive_scheme_from_function` (`check.rs:13689`) clones it; `instantiate`
   (`check.rs:13553`) freshens it; `rename` (`check.rs:13577`) + `unify` solve it. Already correct.
4. **The `<T,U>` name suffix becomes redundant** — once generalization also reads the signature, the
   angle-bracket decl adds nothing (the union makes them equal). 251.5 then strips `<T,U>` from
   every generic defn name (`stream/map`, `with-open-file`, …). Until then, dual-read holds: suffix
   form and bare-var form produce the same `type_params` set.

## The probe (`tests/probe_arc251_implicit_generics.rs` — written + run at HEAD)

Three contracts. F01/F02 lock the FACT (suffix-generics are really checked); R03 is the RED the
build flips:

- **F01 (fact):** `(defn :pair-first<T> [a <- :T b <- :T] -> :T a)` called `(pair-first 1 "two")`
  is **REJECTED** at HEAD → suffix-generics genuinely unify (not tolerated). Stays green forever.
- **F02 (fact):** `(pair-first 1 2)` checks at HEAD. Stays green.
- **R03 (RED→GREEN, load-bearing):** the faithful **bare-var-no-suffix** `(defn :pair-first2
  [a <- :T b <- :T] -> :T a)` with `(pair-first2 1 2)` — **FAILS at HEAD** (spurious mismatch;
  empty `type_params`). The probe asserts `is_err()` at HEAD. **At build time, flip R03's assertion
  to `is_ok()`** (the bare form must now check), AND add R03b: the ill-typed `(pair-first2 1 "two")`
  must still be **REJECTED** (proves the auto-generalized vars are *really* unified, not opaquely
  accepted — the same checked-vs-tolerated discriminator F01 applies to the suffix form).

Build-time additions: **occurs-check** (a signature forcing `T = (List T)` rejected cleanly — the
guard is on the generalized path) and a **two-instantiation** case (call the bare-var defn at
`i64` and at `bool` — distinct fresh vars per call site, no aliasing).

Per examinare: the probe already isolates *checked-vs-tolerated* (F01 is the template); the
foundation is ready.

## Out of scope (named, affirmative cuts — not deferrals)

- **`ann` / `All`** — rejected, not deferred. Off the generics critical path entirely; if a non-fn
  `def` ever needs var-annotation, that is its own stone, unrelated to generics.
- **`<T,U>` name-suffix retirement + corpus migration** — the unified **251.5** sweep owns it;
  this stone only makes the bare-var signature *authoritative*.
- **Generic `defclause` / multimethod dispatch over `∀T`** — the deeper goal in **task #198 / Arc
  256 (generic defclause)**. It *consumes* this model (it needs generalized clause types); it does
  not own it. This stone is its prerequisite, not its body.
- **Spelling sigil** (`K` vs `'a`) — an intueri taste call; bare `K` is the lean; not a blocker.

## Lineage

The ADT-compass cuts, in order this session: anonymous unions → anonymous tuples (`(wat.type/Tuple …)`)
→ `ann`+`All` (this stone). Each a core.typed retrofit a typed-from-birth language doesn't need.
Pairs `feedback_reach_lands_on_great_is_green.md` (the GREEN reach-tell — unification was a great
reached-for-and-found) + `project_typed_clojure_parity_pivot.md` (the campaign breadcrumb).
