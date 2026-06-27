# BRIEF — Stone P12c: the EXPLAIN payload (`DerivationStep` + `:constraints`/`:bindings`/`:pattern`/`rule`)

**Executor:** one **sonnet** Shadowdancer. **No sub-agents. No `git`. No worktrees.** Do NOT run
`cargo wat` (orchestrator-only). `cargo test` only. EMBED + read fully:
`DESIGN-STONE-P12c-explain-payload.md` (the contract) before touching code.

## The work (one paragraph)
Enrich the P12b derivation tree with the operator-legibility payload. Add a `DerivationStep` edge record; give
`DerivationNode` a `rule` (`Option<String>`) and change `via` to `PV<DerivationStep>`. Each step carries: the
supporting fact's own `DerivationNode` (recurse), the matched `pattern` (type FQDN), the per-step `bindings`
(projected to that condition's vars), and `constraints` (the rule's satisfied predicates with the bound values
substituted — `(:wat::core::< -5 0)`). The constraint substitution is a Rust helper that **reuses the matcher's
`resolve_operand` + clause classifier** (faithful by construction). Then un-ignore the 6 P12c probe tests; the
P12 north-star must STAY green (its via-counts are unchanged).

## The records (rete.wat — replace the P12b `DerivationNode`)
```clojure
(:wat::Record::def :wat::rete::DerivationNode
  [fact <- :wat::Record
   rule <- :wat::core::Option<wat::core::String>
   via  <- :wat::core::PersistentVector<wat::rete::DerivationStep>])
(:wat::Record::def :wat::rete::DerivationStep
  [supporting  <- :wat::rete::DerivationNode
   pattern     <- :wat::core::String
   bindings    <- :wat::core::PersistentMap<wat::core::String, wat::core::Value>
   constraints <- :wat::core::PersistentVector<wat::WatAST>])
```
Mutual recursion confirmed type-checks (probe this session). `rule`: `(:wat::core::Some r)` derived / bare
`:wat::core::None` base (both confirmed — `Some` is a call, `None` is a bare value, NOT `(None)`).

## Read in order (the rooms)
1. `wat/rete.wat` — the P12b `DerivationNode` + `explain` walk (search `P12b`), the `Support {rule, token}` +
   `Token {matches, bindings}` + `AlphaNode {id, tests, children}` records. `Support/rule` gives the node's
   rule; `Token/matches` = `PV<(fact, alpha-id)>`; `Token/bindings` = the accumulated bindings.
2. `src/rete/matcher.rs` — **reuse, do not duplicate**: the clause classifier (`alpha_match_inner` ~:217 binder
   `(?v <- :field)`, ~:249 constraint `(:op a b)`), `resolve_operand` (~:325: `?v→bindings`, `:field→fact
   field`, literal→itself), and how a rete Rust primitive is built + registered (`eval-insert`/`build_insert_fact`
   ~:380, its dispatch + check registration). `fact_from_value` extracts a record's fields.
3. `src/rete/kernel.rs` — `get_node`/`node_record`/`kind_of` to fetch an `AlphaNode` by id from the network;
   `token_to_value` / Value-record construction patterns.
4. `src/runtime.rs:4015` (dispatch) + `src/check.rs` (the `fire-rules-explain'` TypeScheme) — register the new
   `step-payload` verb the same way.
5. `tests/probe_arc278_P12c_explain_payload.rs` — the 6 acceptance asserts (un-ignore). `tests/probe_arc278_P12_explain_walk.rs` — the north-star (must stay green).

## Implementation sketch (fill it; reuse matcher.rs primitives)
- **`step-payload` Rust primitive** — `(:wat::rete::step-payload <session> <alpha-id> <bindings> <supporting-fact>)`
  returning a `DerivationStep`-payload (pattern, bindings, constraints) — or three thin readers if cleaner:
  1. `alpha-id → AlphaNode` (kernel `get_node` over `session.network`); read `AlphaNode.tests`.
  2. classify each test clause with the matcher's OWN classifier; **constraints** = for each `(:op a b)`,
     `resolve_operand` each operand (vs the supporting fact's fields + the bindings), rebuild `(:op a' b')` as a
     `WatAST` (operands → literal `WatAST` nodes via runtime quasiquote or a direct constructor) → `PV<WatAST>`.
  3. **bindings (per-step)** = the binder clauses' `?var` names → project the token `bindings` to those.
  4. **pattern** = the condition's fact-type FQDN.
- **`explain` walk (wat, restructure)** — for each `(sfact, alpha-id)` in `(Token/matches (Support/token sv))`:
  `DerivationStep{ supporting = (explain ex sfact),  ...(step-payload (Explained/session ex) alpha-id (Token/bindings (Support/token sv)) sfact) }`.
  Node: `DerivationNode{ fact, rule = (Some (Support/rule sv)) | None, via = <the steps> }`.

## Blast radius (bounded)
- `wat/rete.wat` — the two records + the `explain` restructure + the `step-payload` wrapper. Additive to the
  oracle (no fire-path change).
- `src/rete/matcher.rs` — the `step-payload` helper (REUSING resolve_operand + the classifier). `src/runtime.rs`
  + `src/check.rs` — register it.
- The 2 probes (un-ignore 6 in P12c; the north-star's 2 stay).
- **NOT** any fire path / `Explained` / `Support` / `fire-rules-explain` change (P12a done). **NOT** the flat-DAG
  sharing form, the misfire overlay (stone ③), or a pretty-printer (arc 288).

## STOP triggers
1. STOP if `step-payload` would DUPLICATE `resolve_operand`/the classifier instead of calling them — faithfulness
   depends on reuse (a re-impl can disagree with what actually fired).
2. STOP if `:constraints` does not render as the form `(:wat::core::< -5 0)` (the WatAST-render fix `20722898`
   must be present; if it prints opaque/nil, surface it).
3. STOP if the P12 north-star via-counts change (the restructure must preserve via length = # support edges).
4. STOP if mutual-recursive `DerivationNode`/`DerivationStep` is rejected (probe says it type-checks).
5. STOP if any rete differential (`probe_arc278_P4a`/`P4c`/`P2`/`P12a`) or a floor regresses.

## Acceptance (run yourself, report exact output)
- `cargo test --release -p wat --test probe_arc278_P12c_explain_payload` → **6 passed; 0 failed; 0 ignored**.
- `cargo test --release -p wat --test probe_arc278_P12_explain_walk` → **2 passed** (north-star still green).
- `cargo test --release -p wat --test probe_arc278_P12a_explain_substrate --test probe_arc278_P4a_native_fire_rules --test probe_arc278_P4c_native_retraction` → green (differential).
- `cargo build --release` → clean (25 warnings baseline). lib `cargo test --release -p wat --lib -- --test-threads=1 | grep "test result"` → 941/36 (no NEW failures).

## Prior comparable (copy the shape)
- `eval-insert`/`build_insert_fact` (matcher.rs ~:380) — a pure rete Rust primitive reusing `resolve_operand`,
  registered via runtime dispatch + check TypeScheme. The `step-payload` helper mirrors this.
- The P12b `explain` walk (rete.wat) — the recursion you're enriching.
- `token_to_value` (kernel.rs:503) — native→wat Value record construction.
