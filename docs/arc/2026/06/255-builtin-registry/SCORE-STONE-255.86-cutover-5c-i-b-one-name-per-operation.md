# SCORE — STONE 255.86: STOP-1 on `:wat::vector::concat`

Branch `main` @ the brief draw `9711dfbe4` (parent `279edda38`). **Not pushed.** No name moved. No registry row changed. Main `wat/` is untouched.

## The pair

`:wat::vector::concat` and `:wat::core::concat` do not return the same value on the same inputs.

Programs under `/tmp/g1-diff/`, each a `:user::main` that shows the call and then `assertion-failed!` so the shown text is the message. Constructors are the live spelling `wat.type/…`. Binary `./target/release/wat`.

| program | call | result |
|---|---|---|
| `pv-vec-old.wat` | `(:wat::vector::concat (wat.type/PersistentVector :- [wat.type/i64] 1) (wat.type/Vector :- [wat.type/i64] 2))` | RC=2. Message `#pv[1, 2]` |
| `pv-vec-core.wat` | the same two arguments under `:wat::core::concat` | RC=3. Check error: `:wat::core::concat: parameter #2 expects (wat.type/PersistentVector :- [wat.type/i64]); got (wat.type/Vector :- [wat.type/i64])` |
| `pv-pv-old.wat` / `pv-pv-core.wat` | both PersistentVectors, `1` then `2` | both RC=2, message `#pv[1, 2]` |
| `vv-old.wat` (`:wat::vec::concat`) / `vv-core.wat` | both Vectors, `1` then `2` | both RC=2, message `[1, 2]` |
| `hm-old.wat` (`:wat::hashmap::contains-key?`) / `hm-core.wat` (`:wat::core::contains?`) | `(wat.type/HashMap :- [wat.type/String wat.type/i64] "a" 1)` and `"a"` | both RC=2, message `true` |

The mixed pair is the stop. `:wat::vector::concat` produced a PersistentVector `#pv[1, 2]`. `:wat::core::concat` produced no value. `wat/core.wat:44` aliases `:wat::core::concat` to `:wat::vec::concat`. `infer_concat` (`src/collection/infer.rs`) refuses a different container kind. `persistentvector_concat_inner` (`src/collection/eval.rs`) accepts a Vector as its second argument and returns a PersistentVector. Replacing the name would change that result.

`:wat::vector::concat` stays registered. A search outside `docs/` and `target/` found 42 occurrences in 20 files. They were not rewritten.

## What was not done

No mapping table was committed, because the collection mapping that includes this name is not uniform. No codemod ran. No retirement row was added. Census, clippy, and the release floor were not run: the tree is the draw commit, and a floor of unchanged code would not be a measurement of this stone. R-a was not started. The `contains-key?` / `contains?` pair above matched on the one map that was run; that match does not license moving the rest of the table past this stop.
