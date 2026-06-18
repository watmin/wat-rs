# DESIGN — Stone 4a: production-fire (token → RHS → derived fact, single pass)

The first slice of stone 4. After the equality-join network matches end to end (3b), a `Token` that
reaches a `ProductionNode` must **fire the rule's RHS**: evaluate each `(:wat::rete::insert <fact>)` form
with the token's bindings into a derived `:wat::Record`, and store it in `production-memory`. No cascade, no
truth-maintenance, no re-entry (those are 4b/4c). Cold-and-windy derives exactly one `ColdAndWindy`.

## Why

`fire-rules` today (`wat/rete.wat:778`) runs three passes — alpha → root-join → hash-join — and stops at the
join's beta-memory. The `ProductionNode` (minted per rule in `compile-rule:428`, wired as a child of the final
join at `:431`) is never visited; no rule ever produces an effect. 4a closes that gap with a **production
pass**, the minimal step from "the network matches" to "the rule fires."

## The RHS-eval decision (the question the breadcrumb flagged — grounded)

**RHS fact-construction is a restricted evaluator — the dual of `alpha-match` — NOT the arc-249 macro-eval
engine.** Grounded:

- The north-star RHS (`tests/probe_arc278_northstar_cold_and_windy.rs:43`) is `(:wat::rete::insert
  (:weather::ColdAndWindy ?loc))` — the fact-arg is a bare bound-var `?loc`. The DSL contract (`:14-17`) permits
  "pure exprs over the bound ?vars" in general, but the acceptance test needs only `{?var, literal}`.
- `resolve_operand` (`src/rete/matcher.rs:325-358`) already resolves `{?var → bindings, :field → fact field,
  literal → bare Value}` purely, "NEVER eval_inner." The RHS builder is its **dual**: `alpha-match` is
  `(cond-WatAST, fact) → Option<bindings>`; the RHS builder is `(insert-WatAST, bindings) → fact`. Same pure
  AST-walk + operand resolution — so it **reuses `resolve_operand`**, one resolver for LHS and RHS (no
  divergence class).
- `macro_eval` (`src/macros/eval.rs:75`) takes `&Environment` (expand-time symbol table), not a token's
  `PersistentMap`. Adapting it would be work to *avoid* reusing `resolve_operand`. By the 2a precedent (matcher
  went Rust), the RHS builder belongs in Rust beside `resolve_operand`.

**Banked (NOT 4a):** nested pure exprs as fact-args (e.g. `(:wat::core::i64::+ ?a ?b)` inside a constructed
fact). When that need is real, `macro_eval`'s `is_pure_total` allow-list is the reference, adapted to take a
bindings-map. Do not build that forcing function now.

## What 4a delivers

1. **A new Rust primitive** `(:wat::rete::eval-insert <insert-form: :wat::WatAST> <bindings: :wat::core::PersistentMap>) -> :wat::Record`
   in `src/rete/matcher.rs`, beside `alpha-match`. It:
   - evaluates arg[0] to a `Value::wat__WatAST`; the form must be a List `(:wat::rete::insert <fact-form>)`;
   - takes the inner `<fact-form>` = `(:RecordType arg…)`; the head Keyword → `class_fqdn` (strip leading `:`);
   - resolves each `arg` via `resolve_operand(arg, &[] /*no current fact in RHS*/, &[], &bindings)` — `?var`
     and literal resolve; a `:field` or unresolved var → a TypeMismatch/Resolve error (RHS has no fact to read
     a `:field` from);
   - returns `Value::wat__Record { class_fqdn, struct_form: <resolved args, declaration order> }`.
   Registered exactly as 2a registered `alpha-match`: dispatch arm in `runtime.rs` (~`:3996`), TypeScheme in
   `check.rs` (~`:18845`), module note in `src/rete/mod.rs`. No record-field type-validation (mirrors
   `eval_record_of`, `runtime.rs:12759`, which builds positionally without validating — defrule/checker
   validation is stone 5).

2. **`fire-rules` grows a production pass** (`wat/rete.wat`), after the hash-join pass, mirroring
   `hash-join-pass:736`:
   - fold over `network` node-ids; for each `ProductionNode P`:
     - **parent reverse-lookup** (mirror `alpha-feeding:629`, but match on ANY node whose `node-children-ids`
       contains `P` — a `RootJoinNode` for a 1-condition rule, a `HashJoinNode` for cold-and-windy): yields the
       parent beta node id;
     - read `beta-memory[parent-id]` → the `PV<Token>` that reached `P` (None → no tokens → skip);
     - resolve the rule by `ProductionNode/rule-name` in `Session.rules` (linear find);
     - for each token, for each insert-form in `Rule/rhs`, call `eval-insert(form, Token/bindings)` → a derived
       fact; `conj` it into `production-memory[P-id]`.
   - thread the new `production-memory` into the reconstructed `Session`.

3. **`production-memory[P-id]` = a flat `PV<:wat::Record>` of derived facts.** The `{token → [facts]}` support
   store documented at `rete.wat:121` is the **4c** end-state (the cascade-retraction support chain); storing it
   now is speculative before the cascade algorithm reveals its shape. Adjust the `:121` comment to read: *flat
   `PV` of derived facts in 4a; grows to the `{token → [facts]}` support store in 4c (TM)*.

## The one contract decision (pinned)

`(:wat::rete::eval-insert <insert-form> <bindings>) -> :wat::Record` — total over the v1 RHS surface
(`{?var, literal}` fact-args), raises `RuntimeError` on a malformed insert-form / unresolved operand (never
panics, never silently drops). `production-memory[P-id] : PV<:wat::Record>` (flat, 4a).

## Files touched

- `src/rete/matcher.rs` — add `eval_insert` (reuses `resolve_operand`).
- `src/runtime.rs` — one dispatch arm for `:wat::rete::eval-insert`.
- `src/check.rs` — one TypeScheme for `:wat::rete::eval-insert`.
- `src/rete/mod.rs` — module note (stone 4a).
- `wat/rete.wat` — `fire-rules` grows the production pass + helpers (`production-pass`, parent reverse-lookup,
  rule-by-name find); the `:121` comment adjusted.
- `tests/probe_arc278_4a_production_fire.rs` — the probe (un-ignore).

## Out of scope = REJECTED (not "later")

- **Cascade / fixpoint** (derived facts re-entering the network) — 4b.
- **Truth maintenance / retraction / the support store** (`{token → [facts]}`) — 4c.
- **`:wat::rete::Snapshot`** state blob — 4d.
- **`query` / `collect-rules` / `defrule`** — stone 5 (the 4a probe reads `production-memory` directly, the
  established 3a/3b probe pattern).
- **Nested pure-expr fact-args** — banked (see the RHS-eval decision above).
- **Record-field type-validation of derived facts** — stone 5 (defrule/checker).
- **No 1a/1b/2/3 record or signature change.**
