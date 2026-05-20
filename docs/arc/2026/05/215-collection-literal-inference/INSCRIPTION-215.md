# INSCRIPTION — Arc 215 — Collection literal inference + `:wat::type::Infer` mint

**Closed:** 2026-05-20
**Stones:** 2 (Stone 1 — mint + map/set literal; Stone 2 — Vector unification + keyword-key lift)
**Status:** SHIPPED. LLM-first claim operational.

---

## What this arc declared

Three Clojure-style collection literals — `{...}`, `#{...}`, `[...]` — all route through one unified type-inference mechanism (`:wat::type::Infer`). Type discipline is brutally honest: first-unit determines K/V/T; all subsequent must unify; mixed-type fails at check with position-named diagnostic. Verbose verb form remains for polymorphic-bag cases (`:wat::holon::HolonAST` top-type + explicit construction).

**Two-layer enforcement:**

1. Literal coherence at check time (one literal = one K, one V, one T)
2. Function-signature unification at the call site (the type the function expects vs the type the literal produces)

**The substrate-truth alignment:** every constraint that survives is structurally honest with the runtime Rust types (`HashMap<K,V>`, `HashSet<T>`, `Vec<T>`). No magic auto-coercion. No atomizable-set hazards. No `Atom`-wrapping silently lying about types.

## What this arc retired

- P2's Atom auto-wrap on `{...}` values (Stone 1 — Probe 5's class of failure eliminated)
- P2's pinned `:wat::holon::HolonAST` V-type (Stone 1)
- The dual-mechanism for `[...]` vs `{...}` and `#{...}` (Stone 2 Change A)
- The arbitrary keyword-key parse-time restriction on `{...}` (Stone 2 Change B)
- The `BraceKind::Malformed` variant in the outer brace dispatch (Stone 2 D3 consolidation)
- `tests/wat_arc167_vector_ast.rs` tests 4+5 asserting "vector literals at value position are not supported" (Stone 2 D5 — runtime now supports them)

## What this arc preserves

- `{outcome residue}` struct-destructure in let-binder position (arc 169)
- `[x y z]` tuple-destructure in let-binder position (arc 167)
- P1's explicit verb form `(:wat::core::HashMap :K :V k v ...)` and equivalent Vector / HashSet shapes
- `:wat::holon::Atom` polymorphism for explicit holon construction (atomizable set unchanged; only literal sugar stops forcing values through it)
- D2 fix from P2: check-time rejection of List binders in let-binding position

## What this arc deferred

- **`'(...)` list literal** — PERMANENTLY deferred per LLM-first analysis. Idiomatic Clojure usage of list literal is statistically zero; the verb form plus `:wat::core::List<T>` substrate (task #283) suffice for the rare cases requiring linked-list semantics.
- **`infer_list_constructor` rename** — intueri Level-1 lie (works on Vector despite "list" naming; arc 109 slice 1g retirement leftover). Future arc territory.
- **Match-arm `{...}` / `#{...}` / `[...]` patterns** — task #402 stays separate.
- **ProgramEnv specifics** — arc 214 Slice 4 (#385); now structurally unblocked.

## The LLM-first delivery claim — operational

Any LLM that knows Clojure data literals writes meaningful wat literals with no friction:

```wat
;; Maps: keyword keys, int keys, string keys — all just work
{:foo 42 :bar 100}
{1 "v" 2 "w"}
{"a" 1 "b" 2}

;; Sets: any uniform element type
#{1 2 3}
#{"alice" "bob"}

;; Vectors: any uniform element type
[1 2 3]
[true false true]

;; Nested: the substrate composes
{:outer {:inner 42}}
{:tags #{"prod" "primary"} :limits [100 200 300]}
```

Mixed-type literals fail at check time with clear position-named diagnostics. Polymorphic-bag cases drop to the verb form with explicit `:wat::holon::HolonAST` top-type.

This isn't aspirational. It's structural. The substrate ships this surface as a load-bearing pedagogy for AI co-authors (per `project_wat_llm_first_design`).

## Failure-engineering classes eliminated

Four classes eliminated across Stone 1 + Stone 2:

1. "Literal-syntax produces values incompatible with what algebra needs" (Stone 1)
2. "Dual-mechanism for `[...]` vs `{...}` and `#{...}`" (Stone 2 Change A)
3. "Arbitrary parser-layer restriction blocks Clojure-native syntax" (Stone 2 Change B)
4. "Auto-coerce-with-Atom hides type information" (data-as-default discipline, ratified by both stones)

Each class is **structurally unrepresentable** in the new design, not "less likely."

## Convergence-with-substrate continues

Convergence #7 in the lineage (arc 199 → 214 P1 → Slice 2 → DESIGN forward-correction → 214 P1 second pass → arc 215 Stone 1 → arc 215 Stone 2). Every stone in this arc: the substrate already had the answer; the work was routing literal sugar through existing inference machinery.

- `infer_hashmap_constructor` already had `fresh.fresh()` fallback (Stone 1 routed K + V through it; Stone 2 extended the lift)
- `infer_hashset_constructor` already had the same fallback (Stone 1)
- `infer_list_constructor` already had the same fallback (Stone 2 routed `[...]` through it)
- HashMap key-type substrate (`hashmap_key`) already accepted any `Value` shape (arc 057 slice 3 prefigured Stone 2's lift)

The compression keeps holding: years of failure-engineering discipline applied at high intensity to weeks of substrate work.

## Calibration record

| Stone | Predicted | Actual | Notes |
|---|---|---|---|
| Stone 1 | 60-90 min Mode A | ~60 min | Low end of band; D2 cross-cut absorbed cleanly |
| Stone 2 | 45-75 min Mode A | ~55 min (post-firewall-recovery) | Mid-band; runtime path discovered (D1); arc 167 tests updated (D5) |

Stone 2's first spawn returned ~30s with a "needs bash" complaint — diagnosed as Anthropic firewall pattern-matching on complex agent prompts (`feedback_sonnet_bash_firewall` memory inscribed). Re-spawn with simplified agent prompt + scorecard alignment cleanup shipped cleanly.

## References

- **DESIGN.md** — arc-level design
- **BRIEF-215-STONE-1-INFER-AND-LITERAL-COMPLETION.md** — Stone 1 spec
- **EXPECTATIONS-215-STONE-1-INFER-AND-LITERAL-COMPLETION.md** — Stone 1 scorecard
- **SCORE-215-STONE-1.md** — Stone 1 ship record
- **BRIEF-215-STONE-2-VECTOR-UNIFICATION.md** — Stone 2 spec (bundled scope)
- **EXPECTATIONS-215-STONE-2-VECTOR-UNIFICATION.md** — Stone 2 scorecard
- **SCORE-215-STONE-2.md** — Stone 2 ship record
- INTERSTITIAL entries in `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` (Stone 1 + Stone 2 closure entries; orchestrator-direct voice)
- arc 058 audit history entries in `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/INDEX.md`

## What's next

Arc 215 closes. Arc 214 Slice 4 (#385) resumes — kernel layer + ProgramEnv with the unified literal sugar as the configuration construction surface.

*Three literals. One mental model. The substrate dreamed the inference. So did we.*
