# SCORE — Arc 214 Slice 4 Stone 4.2 — `:wat::program::Env/get` trio

**Mode A target:** 18/18  
**Actual:** 18/18 PASS  
**Time:** ~40 minutes (within the 45-75 min target band)

## Scorecard

| # | Row | Result | Notes |
|---|---|---|---|
| 1 | `/get` verb registered | PASS | `eval_program_env_get` in `src/runtime.rs`; dispatch arm at eval match; `infer_program_env_get` in `src/check.rs` |
| 2 | `/expect-get` verb registered | PASS | `eval_program_env_expect_get`; panic via `std::panic::panic_any(AssertionPayload)` with KeyError-flavored message |
| 3 | `/get-default` verb registered | PASS | `eval_program_env_get_default`; default type unified with T at check time |
| 4 | Probe 1 — `/get` found + correct type | PASS | `Env/get env :foo -> :wat::core::String` → `Some("bar")` |
| 5 | Probe 2 — `/get` not found | PASS | Missing key → `None` |
| 6 | Probe 3 — `/get` wrong type | PASS | `HolonAST::String` stored; requested `i64` → `None` |
| 7 | Probe 4 — `/get` multi-type | PASS | T ∈ {i64, String, bool, keyword} all extract correctly |
| 8 | Probe 5 — `/expect-get` found | PASS | Returns T directly (String "bar") |
| 9 | Probe 6 — `/expect-get` not found | PASS | Panics with AssertionPayload naming the key |
| 10 | Probe 7 — `/expect-get` wrong type | PASS | Panics with type-mismatch diagnostic |
| 11 | Probe 8 — `/get-default` found | PASS | Returns found "bar"; "fallback" default ignored |
| 12 | Probe 9 — `/get-default` not found | PASS | Returns "fallback" |
| 13 | Probe 10 — `/get-default` wrong type | PASS | Returns default 99 when stored String ≠ i64 |
| 14 | Probe 11 — `/get-default` default type unification | PASS | `default=42` vs `-> :String` fails at check with TypeMismatch |
| 15 | Probe 12 — All three on same env | PASS | get → Some("bar"); expect-get → "bar"; get-default → "bar" |
| 16 | Probe 13 — Empty env behavior | PASS | get → None; expect-get → panic; get-default → default |
| 17 | Probe 14 — HolonAST::Atom unwrap | PASS | `(:wat::holon::Atom 42)` → `HolonAST::I64(42)` → `Some(42)` |
| 18 | Probe 15 — Nested holon as wrong type | PASS | `(:wat::holon::Atom (:wat::holon::Atom "x"))` → `HolonAST::Atom(...)` → None/panic/default |

## Verification commands run

```
cargo build --release                                              # PASS (warnings only, pre-existing)
cargo test --release --test probe_arc214_slice4_stone2_env_get_trio -p wat   # 15/15 PASS
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias -p wat  # 6/6 PASS
cargo test --release --test probe_arc215_stone2 -p wat            # 13/13 PASS
cargo test --release --test probe_arc215_collection_literal_inference -p wat  # 12/12 PASS
cargo test --release --test probe_brace_map_literal -p wat        # 9/9 PASS
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat       # 9/9 PASS
cargo clippy --release -- -D warnings                             # 111 pre-existing errors; NONE from new code
```

**Clippy:** 111 pre-existing errors (lines 2704, 2726, 2732, 3129, 4690..., etc. in check.rs; 183, 184, 1706... in runtime.rs). None of the errors are in my new code (check.rs 8814+; runtime.rs 7859+). Pre-existing before Stone 4.2.

## Implementation summary

### Files modified

**`src/runtime.rs`** — new code at lines 7831–8073:
- `holon_ast_extract(h: &HolonAST, target_ty: &TypeExpr) -> Option<Value>` — extracts HolonAST leaf to Value, returns None on composite HolonAST or type-mismatch against `target_ty`
- `program_env_lookup(op, env_val, key_val, target_ty, key_span) -> Result<Option<Value>, RuntimeError>` — shared lookup: validates env is HashMap, looks up key, calls `holon_ast_extract`
- `parse_arrow_ty(op, args, arrow_pos, type_pos) -> Result<TypeExpr, RuntimeError>` — parses `-> :T` from AST args at given positions
- `eval_program_env_get(args, env, sym)` — 4-arg form `[env, key, ->, :T]`; returns `Value::Option`
- `eval_program_env_expect_get(args, env, sym)` — 4-arg form; panics via `panic_any(AssertionPayload)` on None
- `eval_program_env_get_default(args, env, sym)` — 5-arg form `[env, key, default, ->, :T]`; evals default on None

Dispatch arms added after `:wat::core::HashMap/get` in the eval match.

**`src/check.rs`** — new code at lines 8802–9140 (after `infer_kernel_readln`):
- `infer_program_env_get(head_span, args, env, locals, fresh, subst, errors)` — validates 4-arg form; returns `Option<T>`
- `infer_program_env_expect_get(...)` — validates 4-arg form; returns `T` (not Option)
- `infer_program_env_get_default(...)` — validates 5-arg form; unifies default type with T; returns `T`

Dispatch arms added after `:wat::kernel::readln` in the infer match.

**`docs/WAT-CHEATSHEET.md`** — brief accessor trio section added under `:wat::program::Env` (table + example).

**`tests/probe_arc214_slice4_stone2_env_get_trio.rs`** — new probe file, 15 probes.

## Calibration check

- **Target:** 45-75 min
- **Actual:** ~40 min
- **Within prediction band?** Slightly below the lower bound (40 vs 45) — attribution: the three `infer_program_env_*` functions share nearly identical structure (80%+ code reuse as predicted); writing the first composed the pattern, the other two were near-copy. Pattern templates (`eval_hashmap_get`, `infer_kernel_readln`, `option::expect`) composed cleanly with minimal adaptation.

## Honest deltas

1. **Probe 15 reshaping.** The BRIEF specified `HolonAST::Bundle` as the wrong-type test case. `Bundle` construction requires an encoding context (`require_encoding_ctx`) and a `Vec<HolonAST>` — it's not trivially constructable in a type-check-only probe without a live holon encoding substrate. Substituted with `HolonAST::Atom(HolonAST::String("x"))` (nested Atom) — this tests the same "non-primitive-leaf → wrong-type" path (the Atom arm returns `Value::holon__HolonAST`, which fails T-matching for String/i64/bool). The semantic coverage is identical; the probe description was updated to be honest about the substitution.

2. **Type syntax in probes.** WAT type arguments inside `<>` use bare names (`:wat::core::Option<wat::core::String>`, not `<:wat::core::String>`). The BRIEF's examples used `:` inside `<>` which the parser rejects. Fixed in probe source; noted here as a gotcha for future stone authors.

3. **`catch_unwind` for expect-get panic tests.** `FrozenWorld` doesn't implement `UnwindSafe` (contains interior mutability). Used `std::panic::AssertUnwindSafe` wrapper. This is the standard pattern; no behavioral concern (closure owns all values; no shared state corrupted on panic).

4. **`-> :T` in `expect-get` is auto-panicking, not user-message.** Arc 107's `option::expect` takes a user-provided message string. `expect-get` auto-generates the diagnostic from the key and type context. Simpler UX for a key-lookup verb; user doesn't need to write the error message.

5. **Clippy pre-existing.** 111 clippy errors exist before Stone 4.2. None are from new code. The `cargo clippy -- -D warnings` check would have failed on the baseline branch too. Not blocking the stone.
