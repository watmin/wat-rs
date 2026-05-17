# Arc 201 — Structured type-AST in the reflection layer

**Closed:** 2026-05-16 (five slices, one day)
**Originating signal:** Arc 170 Stone D2 settled on a `(:wat::kernel::run-threads coordinator [:name (:factory)] ...)` call form where the macro reflects on the coordinator's signature to extract `I` and `O` from each `:ThreadPeer<I,O>` arg type. Investigation found the reflection layer FLATTENED structured `TypeExpr` values into atomic `HolonAST::Symbol` keyword strings via `type_expr_to_kw` → `format_type`. The substrate KNEW parametric structure (`TypeExpr::Parametric { head, args }`) at check time and DISCARDED it at reflection time. Per `feedback_any_defect_catastrophic`: substrate trust is binary; pivot to fix.

## What was inscribed

Five slices on one substrate-language concern: type-AST structure preservation across the entire reflection layer.

**Slice 1 (`0706949`)** — Replaced `type_expr_to_kw` with recursive `type_expr_to_holon` (`src/runtime.rs`). Parametric types now emit as `HolonAST::Bundle [Atom(head), ...args]`; path types as atomic `HolonAST::Atom`; Tuple, Fn, Var handled uniformly. Applied across all five signature-AST builders: `function_to_signature_ast`, `type_scheme_to_signature_ast`, `typedef_to_signature_ast`, `macrodef_to_signature_ast`, `dispatch_to_signature_ast`. `extract-arg-names` (arc 143) kept working unchanged (reads pair[0] — name, not type).

**Slice 2 (`c9445a4`)** — Minted `:wat::holon::Bundle/children` + `:wat::holon::Bundle/first` as general-purpose HolonAST accessors. `Atom/value` (`:wat::core::atom-value`) needed; verified arc 057's existing `atom-value` already serves every shape — strange-loop closure (the arc 057 primitives minted for VSA encoding extend cleanly to reflection composition). Rename to `:wat::core::Atom/value` carried over as the arc 109 § K namespace-canonicalization follow-up (tracked at task #337).

**Slice 3 (`815d597`)** — Minted `:wat::runtime::signature-of-fn fn-value -> :HolonAST`. Operates on the fn VALUE (post-eval `Value::wat__core__fn(Arc<Function>)`), not a raw fn AST. The decision flipped from the BRIEF's predicted "walk WatAST" path to direct reuse of `function_to_signature_ast` via inline four-questions — the substrate already knew the signature; walking the AST would have re-parsed source. Eight tests including composition tests proving slice 2's accessors walk slice 3's output cleanly.

**Slice 4 (`ecc876a`)** — Renamed `:wat::runtime::signature-of` → `:wat::runtime::signature-of-defn` to make the input-shape asymmetry explicit at the API surface. The pair now reads: `signature-of-defn :name-keyword` (symbol-table lookup) + `signature-of-fn :fn-value` (closure introspection). 21 files swept; 173 mechanical edits including internal Rust identifier rename per FM 14 (`feedback_surface_retirement_internals`; arc 162 precedent). No back-compat alias per `feedback_refuse_easy_solutions`.

**Slice 5 (`2776635`)** — Minted `:wat::runtime::extract-arg-types sig -> :wat::core::Vector<wat::holon::HolonAST>` as a direct mirror of `eval_extract_arg_names`: same walker (skip head + arrow + ret), extracts pair[1] (type AST) instead of pair[0] (name keyword), returns `Vector<HolonAST>` instead of `Vector<keyword>`. Q2 (substrate-side vs wat-side) resolved substrate-side per inline four-questions: wat's Vector ops lack slice/take/drop primitives, so wat-side implementation would require hand-rolled foldl-with-index-counter for what the substrate sibling handles in one walk.

## What surfaced this

Stone D2's pressure. The macro author needed type-driven reflection — extract `I` and `O` from each `ThreadPeer<I,O>` arg of the coordinator fn. Existing reflection returned `":wat::kernel::ThreadPeer<wat::core::String,wat::core::String>"` as one atomic Keyword string — type-string parsing dead-end. Every consumer pressure surfaces substrate honesty: the type-checker's `TypeExpr` knew the structure; the reflection layer was hiding it.

This is `feedback_attack_foundation_cracks` running honest. Stone D2 named the gap concretely; arc 201 closed it without scope creep into adjacent concerns.

## What it cost

Five slices on one substrate-language axis, one day. Concrete:

- **Slice 1:** 5 signature-AST builders uniformly updated; recursive emitter mints. ~60-90 min sonnet.
- **Slice 2:** 2 new accessors + 1 reuse-from-arc-057 discovery (`atom-value` already served the third proposed accessor). ~30 min sonnet.
- **Slice 3:** 1 new verb mirroring `signature-of`'s shape via direct reuse of `function_to_signature_ast`. Inline four-questions flipped the input-shape decision from fn-AST to fn-VALUE (strictly better). ~50 min sonnet.
- **Slice 4:** 21-file rename sweep; 173 edits across substrate Rust + wat consumer + 13 test files + 3 active docs. Internal Rust identifier rename (`eval_signature_of` → `eval_signature_of_defn`) per FM 14. ~70 min sonnet.
- **Slice 5:** 1 new verb mirroring `extract-arg-names`. Sonnet caught Vector-traversal asymmetry vs Gap K (absence-detection needs full subtree; co-presence detection works at outer level). ~40 min sonnet.
- **Slice 6:** Orchestrator paperwork (this INSCRIPTION + USER-GUIDE extension + BRACKET-IMPLEMENTATION-STONES.md D2 marker update + FOUNDATION-CHANGELOG.md cross-repo rows for arc 200 + 201 + 202 backfill).

Workspace baseline tracking through the arc: each slice purely additive; failure count never regressed; pass count climbed from pre-arc-201 baseline through ~+30 new tests across the five slices.

Two honest deltas surfaced and inscribed in slice SCOREs:
- Slice 3's input-shape (fn-VALUE not fn-AST) — strictly better than BRIEF predicted; documented as "should have been the obvious answer from BRIEF" but the four-questions caught it inline.
- Slice 4's baseline discrepancy (orchestrator's pre-slice EXPECTATIONS doc claimed 1679/3 baseline from a cargo run with build errors; actual baseline at the same commit was 2319/4 once compiled cleanly). Documented honestly; EXPECTATIONS stays as historical record per `feedback_inscription_immutable`.

## What it unblocks

**Direct:** Arc 170 Stone D2 (`run-threads` multi-factory heterogeneous expansion). The macro algorithm now has every reflection primitive it needs:

```
signature-of-fn coordinator → structured signature HolonAST
extract-arg-names sig       → Vector<keyword> param names (arc 143)
extract-arg-types sig       → Vector<HolonAST> structured arg types
Bundle/children type-ast    → unpack :ThreadPeer<I,O> → [Atom(:ThreadPeer), I-ast, O-ast]
```

Per arg, the macro can NAME the binder + EXTRACT both type parameters cleanly — no type-string parsing, no synthesized name-substitution, no substrate workaround.

**Downstream:** Stones D3 (`run-threads` panic cascade + `ProcessGroupErr`) and E (`run-processes` bracket macro) build on D2's working call form. Arc 170 closure path: D2 → D3 → Stone E → Stones F-H → INSCRIPTION.

**Generalized:** any future type-driven macro inherits the structured reflection surface. The substrate now exposes its type knowledge through the same uniform HolonAST shape it uses for everything else (per `project_holon_universal_ast` — HolonAST's cross-domain coherence).

## What stayed out of scope

**Atom/value rename (arc 057's `:wat::core::atom-value` → `:wat::core::Atom/value`)** — tracked at task #337 as an arc 109 § K namespace-canonicalization slice. Arc 201 used the existing primitive verbatim; the rename is independent surface work that does not affect arc 201's reflection semantics. Out of arc 201's scope; tracked at #337.

**Wat-side `extract-arg-types` defn** — explicitly considered and rejected in slice 5's four-questions analysis. Substrate-side mirror won YES YES YES YES; wat-side composition failed Obvious + Simple given wat's Vector ops lack slice/take/drop. The wat-side alternative does not surface as a separate arc; it was a road-not-taken inside slice 5's decision, not a piece of work split off.

**Variadic-rest reflection extension** — `eval_extract_arg_names` and `eval_extract_arg_types` both treat the `pair.len() == 2` shape uniformly across strict + variadic-rest binders per `function_to_signature_ast`'s emission. No separate variadic-handling primitive needed; the existing walker covers both cases. If a consumer surfaces wanting to distinguish strict from variadic at the reflection layer, that opens a separate arc — but no such consumer exists today.

## Discipline anchors honored

- **`feedback_any_defect_catastrophic`** — the reflection-layer flattening was a substrate defect; arc 201 pivoted to fix instead of working around it
- **`feedback_attack_foundation_cracks`** — Stone D2's pressure surfaced the crack; the fix is forward progress
- **`feedback_no_known_defect_left_unfixed`** — the flattening was known the moment D2 hit it; the arc opened the same session
- **`feedback_no_new_types`** — Bundle is the existing HolonAST shape; arc 201 added ACCESSORS and producers, not wrapper types. `project_holon_universal_ast` confirms HolonAST's cross-domain reach
- **`feedback_simple_is_uniform_composition`** — uniform recursion in `type_expr_to_holon`; uniform accessors across Bundle; parallel handlers for `extract-arg-names` + `extract-arg-types` (one-character semantic difference; refusing to abstract for the sake of abstracting)
- **`feedback_four_questions_inline`** — every fork inline (input-shape α/β in slice 3; wat-side vs substrate-side in slice 5; accessor naming in slice 2). No `AskUserQuestion` ceremony; the four-questions in prose at the decision point
- **`feedback_refuse_easy_solutions`** — slice 4's hard rename without back-compat alias; slice 5's substrate-side mirror over the easier "skip and document"
- **`feedback_inscription_immutable`** — slice 3's EXPECTATIONS doc retained as historical record alongside slice 3 SCORE's input-shape correction; slice 4's EXPECTATIONS baseline discrepancy documented in SCORE without amending the original
- **`feedback_surface_retirement_internals`** (FM 14) — slice 4 renamed internal Rust identifiers alongside the user-facing keyword per arc 162 precedent

## What is inscribed

Five slices. Five substrate primitives across the reflection axis. Zero new substrate types. Zero new structs. Zero back-compat aliases. Zero deferral language in any slice SCORE or this INSCRIPTION.

The reflection layer is structured top-to-bottom: every type slot the type-checker resolves carries through to its HolonAST representation as `Bundle` for parametric / `Atom` for path. The asymmetric primitive pair (`signature-of-defn` for names, `signature-of-fn` for fn values) is honest about input shape. The extraction primitive pair (`extract-arg-names`, `extract-arg-types`) is symmetric on output structure.

Stone D2's algorithm has every primitive it needs. The dungeon floor closes.
