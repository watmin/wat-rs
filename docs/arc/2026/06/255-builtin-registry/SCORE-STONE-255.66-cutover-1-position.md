# SCORE — STONE 255.66: position decides

Strike. Drawn against `99a2ca905` (ancestor of the draw `3b59494e5`). No
worktree. The old constructor keys stay registered. Nothing here converts
the stdlib or turns a wall on.

## The rule that is true now

`(wat.type/Vector :- [wat.type/i64])` in a parameter slot is the type. The
same form in value position is the empty vector. With elements,
`(wat.type/Vector :- [wat.type/i64] 1 2 3)` equals `[1 2 3]`, and equals
the old head. HashMap, HashSet, PersistentVector, and Tuple with elements
do the same. PersistentMap is not Equatable; the two spellings agree on
`:wat::map::get`. List's constructor does not peel a type bracket
(`infer_linked_list_constructor`), so a value is `(wat.type/List 1 2)` and
the type is the annotation `(xs :- (wat.type/List :- [wat.type/i64]))`.

One helper, `constructor_head_key` (`src/types.rs:141`), sits on
`type_denotation`. If that denotes to `:wat::core::{Vector,HashMap,HashSet,
PersistentVector,PersistentMap,List,Tuple}`, the call uses that key.
Otherwise the head is unchanged. No second `#[wat_intrinsic]`.
`parse_type_form` still stores the head it read, so `wat.type/Bogus` stays
`Bogus`.

`wat.type/AST` denotes `:wat::WatAST` (`src/edn/render.rs:3699`), before
the general `wat.type/` → `wat.core/` strip. Instant and Duration stay
`wat.time/`.

## The 534

The printed conj error was `infer_conj`'s miss arm
(`src/collection/infer.rs`, the `None` branch of
`StreamContainer::of_type`, expected text `(Vector :- [T]), (HashSet :-
[T]), …`). `of_type` (`src/collection/seq_container.rs`) compared the
stored head to the string `wat::core::Vector`. `format_type` had already
denoted the head, so the message showed `(:wat::core::Vector :-
[:wat::WatAST])` for a head the match had not accepted. `select`'s first
gate was the same compare in `infer_select_prime` (`src/check.rs:12283`).
`infer_list` only armed `:wat::core::Vector`; the scrutinee is now
`constructor_head_key` (`src/check.rs:2740`).

The scratch file `wat-scripts/scratch-pad/probe-arc255-66-the-534.wat`
checks: conj of an empty `(wat.type/Vector :- [wat.type/AST])`, nth,
foldl, mapv, and select over an empty vector of `Peer`. `--check` rc 0.
The first select attempt failed only as an outcome-wall discard, after
the vector head and the Peer element had been accepted.

## Where a head check now asks the door

Dispatch, before the existing arm or `registry().lookup`:

| site | what |
|---|---|
| `src/check.rs:2740` `infer_list` | match scrutinee |
| `src/runtime.rs` `dispatch_keyword_head` and `dispatch_keyword_head_value` | shadow `head` before lookup |
| `src/rete/expr_ir/eval.rs` `OpExec::of` | same, so VecNew / List / Tuple / PersistentMap hit the old arms |
| `src/macros/eval.rs` `validate_pure_total` | if the denoted key differs, it is inserted at the front of the allow-list. A macro body may construct `(wat.type/Vector :- [wat.type/i64])` |

Exact `head == "wat::core::…"` constructor compares now call
`parametric_heads_unify` (already the unify door). The functions:

- `src/check.rs`: `infer_list` (List, HashMap, Vector rest), `infer_select_prime`, `infer_poll_prime`, `infer_holon_bundle`, `vector_elem_of`, `map_kv_of`, `set_elem_of`. `is_atomizable` goes through `constructor_head_key`.
- `src/collection/infer.rs`: `infer_contains`, `infer_get`, `extract_lazyable_elem`
- `src/collection/seq_container.rs` `of_type` (parametric heads, and path forms via `type_denotation`, including `:wat::WatAST`)
- `src/collection/map_container.rs` `of_type`
- `src/declare/parse.rs` `try_parse_user_variadic_def_fn_form` (`wat::core::Vec` stays a raw compare)
- `src/function/eval.rs` `select_defclause_clause`
- `src/macros/parse.rs` `is_watast` / `is_watast_vec`
- `src/runtime.rs` `conforms_check`
- `src/lower.rs` `lower_bundle`
- `src/holon/ast.rs` `is_holon_arg_canonical`

No new `"wat::type::Vector"` literal in `src`. The remaining ones are the
comments and `is_known_type` rows that were already there.

## A denoted Tuple with no values stays a type form

`infer_tuple_constructor` rejects `(Tuple :- [A B])` with no values
(expected 2, got 0). That is the old key's answer, and it is still the
answer for `:wat::core::Tuple`. The two `keyword/to-type-form` goldens are
that shape in the new spelling and nothing else:

- `tests/resolve/probe_arc251_keyword_to_type_form__contract-06-tuple.wat`
- `tests/resolve/probe_arc251_keyword_to_type_form__contract-08-nested-tuple.wat`

They are rendered type forms (`probe_arc251_keyword_to_type_form.rs`
compares the string). Routing them into the constructor flipped both
census rows 0→1. The Tuple arm now skips that shape when the head was
denoted (`src/check.rs:3408`): a type bracket and no values falls through
to the known-type accept. `(Tuple :- [])` and a Tuple with values still
construct, both spellings. An old-spelling bracket with types and no
values still reports expected 2, got 0. After that guard the census diff
is 0 flips.

## Tests

`tests/types/probe_arc255_66_position.wat`, driven by
`tests/types/probe_arc255_66_position.rs` (`--test types`, filter
`probe_arc255_66`):

- both spellings equal `[1 2 3]`
- both spellings build an empty vector (lengths equal)
- a parameter `(wat.type/Vector :- [wat.type/i64])` accepts the new head (yields 7)
- conj then nth yields 4; foldl yields 3; mapv equals `[1 2]`
- HashMap, HashSet, PersistentVector, List, Tuple, PersistentMap agree across spellings
- a `defmacro` body may mention `(wat.type/Vector :- [wat.type/i64])` and expands to 1

## Gates

The first floor, `.floor/2026-09-28T02-54-50Z`, was red and was not
re-run:

```
Summary [ 365.417s] 6200 tests run: 6198 passed (19 slow), 2 failed, 24 skipped
```

exit 100. The arms were `one_variant_separator` (`src/edn/render.rs`
`return format!(":wat::core::{tail}")`, the AST branch had moved the rune
off that line) and `keyword_heresy_ledger` (195 → 149). Both are the
door's lint accounting. The rune sits on the format line again. The frozen
ledger is 149: eight functions shrank, twelve rows are gone (`infer_list`
6→2, `is_atomizable` 2→1, `extract_lazyable_elem` 5→2, `infer_contains`
7→1, `infer_get` 7→1, `seq_container::of_type` 10→2,
`try_parse_user_variadic_def_fn_form` 2→1, `conforms_check` 4→2; gone:
`infer_holon_bundle`, `infer_poll_prime`, `infer_select_prime`,
`map_kv_of`, `set_elem_of`, `vector_elem_of`, `map_container::of_type`,
`select_defclause_clause`, `lower_bundle`, `is_watast_vec`,
`dispatch_keyword_head`, `dispatch_keyword_head_value`). Not STOP-2: no
old program was newly admitted, and the red was not outside this stone's
rows as a checker rejection.

Landing floor `.floor/2026-09-28T03-23-54Z`:

```
Summary [ 364.607s] 6200 tests run: 6200 passed (13 slow), 24 skipped
```

exit 0. 6193 at `4bcf0dfbd`, plus these 7 rows.

`cargo clippy --release --all-targets -- -D warnings` rc 0.

Census: pre `.census/2026-09-28T02-45-50Z.txt` (draw `3b59494e5`, 2296
files). Post `.census/2026-09-28T03-21-52Z.txt`, 2296 files, `--diff` rc
0, 0 flips, nonzero 218 → 218. The two new `.wat` files were still
untracked, so `git ls-files` omitted them; each `--check` was rc 0, and
`every_wat_scripts_file_loads` passed with the scratch file on disk.

Delta `.delta/2026-09-28T03-22-48Z`: ORIG-CLEAN 160/179, CONV-CLEAN
158/179, NEW 2, RECOVERY 0. The NEW files are
`wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.

STOP-1 did not fire: the 534 was the spelling compare. STOP-2 did not
fire.
