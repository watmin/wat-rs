# SCORE — STONE 255.39: a surface parameter is consumed by the edge that binds it

Struck against draw `19afe8e40`. `owner-end?` is gone. `Spawned` stays a
surface. Thread and Process still implement it. The parameter is consumed by
those edges.

## Where the check ran

`UnconsumedTypeParam` is decided by `check_type_params_consumed` in
`src/types.rs`. Before this stone it ran at the end of `parse_type_decl`,
which returns the `TypeDef` before `register`, and before the same walk
reaches a later `extend-type`. `register_generic_edge` is the extend-type
arm, after the declaration. At the check, Spawned's edges do not exist.

The check stays that one function. A surface whose members already consume
every parameter is still judged there, with no edges in hand, so a later
edge cannot change a verdict the members settled. A surface with a parameter
no member uses is registered and remembered. At the end of
`register_types_impl` — one walk, stdlib and user both go through it —
`flush_surface_param_debt` calls the same function with the env. A generic
edge consumes a surface parameter when the edge's target head is that
surface and the argument at that slot is present. The binder's names are
the edge's. The slot is what binds the surface parameter. Records, structs,
and enums never take the env. They fail at the declaration, as before.

That is one rule. The flush does not define consumption. It is the call
that can see the edges. A checker-only census, before `spawn.wat` changed,
is `.census/2026-09-25T20-44-44Z.txt` against the pre census
`.census/2026-09-25T20-41-34Z.txt`: 2280 files, 0 rc flips, 215 nonzero.
No existing file changed verdict.

## The surface

The declarator requires a `:features` clause. The brief's form omits it.
The code wins: the clause stays, and it is empty.

```
(:wat::core::defsurface :wat::spawn::Spawned :- [S R] :nature :wat::core::Struct
  :features [])
(:wat::core::extend-type :- [S R]
  (:wat::kernel::Thread :- [S R])
  (:wat::spawn::Spawned :- [S R]))
(:wat::core::extend-type :- [S R]
  (:wat::kernel::Process :- [S R])
  (:wat::spawn::Spawned :- [S R]))
```

The doc comment says what it is: the owner's end of a spawn. `S` is what
the owner sends, `R` what it receives. Its implementors' edges bind them.
A bodiless `extend-type` is a legal edge (`ops` needs the child and the
target; method forms are the rest). `owner-end?` in `wat/` and `src/` is 0.

## Rows

Pre-stone words are `./target/release/wat --check` on the draw binary,
before this stone's rebuild. `rc` is the next statement. Post words are the
same command after the rule and the deletion.

| row | pre | post |
|---|---|---|
| featureless `(Owner :- [S R])` plus one binder edge | rc=1, `UnconsumedTypeParam`, param `S`, decl `:probe::Owner`. "type parameter \"S\" in :probe::Owner's param-spec is declared but never used" | rc=0 |
| the same surface, no edge | rc=1, the same sentence, param `S` | rc=1, the same sentence, param `S` |
| `defstruct` `Unused :- [T]` with a field that does not use `T` | rc=1, param `T`, decl `:probe::Unused`, the same sentence | rc=1, param `T`, the same sentence |

## Gates

Clippy `--all-targets --workspace -- -D warnings` exited 0.

Census `.census/2026-09-25T20-47-20Z.txt` against
`.census/2026-09-25T20-41-34Z.txt`: `census-diff: no STOP-8`. 0 rc flips
either way. 2280 files, 215 nonzero.

Delta `.delta/2026-09-25T20-48-20Z`: NEW 2 / RECOVERY 0. The two NEW files
are `wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.

Ledger 198. `the_heresy_ledger_matches_its_frozen_census` passed. Nothing
moved.

The first floor, `.floor/2026-09-25T20-49-49Z`, was red. Summary:
`6127 tests run: 6126 passed (11 slow), 1 failed, 22 skipped`. The one
failure is `no_loose_string_assert::tests_carry_no_loose_string_assert`,
panicked at `tests/lint/no_loose_string_assert.rs:135`. The assertion names
two `contains` calls in
`tests/types/probe_arc255_39_an_edge_consumes_its_parameters.rs` (then lines
24 and 28). That floor was not re-run. The row test now matches
`TypeErrorKind::UnconsumedTypeParam` and its `decl` and `param`. The floor
below is a new run.

`.floor/2026-09-25T20-57-23Z`: `6127 tests run: 6127 passed (10 slow), 22 skipped`.
