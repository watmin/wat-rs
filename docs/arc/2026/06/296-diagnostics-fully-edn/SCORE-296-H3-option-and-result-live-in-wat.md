# SCORE — 296 H-3: Option and Result live in wat

No commit. Floor and clippy left to the orchestrator. No `wat_enum_from!` generated enum (Option/Result stay `Value::Option` / `Value::Result`).

---

## Row 1 — the params survive, in order

`option_and_result_are_registered_parametric` **PASSES**. `TypeEnv::with_builtins()`:

```
:wat::core::Option  →  ["T"]
:wat::core::Result  →  ["T", "E"]
```

## Row 2 — the declaration is IN WAT

```
wat/core.wat:2125:(:wat::core::defenum :wat::core::Option :- [T] :wat::enum::Pure
wat/core.wat:2128:(:wat::core::defenum :wat::core::Result :- [T E] :wat::enum::Pure
```

Two hits. Variants/fields match the retired literals: Some `value`, None unit, Ok `value`, Err `error`. Purity `Pure` carried verbatim (the decision at `types.rs:1233-1242`).

## Row 3 — the Rust literals are GONE

`grep -n 'name: ":wat::core::Option"' src/types.rs` = **0**. Replaced by:

```
wat_enum_register_from!(env, "wat/core.wat", ":wat::core::Option");
wat_enum_register_from!(env, "wat/core.wat", ":wat::core::Result");
```

## Row 4 — binder capture at arity 3

`types::tests::wat_enum_register_from_captures_binder_at_arity_3` **PASSES**. Registers `:wat::spawn::ServiceEvent` from `wat/spawn.wat` (`:- [I O A]`) into a fresh `TypeEnv`; params are `["I","O","A"]` in declaration order. A fixed-offset bug would have passed at Option's arity 1 and failed here.

## Row 5 — the re-declaration is a NoOp

Stdlib loaded: H-2 probe (wat fixtures) and option_result_tagged ran against the rebuilt binary. No `Duplicate`. The load-time `defenum` is Equivalent to the compile-time registration.

## Row 6 — the wire did not move

`probe_arc296_h2` **3/3**, `#[ignore]` count **0**.

## Row 7 — Option/Result round-trip

`option_result_tagged` **8 passed**. `#wat.core/Option.Some {:value 7}` both directions.

## Row 8 — the floor (ORCHESTRATOR)

Not run.

## Row 9 — clippy (ORCHESTRATOR)

Not run.

## Row 10 — purity unchanged

Both `defenum`s are `:wat::enum::Pure`. The argued decision in `types.rs` (wire-crossing / `:durable`) is carried, not re-derived.

---

## The prerequisite — binder capture

`declared_name` already detected `:- [T …]` to shift the payload offset and discarded the vector. `binder_type_params` now reads it, in order, with the same "bare Symbol, no `/`" rule as `take_declared_binder`.

**Both register sites were fixed** (the brief named the pair):

- `wat_enum_register_from!` — this stone's consumer (Option/Result; ServiceEvent arity-3 proof)
- `wat_record_from!` — the sibling, same hardcoded `Vec::new()`. Not a consumer today (no parametric record goes through it yet); leaving it would prime the next parametric record for the silent drop. Fixed in the same function.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 params empty/reordered | **held.** Probe still green. |
| STOP-2 binder reads payload | **held.** Arity 3 ServiceEvent `["I","O","A"]`. |
| STOP-3 Duplicate | **did not fire.** |
| STOP-4 purity | **held.** Both Pure. |
| STOP-5 a third type | **not taken.** |

## Targeted checks (executor)

```
cargo nextest run --release -E 'test(probe_arc296_h3)'     passed
cargo nextest run --release -E 'test(probe_arc296_h2)'     3 passed
cargo nextest run --release -E 'test(option_result_tagged)'  8 passed
cargo test --release --lib -- wat_enum_register_from_captures_binder_at_arity_3  passed
```

## Files (this stone)

```
crates/wat-source-derive/src/lib.rs   binder_type_params; both register macros emit it
wat/core.wat                          defenum Option / Result
src/types.rs                          wat_enum_register_from! ×2; arity-3 unit test
tests/types/probe_arc296_h3_*.rs      unchanged (GREEN-at-HEAD guard)
```
