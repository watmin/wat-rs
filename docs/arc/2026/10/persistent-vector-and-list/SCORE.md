# SCORE — persistent Vector and List

Clone: `/var/tmp/wat-rs-009` (CARGO_TARGET_DIR=/var/tmp/wat-rs-009-target).
Branch: `the-little-wat-persistent` off `origin/the-little-wat` @ 4f6ebcf12.
GitHub remote `github` pushed after each green commit.

## Baseline

- `cargo build --release` on unmodified `the-little-wat-persistent` @ 4f6ebcf12: green, 4m31s.
  Binary saved at `/var/tmp/009-wat-baseline` for benchmark comparison.
- Floor (`NEXTEST_TEST_THREADS=4 cargo nextest run --release`), clean tree:
  **5405 tests run: 5403 passed (4 slow), 2 failed, 22 skipped** (1616s). The 2 failures are
  exactly the known lint reds named in the brief:
  `wat::lint no_inlined_wat_in_tests::tests_carry_no_inlined_wat`,
  `wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert`.
  Matches the brief's recorded baseline exactly. CONFIRMED.
- Gotcha hit and fixed: a first attempt at this baseline nextest run was started while
  `src/value/pvec.rs` / `src/value/value.rs` were already edited for Row V (background wait +
  foreground edits raced). nextest's own build step picked up the modified source mid-run and
  failed with `PVec` compile errors unrelated to a real baseline. Stashed the edits
  (`git stash push -u`, which also stashed this untracked `docs/arc/2026/10/` — recovered with
  `git stash pop` after), reran nextest on a verified-clean tree (`git status --short` empty)
  to get the number above, then popped the stash back. Lesson recorded here for future
  executors on this clone: never run a gate concurrently with edits to the same tree.

## Row V — Vector -> PVec

Status: DONE.

`Value::Vec(Arc<Vec<Value>>)` -> `Value::Vec(crate::value::pvec::PVec)` (`src/value/value.rs:56`).
The perf fix itself, `vector_conj_inner` (`src/collection/eval.rs:280`):

```rust
// before
Value::Vec(xs) => {
    let mut out = (**xs).clone();
    out.push(item.clone());
    Ok(Value::Vec(Arc::new(out)))
}
// after
Value::Vec(xs) => Ok(Value::Vec(xs.push_back(item.clone()))),
```

`PVec::push_back` is the promoting vector already backing `:wat::core::PersistentVector`
(`src/value/pvec.rs`) — Array (any length) until 8, RRB tree (`rpds::VectorSync`) past that, so
wat-level `conj` is O(log n) amortized once promoted, not O(n) per conj.

Added to `src/value/pvec.rs` (committed as part of Row V): `last()`, `to_vec()` (materialize an
owned contiguous `Vec<Value>` — O(n) on Tree, one clone on Array), `as_cow_slice()` (borrow when
Array, materialize when Tree — unused by the final site list below but kept as the documented
escape hatch), `into_values()` (consume — reuses the buffer in place if sole owner, else clones),
and `impl Index<usize> for PVec` (panics like `Vec` on OOB, for sites that did `xs[i]`).

38 files touched (grep-counted before: 307 occurrences / 61 files — many of those were doc
comments / unrelated `Vec<Value>` mentions, not `Value::Vec` construct/match sites; the real
site count was smaller, matching the brief's ~238/~43 estimate reasonably well: 38 files ended
up with real Rust changes).

**Mechanical (84 sites, ~31 files):** `Value::Vec(Arc::new(EXPR))` ->
`Value::Vec(crate::value::pvec::PVec::from_vec(EXPR))` by `sed`, text-identical semantics —
`PVec::from_vec` is the same "Array, any length, no promotion at bulk build" contract the old
`Arc::new` constructor had. A few sites spelled it `Value::Vec(std::sync::Arc::new(...))`
(fully-qualified) and needed the same substitution with that spelling
(`src/capability/registry.rs`, `src/edn/render.rs`, one test-module literal in
`src/collection/eval.rs`'s `#[cfg(test)]`).

**Sites that needed a genuine contiguous `&[Value]`/`Vec<Value>` (not just element access) —
every one, and what was done:**

- `src/assertion.rs:225` `extract_panics` — returns `Option<Vec<Value>>` to its caller.
  `(**items).clone()` -> `items.to_vec()`.
- `src/collection/transform.rs:66` (`:wat::core::reverse`, Vector arm) — builds the reversed
  copy. `(*items).clone()` -> `items.to_vec()`, then `.reverse()` as before.
- `src/collection/transform.rs:352` (`:wat::core::sort$native`) — `sorted.sort_by(...)` needs an
  owned, independently-sortable `Vec`. `(*xs).clone()` -> `xs.to_vec()` (`xs` now comes from
  `require_vec`, see below).
- `src/collection/transform.rs:1014` (fold-stream-into-vector accumulator) — was
  `Arc::try_unwrap(acc)` (unique-or-clone unwrap of the old `Arc<Vec<Value>>`); `PVec::into_values()`
  (added for exactly this) does the same unique-or-clone unwrap on the Array arm (and collects on
  Tree).
- `src/edn/render.rs:1222` (`:wat::core::with-children`) — iterates + indexes into the children
  list while rebuilding WatAST nodes. `let child_vals: &Vec<Value> = v.as_ref()` -> owned
  `let child_vals: Vec<Value> = v.to_vec()` (the borrow doesn't survive past the match anyway).
- `src/intrinsic/reflect.rs:845` (`:wat::core::type-params-used-in`) — same shape, `v.as_ref()`
  -> `v.to_vec()` with the binding made owned.
- `src/rete/export.rs:325` (`expect_seq`, a cross-container normalizer already producing
  `Vec<Value>` for the `PersistentVector` arm via `.collect()`) — `(**xs).clone()` -> `xs.to_vec()`,
  now symmetric with the sibling arm.
- `src/runtime.rs:6495` `require_vec` — return type changed from `Result<Arc<Vec<Value>>, _>` to
  `Result<PVec, _>` (NOT materialized — most of its ~6 callers only need `.len()`/`.iter()`, which
  `PVec` gives natively; the two that needed a real slice, in `transform.rs`, call `.to_vec()`
  themselves at the point of need — see above).
- `src/stream/mod.rs:224` `eager_container_to_stream`'s `chain_from_slice` helper requires
  `DoubleEndedIterator + ExactSizeIterator`, which `PVec::iter()`'s `Box<dyn Iterator>` does not
  satisfy (same reason the `List`/`PersistentVector` arms already snapshot first). Brought the
  `Vec` arm in line with its two siblings: snapshot via `.to_vec()`, then iterate the snapshot.
- `src/collection/transform.rs:1500` (`seqable_value_to_stream`, Vector arm, lazy indexed stream)
  — turned out NOT to need a slice at all: `indexed_pv_stream` (already written for
  `:wat::core::PersistentVector`, right below `indexed_vec_stream` in the same file) takes a
  `PVec` directly and steps it by `.get(index)` — O(1) Array / O(log n) Tree per step, no
  snapshot. Pointed the Vector arm at it instead of widening `indexed_vec_stream`'s
  `Arc<Vec<Value>>` signature; `indexed_vec_stream` itself stays, still used by the `List` arm's
  own one-time snapshot.

**Test-only sites (Rust construction/match changed because the Rust type changed — not a
wat-level expectation; no STOP):**
- `tests/collection/list.rs` (4 sites), `tests/function/wat_arc170_closure_extraction.rs` (1),
  `tests/value/probe_arc216_stone5a_value_hash.rs` (4), `src/collection/eval.rs`'s own
  `#[cfg(test)]` module (1): `Value::Vec(Arc::new(...))` -> `Value::Vec(PVec::from_vec(...))` /
  `Value::Vec(wat::value::pvec::PVec::from_vec(...))` (integration tests are a separate crate —
  `crate::` doesn't reach the lib, had to spell it `wat::value::pvec::PVec::from_vec`).
- `tests/rete/probe_arc278_export.rs` (3 `as_ref().clone()` -> `.to_vec()` sites — this file
  mutates a cloned `Vec` in place to poke a rule's AST for a negative test, genuinely needs the
  owned Vec).
- `tests/program/wat_arc170_program_contracts.rs` (2 sites) — `v.as_ref()` dropped entirely;
  `PVec`'s own `.len()`/`.iter()`/`Index` cover what the test needed, no slice required.
- `tests/kernel/wat_run_sandboxed.rs:49` — `(*items).clone()` -> `items.into_values()` (owned
  match arm, no residual borrow).

**No identity-sensitive sites found.** Grepped for `Arc::ptr_eq`/`Arc::strong_count` near
`Vec`/`Vector` across `src/` and `tests/` — none. No STOP triggered for this row.

**Compile-error-driven, not a full manual pass:** after the mechanical `sed` and the
`vector_conj_inner` fix, `cargo check --lib --tests --release` was run to exhaustion (5
iterations, v1 -> v5) and every remaining error fixed from the compiler's own site list — this
is the same 38-file set enumerated above; nothing was fixed "blind."

**Floor after Row V** (`NEXTEST_TEST_THREADS=4 cargo nextest run --release`, clean tree):
**5405 tests run: 5403 passed (4 slow), 2 failed, 22 skipped** (1546.217s) — the SAME 2 lint
reds as baseline. Matches baseline exactly. CONFIRMED.

## Row L — List -> rpds::ListSync

Status: DONE.

`Value::wat__core__List(Arc<std::collections::LinkedList<Value>>)` ->
`Value::wat__core__List(rpds::ListSync<Value>)` (`src/value/value.rs:345`). The perf fix,
`list_conj_inner` (`src/collection/eval.rs:294`):

```rust
// before
Value::wat__core__List(xs) => {
    let mut out = (**xs).clone();
    out.push_front(item.clone());
    Ok(Value::wat__core__List(Arc::new(out)))
}
// after
Value::wat__core__List(xs) => Ok(Value::wat__core__List(xs.push_front(item.clone()))),
```

`rpds::List::push_front` is O(1) persistent (shares the tail node, bumps refcounts) — no clone
of the chain at all, vs. the old clone-every-node-then-prepend. `len()` is also O(1) on
`rpds::List` (a stored field), same as `LinkedList`'s.

10 files touched (grep-counted before: 58 occurrences / 17 files — many were match-only sites
that needed NO change at all, since `rpds::List` and `LinkedList` share the same `.iter()` /
`.len()` / `.is_empty()` method shapes; this matches the brief's ~54/~16 estimate well).

**Construction sites fixed (all of them — List has no "bulk build stays the old shape"
mechanical sed the way Vector did, since there's no `Arc::new(LinkedList)` single-call
replacement; each site rebuilt a `LinkedList` by `push_back`-in-a-loop then wrapped it):**

- `src/intrinsic/list.rs` `list_of` (the `(:wat::core::List ...)` constructor) —
  `vals.iter().cloned().collect()` (rpds `List`'s `FromIterator` builds a `Vec` first then
  `push_front_mut`s it in reverse — same front-to-back order as the old push_back loop,
  verified by reading `rpds`'s own `FromIterator` impl before relying on it).
- `src/holon/ast.rs:148` and `src/intrinsic/holon/atom.rs:365` (both a `from_holon_item`
  "List" arm — Holon aggregate decode) — collect the mapped items into a `Vec<Value>` first
  (the inner map is fallible, `?`-threaded), then `.into_iter().collect()` into the list at
  the end.
- `src/edn/render.rs:2324` (`Edn::List` -> `Value::wat__core__List`, the untyped EDN decode
  path) — the `Result<_,_>::collect()` target type changed from `LinkedList<Value>` to
  `rpds::ListSync<Value>` directly (both implement `FromIterator`, no restructuring needed).
- `src/edn/render.rs:2724` (the TYPED EDN decode path, `"wat::core::List"` arm) — built a
  `Vec<Value>` instead of a `LinkedList` in the loop, `.into_iter().collect()` at the end.
- `src/rete/expr_ir.rs:1691` (`OpExec::ListNew`) — dropped the `Arc::new(...)` wrapper;
  `args.iter().cloned().collect()` already produced the right element sequence, just needed
  to target `rpds::ListSync<Value>` instead of `LinkedList<Value>` inside an `Arc`.
- `src/collection/eval.rs:476` (`vector_concat_inner`, `List` arm — `List`+`List` concat) —
  was two `push_back` loops into a `LinkedList`; became
  `l.iter().cloned().chain(r.iter().cloned()).collect()` (no persistent "append" primitive on
  `rpds::List` — it's prepend-only — so this is still an O(n+m) rebuild via a fresh
  collect, same complexity as before, just shorter).
- `src/collection/eval.rs:1352` (`eval_rest`, `List` arm — `:wat::core::rest`) — **improved,
  not just ported**: the old code was `items.iter().skip(1).cloned().collect()` into a new
  `LinkedList` (an O(n) rebuild). `rpds::List::drop_first()` is a documented O(1) persistent
  op (bumps the head pointer to the existing tail node, no clone) — used that instead, so
  `rest` on a `List` goes from O(n) to O(1) as a side effect of this row.
- `src/collection/transform.rs:84` (`eval_vec_reverse`, `List` arm) — **improved**: was
  `items.iter().rev().cloned().collect()` into a `LinkedList`; `rpds::List` has its own
  `.reverse()` method (read its source before using it — it's the same O(n) rebuild
  under the hood, but it's the library's own tested implementation rather than a
  hand-rolled `rev().collect()`), used that instead.

**Test-only sites (Rust construction changed because the Rust type changed — not a wat-level
expectation; no STOP):** `tests/collection/list.rs`, 6 sites — all were
`Value::wat__core__List(Arc::new({ let mut ll = LinkedList::new(); ll.push_back(...); ll }))`
block-expressions; replaced with `vec![...].into_iter().collect()` (non-empty cases) or
`rpds::ListSync::new_sync()` (the empty-list case). Dropped the now-dead
`use std::collections::LinkedList;` import; `use std::sync::Arc` stays (still used for a
keyword value elsewhere in the file).

**Doc-comment accuracy (no behavior change, 2 files):** `src/intrinsic/linkedlist.rs`'s module
doc explicitly said "the builder has ruled that a persistent-backed list is coming" as the
justification for reserving the `:wat::linkedlist::` (not `:wat::list::`) namespace —
corrected to say the swap has now landed, while explicitly NOT renaming the namespace (that's
a separate decision outside this arc's scope: representation only, no wat-facing surface
changes). `src/collection/transform.rs:1445`'s `eval_seqable_to_stream` doc updated
`Arc<LinkedList>` -> `rpds::ListSync` in its per-arm complexity note (substance unchanged — no
indexed access either way, still snapshotted once).

**No identity-sensitive sites found.** Same grep as Row V (`Arc::ptr_eq`/`Arc::strong_count`
near `List`) — none. No STOP triggered for this row either.

**First-check-clean:** unlike Row V, the Row L `cargo check --lib --tests --release` was
ERROR-FREE on the very first run after the construction-site fixes above — every match-only
site (the majority of the 58 original grep hits) compiled unchanged, because `rpds::List` and
`LinkedList` share the same `.iter()`/`.len()`/`.is_empty()` shapes `Value`'s cross-container
code already relied on.

**Floor after Row L** (`NEXTEST_TEST_THREADS=4 cargo nextest run --release`, clean quiescent
tree — confirmed nothing else running first): **5405 tests run: 5403 passed (6 slow), 2
failed, 22 skipped** (1629.121s) — the SAME 2 lint reds as baseline. Matches baseline exactly.
CONFIRMED.

(An earlier attempt at Row L was interrupted mid-edit by an unexpected host reboot; `/var/tmp`
survived, Row V's commit/push survived, and the 8 partially-edited files survived uncommitted.
Resumed by reading the diff of each of those 8 files before writing anything further, per the
"assert every edit applied" lesson — all 8 were intact and consistent with what had been
reported done.)

## Benchmark

Status: IN PROGRESS. Lives at `wat-scripts/bench/conj-build.wat`.

## Stops

None yet.
