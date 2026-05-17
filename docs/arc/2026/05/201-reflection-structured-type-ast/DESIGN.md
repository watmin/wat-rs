# Arc 201 — Structured type-AST in the reflection layer

**Direction:** the reflection layer (`signature-of` + sibling primitives) currently FLATTENS structured `TypeExpr` values into atomic `HolonAST::Symbol` keyword strings via `type_expr_to_kw` → `format_type`. Arc 201 preserves the structure: parametric types become `HolonAST::Bundle (head :args...)`. New general-purpose accessors let macros walk types structurally instead of hand-rolling keyword-string parsing.

**Status:** DESIGN. Slice plan + four-questions surfaced; BRIEFs follow after user review.

**Originating signal:** arc 170 Stone D2 settled on a `(:wat::kernel::run-threads coordinator [:name (:factory)] ...)` call form where the macro reflects on coordinator's signature to extract I and O from each `:ThreadPeer<I,O>` arg type. Investigation found the reflection layer can't return structured types — only flat keyword spellings. Per `feedback_any_defect_catastrophic`: substrate trust is binary; pivot to fix.

---

## The defect concretely

`function_to_signature_ast` (src/runtime.rs:8906) builds the signature HolonAST. Each arg-pair is:
```rust
WatAST::List([
    WatAST::Symbol(param_name),
    type_expr_to_kw(ty),   // ← THIS flattens the TypeExpr
])
```

`type_expr_to_kw` (src/runtime.rs:8895) emits ONE Keyword:
```rust
WatAST::Keyword(crate::check::format_type(ty), Span::unknown())
```

So `:wat::kernel::ThreadPeer<wat::core::String, wat::core::i64>` lands as one atomic keyword string `":wat::kernel::ThreadPeer<wat::core::String,wat::core::i64>"`. The substrate KNOWS the parametric structure (`TypeExpr::Parametric { head, args }`) at check time and DISCARDS it at reflection time.

**Consequence:** any type-driven macro hits a string-parsing dead-end. Future struct-accessor generators, generic wrappers, polymorphism-aware dispatch helpers — all blocked.

## Four-questions on the substrate API shape

### Q1 — Structured type-AST format

**(A) Parametric types emit as `HolonAST::Bundle (head :args...)`**
- `:String` → `HolonAST::Atom("wat::core::String")` (atomic — no change)
- `:ThreadPeer<I,O>` → `HolonAST::Bundle [Atom(:ThreadPeer), I-type-ast, O-type-ast]` (head + args; args may themselves be Bundles for nested generics)
- `:Tuple<A,B>` → `HolonAST::Bundle [Atom(:Tuple), A-type-ast, B-type-ast]`
- `:Fn(A,B) -> R` → `HolonAST::Bundle [Atom(:Fn), A-type-ast, B-type-ast, Atom("->"), R-type-ast]`

- Obvious: YES — Bundle IS the substrate's "structured sequence" shape; matches existing signature head shape
- Simple: YES — uniform recursion; same shape user already sees for sigs
- Honest: YES — preserves the structure the type-checker already has
- Good UX: YES — Bundle accessors walk it uniformly

→ YES YES YES YES.

**(B) Some new wrapper type `TypeAST`** distinct from HolonAST
- Disqualified: `feedback_no_new_types`. HolonAST is the substrate's AST type; adding TypeAST would mint a parallel concept.

### Q2 — Accessor shape

The macro author needs to walk the structured type AST. Two candidates:

**(α) Purpose-specific helpers** — `extract-arg-types`, `extract-type-head`, `extract-type-args` (each new use case adds a new helper)
- Obvious: marginal — each use case needs a new substrate verb
- Simple: NO — surface grows with use cases
- Honest: NO — hides that this is HolonAST iteration
- Good UX: NO — N helpers vs 1 abstraction

→ Disqualified.

**(β) General-purpose HolonAST accessors** — `:wat::holon::Bundle/children`, `:wat::holon::Bundle/head`, `:wat::holon::Atom/value`, etc.
- Obvious: YES — Bundle is the abstraction; accessors reflect the data
- Simple: YES — small fixed surface; composable
- Honest: YES — structured-AST manipulation IS HolonAST manipulation
- Good UX: YES — one set of accessors works for ANY Bundle structure (signatures, types, programs, anything)

→ YES YES YES YES.

`extract-arg-types` (the concrete need) becomes a CONVENIENCE that composes Bundle/children + filtering. Likely lives in `wat/runtime.wat` alongside the existing extract-arg-names sibling, not as a new substrate primitive.

### Q3 — Backward compatibility for existing reflection consumers

Arc 143's `extract-arg-names` walks the signature head looking for pair-Bundles with `(Symbol arg-name, Symbol arg-type)`. Post-arc-201, the type slot becomes a Bundle for parametric types. extract-arg-names should keep working — it only reads pair[0] (the name).

Other consumers:
- `define-alias` (`wat/runtime.wat:22-29`) — uses extract-arg-names + rename-callable-name. rename-callable-name might inspect the head. Need to verify.
- Per the file system audit: no other production consumers of signature-of's TYPE slot. The structured type-AST emission is BACKWARD-COMPATIBLE for consumers that only read names.

If consumers DO read types as Symbol strings (and there are any), they need migration. Sweep is part of slice 2 closure paperwork.

### Q4 — User-facing call form (settled with user 2026-05-16)

Iteration history (each running four-questions YES YES YES YES against alternatives):

- **First proposal:** `(run-threads [[:I :O :factory] ...] :coord-name)` — positional 4-tuple, named coord, brackets group specs. User: "messy."
- **Second proposal:** `(run-threads [[:peer-name :factory-fn<I,O>] ...] :coord)` — types travel with factory ref. User: rejected (DRY violation; types repeated at factory def + call).
- **Third proposal:** `(run-threads :coord [:name (:factory)] ...)` — name-contract via coordinator arg names. User: questioned "why two symbols?"
- **Fourth proposal:** `(run-threads :coord [:name (:factory-1)] [:name (:factory-2)] ...)` — deferred factory-call form, name-contract via coordinator. User: still messy.
- **SETTLED FORM:** coordinator is an anonymous fn at the call site (one-shot; contract is local); factories are variadic deferred-call forms in coordinator's arg order. No bracket-spec naming — coordinator's binder NAMES are the names; binder ORDER is the contract.

```scheme
(:wat::kernel::run-threads
  (:wat::core::fn
    [logger   <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>
     counter  <- :wat::kernel::ThreadPeer<wat::core::i64,wat::core::i64>
     reporter <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
    -> :wat::core::String
    ;; body uses logger, counter, reporter peers
    ...)
  (:app::logger-worker)
  (:app::counter-worker)
  (:app::reporter-worker))
```

Why this wins YES YES YES YES against all alternatives:
- **Obvious** — coordinator's binder ORDER tells you which factory is which; no second naming site
- **Simple** — minimal syntax; no extra brackets; standard Lisp variadic-at-end
- **Honest** — single source of truth (coordinator binders); names and types declared exactly once
- **Good UX** — coordinator declared where it's used (one-shot); refactor is one place; reader sees full picture inline

**Macro algorithm (post-arc-201):**

1. `signature-of-fn coordinator-fn` → structured signature HolonAST (slice 3 — NEW primitive)
2. `extract-arg-names sig` → `[logger counter reporter]`
3. `extract-arg-types sig` → `[Bundle(:ThreadPeer :S :S), Bundle(:ThreadPeer :i64 :i64), Bundle(:ThreadPeer :S :S)]`
4. For each k:
   - `name-k` = arg-names[k]
   - `type-k` = arg-types[k]; `Bundle/children` → take I and O slots
   - `factory-k` = variadic-args[k] (a call form like `(:app::logger-worker)`)
   - emit `thread-{name-k}`, `peer-{name-k}`, `drained-{name-k}` bindings using extracted I,O (names from coordinator → no gensym needed)
5. Emit `(coordinator-fn peer-logger peer-counter peer-reporter)` — apply coordinator to peers in binder order

**Pedagogy:** the user-facing form IS one-line beautiful. The macro source (substrate-internal) IS the educational artifact for macro authors learning type-driven reflection. Both audiences served.

### Q5 — `signature-of-fn` vs `signature-of-defn` (NEW, settled 2026-05-16)

Current `signature-of` takes a NAME keyword and looks up in symbol table. For coordinator-as-anonymous-fn, the macro receives the fn AST directly — needs a different primitive that reads the fn AST's signature without symbol-table lookup.

**Two primitives required (both mandatory long-term):**

- `:wat::runtime::signature-of-fn fn-ast -> :HolonAST` — NEW. Operates on the fn-AST node a macro receives. Required for D2 / run-threads.
- `:wat::runtime::signature-of-defn name-keyword -> :Option<HolonAST>` — RENAMED from current `signature-of` for clarity in the 2-form world. Existing semantics. Required for general reflection (e.g., arc 143 `define-alias`).

Both emit the same structured HolonAST shape (per slice 1's emission rules). Only INPUT differs.

→ YES YES YES YES on having BOTH: each has a distinct legitimate use case; renaming `signature-of` → `signature-of-defn` makes the asymmetry explicit at the API surface.

## Stepping stones (settled slice plan — 2026-05-16)

### Slice 1 — Structured type-AST emission

- Replace `type_expr_to_kw` (`src/runtime.rs:8895`) with a recursive `type_expr_to_holon` (sonnet picks final name via `/gaze` if needed) that emits:
  - `TypeExpr::Path(p)` → `HolonAST::Atom(symbol p)`
  - `TypeExpr::Parametric { head, args }` → `HolonAST::Bundle [Atom(head), ...recurse(args)]`
  - `TypeExpr::Tuple(args)` → `HolonAST::Bundle [Atom(":Tuple"), ...recurse(args)]`
  - `TypeExpr::Fn(args, ret)` → `HolonAST::Bundle [Atom(":Fn"), ...recurse(args), Atom("->"), recurse(ret)]`
  - `TypeExpr::Var(v)` → `HolonAST::Atom(symbol v)`
- Apply uniformly across all signature-AST builders: `function_to_signature_ast`, `type_scheme_to_signature_ast`, `typedef_to_signature_ast`, `macrodef_to_signature_ast`, `dispatch_to_signature_ast`.
- Verify `extract-arg-names` still works (reads pair[0] — unchanged).
- Verify `define-alias` still works (audit `rename-callable-name`'s type expectations).
- Unit test: `signature-of` a known parametric fn → assert structured type emission.

**Predicted:** 60-90 min sonnet. Touches 5 signature-builders + the format helper.

### Slice 2 — General-purpose HolonAST accessors

- Mint `:wat::holon::Bundle/children` — `(Bundle/children :HolonAST) -> :Vec<HolonAST>` (errors if input is not Bundle)
- Mint `:wat::holon::Bundle/head` — `(Bundle/head :HolonAST) -> :HolonAST` (first child; errors if empty or not Bundle)
- Mint `:wat::holon::Atom/value` — `(Atom/value :HolonAST) -> :HolonAST::Symbol` or similar (returns wrapped value; errors if not Atom)
- Naming via `/gaze` if any feel off.
- Tests: round-trip Bundle/Atom construction + accessor on each.

**Predicted:** 30-60 min sonnet.

### Slice 3 — `signature-of-fn` primitive (REQUIRED for D2)

- Mint `:wat::runtime::signature-of-fn fn-ast -> :HolonAST` — operates on the fn AST node a macro receives (the `(:wat::core::fn [params] -> :T body)` form).
- Extracts the signature in the same structured shape slice 1 emits.
- Tests: signature-of-fn on a parametric-typed inline fn → assert structured signature comes back.

**Predicted:** 45-75 min sonnet. Reuses slice 1's emission machinery applied to a different input.

### Slice 4 — Rename `signature-of` → `signature-of-defn` + consumer sweep

**Scope concrete (post-grep 2026-05-16):**

User-facing keyword: `:wat::runtime::signature-of` → `:wat::runtime::signature-of-defn`. Internal Rust identifiers rename to match per FM 14 (`feedback_surface_retirement_internals`; arc 162 precedent).

- **Substrate Rust** — `src/runtime.rs` (eval handler `eval_signature_of` → `eval_signature_of_defn`; dispatch arm at `:4046`; OP constant; 3 docstring/comment refs at `:9018`, `:9532`, `:9774`); `src/check.rs` (string-literal callee match at `:4721`; `env.register` entry at `:14192`; check-side comment refs); `src/freeze.rs` comments (2); `src/stdlib.rs` comment (1).
- **Wat consumer** — `wat/runtime.wat` `define-alias` macro (calls `(:wat::runtime::signature-of target-name)` in its expansion body).
- **Tests** — 13 test files reference `signature-of`: `tests/wat_arc143_lookup.rs`, `wat_arc143_define_alias.rs`, `wat_arc143_manipulation.rs`, `wat_arc136_do_form.rs`, `wat_arc144_lookup_form.rs`, `wat_arc144_uniform_reflection.rs`, `wat_arc144_special_forms.rs`, `wat_arc144_hardcoded_primitives.rs`, `wat_arc146_dispatch_mechanism.rs`, `wat_arc150_variadic_define.rs`, `wat_arc201_signature_of_fn.rs`, `wat_arc201_holon_ast_accessors.rs`, `wat_arc201_structured_signature_types.rs`. Mechanical sweep: every literal call `(:wat::runtime::signature-of ...)` → `(:wat::runtime::signature-of-defn ...)`; every Rust identifier `signature_of` → `signature_of_defn` in test fixture names (only where it refers to THIS primitive — preserve `eval_signature_of_fn` / `signature_of_fn` references unchanged).
- **Active docs** — `docs/USER-GUIDE.md` (4 hits — explanatory sections), `docs/ZERO-MUTEX.md` (1 hit), `docs/MODULARIZATION-NOTES.md` (1 hit). Mechanical text replace.
- **Historical artifacts (NOT touched per `feedback_inscription_immutable`):** past INSCRIPTIONs, SCOREs, BRIEFs, DESIGNs in `docs/arc/2026/05/143-*/`, `144-*/`, `146-*/`, `148-*/` — these describe state at write-time; the rename is forward-only.

**Alias decision: RESOLVED — no back-compat alias.** Per `feedback_refuse_easy_solutions`: short-term sweep churn is the honest cost; alias would mint a synonym that violates `project_wat_llm_first_design` (one canonical path per task).

**Predicted:** 60-90 min sonnet. Scope corrected from "likely small" hedge — concrete is ~150 mechanical edits across ~18 files. Still purely mechanical (no substrate-shape change); but the sweep needs careful preservation of `signature-of-fn` (slice 3 sibling) references which contain the substring.

### Slice 5 — `extract-arg-types` substrate primitive (mirror of `extract-arg-names`)

**Q2 RESOLVED 2026-05-16 (substrate-side over wat-side):**

Original lean was wat-side composition via `Bundle/children` + Vector ops. Verification (post-`feedback_assertion_demands_evidence` discipline) showed wat's Vector ops are limited to `{length, empty?, contains?, get, conj, concat}` + `map`/`foldl`/`foldr` — **no slice / take / drop primitives.** Implementing pair-extraction-from-signature in wat would require hand-rolled foldl-with-index-counter to filter out head + arrow + ret slots by position. Four questions on the wat-side candidate failed Obvious + Simple. Substrate-side mirror of `extract-arg-names` passes YES YES YES YES — direct sibling, same walker, same layer, symmetric API.

**Scope:**

- Mint `:wat::runtime::extract-arg-types` substrate primitive at `src/runtime.rs` — eval handler mirrors `eval_extract_arg_names` (`src/runtime.rs:10165`); changes pair extraction from index 0 (name keyword) to index 1 (type AST); returns `Vector<HolonAST>` instead of `Vector<keyword>`
- Type-scheme registration at `src/check.rs` (mirror `extract-arg-names` registration at `src/check.rs:14385+`)
- Dispatch arm at `src/runtime.rs:4051` area (mirror `extract-arg-names` dispatch arm)
- Reuses slice 1's structured HolonAST emission for type slots (the walker already gets structured Bundles/Atoms for type AST)
- Test: `extract-arg-types` on a known parametric-typed fn signature → assert structured Bundles for parametric args, Atoms for path args

**Reuse path:** `eval_extract_arg_names` walks the signature head; for each arg-pair Bundle, extracts pair[0] (a `HolonAST::Symbol` keyword). New eval handler walks the SAME signature head; for each arg-pair Bundle, extracts pair[1] (a structured HolonAST representing the type, per slice 1 rules). Skipping head + arrow + ret slots happens identically.

**Predicted:** 30-60 min sonnet.

### Slice 6 — Closure paperwork

- `INSCRIPTION.md` — capture the four-questions, the lesson (consumer pressure surfaced reflection-layer flattening), the API surface, the `:any-defect-catastrophic` principle inscribed mid-arc
- USER-GUIDE update — new section on type-driven macros + the reflection primitives + the worked example (run-threads internals)
- 058 PROPOSAL row + CHANGELOG row — substrate language change documented
- Mark arc 170 D2 unblocked in STONES.md

**Predicted:** 60-90 min orchestrator.

## Knock-on effects

**Unblocks:**
- Arc 170 Stone D2 (run-threads multi-factory) — was the originating consumer
- Arc 170 Stone D3 (panic cascade) — depends on D2
- Arc 170 Stone E (run-processes) — same pattern
- Any future type-driven macro

**Doesn't affect:**
- Arc 167's "vectors at value position" — separate concern, intentional design
- Arc 173 (gensym) — separate; D2's fresh-name need now solved by name-contract (coordinator's args ARE the names)

**Cleanup that should land alongside or before arc 201:**
- `MacroError::SpliceNotList` → `SpliceNotSequence` rename (post-arc-200 cleanup; orchestrator-direct)
- arc 167 comment: drop the "future arc enables vector literals as Value::Vec values" speculation (per user clarification 2026-05-16, this is NOT happening)

## Discipline anchors

- `feedback_any_defect_catastrophic` (2026-05-16, NEW) — substrate trust is binary; reflection bears a defect; we pivot
- `feedback_attack_foundation_cracks` — the crack IS diagnostic; fix is forward progress
- `feedback_no_known_defect_left_unfixed` — we know the gap; we fix now
- `feedback_no_new_types` — Bundle is the existing abstraction; we add ACCESSORS, not new wrapper types
- `feedback_simple_is_uniform_composition` — uniform recursion in type_expr_to_holon; uniform accessors across Bundle

## Open questions for user

1. **Accessor naming** — RESOLVED in slice 2 (sonnet's call: `Bundle/children`, `Bundle/first`; `Atom/value` covered by existing arc 057 `:wat::core::atom-value`).
2. **`extract-arg-types` location** — RESOLVED 2026-05-16 (substrate-side mirror of `extract-arg-names`). See § Slice 5 for verification.
3. **Slice 1 atomicity** — RESOLVED (shipped as single slice; consumer sweep was trivial).

All open questions closed.
