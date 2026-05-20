# Arc 215 — Collection literal inference + `#{...}` set literal mint

**Status:** DESIGN inscribed 2026-05-20.
**Trigger:** P2 (arc 214 parser-pivot, commit 3230a9d) shipped `{...}` map literal with Atom auto-wrap on every value. Probe 5 surfaced: nested `{:outer {:inner 42}}` fails at runtime because `value_to_atom` doesn't accept HashMap values. The class of failure: literal desugar uses uniform HolonAST V-type via Atom wrap; Atom polymorphism is real but bounded; nested-collection cases fall outside the atomizable set.
**Discipline:** Failure engineering — eliminate the class, don't patch the symptom.

## What this arc declares

The `{...}` and `#{...}` literals desugar to Rust collection-constructor verb-calls with **type inference** delegated to the type-checker via a new substrate primitive: `:wat::type::Infer`.

Concretely:

```
[2 5 6]                       → existing arc 167 WatAST::Vector path (unchanged)
{:foo 42 :bar 43}             → (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :foo 42 :bar 43)
{:outer {:inner 42}}          → (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :outer {:inner 42})
                                 — recursion: inner {...} desugars same way; V infers to HashMap<keyword, i64>
{:foo "x" :bar "y"}           → (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :foo "x" :bar "y")
                                 — V inferred to :wat::core::String
#{1 2 3}                      → (:wat::core::HashSet :wat::type::Infer 1 2 3)
                                 — T inferred to :wat::core::i64
```

K for map literals is always `:wat::core::keyword` (structural rule at parse; non-keyword keys fail at parse with `MalformedBraceLiteral`).

V (HashMap) and T (HashSet) are inferred from the first value/element; mismatched subsequent values fail at check time with `TypeMismatch` diagnostic naming the offending position.

## What this retires

- **P2's Atom auto-wrap.** Probe 5's `Atom(HashMap)` runtime failure class dissolves. No value gets auto-wrapped; values pass through to the inference machinery as-is.
- **P2's pinned `:wat::holon::HolonAST` V-type.** V is now inferred per-literal from the actual value types.

## What this preserves

- P2's structural rule: map-literal keys MUST be keywords at parse (enforced by `MalformedBraceLiteral`)
- P2's struct-destructure path: `{outcome residue}` in let-binder position still parses as `StructPattern` (arc 169)
- P2's D2 fix: `process_let_binding` rejects List binders at check time
- P1's HashMap explicit verb form: `(:wat::core::HashMap :K :V k v ...)` continues to work for callers wanting explicit types

## Substrate primitive: `:wat::type::Infer`

A type-placeholder. Rust's `_` in type position; Haskell's `_`. Documented semantics:

- Appears in type-arg slots of parametric constructor calls
- Tells check.rs to infer this type from the values
- Failure to infer (e.g., empty literal with no values) → fresh type variable (existing HM-style behavior)
- Documented at the substrate level (CONVENTIONS + WAT-CHEATSHEET); future arcs may extend usage

Mint as a registered keyword-type in `types.rs`; parse_type_expr returns it; the check.rs constructor handlers detect it and switch to inference mode.

**User-facing benefit beyond literals:** writers can use the explicit verb form with inference:
```
(:wat::core::Vector :wat::type::Infer 1 2 3)         ; same as [1 2 3]
(:wat::core::HashMap :wat::type::Infer :wat::type::Infer :foo 1 :bar 2)  ; full inference
```

## Inference rules

**HashMap** `(:wat::core::HashMap K V k1 v1 k2 v2 ...)`:
- K is `:wat::type::Infer` → infer from first key; verify all subsequent keys unify
- K is a real type → use it; verify all keys unify (existing P1 behavior)
- V is `:wat::type::Infer` → infer from first value; verify all subsequent values unify
- V is a real type → use it (existing P1 behavior)
- Empty literal `{}` → K, V both fresh type variables (acceptable; concrete-type unification happens at first use)

**HashSet** `(:wat::core::HashSet T x1 x2 ...)`:
- T is `:wat::type::Infer` → infer from first element; verify all subsequent unify
- T is a real type → use it (existing behavior)
- Empty literal `#{}` → T is fresh type variable
- Dedup happens at construction (existing HashSet behavior); duplicates in source are not an error

## Failure modes (all eliminated structurally)

| Mode | Old behavior (P2) | New behavior |
|---|---|---|
| Nested map literal `{:k {:k 1}}` | Runtime TypeMismatch (Atom can't wrap HashMap) | Check infers V as nested HashMap; type-check passes; runtime works |
| Mixed-value-type map `{:a 1 :b "x"}` | Auto-wraps both via Atom → silently accepts polymorphic HolonAST | Check fails at the offending value with TypeMismatch; position-named |
| Mixed-element-type set `#{1 :foo}` | (new path) | Check fails at first non-unifying element |
| Empty literals `{}`, `#{}` | (P2 handles empty `{}` via P2 fix) | Both produce empty collection with fresh type variable; type concretizes at first use |
| Key non-keyword `{42 :v}` | MalformedBraceLiteral at parse (P2 rule preserved) | Same — preserved |
| Keyword in binder `(let [{:foo bar} ...] ...)` | MalformedForm at check via D2 fix | Same — preserved |

## Out of scope (deferred)

- **`'(...)` list literal** — substrate `:wat::core::List<T>` (task #283) must land first; reader macro then mechanical
- **`[...]` retarget to `Infer` form** — existing `WatAST::Vector` path stays; future arc may unify all literal paths through verb-call + `Infer`
- **`(:wat::holon::*)` constructions from literals** — algebra opt-in via explicit conversion verbs; not literal default per user direction "literals are data; holon is the algebraic view"
- **Match-arm patterns** for `{...}` / `[...]` / `#{...}` — task #402 remains separate
- **WARD-PASS** — parser + check + types out-of-zone per `feedback_ward_zone_comms_only`

## Convergence-with-substrate (the meta-pattern this arc continues)

- arc 199 — REJECTED (substrate already sufficient)
- arc 214 P1 — HashMap verb-form already had constructor; refactor to symmetric, not mint
- arc 214 Slice 2 forward-correction — bounded(N) retired; pair() at mini-TCP depth 1 already what 22/22 callers used
- arc 214 DESIGN forward-correction — io_uring depth knob rejected; ring-rebuild on structural need
- arc 215 (this) — Atom polymorphism was load-bearing; the fix is not "tighten Atom" but "stop forcing literals through Atom"; the substrate's inference machinery already does the right thing — we just route literals through it

Convergence #7 inside this lineage. The substrate has the answer; the literal sugar exposes it.

## Stages

Single stone covers the full pivot. Substrate change is small (one new placeholder type + inference detection in two existing handlers); parser change is mechanical; probe matrix is parallel to P1/P2.

- **Stone 215.1** — mint `Infer` + extend inference mode + adjust `{...}` parser + add `#{...}` parser dispatch + probes + docs + retroactively amend P2's SCORE to mark Probe 5's LIMITATION resolved.

Estimated 60-90 min Mode A.

## Closure paperwork

After Stone 215.1 lands:
- INSCRIPTION-215.md
- arc 058 row entry
- USER-GUIDE updates
- WAT-CHEATSHEET § 8 final state
- INTERSTITIAL entry (orchestrator-direct per `feedback_sonnet_no_realization_voice`) capturing convergence #7

*Literals are data. Holon is the algebraic view. The substrate already had the inference.*
