# DESIGN — Stone S-A — the is-a hierarchy mechanism (`typesub` + `subtype?`)

**Arc:** 237, records-first-class thread (see `DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md`).
**Status:** READY (sub-DESIGN). First substrate stone of the records thread.
**Gate:** S0 GREEN (`da059f42`) — macro-emitted type decls self-register; T1 satisfied.

## Why this stone

Records can't be first-class types and can't carry the `:wat::holon::Record <:
:wat::Record` flavor relation until wat has an **open directional is-a
hierarchy** (Clojure's `isa?`/`derive` axis — distinct from `typeunion`'s closed
sum and `defprotocol`'s behavior). This stone mints that mechanism — and ONLY
the mechanism. It touches **zero** `unify` call sites; it is self-contained and
testable through `subtype?` + `conforms?` directly.

**Scope discovery (S0 recon, 2026-05-25):** check.rs has **76** `unify(` call
sites, ~50+ of them the open-coded shape `unify(actual, expected).is_err()`
("does this actual type satisfy this expected slot"), spanning arg-binding /
return-position / let-binding / collection-element / leaf-invariant contexts.
There is NO shared arg-satisfaction helper today. Wiring those sites to consult
the hierarchy is a SEPARATE concern (Stone S-A1: mint `assignable`, route the
value-into-slot sites through it). **S-A does not touch them** — it builds the
thing they will later consult. This keeps S-A small, green, and isolated.

## What this stone delivers

Four pieces, all in `src/types.rs` + `src/runtime.rs` + `src/check.rs`
(inference scheme only), NO unify-site edits:

1. **The edge registry** — `TypeEnv` gains a child→parents map (the `typesub`
   relation) + `register_subtype(child, parent)`. Standalone from `TypeDef`
   kinds (a tag can derive regardless of what kind it is) — mirrors Clojure's
   hierarchy being orthogonal to what the tags ARE.
2. **`is_subtype(sub, super, types) -> bool`** — the directional, transitive,
   reflexive walk UP `sub`'s parent chain. The ONE internal authority; consumed
   by `subtype?`, by `conforms?`, and later by S-A1's `assignable`.
3. **`:wat::core::subtype?`** — the wat-surface predicate `keyword × keyword ->
   bool`, sibling of `conforms?` (which is `value × type -> bool`). Body routes
   to `is_subtype`.
4. **`conforms?` gains the parent-walk** — when nominal identity fails, consult
   `is_subtype(value's declared type, target)`. So `(conforms? holonic-val
   :wat::Record)` → true via the edge, not just exact match.
   Plus: the two built-in roots — `:wat::Record` (exists) and the NEW
   `:wat::holon::Record`, with `register_subtype(":wat::holon::Record",
   ":wat::Record")` seeded in `register_builtin_types`.

## The algorithm — `is_subtype(sub, super, types)`

```
is_subtype(sub, super):
    if sub == super: return true                       # reflexive
    # walk sub's parent edges transitively, looking for super
    visited = {}
    stack = parents_of(sub)                            # registry lookup
    while stack not empty:
        p = stack.pop()
        if p == super: return true
        if p in visited: continue
        visited.insert(p)
        stack.extend(parents_of(p))                    # transitive
    return false
```

- **Directional:** walks `sub` UP toward `super`. `is_subtype(super, sub)` is
  NOT the same call and returns false unless an edge exists the other way.
  (This is precisely why it is NOT `unify` — `unify` is symmetric.)
- **Transitive:** `Sphere → holon::Record → Record` ⇒ `is_subtype(Sphere,
  Record)` true.
- **Acyclic:** edges are registered acyclically (defensive `visited` guard
  bounds the walk regardless; cycle-rejection at `register_subtype` time mirrors
  `check_union_no_cycle`).
- **Leaf-safe:** a type with no parent edges (bool, keyword, i64, …) → walk is
  empty → false. So `is_subtype` is inert for everything outside the hierarchy.

## Error / behavior contract

- `subtype?` on two well-formed type names → `bool` (true/false), never error
  for a legitimate non-edge. Unknown type name → error (mirror `conforms?`'s
  unknown-name contract: bad input, not a false).
- `conforms?` semantics UNCHANGED except the added parent-walk fallback: a value
  that nominally-conforms still returns true (fast path first); a value whose
  declared type *derives* the target now ALSO returns true (new). Nothing that
  returned true before returns false.
- **Baseline invariant:** lib tests 827/0 MUST hold. No `unify` site changes →
  no arg-checking behavior change → existing tests unaffected. (This is the
  green-isolation guarantee that makes S-A a safe stepping stone.)

## Surface / files

- `src/types.rs` — the edge registry field on `TypeEnv` + `register_subtype` +
  cycle-check + `is_subtype` (or place `is_subtype` here as the type-level
  authority) + seed the two roots in `register_builtin_types`.
- `src/runtime.rs` — register `:wat::core::subtype?` in the eval dispatch
  (mirror `:wat::core::conforms?` at ~5291 / `eval_conforms` at ~16087);
  extend `conforms_check` (~16130) Path-arm with the `is_subtype` fallback.
- `src/check.rs` — inference scheme for `subtype?`: `(:fn(:keyword, :keyword) ->
  :wat::core::bool)` — both args type-position keywords (mirror how `conforms?`'s
  type-arg is handled; labels-are-ASTs, no String→keyword wrap).
- NO new `Value` variant. NO holon-rs (STOP-5). **NO `unify` call-site edits**
  (that is S-A1).

## Proven-moves template (mirror these — arcs 234.0 / 232.0 / 237.5 / 226)

Surveyed 2026-05-25 across the last four `:wat::core` primitive mints. The
canonical maneuver + the trap-doors we have ALREADY paid for — the BRIEF must
mirror these so Sonnet ships one-shot:

**The wiring (2 rounds: runtime.rs, then check.rs; types.rs additionally here for the registry):**
- `runtime.rs` dispatch arm in `dispatch_keyword_head_value` (~5266), beside the
  `:wat::core::conforms?` arm (~5291): `":wat::core::subtype?" => eval_subtype(...)`.
- `eval_subtype` mirrors `eval_conforms` (@16087): arity-2, parse BOTH args via the
  type-slot parser (both are type-position keywords — neither is `eval`'d as a
  value), acquire `sym.types()` via `.ok_or_else(...)`, call `is_subtype`.
- `check.rs`: **the `infer_list` special-case arm (~5561, beside conforms?) is the
  load-bearing checker piece — the `register_builtins` TypeScheme (~19317) is only
  a sentinel.**
- NO new `Value` variant → **0 cascade files** (same as 234.0 / 237.5).

**Trap-doors (pre-empt all):**
1. **Type-keyword-infers-as-`Fn`** (237.5): a type keyword that names a registered
   constructor (`:my::Circle`) infers as `Fn(…)->Record`, NOT `:keyword`. The
   `register_builtins` TypeScheme alone does NOT fix this. The `infer_list` arm must
   validate each arg is `WatAST::Keyword(_,_)` and **skip inference**. For
   `subtype?`, BOTH args are type-keywords → skip BOTH. NO `_discard` drain needed
   (unlike conforms?, which has a value arg0).
2. **`declared_type_name`, never `type_name()`** (237.5.fix): the `conforms?`
   parent-walk reads `value.declared_type_name()` (runtime.rs:1298). `type_name()`
   returns the generic kind (`"wat::Record"` / `"wat::core::Enum"`) and silently
   drifts. The 237.5.fix consolidated this to one wildcard-free authority — route
   through it.
3. **`t_var()` not `TypeExpr::Var("T")`** (234.0): `Var` takes `u64`. MOOT for
   `subtype?` — no type var; `type_params: vec![]`, both params
   `:wat::core::keyword`, ret `bool`.
4. **records-aren't-in-TypeEnv** (237.5): a record's identity is `class_fqdn`, not a
   `TypeDef`. So S-A's walk operates on hand-registered edges + the seeded roots;
   records-derive-an-edge is S-B. (Probe #8 constructs it via a hand-registered
   edge + a struct value, not a record.)

**The one place the precedent MISLEADS — inscribe explicitly:** do NOT make
`is_subtype` delegate to `collect_union_members`. That is typeunion *membership*
(a closed sum); the hierarchy is the NEW `typesub` child→parent edge-registry — a
distinct relation. `is_subtype` walks the new map, full stop. Conflating them
collapses the isa?-vs-typeunion distinction this whole thread rests on.

**SCORE shape:** mirror SCORE-STONE-237.5 — scorecard (LOAD-BEARING probe row + lib
baseline 827 + predecessor-regression guards + holon-rs-untouched) → Final API
shape → Line count → Cascade depth → Honest deltas → Working tree. Cite 237.5 SCORE
in the BRIEF.

## Out of scope (REJECTED — not deferral)

- **The `assignable` choke point + `unify`-site routing** — Stone S-A1. Until it
  ships, `[v <- :wat::Record]` does NOT yet accept a holonic-typed arg at the
  static boundary; `subtype?`/`conforms?` work directly. Affirmative cut: S-A is
  the mechanism, S-A1 is the consumption.
- **Records registering `typesub` edges** — Stone S-B (records-as-TypeDef);
  the macro calls `register_subtype` once it exists here.
- **The macro split** (`:wat::Record::def` / `:wat::holon::Record::def`) — S-C.
- **User-facing `:wat::core::derive`** — minimal-form: the roots are seeded
  internally; records derive via S-B. A user-facing derive verb ships only when
  a non-record user hierarchy surfaces.

## FM 2-bis probe (NEW — committed before the BRIEF)

`tests/probe_arc237_sA_hierarchy.rs`. Pre-stone: does not compile
(`register_subtype` / `is_subtype` / `:wat::core::subtype?` don't exist).
Post-stone: all PASS. Contracts:

1. **edge + directional** — `register_subtype(:Child, :Parent)`; `is_subtype(Child,
   Parent)` true; `is_subtype(Parent, Child)` false.
2. **transitive** — A→B, B→C ⇒ `is_subtype(A, C)` true.
3. **reflexive** — `is_subtype(T, T)` true.
4. **leaf-safe** — `is_subtype(:wat::core::bool, :wat::core::i64)` false (no edges).
5. **cycle rejected** — `register_subtype` closing a cycle → error.
6. **built-in roots** — `is_subtype(:wat::holon::Record, :wat::Record)` true;
   reverse false.
7. **wat surface `subtype?`** — `(subtype? :wat::holon::Record :wat::Record)` →
   true; `(subtype? :wat::Record :wat::holon::Record)` → false.
8. **conforms? parent-walk** — a value whose declared type derives `:wat::Record`
   → `(conforms? v :wat::Record)` true (exercises the new fallback). (Constructed
   via a hand-registered edge + a struct value, since records-derive is S-B.)
9. **conforms? unchanged for nominal** — existing nominal conformance still true;
   non-conformant still false (no regression).
10. **subtype? unknown name** → error (input-validation contract).

Plus baseline: `cargo test --release --lib` ≥ 827/0 (no unify-site change).

## Calibration

New registry field + a graph walk + one wat primitive (mirror `subtype?` on
`conforms?` 237.5) + a conforms? fallback arm + two seeded roots. Comparable to
237.5 (new primitive + recursive walk) but simpler (no full grammar recursion —
a flat parent-chain walk). Sits in the proven 234.0/237.5 tier (38 min / in-band
of 40–75). **Target band: 40–70 min Mode A; 90 STOP-3; 120 STOP-4. Cascade: 2
rounds, 0 forced files (no new Value variant).** Mirror Stone 237.5 SCORE
structural shape; cite 237.5 SCORE in the BRIEF.
