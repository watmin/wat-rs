# SCORE — STONE 255.74: a set element or map key must be data, refused by the checker

**Executor: a Sonnet subagent.** Branch `main`, drawn against `70a157245`, struck at `ec77a35e12b`.
Not pushed.

## 1. Census — every hashing context (item 1)

**Guarded (checker-visible construction, item 2's door) — now wired to `is_atomizable` through
ONE function, `key_eligible_or_error` (`src/check.rs:1708`):**

| site | file:line | type position |
|---|---|---|
| `HashSet` constructor | `src/check.rs:13136` (`infer_hashset_constructor`) | declared element type |
| `HashMap` constructor | `src/check.rs:15685` (`infer_hashmap_constructor`) | declared KEY type only |
| `PersistentMap` constructor | `src/check.rs:15812` (`infer_persistentmap_constructor`) | declared KEY type (bracket-less calls are already illegal, separately) |
| `{}` map literal, bottom-up | `src/check.rs:15943` (`infer_map_literal`) | inferred KEY type |
| `#{}` set literal, bottom-up | `src/check.rs:15988` (`infer_set_literal`) | inferred element type |
| `{}` map literal, expected-type-directed (call-arg/ann-form) | `src/check.rs:16434` (`check_map_literal_against`) | expected KEY type |
| `#{}` set literal, expected-type-directed (call-arg/ann-form) | `src/check.rs:16489` (`check_set_literal_against`) | expected element type — **this is `probe-compound-upcast.wat`'s old Set case** |
| `conj` onto a HashSet | `src/collection/infer.rs:251` (`infer_conj`) | element type, when resolved fresh from the argument (e.g. `(conj #{} x)`) |
| `assoc` onto a HashMap/PersistentMap | `src/collection/infer.rs:485` (`infer_assoc`) | KEY type, when resolved fresh from the argument (e.g. `(assoc {} k v)`) |

`conj`/`assoc` do **not** need the wall when the target collection's element/key type is already
concrete (it was already validated at construction); the wall only fires when the type is still a
free variable the argument is about to bind — the gap the Why section named.

**Guarded (runtime, `value_is_hashable` family, `src/runtime.rs:6791` — unchanged call sites, just
the body) — every caller that inserts into a `HashSet<Value>`/`HashMap<Value,_>` at the WAT
surface:**

`src/collection/eval.rs:205,232,328,406,448,481,667,687,718,748,825,1864,1938,2157`,
`src/runtime.rs:1641,1664`, `src/rete/expr_ir/eval.rs:1252`, `src/rust_deps/cache.rs:125,137`,
`src/edn/render.rs:2342,3519` (EDN read — the path that caught this stone's own first-draft
regression, below).

**Unguarded by either layer, found and left alone (out of this stone's boundary — item 1's
"whatever the code has"):** RETE's internal indexing structures build `HashMap<Value,_>` /
`FxHashSet<Value>` / `FxHashMap<Value,_>` directly from fact field values, never through
`value_is_hashable`: `src/rete/alpha_tree.rs:53,56`, `src/rete/where_tree.rs:42,85`,
`src/rete/compiled_cond.rs:1026`, `src/rete/kernel/fire/delta.rs:196,872`,
`src/rete/kernel/session.rs:166`. These rely on facts never carrying an opaque-handle field
(unverified by this stone); `distinct`/`frequencies`/`group-by` (`wat/seq.wat`, `wat/rete/acc.wat`)
are ordinary `wat`-level library functions built on the guarded `HashSet`/`PersistentMap`
constructors + `conj`, so they inherit this stone's wall/guard without their own wiring.

## 2. The static wall (item 2)

One door, `key_eligible_or_error` (`src/check.rs:1708`), mirrors `to-holon`/`leaf`'s existing
record-subtype carve-out (`:3960-3966`): a Record subtype is atomizable via its `holon_form` even
though `is_atomizable` only knows the exact root names. Wired into the nine sites above. Error
names the container and the offending type, e.g.:

```
:wat::core::HashSet: parameter element type expects key-eligible type (is_atomizable); got [:wat::core::i64 :-> :wat::core::i64]
#{…} set literal: parameter element type expects key-eligible type (is_atomizable); got :wat::capability::Capability
```

Both original probes now refuse (moved to permanent negative fixtures, see §4), and a `Capability`
set element (the compound-upcast bug) refuses the same way.

## 3. The runtime guard, deepened (item 3)

`value_is_hashable` (`src/runtime.rs:6791`) is rewritten to classify every LEAF from
`Value::key_eligibility()` (`src/value/value.rs:1341`) instead of a hand-rolled 13-variant list —
closing the `wat__stream__Stream` omission the Why section measured.

**Load-bearing correction, found by the floor (§5):** `key_eligibility()`'s `NeverAKey` carries a
reason (`NotAKeyReason`), and only `InteriorMutable`/`OpaqueHandle` are an actual `Hash`-panic
risk. `ExcludedByDesign` ("the `Hash` arm is real/structural, but `is_atomizable` does not admit
it" — its own doc, covering `PersistentMap`/`PersistentVector`/`Option`/`Result`/`Enum`/
`ForeignRecord`/`ForeignVariant`/`u8`) is a static-admission policy gap, **not** a safety one —
rejecting it uniformly (my first draft) broke a real, proven-safe capability (a `PersistentMap`
nested as a key inside another `PersistentMap`). The guard now rejects only on
`InteriorMutable`/`OpaqueHandle`, found at ANY depth, and recurses into every variant
`impl Hash for Value` itself recurses into: `Tuple`, `Vec`, `wat__core__List`, `wat__std__HashSet`,
`wat__std__HashMap`, `Aggregate`'s fields, **and now also** `PersistentMap`, `PersistentVector`,
`Option`, `Result`, `Enum`, `ForeignRecord`, `ForeignVariant` — closing a latent gap the *pre-255.74*
guard also had (an `Option` wrapping a fn was never on its old list either; `impl Hash for Value`'s
`Option` arm recurses into the wrapped value the same as `Vector` does).

**Paths this guard defends that the checker cannot see** (item 3's ask): an unresolved generic
`:T` instantiated at runtime to a concrete type the checker only verified abstractly
(`is_atomizable(Var) == true`, conservative), and `:wat::eval-ast!` (`T → Result<T, EvalError>`,
caller-annotated, not re-validated). Driven directly in `src/collection/eval.rs`'s
`arc255_74_deep_runtime_guard` test module via hand-built `Value`s and `hashset_conj_inner` — no
`.wat` source at all, isolating exactly this function.

The rune at `src/value/value.rs:883-896` (`impl Hash for Value`'s non-atomizable cluster) is
corrected: it attributed the dead-arm guarantee solely to the static `is_atomizable` predicate,
which was false even before this stone (it had one caller); now it names BOTH layers and the one
path neither covers (RETE's internal maps, §1).

## 4. The probes (item 4)

- `wat-scripts/probes/arc-255/probe-255.74-a-key-must-be-data.wat.bad` and
  `…-deep.wat.bad` — the two scratch probes, **moved** (not copied) from
  `wat-scripts/scratch-pad/arc-255/`, `.wat.bad` extension (excluded from
  `every_wat_scripts_file_loads`'s `*.wat` glob, matching
  `wat-scripts/fmt/fixtures/spelling-dotted.wat.bad`'s existing convention). Driven by
  `tests/types/probe_arc255_74_key_must_be_data.rs`'s
  `hashset_fn_element_shallow_wat_bad_is_refused` / `hashset_vector_of_fn_deep_wat_bad_is_refused`.
- `wat-scripts/probes/arc-255/probe-255.74-set-of-capability.wat.bad` — new, extracted verbatim
  from `probe-compound-upcast.wat`'s retired Set case. Driven by `set_of_capability_wat_bad_is_refused`.
- A driven runtime test for the deep guard: `src/collection/eval.rs`'s
  `conj_vector_of_fn_into_hashset_is_type_mismatch_not_panic` (the exact deep-probe shape, as a
  `Value`) and `conj_option_of_fn_into_hashset_is_type_mismatch_not_panic` (the §3 correction's own
  regression case, inverted) — both assert `TypeMismatch`, never a panic.
- `probe-compound-upcast.wat` — Set case **removed** (its up-cast of a handle into
  `(HashSet :- [Capability])` is now illegal); Tuple and Map claims kept unchanged. `wat --check`
  and a full run both confirmed rc 0 (manually and via the new driven test
  `tests/process/probe_arc255_74_compound_upcast_runs.rs`, the `probe_arc170_s3b_astsplice.rs`
  shape — this probe had NO driven test before this stone).

## 5. Reds found and cured (none were STOP-1/STOP-2; both are "red caused by this stone's own change")

**Red 1 — `no_inlined_edn`/`no_inlined_wat_in_tests` (lint).** My own first test-file draft
embedded the diagnostic's exact type-expression substrings (`"[:wat::core::i64 :-> …]"`,
`"#{…} set literal"`, a full `"(:wat::core::Vector :- […])"` form) as `needle_count` arguments —
each one is itself EDN-esque or a parseable `(head …)` wat form. Cured by dropping the leading
delimiter character from each needle (still a valid substring of the real diagnostic text, no
longer independently EDN/wat-shaped) — restructuring per both lints' own stated remedy, no rune.

**Red 2 — `value::pmap::tests::a_map_used_as_a_key_is_found_across_arms`.** Verbatim, from
`.floor/2026-10-01T02-10-37Z/ARM.txt`:

```
thread 'value::pmap::tests::a_map_used_as_a_key_is_found_across_arms' (2956628) panicked at src/value/pmap.rs:450:64:
round-trip parse: EdnReadError { span: Span { file: "src/edn/render.rs", line: 3520, col: 49, end: None }, kind: Other("non-hashable PersistentMap key: wat::core::PersistentMap") }
```

My first draft of `value_is_hashable` rejected every `NeverAKey` reason uniformly. Root cause and
cure: §3 above. Fixed, then two new unit tests lock the corrected boundary in
(`conj_option_of_i64_into_hashset_still_succeeds` / `conj_option_of_fn_into_hashset_is_type_mismatch_not_panic`).

**Red 3 — `wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`.**
Verbatim:

```
2 of 798 wat-scripts/ files do not load on the current runtime (rotted):
  wat-scripts/scratch-pad/probe-hashmap-fn-type-bracket.wat
      … ":wat::core::HashMap: parameter key type expects key-eligible type (is_atomizable); got [:wat::core::i64 :-> :wat::core::bool]" …
  wat-scripts/scratch-pad/probe-hashset-fn-type-bracket.wat
      … ":wat::core::HashSet: parameter element type expects key-eligible type (is_atomizable); got [:wat::core::i64 :-> :wat::core::bool]" …
```

Two **pre-existing** scratch probes (unrelated "three-orphans stone" investigation, predating this
draw) that happened to build exactly the construct this stone now refuses. Cured: renamed
`.wat` → `.wat.bad` (the file's own claim was always "does the runtime accept what the checker
accepts" — now moot, since the checker refuses it first; header updated to say so and point at the
promoted fixture).

## 6. Gates

| what | how | result |
|---|---|---|
| the three negative probes | `./target/release/wat --check` | all three refused, naming the type and container (verified manually + via `tests/types/probe_arc255_74_key_must_be_data.rs`, 12 tests) |
| release floor | `scripts/floor.sh` | **`.floor/2026-10-01T02-46-32Z` — `6254 tests run: 6254 passed (26 slow), 24 skipped`** (no `ARM.txt`). 6236 (prior green, `.floor/2026-10-01T01-32-01Z`) + 18 new tests (12 `tests/types`, 1 `tests/process`, 5 `src/collection/eval.rs` unit tests), zero regressions, zero skips added |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 (one `#[allow(clippy::too_many_arguments)]` added to `check_map_literal_against`, matching five sibling `check_*`/`infer_*` fns in the same file that already carry it) |
| census | `scripts/replay/census.sh` pre (`.census/2026-10-01T02-40-17Z.txt`, 2271 files, unmodified draw @ `ec77a35e12b`) vs post (`.census/2026-10-01T02-43-11Z.txt`, 2267 files) `--diff`, with the 4 renamed `.wat.bad` fixtures passed as owned/produced | `census-diff: no STOP-8` (rc 0) — **zero unowned rc flips** across the other 2267 shared tracked `.wat` files. The only files whose check-result changed are exactly the ones this stone touched: the 2 promoted probes + 2 pre-existing scratch probes (all held a non-key Fn-typed HashSet/HashMap element/key, §5 Red 3), plus `probe-compound-upcast.wat` which stays rc-0→rc-0 net (its Set case made it rc-1 mid-stone; removing that case restored rc-0) |

## 7. STOPs

None fired. No real corpus/stdlib site was found keeping a resource in a set/map on purpose
(STOP-1 — the two census'd scratch files were demonstration bugs, not intentional designs, and are
cured in §5/§4); no type `key_eligibility()` classifies `Hashable` was ever refused by the wall
(STOP-2) — the opposite drift (§3/§5 Red 2) was caught and cured instead.

## 8. Doctrine notes

`holon/CLAUDE.md` read in full before starting. These are a handful of named fixture files (two
scratch-probe promotions, one extraction, two pre-existing scratch renames) — edited/renamed
directly, not a corpus-wide codemod, per the brief's own doctrine carve-out. `rc=$?` captured on
every `--check`/census invocation. No `pgrep -f` waits. Every number/file:line in this report was
measured in this session (builds, `--check` runs, the floor, the census diff) — none carried over
unverified.
