# BRIEF — STONE E: the hashability check looks inside the key

Read `DESIGN-stone-E-hashability-looks-inside.md` first.

## The work

Make `value_is_hashable` (`src/runtime.rs`, ~`:6735-6770`) deep and single-sourced: shallow refusal
from `Value::key_eligibility()` (`InteriorMutable`/`OpaqueHandle` only), recursion mirroring
`impl Hash for Value` (`src/value/value.rs:752-960`), exhaustive `match`, no `_ =>`. Keep
`value_is_key_hashable`/`value_is_set_hashable` as the thin wrappers they are.

## Rooms

1. `src/value/value.rs:752-960` — `impl Hash for Value`: the recursion to mirror, arm by arm.
2. `src/value/value.rs:~1195-1260` — `KeyEligibility`, `NotAKeyReason`, `key_eligibility()`,
   `all_key_eligibility()`.
3. `src/runtime.rs:~6735-6770` — `value_is_hashable` and its two wrappers.
4. Callers, which should need NO change: `src/collection/eval.rs` (HashMap/HashSet guards),
   `src/rust_deps/cache.rs` (`put`/`get`), `src/edn/render.rs:2331,3483`.
5. `src/check.rs:25186-25220` and `tests/value/probe_arc216_stone5a_value_hash.rs:354` — the gates
   binding `key_eligibility()`; they must stay green.

## STOP triggers

1. **An `Aggregate` with a stamped identity** (`Hash` hashes `a.identity`, not the fields): decide
   whether its fields can still hold a handle that reaches `unreachable!()`. If the recursion rule
   is unclear, STOP and report what you measured.
2. **A container whose `Hash` arm recurses in a way the predicate cannot mirror** (e.g. through a
   `dyn` or an opaque payload). Report it; do not approximate.
3. **Any gate reddens that you did not add** — capture whole, name the arm. ⛔ Do not re-run first.

## Prove it — a probe over the containers

For each of `Option.Some`, `Tuple`, `Vector`/`List`, a `HashMap` value, a record field and an enum
variant field, each wrapping an opaque handle, used as a key in: `Lru/put`, `Lru/get`, `HashMap`
insert, `HashSet` conj. Where the checker already refuses the shape, launder it through a generic
`K`/`T` (stone C's probe shows how). Assert: **no `panicked at` on stderr**, and the verb's normal
refusal (`Err` value / miss / wat `TypeMismatch`). Also a positive control: a deep key of pure data
(e.g. `Option.Some (Tuple 1 "a")`) still inserts and is found.

**Mutations:** (a) make the recursion shallow again — the nested cases RED; (b) add a `_ =>` arm
returning `true` — the exhaustiveness is what makes drift unrepresentable, so say what reds.

## Mechanics — ⛔ read

- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks. Do not end your turn while it
  runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines, never piped exit codes.
- `cargo fmt` reformats the whole workspace — `rustfmt <file>` or neither.
- `git add` BEFORE running `git ls-files`-based gates. New files under `tests/` answer to
  `no_loose_string_assert`, `no_inlined_edn`, `no_inlined_wat_in_tests`, `every_tracked_wat_parses`,
  `every_wat_bad_fixture_actually_fails`.
