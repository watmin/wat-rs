# BRIEF — Stone 237.7b-i — `:wat::core::empty?` reborn as a `∀T` intrinsic

Paired with `DESIGN-STONE-237.7b.md` + the proven recipe probe
`tests/probe_arc237_7b_intrinsic_typing.rs` (the regression contract — keep its
`empty_q_*` tests green). This is the **exact shape of 237.7a `length`**, with a
`bool` return in place of `i64`. Tier A (concrete return) — plain ∀ scheme; no
custom inference needed.

## The work — mirror `length` precisely

`:wat::core::empty?` is currently a `define-dispatch` entity (`wat/core.wat`,
the `(:wat::core::define-dispatch :wat::core::empty? ...)` decl with the
Vector/HashMap/HashSet clauses). Reborn it as a Rust `∀T` intrinsic:

1. **Register the scheme** in `register_builtins` (src/check.rs), mirroring
   `:wat::core::length` at **check.rs:19610**: `type_params: ["T"]`,
   `params: [t_var()]`, `ret: <bool type>`, `rest_param_type: None`.
   (`length` is `∀T. T -> i64` via `i64_ty()`; `empty?` is `∀T. T -> bool` —
   use the bool type helper; grep how other `-> bool` builtins express it,
   e.g. `conforms?` / `contains?`-era registrations, or a `bool_ty()` helper.)

2. **Add the eval handler** `eval_empty` in src/runtime.rs, mirroring
   `eval_length` (**runtime.rs:16155**): arity-check (1 arg) → eval the arg →
   `match` the raw `Value`:
   - `Value::Vec(xs)` → `Value::bool(xs.is_empty())`
   - `Value::wat__std__HashMap(m)` → `Value::bool(m.is_empty())`
   - `Value::wat__std__HashSet(s)` → `Value::bool(s.is_empty())`
   - anything else → a teaching `RuntimeError::TypeMismatch` (same shape as
     `eval_length`'s `other` arm; expected "Vector<T>, HashMap<K,V>, or HashSet<T>").
   Wire the dispatch arm next to `":wat::core::length" => eval_length(...)` at
   **runtime.rs:5323**: `":wat::core::empty?" => eval_empty(...)`.
   You may route to the existing per-type leaf logic
   (`eval_vector_empty`/`eval_hashmap_empty`/`eval_hashset_empty` if they exist)
   or inline `.is_empty()` — whichever reads cleanest, matching `eval_length`.

3. **Delete** the `(:wat::core::define-dispatch :wat::core::empty? ...)` decl in
   `wat/core.wat`.

4. **KEEP**: the per-type leaves (`:Vector/empty?` etc.), the `DispatchRegistry`,
   and every OTHER `define-dispatch` decl (`contains?`/`get`/`conj` + arithmetic).
   Only `empty?` evacuates this stone.

## Scope note — NO List arm (consistency)

`length` (the shipped exemplar) covers Vector/HashMap/HashSet only — NOT
`wat__core__List`. Mirror that EXACTLY: do **not** add a List arm to `empty?`
here. List coverage for the collection-op intrinsics is a separate uniform stone
(it must cover `length` too, or they diverge). If you feel the urge to add List,
STOP — it's out of scope for 7b-i.

## Verify against the substrate

`cargo build --release -p wat`, then:
- `cargo test --release --test probe_arc237_7b_intrinsic_typing` → the `empty_q_*`
  tests green (regression guard).
- `./scripts/green-gate.sh` → lib 834/0 + test-build 0 errors.
Deleting the decl shifts `empty?`'s call-site resolution from the dispatch branch
to ordinary scheme lookup → it must find the new builtin. If anything ripples,
the errors name the sites — fix them (substrate-as-teacher).

## STOP triggers (REJECTION — surface, do not work around)

- If `(:wat::core::empty? [...])` won't resolve to the new builtin after the decl
  delete (hidden define-dispatch coupling) — STOP, surface the coupling; do NOT
  re-add a shim.
- If the `∀T. T -> bool` scheme doesn't type-check the way `length`'s `∀T. T ->
  i64` does — STOP, show the diff vs `length`.
- Any urge to add a List arm, touch other ops, the registry, or holon-rs — STOP.

## Definition of done

- `probe_arc237_7b_intrinsic_typing` `empty_q_*` green; lib 834/0; build gate 0.
- `wat/core.wat` no longer has `define-dispatch :wat::core::empty?`;
  `register_builtins` has the `:wat::core::empty?` scheme; runtime has the eval arm.
- Only `src/check.rs` + `src/runtime.rs` + `wat/core.wat` touched. NO holon-rs,
  NO List arm, NO other ops, NO registry deletion.
- Write `SCORE-STONE-237.7b-i.md` (sibling); do NOT commit (orchestrator scores + commits).
