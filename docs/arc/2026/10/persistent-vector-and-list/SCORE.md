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

Status: NOT STARTED.

## Benchmark

Status: NOT STARTED. Will live at `wat-scripts/bench/conj-build.wat`.

## Stops

None yet.
