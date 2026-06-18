# BRIEF — Stone 0b: `:wat::core::PersistentVector` (rpds `VectorSync`)

Single-hop sonnet in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** Build, run the named
tests, report verbatim. Another agent weighs independently.

## The work
Add `:wat::core::PersistentVector` backed by `rpds::VectorSync<Value>` — a persistent (structural-sharing,
O(log n) `conj`) vector, beside the std `:wat::core::Vector` (untouched). It is the VECTOR MIRROR of the
just-shipped stone 0a (`:wat::core::PersistentMap`). **The shipped `persistentmap_*` code is your live,
correct template — read it and mirror it for the vector.**

## Read first (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-0b-persistent-vector.md` — the contract + the 0a→0b delta
   table (the substitutions to make). The ONE decision: EDN tag `#wat.core/PersistentVector [...]`.
2. **The SHIPPED stone 0a code** (commits `3daea024`+`70835637`) — your template. Grep `PersistentMap` /
   `persistentmap_` across `src/` and mirror each site for the vector:
   - `src/value/value.rs` — the `wat__core__PersistentMap` variant + its `PartialEq`/`Hash`/`type_name`/
     `type_path` arms. For the vector, ALSO mirror the std `Value::Vec` arms (value.rs:606/728/1040/1146).
     **Hash: order-DEPENDENT** for a vector (mirror std `Vec`'s `hash_sequence` at value.rs:728, NOT the
     map's sorted-pairs hash — a vector's order is semantic).
   - `src/collection/eval.rs` — mirror `persistentmap_*` → `persistentvector_*`, using the std `vector_*`
     fns as the op shapes: `length`(:27) `empty?`(:127) `contains?`(:225) `get`(:353, index→Option)
     `conj`(:542) + `first`/`rest`. rpds: `VectorSync::new_sync()`, `.push_back(v)->Self` (for conj — NO
     clone), `.get(i)->Option<&V>`, `.len()`, `.iter()`.
   - `src/runtime.rs` — dispatch arms `:wat::core::PersistentVector/<op>` + ctor; Layer-1 generic arms on
     `eval_conj`(:3834) `eval_get`(:3839) `first`(:4016) `rest`(:4028).
   - `src/collection/infer.rs` — mirror the `wat::core::Vector` arms in `infer_conj`(:158) `infer_get`(:252,
     index=i64) `infer_contains`(:60) → `PersistentVector<T>`.
   - `src/check.rs` — `BARE_CONTAINER_HEADS` entry + ctor dispatch + `infer_persistentvector_constructor`
     (mirror `infer_persistentmap_constructor`; infer `T` from the elements).
   - `src/edn_shim.rs` — mirror the 0a tag write/read for `#wat.core/PersistentVector` over an EDN VECTOR body.
3. `tests/probe_arc278_0b_persistent_vector.rs` — remove its `#[ignore]`.

## Also (close 0a's lesson up front, in THIS stone)
Add a lib unit test in `src/edn_shim.rs` `mod tests` (mirror the existing `persistent_map_edn_round_trip`):
build a `Value::wat__core__PersistentVector(...)`, `value_to_edn_string` → `edn_string_to_value`, assert it
comes back EQUAL **and** still `matches!(.., Value::wat__core__PersistentVector(_))` (not a std Vector).

## Method
Add the variant → `cargo build` → mirror the `PersistentMap` arm at every exhaustive match the compiler
flags (same ~7-file ripple 0a hit; 0a's arms show exactly where + how). Logic in `collection/`+`value/`
homes; only forced thin arms in the megafiles.

## STOP triggers
1. If the EDN tag does NOT round-trip to a PersistentVector (collapses to std Vector / errors) — STOP, report.
2. If std `Vector` behavior or its tests change — STOP.
3. If any floor moves beyond the two new tests (+1 probe, +1 lib EDN test) — STOP and report.

## Verify (paste verbatim)
```
cargo test --release -p wat --test probe_arc278_0b_persistent_vector -- --include-ignored   # 1/1 GREEN
cargo test --release -p wat --lib persistent_vector_edn_round_trip 2>&1 | grep "test result" # 1/1 GREEN
cargo test --release -p wat --lib 2>&1 | grep "test result"                                  # 931 passed / 36 (was 930, +1)
cargo test --release --test test 2>&1 | grep "test result"                                   # 264 / 1 (UNCHANGED)
cargo test --release -p wat --test nursery -- --test-threads=1 | grep "test result"          # ~893 / 4 (UNCHANGED)
cargo test --release --test test_stdlib_load_order | grep result                             # 1 / 0
cargo build --release 2>&1 | tail -2                                                          # clean
```
Report: the diff (files + variant + each mirrored arm), outputs verbatim, any STOP hit, any delta. Do not
claim a green you did not see. Un-ignore the probe. No git.

## Blast radius
`src/value/value.rs` · `src/collection/eval.rs` · `src/collection/infer.rs` · `src/check.rs` ·
`src/runtime.rs` · `src/edn_shim.rs` (+ the lib test) · forced arms in `observe.rs`/`ast.rs`/`closure_extract.rs`.
Cargo.toml UNCHANGED (rpds already present). + un-ignore the probe. No wat-side files. No git.
