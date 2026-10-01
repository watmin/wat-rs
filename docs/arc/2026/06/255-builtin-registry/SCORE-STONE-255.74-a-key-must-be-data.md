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

---

# AMEND — D3: the key door is `Equatable`, not `is_atomizable`

Continues at local commit `3e214a67e` (not pushed), per
`AMEND-STONE-255.74-the-key-door-is-equatable.md` (committed `8f30e7b77`). The builder's ruling:
the brief above named `is_atomizable` as the key door; that predicate answers *"can be encoded
as a holon atom"*, a different property, and it refused real data (measured on `3e214a67e`: a
map key of `u8`, `bigint`, `rational`, `Instant`, `(Option :- [i64])`, `(PersistentVector :- [i64])`
was refused by `--check` while the runtime hashed every one of them). The door is now
`:< :wat::core::Equatable` (`wat/class.wat`), asked via `require_class` — the same predicate `=`
already asks. **The nine sites and the deep runtime guard from the base stone are unchanged** —
only the door's own predicate, and `key_eligibility()`'s per-variant table, move.

## 1. The swap (item 1)

`key_eligible_or_error` (`src/check.rs:1708`) now calls `require_class(container, span, ty,
":wat::core::Equatable", env)` instead of `is_atomizable(ty)` + the hand-rolled record carve-out
(`require_class` already carries record membership via `wat/class.wat`'s `Record` edge, Q1 — no
separate carve-out needed). `is_atomizable` is untouched and stays `to-holon`/`leaf`'s own door.

## 2. Type variables (item 2)

**Declared generic type parameter** (a bare name like `T`): no new code needed.
`require_class` -> `assignable` already consults `env.bound_of("T")` (`src/check.rs:17455` area,
Stone 255.51), so `[T :< Equatable]` passes immediately and an unbounded `T` is refused
immediately, by name (`got ":T"`).

**Fresh inference variable** (an empty `#{}`/`{}` literal, or a `conj`/`assoc` target still
unbound at the point the wall runs): `key_eligible_or_error` now checks `if let
TypeExpr::Var(id) = ty` FIRST and, when true, calls `env.record_pending_bound(PendingBound { var,
letter: position, bound: Equatable, span })` and returns `None` (no immediate error) — the
existing `[T :< X]` pending-bound machinery (`enforce_type_bounds`/`flush_pending_bounds`, Stone
255.51-53, already called once per function body and once per top-level form) resolves it once
the var is bound by something later in the same scope, and refuses it (`BoundUnresolved`, naming
the function) only if it is STILL unresolved at the end — the ruling's own "Refuse" word, verified
directly:

```
(:wat::core::let [s (:wat::core::conj #{} 1)] …)                    → checks clean (resolves to i64)
(:wat::core::let [s (:wat::core::conj #{} :probe::inc)] …)          → refused (BoundNotSatisfied, Fn)
(:wat::core::let [s #{}] "unused") …                                → refused (BoundUnresolved)
```

**Corpus/stdlib sites needing the bound (item 2 bullet 1) — found: 2, both in `wat/seq.wat`,
well under STOP-1's 15:**

| site | fix |
|---|---|
| `wat/seq.wat:580` `:wat::core::distinct-walk :- [T]` | `:- [[T :< :wat::core::Equatable]]` |
| `wat/seq.wat:591` `:wat::core::distinct :- [T]` | `:- [[T :< :wat::core::Equatable]]` |

Found by measuring directly: even a trivial one-line user program failed `--check` pre-fix, both
errors located in these two stdlib functions (`distinct-walk`'s `conj seen value` and
`distinct`'s own `(HashSet :- [:T])` construction) — neither is a corpus file "deliberately
instantiating a non-Equatable type" (STOP-1's other trigger); `distinct`'s whole point is
deduplication via equality, so `Equatable` is the natural bound, not a deliberate violation.
`every_wat_scripts_file_loads` (all 798 `wat-scripts/` files, which also load the full stdlib)
confirmed green after the two-line fix — no third site found.

## 3. Tying the two layers (item 3)

**The gate.** `check::tests::every_key_eligibility_row_agrees_with_equatable` (`src/check.rs`,
replacing — not alongside — the retired is_atomizable-keyed gate below) iterates
`Value::all_key_eligibility()`'s 46+ rows against `require_class(…, Equatable, …)` on a
STDLIB-LOADED `CheckEnv` (`wat/class.wat`'s `extend-type` edges only exist once stdlib is
loaded — a bare `TypeEnv::new()`, as the pre-existing `arc109_two_iii_…` unit tests used, has
NONE of them registered): `Hashable` rows must be Equatable, `NeverAKey` rows must not.
**Mutation-proved once** (per the brief's instruction), by hand: flipped `u8`'s row to
`NeverAKey(ExcludedByDesign)` (its REAL classification was already `Hashable`, as reclassified
below), ran the gate, captured the red —

```
Path(":wat::core::u8") is declared NeverAKey(ExcludedByDesign) but IS `:< :wat::core::Equatable`
— the static door would admit a type the runtime guard hard-rejects (InteriorMutable/
OpaqueHandle) or never recurses into expecting it to be excluded (ExcludedByDesign); reclassify
the row or add the missing `wat/class.wat` edge, whichever is true
```

— then reverted and reran green (`cargo test --release --lib
every_key_eligibility_row_agrees_with_equatable`).

**Disagreements found (measured via a throwaway diagnostic test, run once, then deleted) —
10, all in the SAME direction (Rust table says `NeverAKey`, `wat/class.wat` already says
`:< Equatable`) and all cured by RECLASSIFYING the Rust row, never by adding a `class.wat` edge
(every edge these ten needed already existed):**

| row (`src/value/value.rs`) | was | now | gate probe change |
|---|---|---|---|
| `u8` | `NeverAKey(ExcludedByDesign)` | `Hashable` | none (leaf) |
| `wat__core__PersistentVector` | `NeverAKey(ExcludedByDesign)` | `Hashable` | none (already parametric) |
| `Option` | `NeverAKey(ExcludedByDesign)` | `Hashable` | bare `Path` → parametric `(Option :- [i64])` (the bare path has no unconditional edge; only the applied conditional edge does) |
| `Result` | `NeverAKey(ExcludedByDesign)` | `Hashable` | bare `Path` → parametric `(Result :- [i64 i64])`, same reason |
| `wat__core__List` | `NeverAKey(ExcludedByDesign)` | `Hashable` | bare `Path` → parametric `(List :- [i64])`, same reason |
| `Vector` (`:wat::holon::Vector`, the VSA byte vector) | `NeverAKey(ExcludedByDesign)` | `Hashable` | none (leaf) |
| `Instant` | `NeverAKey(ExcludedByDesign)` | `Hashable` | none (leaf) |
| `Duration` | `NeverAKey(ExcludedByDesign)` | `Hashable` | none (leaf) |
| `wat__core__Rational` | `NeverAKey(ExcludedByDesign)` | `Hashable` | none (leaf) |
| `wat__core__BigInt` | `NeverAKey(ExcludedByDesign)` | `Hashable` | none (leaf) |

None of these ten needed a code change to `value_is_hashable` itself: its rejection criterion was
already narrowed (base-stone §3's own correction) to `NotAKeyReason::InteriorMutable`/
`OpaqueHandle` only, so these ten were ALREADY runtime-admitted; only the Rust table's LABEL (and,
for three of them, the gate's representative probe shape) was stale.

**Confirmed NOT a disagreement, no change (`NeverAKey(ExcludedByDesign)` stays, verified against
BOTH the bare-path gate probe AND, for `PersistentMap`, a full parametric probe):**

| row | why it correctly stays `NeverAKey` |
|---|---|
| `wat__core__PersistentMap` | `wat/class.wat`'s own comment: *"PersistentMap is not a member"* — confirmed even with a fully-applied `(PersistentMap :- [i64 i64])` probe, still not Equatable. The one `ExcludedByDesign` row that stays `NeverAKey` under EITHER door. |
| `:wat::core::Struct` | never a class member (mixes code+data) |
| `Enum` | only a `:wat::enum::Pure`-declared enum is `:< Equatable`; this row's bare-path probe is the generic/non-Pure representative. The checker sees a SPECIFIC enum's registered type at a real call site and admits a Pure one via its own edge (verified: `tests/types/probe_arc255_74_key_must_be_data__acceptance.wat`'s `(HashSet :- [:probe::Color])` for a `:wat::enum::Pure`-declared enum checks and runs clean); `key_eligibility()` has no per-instance purity to read from a bare `Value::Enum`, so it stays conservative for the variant as a whole — not a STOP-2 case (not "a type the two layers disagree on": the checker and the runtime guard agree exactly once purity is known: a Pure enum is accepted by both, an impure one's safety still depends on its fields, which `value_is_hashable` already recurses into). |
| `ForeignRecord` / `ForeignVariant` | dynamic/foreign, never registered against any class |

**STOP-2 did not fire.** No disagreement needed a genuinely new `wat/class.wat` edge, and none
was a type the two layers disagree on outside the "stale Rust label" shape above.

`src/runtime.rs:6750-6805`'s `value_is_hashable` doc comment and `src/value/value.rs`'s
`NotAKeyReason::ExcludedByDesign` doc comment are both rewritten to name `Equatable` (not
`is_atomizable`) as the static door and list the surviving `ExcludedByDesign` members precisely
(`PersistentMap`/`Struct`/`Enum`/`ForeignRecord`/`ForeignVariant` only). The base stone's own
`check::tests::every_interior_mutable_variant_is_rejected_as_a_key` (which asserted FULL agreement
with `is_atomizable`, including the now-false "Hashable ⇒ is_atomizable accepts" direction — driven
red the moment D3 landed, by exactly the ten reclassified rows) is narrowed to
`is_atomizable_still_rejects_every_interior_mutable_or_opaque_handle_variant`: the HALF of the
original claim that is still true and still matters (`is_atomizable` must never accept an
`InteriorMutable`/`OpaqueHandle` variant, or `to-holon` would try to encode it as a holon atom) —
asserted as its own, narrower gate.

## 4. Tests (item 4)

- `tests/types/probe_arc255_74_key_must_be_data__acceptance.wat`: extended with `u8`, `bigint`,
  `rational`, `Instant`, `(Option :- [i64])`, `(PersistentVector :- [i64])`, a `:wat::enum::Pure`
  enum, and `f64` — every type D3's own measurement named as newly-admitted, as both a HashSet
  element and (where applicable) a HashMap key — alongside the originally-admitted i64/String/
  keyword/record/vector/tuple rows. Checks AND runs clean (verified).
- `generic_bounded.wat` / `generic_unbounded.wat` (new): the same one-line generic function
  (`(defn singleton :- [T …] [x <- :T] -> (HashSet :- [:T]) …)`), with and without
  `[T :< :wat::core::Equatable]`. Bounded checks clean; unbounded is refused naming `T` (`got
  ":T"`, not a resolved type — proving this is the declared-parameter path, not the
  fresh-variable path). Two new tests drive both.
- All pre-existing refusal rows (fn, `Capability`, vector of fns) re-verified red-for-the-right-
  reason: the diagnostic text changed (`expects :wat::core::Equatable`, and for the
  `Vector :- [T :< Equatable]` conditional-edge case, a `MembershipBound` naming `T` instead of a
  plain `TypeMismatch`) but every row still refuses, still names the offending type. The shared
  `WALL_MESSAGE` constant and one test's exact-shape assertion were updated to match; no row's
  PASS/FAIL verdict changed.

## 5. Reds found and cured (none were STOP-1/STOP-2)

Floor run 1 (`.floor/2026-10-01T04-49-35Z`) — **28 failures**, all caused by this amendment's own
change, in four shapes:

1. **3 pre-existing in-crate unit tests** (`check::arc109_two_iii_check_time_ctor_guard_widening::
   row1_infer_hashset_constructor_*`, `row3_…canonical_keyword_first_arg_works`) called
   `infer_hashset_constructor` against a bare `TypeEnv::new()` — `is_atomizable` needed no
   registry; `require_class`/Equatable does, so even `:wat::core::i64` failed. Cured: the shared
   `env_and_types()` helper now clones the real stdlib snapshot
   (`crate::freeze::env::stdlib_snapshot()`), harmless for the sibling `infer_list_constructor`
   rows (never walled).
2. **20 probes across 3 shared `.wat` fixtures** (`tests/collection/
   probe_arc215_collection_literal_inference.wat`, `probe_arc216_stone1_hashset_roundtrip.wat`,
   `probe_brace_map_literal.wat`) — ALL failed together each time, because `call_beside_value`
   freezes the WHOLE fixture file once and shares it: ONE genuinely-unresolved empty `{}`/`#{}`
   literal (used only via `length`/`to-holon`, which propagate no element-type constraint) poisons
   every other probe in the same file. Verbatim (one of three, same shape all three times):
   ```
   call_beside_value: fixture beside ".../probe_arc215_collection_literal_inference.rs" failed to freeze: #wat.check/CheckErrors {:message "2 type-check errors" … #wat.check/BoundUnresolved {:message ":t::p6-empty-map-len: type parameter key type bounded by :wat::core::Equatable is still unresolved" … } #wat.check/BoundUnresolved {:message ":t::p7-empty-set-len: type parameter element type bounded by :wat::core::Equatable is still unresolved" …}]}
   ```
   Cured: pinned each offending empty literal's type with `:wat::core::ann-form` (`{}`/`#{}`
   syntax kept — only the TYPE is pinned, not the literal form each probe exists to exercise) —
   3 sites total (`probe_arc215_collection_literal_inference.wat` p6/p7,
   `probe_arc216_stone1_hashset_roundtrip.wat` p3, `probe_brace_map_literal.wat` p1).
3. **`probe_stone255_71_the_wall::exactly_fourteen_errors_one_per_bracketless_call`** — `left: 16,
   right: 14`. Root cause: `infer_hashmap_constructor`'s bracket-less fallback
   (`(None, _) => … (fresh.fresh(), fresh.fresh(), &args[2..])`) drops BOTH leading args as if
   they were the missing bracket slots — for a 2-arg bracket-less call (`(HashMap 1 "a")`) that
   leaves ZERO pairs to unify, so `k_ty` stays an unresolved var my wall then registered a pending
   bound for, duplicating the already-reported `MalformedForm` with a redundant
   `BoundUnresolved` (×2, the keyword+symbol spelling rows). Cured: both `infer_hashmap_constructor`
   and `infer_hashset_constructor` now only run the key-door wall when a REAL `:- […]` bracket was
   present (`bracket_declared`, mirroring the base stone's existing `declared.is_some()` guard on
   `infer_persistentmap_constructor`) — a bracket-less call already gets its own located
   `MalformedForm`; a second, redundant diagnostic for the same broken call adds noise, not
   information.
4. (From the base-stone work, confirmed NOT recurring here: no PersistentMap-as-key regression,
   no lint false-positives — both already covered by the base stone's own fixes.)

Floor run 2 (`.floor/2026-10-01T05-09-04Z`) — **green**:
`6257 tests run: 6257 passed (25 slow), 24 skipped`, no `ARM.txt`.

## 6. Gates

| what | how | result |
|---|---|---|
| release floor | `scripts/floor.sh` | **`.floor/2026-10-01T05-09-04Z` — `6257 tests run: 6257 passed (25 slow), 24 skipped`** (no `ARM.txt`). Baseline was `.floor/2026-10-01T02-46-32Z` (6254/6254, `3e214a67e`); +2 new tests (`generic_bounded`/`generic_unbounded`), net `.rs` test-count otherwise unchanged (fixture edits, not new `#[test]` fns), zero regressions |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census | `scripts/replay/census.sh --diff .census/2026-10-01T02-43-11Z.txt .census/2026-10-01T05-15-55Z.txt` (pre = the base stone's own post-census, at `3e214a67e`; post = current, `2276` tracked `.wat` files vs `2267` — the +9 are this stone's own `tests/types/probe_arc255_74_key_must_be_data__*.wat` fixtures, untracked at the moment the base stone's post-census ran, committed since) | **`census-diff: no STOP-8`** — zero unowned rc flips. Direct `join` over every path present in BOTH census files: **zero rc differences at all**, either direction — every file this amendment touched (`wat/seq.wat`, the 3 pinned test fixtures) was ALREADY fixed before this gate ran, so its own standalone `--check` rc never actually moved across the amendment's net effect |

## 7. STOPs

Neither fired.
- **STOP-1** (>15 generic declarations needing the bound, or a deliberate non-Equatable
  instantiation): 2 found (`wat/seq.wat`'s `distinct-walk`/`distinct`), both genuinely needing
  `Equatable` (deduplication requires equality), well under 15.
- **STOP-2** (a layer disagreement that isn't a plain missing `class.wat` edge): every disagreement
  found (§3) was a stale Rust-side `key_eligibility()` LABEL, cured by reclassification; the one
  type needing a closer look (a Pure enum) turned out to already agree once instance-level purity
  is available (verified end-to-end via the acceptance fixture), not a genuine three-way
  disagreement.

## 8. Doctrine notes

`holon/CLAUDE.md` re-read before resuming. Every number/file:line/diagnostic in this amendment
section was measured in this session (two full floor runs, `--check` on hand-built throwaway
fixtures before committing any corpus/test-fixture edit, the mutation-proof, the census diff) —
none carried over unverified from the base stone's own report.
