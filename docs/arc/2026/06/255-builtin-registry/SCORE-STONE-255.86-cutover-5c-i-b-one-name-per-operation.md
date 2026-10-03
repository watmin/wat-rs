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

## Amend: `into` covers the pairs `:wat::vector::concat` accepts

Continues at `6c48ae7d8`. Pre-edit census `.census/2026-10-03T02-19-03Z.txt`, files=2284, RC=0.

`:wat::vector::concat` accepts a PersistentVector `to` and a `from` that is a Vector or a PersistentVector. A List `from` is a check error (`parameter #2 expects (Vector :- [T]) or (PersistentVector :- [T])`). Before this amend, `(:wat::core::into …)` on PersistentVector×PersistentVector was `NoMatchingClauseAtCallSite` (five clauses, none of them that pair). PersistentVector×Vector already matched and showed the same value as `:wat::vector::concat`.

`wat/seq.wat` gained one clause, body `(:wat::vector::concat to from)`:

```
([to <- (wat.type/PersistentVector :- [T]) from <- (wat.type/PersistentVector :- [T])] -> (wat.type/PersistentVector :- [T])
  (:wat::vector::concat to from))
```

Release rebuild RC=0. Shown values after that clause, same programs as the section above (`assertion-failed!` message is `:wat::core::show` of the call), binary `./target/release/wat`:

| pair | `:wat::vector::concat` | `:wat::core::into` |
|---|---|---|
| PV `[1]` × Vector `[2]` | `#pv[1, 2]` RC=2 | `#pv[1, 2]` RC=2 |
| PV `[1]` × PV `[2]` | `#pv[1, 2]` RC=2 | `#pv[1, 2]` RC=2 |
| empty PV × PV `[2]` | `#pv[2]` RC=2 | `#pv[2]` RC=2 |
| PV `[1]` × empty PV | `#pv[1]` RC=2 | `#pv[1]` RC=2 |
| PV `[1]` × empty Vector | `#pv[1]` RC=2 | `#pv[1]` RC=2 |
| PV `[1 2]` × Vector `[3 4]` | `#pv[1, 2, 3, 4]` RC=2 | `#pv[1, 2, 3, 4]` RC=2 |
| PV `[1 2]` × PV `[3 4]` | `#pv[1, 2, 3, 4]` RC=2 | `#pv[1, 2, 3, 4]` RC=2 |
| PV `["a"]` × Vector `["b"]` | `#pv["a", "b"]` RC=2 | `#pv["a", "b"]` RC=2 |

`:wat::vector::concat` maps to `:wat::core::into` on every pair it accepts that was run. The name was not retired: `into`'s clause still calls it, and the rest of the table is not uniform.

## STOP-1 on `:wat::map::dissoc`

Sixty-nine programs under `/tmp/g1-diff2/`, each binding the old call and the proposed replacement and returning nil when `(:wat::core::= a b)`. RC=0 on 63. The six RC=3 are all `:wat::map::*` on `(wat.type/PersistentMap :- [wat.type/String wat.type/i64] "a" 1)`.

`:wat::map::assoc` of that map with `"b" 2`, and the overwrite of `"a"` to `9`, both type-check under `:wat::core::assoc`. `=` cannot compare them (`parameter #1 expects :wat::core::Equatable; got (wat.type/PersistentMap :- [wat.type/String wat.type/i64])`). Shown text matches: `#pm{"a": 1, "b": 2}` and `#pm{"a": 9}`.

`:wat::map::dissoc` does not.

| program | call | result |
|---|---|---|
| `/tmp/g1-pm/dissoc-old.wat` | `(:wat::map::dissoc (wat.type/PersistentMap :- [wat.type/String wat.type/i64] "a" 1) "a")` | RC=2. Message `#pm{}` |
| `/tmp/g1-pm/dissoc-miss-old.wat` | the same map, key `"z"` | RC=2. Message `#pm{"a": 1}` |
| `/tmp/g1-pm/dissoc-core.wat` / `dissoc-miss-core.wat` | the same arguments under `:wat::core::dissoc` | RC=3. `:wat::core::dissoc: parameter #1 expects (wat.type/HashMap :- [_ _]); got (wat.type/PersistentMap :- [wat.type/String wat.type/i64])` |

`wat/core.wat:41` aliases `:wat::core::dissoc` to `:wat::hashmap::dissoc`. Replacing the name would change that result. `:wat::map::dissoc` stays registered.

The same check refusal, measured in the same pass, hits the next two map verbs. `:wat::map::keys` of that map shows `["a"]` (RC=2); `:wat::core::keys` is RC=3, `parameter #1 expects (wat.type/HashMap :- [_ _])`. `:wat::map::values` shows `[1]` (RC=2); `:wat::core::values` is the same HashMap expectation (RC=3). Those aliases are `wat/core.wat:42` and `:43`. They are the same stop, not a license to keep going.

`:wat::map::{length,empty?,contains-key?,get}` on that map compared equal to `:wat::core::{length,empty?,contains?,get}` (hit and miss, empty and non-empty) under `=`. `:wat::hashmap::*` (all eight), `:wat::vec::*` including `extend` against `into` on Vector×Vector and Vector×PersistentVector, `:wat::vector::*` including `concat` against `into`, `:wat::hashset::*`, and `:wat::linkedlist::*` compared equal on the values in `/tmp/g1-diff2/results.txt`. Those matches do not license a codemod past this stop.

No mapping EDN was committed. No call site moved. No retirement row was added. R-a was not started. Clippy and the release floor were not run.
