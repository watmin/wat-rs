# BRIEF — Stone 237.7a — `:wat::core::length` reborn as a `∀T` intrinsic

Paired with `DESIGN-STONE-237.7-intrinsic-kill.md` + `EXPECTATIONS-STONE-237.7a.md`. The probe
`tests/probe_arc237_7a_length_intrinsic.rs` is committed as the contract — make/keep it green.

## The work

`:wat::core::length` is currently a `define-dispatch` entity (declared at `wat/core.wat:12`) routing
to per-type leaves. Reborn it as a plain Rust `∀T` **intrinsic** — the exact shape of
`:wat::core::type`:

1. **Register the scheme** in `register_builtins` (src/check.rs), mirroring `:wat::core::type`
   (check.rs:~19419): `type_params: ["T"]`, `params: [t_var()]`, `ret: :wat::core::i64`,
   `rest_param_type: None`. (`type` is `∀T. T -> String`; `length` is `∀T. T -> i64`.)

2. **Add the eval handler** in src/runtime.rs, mirroring `eval_type` (runtime.rs:16119): arity-check
   (1 arg) → eval the arg → `match` the `Value`:
   - `Value::Vector(..)` → its length
   - `Value::wat__std__HashMap(..)` (or the current HashMap variant) → its length
   - `Value::wat__std__HashSet(..)` (or current HashSet variant) → its length
   - anything else → a teaching `RuntimeError` ("length expects a collection; got <type>")
   Route to the existing per-type logic (`eval_vector_length`/`eval_hashmap_length`/
   `eval_hashset_length` at runtime.rs:5760-5762) or inline `.len()` on the inner — your call,
   whichever reads cleanest. Wire the dispatch arm next to `":wat::core::type" => eval_type(...)`.

3. **Delete** the `(:wat::core::define-dispatch :wat::core::length ...)` decl at `wat/core.wat:12-15`.

4. **Keep**: the per-type leaves (`:Vector/length` etc.), the `DispatchRegistry`/`dispatch.rs`, and
   every OTHER `define-dispatch` decl (`empty?`/`contains?`/`get`/`conj` + arithmetic `+'2` etc.).
   They still tenant the registry; only `length` evacuates this stone.

## Verify against the substrate as you go

Run `cargo test --release --workspace --no-fail-fast`. Deleting the decl shifts `length`'s call-site
resolution from the dispatch branch (check.rs:6859) to ordinary scheme lookup → it must find the new
builtin. If anything ripples, the errors name the sites — fix them (substrate-as-teacher).

## STOP triggers (REJECTION criteria — surface, do not work around)

- If `(:wat::core::length [1 2 3])` will not resolve to the new builtin after the decl delete (some
  hidden `define-dispatch` coupling for `length`) — STOP and surface the exact coupling. Do NOT
  re-add a shim or leave the decl.
- If the `∀T. T -> i64` scheme does not type-check a call the way `:wat::core::type`'s does — STOP and
  show the diff vs `type`. (It should be identical shape.)

## Definition of done

- `tests/probe_arc237_7a_length_intrinsic.rs` — all 6 green.
- lib baseline ≥ 834 / 0 failed; workspace 0 FAILED.
- `wat/core.wat` no longer contains `define-dispatch :wat::core::length`; `register_builtins` contains
  the `:wat::core::length` scheme; runtime has the eval arm.
- NO holon-rs edits; NO namespace renames; NO touching other ops or the registry.
- Write `SCORE-STONE-237.7a.md` (sibling); do NOT commit (orchestrator scores + commits).
