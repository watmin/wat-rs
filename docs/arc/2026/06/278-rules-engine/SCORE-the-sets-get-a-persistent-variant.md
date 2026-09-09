# SCORE — the sets get a persistent variant

**SCORED.** Executor: grok, 2026-09-09. Did not commit.

```
Summary [ 498.304s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-09T00-55-30Z/`

Two tests added: `persistent_set_edn_round_trip`, `persistentset_conj_disj_leave_original_unchanged`. **5237 = 5235 + 2**. 0 FAIL, 0 TIMEOUT.

First floor `.floor/2026-09-09T00-43-06Z/` was RED — two arms this stone minted, captured whole, then fixed. Not re-run: the green floor is a new run after the fix.

## WHAT LANDED

`:wat::set::` over `rpds::HashTrieSetSync`. Five verbs: `conj` `disj` `contains?` `empty?` `length`. New `Value::wat__core__PersistentSet(Arc<rpds::HashTrieSetSync<Value>>)`. Constructor `(:wat::core::PersistentSet :- [T] …)`. EDN tagged `#wat.core/PersistentSet #{…}`.

`conj` is `(**s).insert(item.clone())` — a new trie sharing structure. No `(**s).clone()` on the `:wat::set::` path. `disj` of an absent element returns the set unchanged.

`:wat::hashset::` is untouched. `RedBlackTreeSetSync` is absent.

## VALUE MATCH SITES

The compiler named four exhaustive `Value` matches. Each got an explicit arm. **No `_ =>` was added to silence any of them.**

| site | what the arm does |
|---|---|
| `src/value/observe.rs` `render_value` | `#ps#{…}` |
| `src/edn/render.rs` `value_to_edn_with` | tagged `#wat.core/PersistentSet #{…}` |
| `src/runtime.rs` `val_type_path` | `":wat::core::PersistentSet"` |
| `src/closure_extract.rs` `encode_value_with_path` | `(:wat::core::PersistentSet :- [T] …)` sorted elems |

Also handled at sites that already had a catch-all (so the compiler did not name them). Arms added *before* the existing wildcard, not by widening one:

- `src/closure_extract.rs` `value_static_type_keyword` — explicit, same as HashSet (the existing `other =>` would have swept it)
- `src/runtime.rs` `values_equal` — needed for `assert-eq` / the EDN probe (`_ => None` would have TypeMismatch'd)
- `src/edn/render.rs` `tagged_to_value` — parse of the tagged form
- `src/runtime.rs` constructor dispatch
- `src/rete/purity.rs` `intrinsic_meta` — first floor named the six unclassified heads

## FIRST FLOOR (captured, not re-run)

```
Summary [ 500.116s] 5237 tests run: 5235 passed (7 slow), 2 failed, 22 skipped
```

`.floor/2026-09-09T00-43-06Z/`

1. **`no_rpds_rebuild_loop::a_persistent_structure_built_in_a_loop_is_a_transient`** — ctor and EDN parse used `set = set.insert(v)`. Shape is the copy-per-element rebuild. Switched to `insert_mut`. `conj` stays `insert` (returns a new set; the original is live).
2. **`rete::purity::completeness_gate::every_dispatched_verb_is_classified_or_disposed`** — six heads with no purity ruling: `:wat::core::PersistentSet` and the five `:wat::set::*` verbs. Classified pure∧deterministic next to HashSet.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ insert shares structure | ✅ `conj` is `HashTrieSetSync::insert`; no `(**s).clone()` on the `:wat::set::` path |
| 2 | ★ five verbs exist and type-check | ✅ probe drives all five; sharing unit test drives them at the inner |
| 3 | ★ EDN round-trip | ✅ tagged write+read; rust test + probe `assert-eq back s12` |
| 4 | `:wat::hashset::` untouched | ✅ `hashset.rs` absent from the diff; cloning `conj` unchanged |
| 5 | unmarked name is persistent | ✅ `:wat::set::` is `HashTrieSetSync` |
| 6 | no ordered-set surface | ✅ `RedBlackTreeSetSync` absent |
| 7 | variant handled, no silencing `_ =>` | ✅ four compiler-named sites each have an explicit arm |
| 8 | corpus loads | ✅ `every_wat_scripts_file_loads` PASS (the new probe included) |
| 9 | floor | ✅ **5237** passed, 22 skipped, 0 FAIL, 0 TIMEOUT |

## BLAST

`src/intrinsic/set.rs` (new), `src/intrinsic/mod.rs`, `src/value/value.rs`, `src/collection/eval.rs`, `src/check.rs`, `src/runtime.rs`, `src/edn/render.rs`, `src/closure_extract.rs`, `src/value/observe.rs`, `src/rete/purity.rs`, `wat-scripts/scratch-pad/probe-the-sets-get-a-persistent-variant.wat`.
