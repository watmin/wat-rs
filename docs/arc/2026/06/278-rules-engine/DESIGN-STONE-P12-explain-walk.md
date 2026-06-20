# DESIGN — Stone P12: the EXPLAIN walk ("how did this get derived"), proven in wat

The guiding light. Operator diagnostic, non-negotiable: given a derived fact, render the **why-tree** back to the
input facts — through which rule, which gates, which supporting facts — recursively. Prove it **in wat**.

## ⚠ NAMING — BLESSED by intueri (cast a82b86, 2026-06-19); the worked surface below is PRE-RENAME
intueri graded the original `Why*` model and recommended the `Derivation*` family + transparent field names.
**Adopt these** (P12b's `Why` is already renamed → `DerivationNode`; the rest land in P12c when the fields are
built). The worked-surface block BELOW still shows the OLD names (`#rete/Why`, `:via`, `:met`, `gate`, `bound`) —
read it for STRUCTURE only; the names are superseded by this table:

| old (worked surface below) | BLESSED (use this) | why (intueri) |
|---|---|---|
| `Why` (node) | **`DerivationNode`** | "Derivation is domain-precise to RETE; not a 'why'" |
| `WhyEdge` (the via entry / edge) | **`DerivationStep`** | "one step in the proof chain" (P12c) |
| `WhyGraph` (future flat DAG) | **`DerivationGraph`** | systematic `Derivation*` family |
| `:met` ⚠ | **`:constraints`** | *Level-1 LIE, worst offender* — "`met` is a past participle, says nothing; these are the satisfied constraint predicates" |
| `gate` | **`:pattern`** | "what was matched, not engine jargon" |
| `bound` | **`:bindings`** | unification-standard |
| `from` / `to` (edge dirs) | **`:supporting` / `:derived`** | the actual roles, not directions |

## The worked surface (STRUCTURE reference — names superseded by the table above; four-questioned)

```clojure
;; rule:  :when [(:weather::Temperature (?t <- :celsius) (:wat::core::< ?t 0))
;;               (:weather::WindSpeed   (?w <- :kmh)     (:wat::core::> ?w 30))]
;;        :then (:weather::ColdAndWindy ?t ?w)        ;; facts: (Temperature -5) (WindSpeed 40)

(:wat::rete::explain (:wat::rete::fire-rules-explain staged) (:weather::ColdAndWindy -5 40))
;; →
#rete/Why
{:fact  (:weather::ColdAndWindy -5 40)
 :rule  "weather::cold-and-windy"
 :via   [ {:type  :weather::Temperature
           :fact  (:weather::Temperature -5)
           :bound {?t -5}
           :met   [ (:wat::core::< -5 0) ]}     ;; (< ?t 0), ?t=-5 → -5 < 0 ✓   (no :why → base fact, leaf)
          {:type  :weather::WindSpeed
           :fact  (:weather::WindSpeed 40)
           :bound {?w 40}
           :met   [ (:wat::core::> 40 30) ]} ]} ;; (> ?w 30), ?w=40 → 40 > 30 ✓ (no :why → base fact, leaf)

;; a cascade level — a derived supporting fact carries a :why (the DAG recurses to inputs):
(:wat::rete::explain fired (:weather::WeatherAlert -5 40))
;; → #rete/Why {:fact (:weather::WeatherAlert -5 40) :rule "weather::alert"
;;              :via [ {:type :weather::ColdAndWindy :fact (:weather::ColdAndWindy -5 40)
;;                      :bound {?t -5, ?w 40} :met []
;;                      :why  #rete/Why { …the cold-and-windy tree above… }} ]}
```

- **Nodes = facts; edges = gates carrying the conditions that fired.** Each `:via` entry: the `:type` matched,
  the `:fact` (the supporting fact's **data-form**), the vars it `:bound`, and **`:met` — the rule's constraint
  predicates with the concrete bound values substituted in** (`(< -5 0)`, `(> 40 30)`), each shown as it
  evaluated true. The operator reads the activation off the page without knowing the rule: *cold = `(< -5 0)`* is
  obvious on sight. **This is the load-bearing payload** — with dozens-to-hundreds of arbitrary-complexity rules,
  no operator knows them all; the concrete satisfied condition is the diagnostic.
- **A `:via` entry with NO `:why` is a base/asserted fact** — the `:fact` data-form is the leaf, nothing lower.
  A `:why` present (a nested `#rete/Why`) means the supporting fact is derived → drill in. No `:input` sentinel:
  the data-form already says what the fact is, and absence-of-`:why` says "nothing lower." On a **shared
  sub-derivation** (a fact reached two ways), the canonical output is the **DAG** — each fact node once, `:why`
  referenced not re-expanded (acyclic by the fixpoint's round structure, so the walk-back to inputs always
  terminates); the nested form above is the rooted readable *projection*.
- `#rete/Why` is a plain wat Record → `println` is ∀T→EDN; the operator gets readable output for free.

### `:met` provenance — no ③ dependency for why-TRUE
The concrete conditions are available at fire time without the structured-Condition stone: the edge's `alpha_id` →
the alpha node's condition form (`alpha_cond`, hoisted in P8) → extract its constraint clauses (the
`(:op a b)` clauses, distinct from the `(?v <- :field)` binders — the matcher already classifies these) →
substitute the token's `bindings` (`?t → -5`) into them → render. So `:met` ships **in P12**. (③ is needed only
for the *negative* view — "which gate would have fired but didn't" — which needs the full rule structure to know
what *should* have matched. Different stone.)

## The principle — diagnostics are OPT-IN, and it costs nothing (R5 applied to the why-tree)
Explainability defaults **off**. The default `fire-rules'` is the line-rate path (clears beta, no provenance
index); `fire-rules-explain` is a separate mode you opt into. This is not a perf compromise — it is the
strongest possible design, justified by purity:

- The snapshot is `{facts, rules}` — a thunk; firing forces it (R5).
- The engine is **pure**: same `{facts, rules}` → the same derivation, deterministically.
- Therefore the **why-tree is itself a pure function of `{facts, rules}`** — a deferred computation exactly like
  the derived facts. The provenance carries zero information not already in the inputs.
- So retaining the support chain at fire time is **redundant for storage** (the identical R5 argument that drops
  derived facts from the blob). You can always re-force it.

**Consequence:** you never trade explainability for speed, because explainability is not *stored*, it is
*recomputed* — and the recompute is free (P11-fast) and **faithful** (purity guarantees the re-fire reproduces
the exact derivation the fast path performed and discarded). The operator decides at triage time: pull the
stored `{facts, rules}`, `fire-rules-explain`, walk the tree — bit-identical to what prod did. This is the AWS
S3-triage workflow made principled, and the inverse of Clara's heavyweight durability blob: Clara had to *store*
provenance because its impure RHS could not re-derive (R5); we do not store it because we can re-derive it,
identically. Default opt-**out**, justified by: *the explanation is never lost, only deferred — and forcing it is
cheap and exact.*

## The contract (decisions)

### Decision A — the walk lives in WAT (builder-readable), over an explain-mode-exposed graph
The fast `fire-rules'` clears beta (P11, line-rate). A **second mode**, `fire-rules-explain`, retains the support
graph and exposes it as Values: it does NOT clear beta, and `production_pass` records a **fact→producing-token
index** (re-introducing the 4c cut). `explain` is then a **wat function** that walks those Values recursively —
the builder reads and can modify the diagnostic logic in the language he reads. (Walk-in-Rust would be faster but
opaque; EXPLAIN is a rare diagnostic, not line-rate, so readability wins. wat-orchestrates-Rust: the fire is
Rust, the walk is wat.)

### Decision B — the fact→producing-token index is the 4c cut, re-introduced, explain-mode only
In `fire-rules-explain`, `production_pass` records `derived-fact → (rule-name, producing-token)` alongside the
flat production memory. The token carries `matches = [(fact, alpha_id)]` (the condition-edges). The fast fire
does NOT build it (no cost on the line-rate path). Exposed in the frozen explain-Session as a Value map the wat
walk reads.

### The walk (wat, recursive)
```
explain(session, fact):
  let (rule, token) = index[fact]        ;; not found → fact is an INPUT → return a leaf
  via = for (sf, alpha_id) in token.matches:
          { :gate (type-of alpha_id), :fact sf, (if index[sf]: :why (explain session sf)) }
  #rete/Why { :fact fact, :rule rule, :via via }
```
Terminates at input facts (not in the index). Multiple derivations of one fact → the index keeps the first
producing token (v1; a fact derived two ways is a follow-on, noted).

## Out of scope = rejected (this stone)
- **"Which gate MISFIRED"** (a fact that SHOULD have derived but didn't) — needs the full structured rule
  (all conditions) to diff against the runtime edges → that is stone ③ (structured Condition, DESIGN-STONE-S).
  This stone does the **why-TRUE** walk only.
- The full snapshot/revive (DESIGN-STONE-S).
- Multi-derivation fan-in in the why-tree (v1 keeps the first producing token; named).
- No change to the fast `fire-rules'` path (P11 stays lean) — `explain` is a separate mode.

## Foundation probe (write FIRST, RED at HEAD)
A wat demo (`wat-scripts/perf/` sibling, or a probe): fire the 2-rule cold-and-windy cascade (A: Temp+Wind →
ColdAndWindy; B: ColdAndWindy → WeatherAlert) in explain mode; `explain` the top derived fact (WeatherAlert);
assert the why-tree reaches the two input facts (Temperature, WindSpeed) through the right gates, with the
intermediate ColdAndWindy carrying its own `:why`. RED at HEAD (`fire-rules-explain` / `explain` are
UnknownFunction). This proves the diagnostic **in wat**, end to end.

## Why this is the close-condition's heart
The whole arc exists so an operator can be handed a concrete answer to "why did the engine decide this." This
stone delivers it, in the language the builder reads, over the cheap property graph P11 built. The bench proved
the engine; this proves it can explain itself.
