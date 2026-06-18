# BRIEF — Stone 0c: full PersistentVector transform/sequence parity

Single-hop sonnet in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A bounded, mechanical
Rust stone (the 0a/0b mirror pattern). Build, run the named tests, report verbatim. Another agent weighs.

## The work
Give `PersistentVector` the same transform/sequence ops std `Vec` has, so `Vec` and `PersistentVec` behave
identically and the persistent collections are COMPLETE (no more engine trips). Add a
`Value::wat__core__PersistentVector(pv)` arm to each of these `eval_vec_*` fns in
`src/collection/transform.rs`, mirroring the existing std-`Vec` arm:

| op | fn | signature (grounded) | PersistentVector arm returns |
|---|---|---|---|
| `map`     | `eval_vec_map`     | `(map f xs)` fn-first      | PersistentVector (type-preserving) |
| `filter`  | `eval_vec_filter`  | `(filter pred xs)` fn-first| PersistentVector |
| `foldl`   | `eval_vec_foldl`   | `(foldl f init xs)` fn-first| the accumulator |
| `foldr`   | `eval_vec_foldr`   | `(foldr f init xs)` fn-first| the accumulator |
| `reverse` | `eval_vec_reverse` | `(reverse xs)`             | PersistentVector |
| `take`    | `eval_vec_take`    | `(take xs n)` coll-first   | PersistentVector |
| `drop`    | `eval_vec_drop`    | `(drop xs n)` coll-first   | PersistentVector |
| `concat`  | (Vector/concat + generic) | `(concat a b)`      | PersistentVector |

Iterate the PersistentVector via `pv.iter()`; build vector-returning results with `rpds::VectorSync`
(`new_sync()` + `.push_back`). Type-preserving: a PersistentVector in → a PersistentVector out (for
map/filter/reverse/take/drop/concat); foldl/foldr return the accumulator value.

## Read first
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-0c-persistent-parity.md` — the contract + the 8-op table.
2. `src/collection/transform.rs` — each `eval_vec_*` fn; mirror its std-`Vec` arm for `PersistentVector`.
   (`eval_vec_map` doc says "Arc 247: fn-first — (map f xs)"; foldl/foldr/filter likewise; take/drop coll-first.)
3. `src/collection/eval.rs` — the existing `persistentvector_*` arms (0a/0b) for the rpds idiom.
4. `src/collection/infer.rs` — mirror the std `Vector<T>` infer arms for these ops onto `PersistentVector<T>`
   (type-preserving: map → PersistentVector<U>; filter/reverse/take/drop/concat → PersistentVector<T>;
   foldl/foldr → Acc).
5. `tests/probe_arc278_0c_persistent_parity.rs` — remove its `#[ignore]`; it runs a PersistentVector through
   all 8 ops. It is your contract.

## STOP triggers
1. If `concat` isn't routed through a `eval_vec_*` fn you can add an arm to (it may be `Vector/concat` only) —
   STOP, report how concat dispatches, do not invent.
2. If any std `Vec` op's behavior would change — STOP (additive arms only).
3. If a floor moves beyond the new probe (+1) — STOP, report.

## Verify (paste verbatim)
```
cargo test --release -p wat --test probe_arc278_0c_persistent_parity -- --include-ignored   # 1/1 GREEN
cargo test --release -p wat --test probe_arc278_0a_persistent_map -- --include-ignored        # still GREEN
cargo test --release -p wat --test probe_arc278_0b_persistent_vector -- --include-ignored      # still GREEN
cargo test --release -p wat --lib 2>&1 | grep "test result"                                   # 931/36 (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                    # 264/1
cargo test --release -p wat --test nursery -- --test-threads=1 | grep "test result"           # ~893/4
cargo build --release 2>&1 | tail -2                                                           # clean
```
Report: the diff (the 8 arms + infer arms), all outputs verbatim, any STOP hit. Do not claim a green you did
not see. Un-ignore the 0c probe. No git.

## Blast radius
`src/collection/transform.rs` (8 PersistentVector arms) · `src/collection/infer.rs` (mirror infer arms) ·
maybe `src/collection/eval.rs`/`src/runtime.rs` if `concat` lives there · the probe. NO std-Vec behavior
change. No git.
