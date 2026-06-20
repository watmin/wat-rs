# DESIGN — Stone P12c: the EXPLAIN payload (`:constraints` / `:bindings` / `:pattern` — operator legibility)

The third and final P12 sub-stone. P12a built the substrate (`fire-rules-explain` + `Explained{session,
support}`); P12b walked it into a `DerivationNode` tree (north-star green). P12c adds the **per-edge payload**
that turns "what derived this" into "*why*, concretely" — the operator reads the activation off the page
without knowing the rule.

## What it delivers
A `DerivationStep` edge type carrying, for each supporting fact: the **rule's satisfied constraint predicates
with concrete bound values** (`(:wat::core::< -5 0)`), the **bound vars** (`{?c -5}`), and **which condition
type** matched. The `via` of a `DerivationNode` becomes `PV<DerivationStep>`.

## The records (BLESSED names — intueri a82b86; the rename adopted)
```clojure
(:wat::Record::def :wat::rete::DerivationNode
  [fact <- :wat::Record
   rule <- :wat::core::Option<wat::core::String>     ;; Some(r) = derived by rule r; None = base/asserted (leaf)
   via  <- :wat::core::PersistentVector<wat::rete::DerivationStep>])

(:wat::Record::def :wat::rete::DerivationStep
  [supporting  <- :wat::rete::DerivationNode          ;; the supporting fact's own derivation (recurse; leaf = empty via)
   pattern     <- :wat::core::String                  ;; the matched condition's fact-type FQDN ("weather::Temperature")
   bindings    <- :wat::core::PersistentMap<wat::core::String, wat::core::Value>  ;; per-step: vars THIS condition bound
   constraints <- :wat::core::PersistentVector<wat::WatAST>])  ;; satisfied predicates, values substituted: (< -5 0)
```
Mutual recursion `DerivationNode → via → DerivationStep → supporting → DerivationNode` — **confirmed
type-checks** (probe this session). Base fact = a `DerivationNode` with `rule=None`, `via=[]`. No Option needed
for the recursion (empty `via` = leaf); Option only for `rule` (`(Some r)` / bare `None` both confirmed).

## The decisions (four-questioned — full table in the session log; YES-winners only)
- **A `:constraints <- PV<WatAST>`** — the constraints ARE predicate expressions; an AST is honest (not the
  data-as-ast hack `bound` was), renders as the form `(< -5 0)` (post the WatAST-render fix, `20722898`), and
  stays inspectable. (String fails Honest; typed `Constraint{}` fails the legibility hard-constraint.)
- **B `DerivationNode`+`DerivationStep`** — payload on the EDGE, where "how-matched" honestly belongs (a
  self-recursive node-with-payload conflates the fact with how-its-parent-matched-it).
- **C `rule: Option<String>`** — None=base, Some=derived; matches Rust (`None` bare / `Some(x)` call).
- **D mechanism: a Rust helper reusing `resolve_operand` + the clause classifier** — faithful by construction
  (same resolver as the match → `:constraints` cannot drift from what fired); the walk stays wat.
- **E `:pattern` = matched fact-type FQDN**; **F `:bindings` per-step** (projected to the condition's binders).

## The `:constraints` + `:bindings` mechanism (Rust helper — `resolve_operand` reuse)
Per support edge the walk holds `(supporting-fact, alpha-id)` (from `token.matches`) + the token `bindings`.
A Rust primitive — `(:wat::rete::step-payload <session> <alpha-id> <bindings>) -> StepPayload{pattern, bindings,
constraints}` (or two thin verbs) — does, reusing the matcher's EXISTING machinery (`src/rete/matcher.rs`):
1. `alpha-id → AlphaNode` (via `session.network`), read `AlphaNode.tests : PV<WatAST>`.
2. **classify** each clause with the matcher's own classifier (matcher.rs:217 binder `(?v <- :field)` vs :249
   constraint `(:op a b)`) — no duplication.
3. **constraints**: for each `(:op a b)` clause, `resolve_operand` each operand against `bindings`+fact
   (matcher.rs:325), rebuild `(:op a' b')` as a `WatAST` (operands → literal nodes) → push to `PV<WatAST>`.
   Faithful: same `resolve_operand` the match used.
4. **bindings (per-step)**: the binder clauses name the ?vars THIS condition bound → project the token
   `bindings` to just those → `PM<String,Value>`.
5. **pattern**: the condition's fact-type FQDN (the AlphaNode's matched type).
The walk (wat, Decision A) calls this per edge; wat orchestrates the Rust primitive (like `alpha-match`,
`eval-insert`). The AST-rebuild reuses runtime quasiquote (proven, R3) or a direct `WatAST` constructor.

## The walk (wat — restructure of P12b's `explain`)
```
explain(ex, fact):
  match support[fact]:
    Some(sv):  DerivationNode{ fact, rule = Some(Support/rule sv),
                 via = for (sfact, alpha-id) in (Token/matches (Support/token sv)):
                         let p = (step-payload (Explained/session ex) alpha-id (Token/bindings (Support/token sv)))
                         DerivationStep{ supporting = explain(ex, sfact),     ;; recurse
                                         pattern = p.pattern, bindings = p.bindings, constraints = p.constraints } }
    None:      DerivationNode{ fact, rule = None, via = [] }                  ;; base leaf
```

## The predicted UX (raw `println`→EDN, with the render fix live)
```clojure
#wat.rete/DerivationNode
{:fact #wat.weather/ColdAndWindy {:celsius -5 :kph 40}
 :rule "weather::cold-and-windy"
 :via [#wat.rete/DerivationStep
       {:supporting #wat.rete/DerivationNode {:fact #wat.weather/Temperature {:celsius -5 :location "Oslo"} :rule nil :via []}
        :pattern "weather::Temperature" :bindings {?c -5} :constraints [(:wat::core::< -5 0)]}   ;; ← legible
       #wat.rete/DerivationStep
       {:supporting #wat.rete/DerivationNode {:fact #wat.weather/WindSpeed {:kph 40 :location "Oslo"} :rule nil :via []}
        :pattern "weather::WindSpeed" :bindings {?k 40} :constraints [(:wat::core::> 40 30)]}]}
```
Reads off the page: *ColdAndWindy ← cold-and-windy, because Temperature `(< -5 0)` and WindSpeed `(> 40 30)`.*
(Structural tag-noise — `#wat.core/PersistentVector` etc. — is the general pretty-printer's job, arc 288, not P12c.)

## Build-steps (verify before the brief)
1. **`resolve_operand` + the classifier are reusable** from a new helper without refactor (they're `pub(crate)`
   in matcher.rs). If a constraint operand is a `:field` ref (not a `?var`), it resolves against the supporting
   fact — confirm the helper has the fact (it does: `sfact`).
2. **AST-rebuild of `(:op a' b')` as a `WatAST`** with resolved literal operands (runtime quasiquote or direct
   constructor). Confirm a `Value` (e.g. `i64(-5)`) → a `WatAST` literal node cleanly.
3. **Per-step binding projection** — the binder clauses give the ?vars; project the token bindings. Confirm the
   binder ?var names are extractable from the `(?v <- :field)` clauses.

## The probe (extend the north-star or a sibling — RED at HEAD)
The P12 north-star's via-COUNTS still hold (via length unchanged: 2 and 1) — keep them green. ADD P12c assertions
on the cold-and-windy explain:
- `DerivationStep/constraints` of the Temperature step renders/contains `(:wat::core::< -5 0)` (the legible
  payload — the load-bearing assertion).
- `DerivationStep/pattern` == "weather::Temperature".
- `DerivationStep/bindings` has `?c -5`.
- `DerivationNode/rule` of the root = `Some("weather::cold-and-windy")`; of a base leaf = `None` (renders nil).
- recursion: the WeatherAlert step's `supporting` is a `DerivationNode` whose own `via` reaches the inputs.

## Blast radius
- `wat/rete.wat` — `DerivationNode` gains `rule` + `via:PV<DerivationStep>`; new `DerivationStep` record; the
  `explain` walk restructured; the `step-payload` verb wrapper.
- `src/rete/matcher.rs` — the `step-payload` helper (reuses `resolve_operand` + classifier; AST-rebuild).
- `src/runtime.rs` (dispatch) + `src/check.rs` (TypeScheme) — register `step-payload`.
- The P12 north-star probe (via element type DerivationNode→DerivationStep; counts unchanged) + the new P12c
  asserts.
- **NOT** the flat-DAG sharing form (follow-on), **NOT** the "which gate MISFIRED" overlay (stone ③), **NOT** a
  pretty-printer (arc 288), **NOT** any fire-path / `Explained` / `Support` change (P12a is done).

## STOP triggers
1. STOP if the `step-payload` helper needs to DUPLICATE `resolve_operand`/the classifier rather than reuse them
   (faithfulness depends on reuse — a re-impl can drift from what fired).
2. STOP if mutual-recursive `DerivationNode`/`DerivationStep` is rejected by the checker (probe says it
   type-checks; if your form differs and fails, surface it).
3. STOP if `:constraints` renders as anything but the legible form (the WatAST-render fix `20722898` must be in;
   if it prints opaque/nil, the fix regressed — surface it).
4. STOP if the north-star via-counts change (the restructure must preserve them: via length = # support edges).
5. STOP if any rete differential or floor regresses (additive to rete.wat; the differential is the guard).

## Four-questions
- **Obvious?** YES — node = fact + its derivation; step = how a supporting fact was matched.
- **Simple?** YES — reuses the matcher's resolver/classifier; mutual recursion confirmed; one new helper.
- **Honest?** YES — `:constraints` faithful by construction (same resolver); payload on the edge where it belongs.
- **Good UX?** YES — the operator reads `(< -5 0)` on sight; the structured tree stays programmatically walkable.
