# BRIEF — Stone 4a: production-fire (token → RHS → derived fact, single pass)

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A MIXED
stone: one new pure Rust primitive (`src/rete/matcher.rs`) + a new pure-WAT pass (`wat/rete.wat`). Build, run
the named tests, report verbatim. Another agent weighs.

## The work
After the hash-join network matches (3b), make a rule **fire**: a `Token` reaching a `ProductionNode` runs the
rule's RHS — each `(:wat::rete::insert <fact>)` form is evaluated with the token's bindings into a derived
`:wat::Record`, stored in `production-memory`. ONE pass, no cascade, no truth-maintenance, no re-entry
(those are 4b/4c). Cold-and-windy derives exactly one `ColdAndWindy`.

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-4a-production-fire.md` — the full contract: the RHS-eval
   decision (restricted evaluator = the dual of `alpha-match`, reusing `resolve_operand`; NOT `macro_eval`),
   the production-pass shape, flat `production-memory`, the pinned `eval-insert` signature, out-of-scope.
2. `src/rete/matcher.rs` — `eval_alpha_match` (`:75-151`, the registration shape to MIRROR), and
   `resolve_operand` (`:325-358`, which `eval-insert` REUSES). Read both fully.
3. `src/runtime.rs:3996` (the `alpha-match` dispatch arm — add a sibling) + `:12759-12816` (`eval_record_of` —
   how a `Value::wat__Record { class_fqdn, struct_form }` is built; `eval-insert` builds the same value).
4. `src/check.rs:18835-18860` (the `alpha-match` TypeScheme — add a sibling).
5. `wat/rete.wat` — `fire-rules` (`:778-816`), `hash-join-pass` (`:736-770`, the pass shape to MIRROR),
   `alpha-feeding` (`:629-650`, the reverse-lookup shape to MIRROR), `compile-rule` (`:411-435`, where the
   ProductionNode is minted + wired as the final join's child), the `Session` record + `production-memory`
   field (`:116-131`), `node-children-ids` (`:155-166`), `node-kind-label` (`:139-149`), `append-token`
   (`:556`, the `conj`-into-a-node-memory idiom).
6. `tests/probe_arc278_4a_production_fire.rs` — remove the `#[ignore]`s if any (there are none; it is live and
   RED). It is your contract.

## Part 1 — the Rust primitive `eval-insert`
Add `eval_insert(args, list_span, env, sym) -> Result<Value, EvalBreak>` to `src/rete/matcher.rs`, mirroring
`eval_alpha_match`:
- 2 args. Evaluate arg[0] → must be `Value::wat__WatAST` wrapping a List `(:wat::rete::insert <fact-form>)`
  (else `RuntimeError::TypeMismatch`). Evaluate arg[1] → must be `Value::wat__core__PersistentMap` (the token
  bindings; else TypeMismatch).
- Validate the form: head Keyword is `:wat::rete::insert`, exactly 2 children; `<fact-form>` = child[1], itself
  a List `(:RecordType arg…)` with a Keyword head.
- `class_fqdn` = the fact-form head keyword with the leading `:` stripped (mirror `eval_record_of:12790`).
- For each `arg` in the fact-form tail: `resolve_operand(arg, &[], &[], &bindings)` — empty fact-fields/names
  (RHS has NO current fact; only `?var` + literal resolve). `None` → `RuntimeError` (an unresolved operand /
  a `:field` in RHS is a malformed rule, not a silent drop).
- Return `Value::wat__Record { class_fqdn: Arc::new(class_fqdn), struct_form: Arc::new(resolved_args) }`.
- **Register it** exactly as `alpha-match` is registered: dispatch arm `":wat::rete::eval-insert" =>
  crate::rete::matcher::eval_insert(args, list_span, env, sym)` in `runtime.rs` beside `:3996`; a TypeScheme in
  `check.rs` beside `:18845` — `params: [:wat::WatAST, :wat::core::PersistentMap]`, `ret: :wat::Record`, NO
  type_params; a one-line note in `src/rete/mod.rs`.

## Part 2 — the WAT production pass
Grow `fire-rules` with a production pass AFTER the hash-join pass, mirroring `hash-join-pass`. Suggested helpers:
- `node-parent` (reverse-lookup, mirror `alpha-feeding`): given a child-id + network, return the id of the node
  whose `node-children-ids` contains child-id (-1 if none). Works for BOTH a `RootJoinNode` parent (1-condition
  rule) and a `HashJoinNode` parent (cold-and-windy). Use `node-children-ids` (`:155`) so it is kind-agnostic.
- `rule-by-name` (linear find over `Session/rules`): name `String` → the `Rule` (the ProductionNode stores
  `rule-name`). A `foldl` carrying an `Option`, or reuse an existing find idiom if one exists.
- `fire-production` (per ProductionNode): read `beta-memory[node-parent(P)]` (None → no tokens → unchanged
  production-memory); for each token, for each insert-form in `Rule/rhs`, call
  `(:wat::rete::eval-insert form (:wat::rete::Token/bindings token))` → derived fact; `conj` it into
  `production-memory[P-id]` (mirror `append-token`'s match-Some/None-then-conj idiom).
- `production-pass` (fold step over node-ids): if the node is a `ProductionNode`, run `fire-production`; else
  pass through. Fold it over `node-ids` in `fire-rules`, seeded with the existing (empty) `production-memory`,
  threading the result into the reconstructed `Session`'s production-memory slot.
- Adjust the `Session` `production-memory` comment (`:121`): *flat `PV` of derived facts in 4a; grows to the
  `{token → [facts]}` support store in 4c (TM)*.
- Update the `fire-rules` doc comment (`:772-774`) — it now ALSO fires productions (still no cascade/TM).

## Builder directive: build missing deps, never hack around
Deps SHOULD all exist (`resolve_operand`, `eval_record_of`'s record build, `Token/bindings`, `Rule/rhs`,
`ProductionNode/rule-name`, `node-children-ids`, `PersistentMap/get`/`assoc`, `PersistentVector/conj`,
`node-kind-label`, the `Option` match idiom). **If a core primitive is genuinely missing → STOP + name it.**
Do NOT hack around it.

## Engine-source bar (DOGFOOD)
LINT-CLEAN — `format`/`interpolate` over nested `concat`; `cond`/`contains?` over nested `if`. The ONLY
below-bar spot is the EXISTING `render-dag` compound-concat FIXTURE — do NOT touch it.

## STOP triggers
1. A needed core primitive is missing → STOP, name it (do NOT improvise a workaround).
2. The parent reverse-lookup cannot be made kind-agnostic with `node-children-ids` (e.g. a guard rejects the
   call) → STOP, describe what you found.
3. You reach for cascade / re-entry / truth-maintenance / the `{token → [facts]}` support store / `Snapshot` /
   `query` / `defrule` / nested-pure-expr fact-args → that is 4b/4c/4d/stone-5 / banked; STOP.
4. `eval-insert` needs an `Environment`/`eval_inner` to resolve a fact-arg → STOP (it must be PURE, resolving
   only `?var`+literal via `resolve_operand`; reaching for eval means the surface grew beyond 4a).

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored   # 4/4 GREEN
cargo test --release -p wat --test probe_arc278_3b_hash_join -- --include-ignored          # 4/4 (join still green)
cargo test --release -p wat --test probe_arc278_3a_root_join -- --include-ignored          # 3/3
cargo test --release -p wat --test probe_arc278_2b_insert_alpha -- --include-ignored        # 3/3
cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored          # 3/3
cargo test --release -p wat --test probe_arc278_1a_data_model -- --include-ignored           # 1/1
cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored              # 2/2
cargo test --release --test test_stdlib_load_order | grep result                            # 1/0
cargo test --release -p wat --lib 2>&1 | grep "test result"                                 # 931/36 (UNCHANGED — was 930+1 stone-0a; confirm baseline)
cargo test --release --test test 2>&1 | grep "test result"                                  # 264/1 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                         # Finished; no NEW warnings
```
Report: the `eval_insert` source + its 3 registration sites; the production-pass source + helpers
(`node-parent`, `rule-by-name`, `fire-production`, `production-pass`) + the `fire-rules` change; all outputs
verbatim; any STOP hit. No git.

## Blast radius
`src/rete/matcher.rs` (+`eval_insert`) · `src/runtime.rs` (1 dispatch arm) · `src/check.rs` (1 TypeScheme) ·
`src/rete/mod.rs` (1 note) · `wat/rete.wat` (`fire-rules` + helpers + 2 comment touches) ·
`tests/probe_arc278_4a_production_fire.rs` (already live). NO other record/signature change. No git.
