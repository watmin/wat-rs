# DESIGN — Stone 251.2: the `wat.type/` namespace (type atoms become symbols)

**Status: STRIKE-READY (drawn 2026-06-10). Decision by four-questions, grounded against
the disk + the arc DESIGN. Probe RED at HEAD before build.**

Predecessor: 251.1 (resolve/ warded + stamped; the first symbol head resolves via the
normalize-layer). Successor: 251.3 (parametrics-as-FORMS). Home: `src/types.rs` +
`src/types/` (already warded — re-earn the stamp).

## The decision (four-questions on the cut width)

The arc DESIGN draws the line; the disk confirms why it is the only coherent one.

- **Obvious?** YES. `WatAST::Keyword` is read as a TYPE in type position (`parse_type_expr_with_span`,
  types.rs:2185) and as a value / call-head elsewhere — the "type/value split" the stone
  dissolves. Moving a scalar atom `:wat::core::i64` (Keyword) → `wat.type/i64` (Symbol) *is*
  that dissolution for atoms. So "move atoms but keep the keyword split" is a FALSE option —
  it cannot exist.
- **Simple?** YES. The atomic cut is scalar type ATOMS (`i64 f64 bool String char nil unit`)
  → `wat.type/` symbols. Parametric CONSTRUCTORS (`Vector`×126, `Option`×50, `HashMap`,
  `Result`, `HashSet`) are type *application*, a different shape that becomes FORMS
  `(wat.type/Vector wat.type/i64)` at 251.3. Atoms now; application-forms next.
- **Honest?** YES. Dual-read (the 251.1b precedent) lets scalars move while parametrics stay
  keyword-spelled — the corpus churns once, not twice.
- **Good UX?** YES. One coherent stone with a clear successor.

**Verdict: 251.2 = scalar type atoms → `wat.type/` symbols (= the keyword type/value split
dissolution for atoms) + subsume the bare-type lint. Parametrics defer to 251.3. Dual-read
bridges `:wat::core::T` (old) · `:wat::type::T` (new FQDN) · `wat.type/T` (surface symbol).**

## The mechanism (most of it already exists)

The 251.1b normalize-layer rewrites ANY `WatAST::Symbol` containing `/` (outside data
positions) to its keyword FQDN via `ns_to_wat_path`. So `wat.type/i64` already normalizes:
`ns_to_wat_path("wat.type","i64")` = `:wat::type::i64`, which is under the reserved `:wat::`
prefix → `is_resolvable_call_head` accepts it (reserved-prefix shortcut). Type-position refs
are not call heads, so resolve does not reject them. **The only NEW substrate work is teaching
the type parser the new FQDN.**

1. **Parser dual-read (`src/types.rs`):** `parse_type_expr_with_span` accepts the
   `:wat::type::<atom>` namespace as an alias for the current `:wat::core::<atom>` for the
   scalar atom set — `TypeExpr::Path` normalizes both to one canonical spelling. (Choose the
   canonical: `:wat::type::i64` is the destination; `:wat::core::i64` and bare `:i64` are the
   legacy reads that map onto it.) Parametric heads (`Vector`/`HashMap`/…) stay `:wat::core::`
   until 251.3.
2. **`ns_to_wat_path` already generic** — no change; `wat.type/i64` → `:wat::type::i64` falls
   out. Confirm the normalize-layer does not need a type-position special case (it should not:
   a type atom in a binder/return slot is a bare `WatAST::Symbol` with `/`, rewritten like any).
3. **Corpus migration:** `:wat::core::i64`(118) / bare `:i64`(126) and the other scalar atoms
   (`f64` 85+70, `String` 46+29, `bool` 12+7, `nil` 161) → `wat.type/<atom>`. Parametric
   spellings UNTOUCHED this stone.
4. **Bare-type lint subsumption (`src/types.rs` `BareLegacyPrimitive`):** stays as a deprecation
   guard during migration; once the corpus is `wat.type/`-clean it has nothing to lint and
   retires (track the retirement in the ward close, not a dangling deferral).

## The probe (RED at HEAD)

`tests/probe_arc251_stone2_type_namespace.rs`:
- **C01 (RED→GREEN):** a `wat.type/i64` type annotation type-checks identically to
  `:wat::core::i64` — e.g. `(:wat::core::defn id [x :- wat.type/i64] -> wat.type/i64 x)` freezes
  and checks clean. RED at HEAD (the type parser does not yet know `:wat::type::i64`).
- **C02 (dual-read preserved):** the legacy `:wat::core::i64` / bare `:i64` spelling still
  type-checks (no regression during transition).

Verify C01 disconfirms at HEAD before any build (probe-first discipline).

## Sequencing within 251.2 — CHURN-ONCE refinement (four-questions, 2026-06-10)

The arc DESIGN's sequencing verdict is **"the corpus churns once"** — 251.4 folds the
corpus-wide `<-`→`:-` + keyword-head→symbol-head + type-spelling rewrite into ONE sweep. A
SEPARATE scalar-type corpus migration here would churn the corpus for types now and again at
251.4 (3× total) — fails Simple + Honest against that verdict. So 251.2 ships the SUBSTRATE
CAPABILITY only; the corpus moves wholesale at the unified sweep.

- **251.2a ✓ DONE** — parser dual-read: `:wat::type::<atom>` aliases to `:wat::core::<atom>`
  on the type-checker path (`parse_type_inner`, types.rs ~2364). Probe RED→GREEN (C01 i64
  load-bearing; C03 f64/bool/String load-bearing; C02 legacy dual-read preserved). Internal
  canonical stays `:wat::core::` for dual-read — the flip is 251.5. ~12 lines.
- **251.2w** — diff-scoped re-ward of the `src/types.rs` touch (the home carries vigilatum
  stamps at `src/types/defstruct.rs` / `error.rs`; the 251.2a addition is gated + clippy-clean;
  cast the applicable spells on the diff, not a full from-scratch vigilia).
- **(corpus migration DEFERRED)** — scalar `:wat::core::i64`/`:i64` → `wat.type/i64` rides the
  unified 251.4/5 sweep, NOT a separate 251.2b. The `BareLegacyPrimitive` lint stays ACTIVE
  until then — it is the migration guide, not yet subsumed. (Subsumption lands when the corpus
  is `wat.type/`-clean at the unified sweep.)

## Out of scope (named, not deferred-prose)

- Corpus migration of scalar type spellings → the **unified 251.4/5 sweep** (churn-once).
- Parametrics-as-forms → **251.3** (the `(wat.type/Vector wat.type/i64)` form surface +
  `lex_keyword` bracket-machinery deletion).
- `:rust::` interop → **251.5** corpus cut (DESIGN move 6).
- Hard-cut of the legacy keyword type spellings + the internal-canonical flip to
  `:wat::type::` → **251.5** (one-canonical-path).
