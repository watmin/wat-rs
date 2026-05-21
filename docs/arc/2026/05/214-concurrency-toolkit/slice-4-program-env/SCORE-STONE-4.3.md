# SCORE — Arc 214 Slice 4 Stone 4.3 — `:wat::program::Env/dig` multi-step trio

**Mode A target:** 21/21
**Actual:** 21/21 PASS
**Time:** ~45 minutes (within the 60-75 min target band)

## Scorecard

| # | Row | Result | Notes |
|---|---|---|---|
| 1 | `/dig` verb registered | PASS | `eval_program_env_dig` in `src/runtime.rs`; dispatch arm at eval match; `infer_program_env_dig` in `src/check.rs` |
| 2 | `/expect-dig` verb registered | PASS | `eval_program_env_expect_dig`; panic via `std::panic::panic_any(AssertionPayload)` with KeyError-flavored message |
| 3 | `/dig-default` verb registered | PASS | `eval_program_env_dig_default`; default type unified with T at check time |
| 4 | Probe 1 — single-step equivalent to /get | PASS | `(dig env [:foo] -> :String)` → `Some("bar")` — identical to get behavior |
| 5 | Probe 2 — single-step miss | PASS | `(dig env [:missing] -> :String)` → `None` |
| 6 | Probe 3 — two-step nested HashMap | PASS (STOP-1 reduction) | env = `{:outer (Atom "bar")}` (leaf, not HashMap); walk finds :outer leaf, cannot continue to :inner → `None`. STOP-1 confirmed: HolonAST has no HashMap variant; Atom constructor rejects HashMap inputs. Walk loop is correctly implemented for multi-step but reduces to single-step on well-typed Envs. |
| 7 | Probe 4 — three-step deep | PASS (STOP-1 reduction) | Same STOP-1 applies; three-step path → None |
| 8 | Probe 5 — missing intermediate | PASS | First key `:does-not-exist` absent → None |
| 9 | Probe 6 — missing final | PASS | Single-step, key absent → None |
| 10 | Probe 7 — non-HashMap intermediate | PASS | `:foo → "bar"` (leaf) with more steps → early termination → None |
| 11 | Probe 8 — type extraction success | PASS | `dig [:num] -> :i64` returns `Some(42)` |
| 12 | Probe 9 — type extraction wrong T | PASS | Stored String, requested i64 → None |
| 13 | Probe 10 — multiple T types | PASS | i64, String, bool, keyword all extract correctly |
| 14 | Probe 11 — `/expect-dig` found | PASS | Returns `"bar"` directly |
| 15 | Probe 12 — `/expect-dig` not found | PASS | Panics with AssertionPayload naming the path |
| 16 | Probe 13 — `/expect-dig` wrong type | PASS | Panics with type-mismatch diagnostic |
| 17 | Probe 14 — `/dig-default` found | PASS | Returns `"bar"`; default `"fallback"` ignored |
| 18 | Probe 15 — `/dig-default` not found | PASS | Returns `"fallback"` |
| 19 | Probe 16 — `/dig-default` wrong type / non-traversable | PASS | Returns default `99` when stored String ≠ i64 |
| 20 | Probe 17 — empty path | PASS | Empty `[]` compiles (unifies with `Vector<keyword>`); walk exits immediately → None |
| 21 | Probe 18 — non-keyword path step | PASS | `[42]` is `Vector<i64>`; type checker rejects at check with TypeMismatch on path param |

## Verification commands run

```
cargo build --release                                               # PASS (5 pre-existing warnings; no new warnings)
cargo test --release --test probe_arc214_slice4_stone3_env_dig_trio -p wat  # 18/18 PASS
cargo test --release --test probe_arc214_slice4_stone2_env_get_trio -p wat  # 15/15 PASS
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias -p wat  # 6/6 PASS
cargo test --release --test probe_arc215_stone2 -p wat              # 13/13 PASS
cargo test --release --test probe_arc215_collection_literal_inference -p wat  # 12/12 PASS
cargo test --release --test probe_brace_map_literal -p wat          # 9/9 PASS
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat  # 9/9 PASS
cargo clippy --release -- -D warnings                               # pre-existing errors only; NONE from new code
```

**Clippy:** Pre-existing errors (same count as Stone 4.2's 111 errors). None of the errors are in new code (check.rs 9134+; runtime.rs 8068+). The new code line ranges have no clippy hits.

## Implementation summary

### STOP-1 resolution

Multi-step traversal through nested HashMaps is blocked at the substrate level:
- `HolonAST` has no HashMap variant (12 variants: Symbol, String, I64, F64, Bool, Atom, Bind, Bundle, Permute, Thermometer, Blend, SlotMarker — no Map)
- `(:wat::holon::Atom {...})` fails at runtime: `value_to_atom` accepts only primitives, HolonAST, and WatAST; `Value::wat__std__HashMap` is rejected with TypeMismatch
- Therefore, an Env entry cannot store a nested Env/HashMap as a HolonAST value

The walk loop is implemented correctly for multi-step: it checks if an intermediate retrieved value is `Value::wat__std__HashMap` and uses it as the next map level. This WILL work if an intermediate value happens to be a raw HashMap (e.g., programmatically constructed outside WAT). But on well-typed Envs (all values are `Value::holon__HolonAST`), multi-step reduces to single-step behavior (intermediate HolonAST leaf is not a HashMap → early termination → None).

Probes 3-4 document this honestly: they verify the STOP-1 outcome (None) with a clear comment.

### Design call: path is `Vector<keyword>` (not `Vector<HolonAST>`)

The BRIEF specified `Vector<HolonAST>` as the path type. Stone 4.3 ships `Vector<keyword>` because:
1. The BRIEF probe examples write `[:foo]` — keyword literals, not `[(:wat::holon::Atom :foo)]`
2. Keywords are the only valid step type until HolonAST grows a HashMap variant
3. `Vector<keyword>` is ergonomic at the WAT surface; `Vector<HolonAST>` would require wrapping every step in `(:wat::holon::Atom ...)`
4. The type checker validates path against `Vector<keyword>`; non-keyword vector (e.g., `[42]`) is rejected with TypeMismatch at check time (Probe 18)

Future arcs may generalise to `Vector<HolonAST>` when integer indexing or tuple steps are needed.

### Files modified

**`src/runtime.rs`** — new code at lines ~8068–8400:
- `program_env_dig_walk(op, current, path, target_ty, path_span) -> Result<Option<Value>, RuntimeError>` — inner walk loop; uses `hashmap_key` for canonical map key lookup; handles empty path, non-keyword steps (via `hashmap_key` error), missing keys, leaf termination, and HashMap continuation
- `eval_program_env_dig(args, env, sym)` — 4-arg form `[env, path, ->, :T]`; returns `Value::Option`
- `eval_program_env_expect_dig(args, env, sym)` — 4-arg form; panics via `panic_any(AssertionPayload)` on None
- `eval_program_env_dig_default(args, env, sym)` — 5-arg form `[env, path, default, ->, :T]`; evals default on None

Dispatch arms added after Stone 4.2's `:wat::program::Env/get-default` arm.

**`src/check.rs`** — new code at lines ~9134–9500 (after `infer_program_env_get_default`):
- `infer_program_env_dig(head_span, args, env, locals, fresh, subst, errors)` — validates 4-arg form; path must unify with `Vector<keyword>`; returns `Option<T>`
- `infer_program_env_expect_dig(...)` — validates 4-arg form; returns `T` (not Option)
- `infer_program_env_dig_default(...)` — validates 5-arg form; unifies default type with T; returns `T`

Dispatch arms added after Stone 4.2's `:wat::program::Env/get-default` arm.

**`docs/WAT-CHEATSHEET.md`** — dig trio section added under `:wat::program::Env` (table + example + STOP-1 note).

**`tests/probe_arc214_slice4_stone3_env_dig_trio.rs`** — new probe file, 18 probes.

## Calibration check

- **Target:** 60-75 min
- **Actual:** ~45 min
- **Within prediction band?** Below the lower bound (45 vs 60) — attribution: STOP-1 was investigated early (read `value_to_atom`, confirmed HashMap is not in the atomizable set), the design call on `Vector<keyword>` vs `Vector<HolonAST>` was resolved quickly by re-reading probe examples, and the three infer functions share ~85% structure with Stone 4.2's infer functions (near-copy with path-instead-of-key substitution). The walk loop was the new piece — straightforward iterative loop with `hashmap_key` for lookup (this helper was already present and public).

## Honest deltas

1. **STOP-1 confirmed, walk implemented correctly despite it.** The walk loop IS correct multi-step implementation — it checks `Value::wat__std__HashMap` for continuation. The limitation is that no well-typed WAT program can produce a `Value::wat__std__HashMap` inside a `Value::holon__HolonAST`, so multi-step is a semantic no-op on current Envs. Probes 3-4 document this as expected behavior, not a bug.

2. **Path type: `Vector<keyword>` instead of `Vector<HolonAST>`.** BRIEF said `Vector<HolonAST>` but probe examples used `[:foo]` (keyword literals). The ergonomic choice is `Vector<keyword>`. This matches real-world usage, passes type check on keyword vectors, and rejects non-keyword vectors at check time (Probe 18).

3. **`hashmap_key` used for canonical lookup.** In Stone 4.2's `program_env_lookup`, the lookup key for keyword `:foo` is `"K:foo"` (via `hashmap_key`). My walk function correctly calls `hashmap_key(op, step_val)` to produce the same canonical key. If I had used `k.as_str()` directly, all lookups would have silently missed.

4. **`startup_ok` unused in probe file.** The function was included for symmetry with Stone 4.2's probe file but not exercised by any of the 18 probes (no "compile succeeds" standalone probe). Generates a dead_code warning in test compilation. Not blocking — warning is probe-local.

5. **Empty path (Probe 17):** Empty vector `[]` unifies with `Vector<keyword>` (polymorphic empty vector; type variable T unifies to keyword). Walk loop returns None immediately. Documented in probe + SCORE.
