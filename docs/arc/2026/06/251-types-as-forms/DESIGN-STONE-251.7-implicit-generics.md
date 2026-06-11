# DESIGN — Stone 251.7: implicit generics (Hindley-Milner, bare type-vars)

**Status: DESIGN-CAPTURED, BUILD-DEFERRED (drawn 2026-06-10 on a grounded crawl; no probe yet —
this stone is design-only by direction "we haven't built it; get the model on disk").** The
build is a real checker feature (a modest, bounded one — see The strike); it lands when 251 reaches
it. Home: `src/check.rs` (+ a small parser touch for bare type-vars).

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

## The disk (grounded, 2026-06-10) — and a premise correction

The HM machinery is **all present**:

- `TypeScheme { type_params: Vec<String>, params, ret, rest_param_type }` (`check.rs:79`).
  `type_params` is documented as the `∀`-bound variable names (`check.rs:64`).
- `derive_scheme_from_function` (`check.rs:13678`) builds a fn's scheme via
  `type_params: func.type_params.clone()` (`check.rs:13689`).
- `instantiate(scheme, fresh)` (`check.rs:13553`) — at each call site, one fresh var per
  `type_params` entry (the HM *instantiate* step).
- `unify` (`check.rs:13149`), `unify_types` (`check.rs:13417`), `fresh_var` (`check.rs:13409`);
  rigid type-vars unify only with the same name (`unify_rigid_vars_require_same_name`,
  `check.rs:18477`); unresolved TypeVars are leniently accepted/deferred (`check.rs:11493`).

**The correction the crawl forced** (the breadcrumb over-stated this — weighed against disk):
`Function.type_params` (`value/environment.rs:44`) is **never populated non-empty**. Every
constructor sets `vec![]` / `Vec::new()` (`environment.rs:224/245`, `function/eval.rs:64`); there
are **zero** `.type_params =` mutations anywhere in `src/`; and `parse_declared_name` (the `<T>`
stripper) is called **only for type declarations** (typeunion/newtype/typealias `types.rs:1669/1832/
1874/1925`, struct `types/defstruct.rs:335`), **never for a fn name**.

→ Therefore a user generic fn (`stream/map`'s `<T,U>`) does **not** carry real `∀`-checked params
today. It passes only because **free type-vars are leniently accepted** (`check.rs:11493`) — not
because the checker truly generalizes-and-instantiates them. **This stone is the first REAL
population of `Function.type_params` for user defns** — it turns leniently-tolerated generics into
genuinely-`∀`-checked ones. (This is a *stronger* motivation than "redirect an existing population":
there is no existing population.)

## The strike (the build, when reached)

The contract decision, pinned to ONE site and ONE shape:

> **At fn registration, AUTO-GENERALIZE: populate `Function.type_params` (today always `[]`) by
> collecting the free, un-namespaced (bare-`Path`, no `::`/`.`) type-variable symbols that appear
> in the signature's param `TypeExpr`s and return `TypeExpr`, in first-occurrence order. Everything
> downstream — `derive_scheme_from_function`, `instantiate`, `unify` — is UNCHANGED; it already
> consumes `type_params` correctly. The build changes only WHERE `type_params` comes from: the
> signature's free vars, not a `<T>` name suffix.**

Sketch (rooms named; the build's lair-study pins exact line ranges):

1. **A free-var collector** over `TypeExpr` — walk a `TypeExpr`, emit each bare `Path` (no `::`/`.`
   → a type-var per the FQDN-always law; an FQDN `Path` is a named type, skipped) and recurse into
   `Parametric.args`, `Fn.{args,ret}`, `Tuple.elems`. First-occurrence order, deduped. (Mirrors the
   discriminator `keyword/to-type-form` already encodes: `Path` with `::` → named, without → var —
   `edn_shim`, shipped `18c6c3c0`.)
2. **Populate `Function.type_params`** at the fn-construction site that today hard-codes `vec![]`
   (the build's first task: confirm which of `function/eval.rs:64` / `value/environment.rs:224/245`
   is the live user-defn path; the others are built-in/closure paths that legitimately stay empty).
   The generalization is: `func.type_params = free_vars(params ∪ ret)`.
3. **No downstream change.** `derive_scheme_from_function` clones it; `instantiate` freshens it;
   `unify` solves it. The rigid-var-by-name unify (`check.rs:18477`) already gives the in-body
   discipline (a `K` in the body unifies only with `K`).
4. **The `<T,U>` name suffix becomes droppable** — once generalization reads the signature, the
   angle-bracket decl carries no information. The 251.5 corpus sweep / hard-cut then strips `<T,U>`
   from every generic defn name (`stream/map`, `with-open-file`, …). Until then, dual-read: a name
   with `<T,U>` and a bare-var signature both produce the same `type_params`.

## The probe (RED at HEAD — to be written at build time)

`tests/probe_arc251_implicit_generics.rs`, the load-bearing disconfirmer:

- **C01 (RED→GREEN):** a `user/put`-shaped generic defn written with **bare-var signature and NO
  `<T,U>` name suffix**; call it at two distinct instantiations (`String/i64` and `bool/String`),
  each load-bearing — both must check. RED at HEAD because `Function.type_params` stays `[]`, so
  `instantiate` has nothing to freshen and the bare `K`/`V` either leak as rigid mismatches or pass
  only by the lenient-accept path (the probe must distinguish *truly checked* from *leniently
  tolerated* — e.g. a deliberately ill-typed call `(user/put some-map "name" 42)` against a
  `HashMap<i64,i64>` map must be **rejected**, which lenient-accept would wrongly pass).
- **C02 (occurs-check):** a signature that would force `T = (List T)` is rejected cleanly (proves
  the unifier's occurs-guard is on the generalized path).
- **C03 (dual-read):** the `<T,U>`-name spelling still checks identically (PRESERVATION — the name
  suffix retires at 251.5, not here).

Per examinare: if the probe cannot isolate *checked-vs-tolerated*, the foundation is not ready —
build the distinguishing harness first.

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
