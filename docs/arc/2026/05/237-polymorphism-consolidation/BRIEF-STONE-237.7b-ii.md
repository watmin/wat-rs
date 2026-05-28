# BRIEF — Stone 237.7b-ii — `:wat::core::contains?` reborn as a `∀T` intrinsic with a CUSTOM inference arm

Paired with `DESIGN-STONE-237.7b.md` (refined slicing section) and the proven
recipe probe `tests/probe_arc237_7b_intrinsic_typing.rs` (7/7, the regression
contract — the `contains_q_*` tests MUST stay green). **Tier B** — element-typing
must be enforced; plain ∀ scheme is insufficient (probe `contains_q_wrong_element_rejected_at_check`
proves current behavior rejects wrong-elem; the intrinsic preserves that).

## The work — a CUSTOM inference arm (NOT a plain TypeScheme)

Unlike 237.7a `length` / 237.7b-i `empty?` (Tier A, pure ∀ scheme), `contains?`
needs a custom inference handler in `src/check.rs`'s `infer_list` dispatch
(mirroring how `:wat::core::first` / `:wat::core::get` are handled — they have
custom arms, not bare TypeSchemes, because they need to extract the collection's
element type and unify the second arg against it). The pattern to mirror:
`infer_positional_accessor` (the function that handles first/second/third — it
reduces the arg type, matches the collection Parametric, extracts the element).

### 1. `src/check.rs` — add the custom arm

In `infer_list` find the `match` block that dispatches on the call's head
symbol (where `:wat::core::first` / `:wat::core::get` / `:wat::core::Vector/of`
have their custom handlers — grep `:wat::core::first` to locate the family).
Add a `":wat::core::contains?"` arm calling a new `infer_contains` helper:

```rust
":wat::core::contains?" => {
    let (val, mut errs) = infer_contains(args, head_span, env, locals, fresh, subst).into_parts();
    // (return shape: CheckResult<TypeExpr>, value = bool, errors propagated)
}
```

`infer_contains` mirrors `infer_positional_accessor` (~check.rs:12095) but:
- Arity 2 (collection + elem).
- Match `reduce(arg0_ty, subst, env.types())` against three collection shapes:
  - `Parametric { head == "wat::core::Vector", args: [X] }` → unify arg1's type with `X`; return `bool`.
  - `Parametric { head == "wat::core::HashSet", args: [X] }` → unify arg1's type with `X`; return `bool`.
  - `Parametric { head == "wat::core::HashMap", args: [K, V] }` → unify arg1's type with **`K`** (HashMap.contains-key? checks the KEY); return `bool`.
  - else → teaching `CheckError::TypeMismatch` (`expected: "Vector<T>, HashSet<T>, or HashMap<K,V>"`).
- On unify-fail of arg1 with element-type → push a `CheckError::TypeMismatch`
  naming the actual element type (`got: "<arg1's type>"`, `expected: "<element type>"`).

### 2. `src/runtime.rs` — add `eval_contains` + wire the dispatch arm

Mirror `eval_length` shape (runtime.rs:16155): arity-check 2 → eval arg0 + arg1
→ match raw `arg0_val`:
- `Value::Vec(xs)` → `Ok(Value::bool(xs.contains(&arg1_val)))`
- `Value::wat__std__HashSet(s)` → `Ok(Value::bool(s.contains(&arg1_val)))`
- `Value::wat__std__HashMap(m)` → `Ok(Value::bool(m.contains_key(&arg1_val)))`
- else → teaching `RuntimeError::TypeMismatch` (same shape as `eval_length`).
Wire next to length/empty? in the dispatch at runtime.rs:5323:
`":wat::core::contains?" => eval_contains(...)`.

You may instead route to the existing per-type leaves
(`eval_vector_contains`/`eval_hashset_contains`/`eval_hashmap_contains_key`)
if they exist + match cleanly — your call, whichever reads cleanest.

### 3. `wat/core.wat` — delete the `define-dispatch :wat::core::contains?` decl

Same shape as 237.7a/b-i deletes; leave a one-line tombstone comment ("arc 237
Stone 237.7b-ii — `:wat::core::contains?` is now a Rust ∀T intrinsic; see
src/runtime.rs::eval_contains") and keep the per-type leaves intact.

### 4. KEEP

- The per-type leaves (`:Vector/contains?`, `:HashSet/contains?`,
  `:HashMap/contains-key?`).
- The DispatchRegistry.
- `get` + `conj` `define-dispatch` decls (those evacuate in 237.7b-iii/iv).
- All arithmetic `define-dispatch` decls (237.8).

## Scope note — NO List arm (consistency)

`length` + `empty?` (the shipped exemplars) are Vector/HashMap/HashSet only —
NOT `wat__core__List`. Mirror that exactly: do **not** add a List arm to
`contains?`. List coverage for the collection-op intrinsics is a separate
uniform stone. If you feel the urge, STOP — out of scope.

## Verify

- `cargo build --release -p wat` → 0 errors.
- `cargo test --release --test probe_arc237_7b_intrinsic_typing` → `7 passed; 0 failed` (regression guard; `contains_q_*` tests must stay green).
- `./scripts/green-gate.sh` → `green-gate: PASS` (lib 834/0 + test-build 0 errors).

## STOP triggers (REJECTION — surface, do not work around)

- If `(:wat::core::contains? <coll> <elem>)` won't resolve to the new intrinsic
  after the decl delete (hidden coupling) — STOP, surface the coupling.
- If a wrong-elem call (`(contains? (Vector :i64 1 2) "x")`) is NO LONGER
  rejected at check post-migration — STOP. The probe's
  `contains_q_wrong_element_rejected_at_check` test will catch this; if it
  flips from pass to fail, the custom arm isn't enforcing element-typing.
- If the HashMap arm checks `arg1` against `V` (value type) instead of `K`
  (key type) — STOP and fix. `contains?` on HashMap is `contains-key?`.
- Any urge to add a List arm, touch other ops, the registry, or holon-rs — STOP.

## Definition of done

- All 7 probe tests green; lib 834/0; build gate 0; green-gate PASS.
- `wat/core.wat` no longer has `define-dispatch :wat::core::contains?`;
  `src/check.rs` has the custom arm + `infer_contains` helper; `src/runtime.rs`
  has `eval_contains` + the dispatch arm wired.
- Only `src/check.rs` + `src/runtime.rs` + `wat/core.wat` touched. NO holon-rs,
  NO List arm, NO other ops, NO registry deletion, NO probe edits.
- Write `SCORE-STONE-237.7b-ii.md` (sibling); do NOT commit (orchestrator scores
  + commits).
