# SCORE — name resolution, measured first then the hasher

Executor's own clone: `/var/tmp/wat-rs-names`, branch `the-little-wat`, started at `732f357d4`
(the brief commit, which is `e67f9da99` + one docs commit — `e67f9da99` confirmed an ancestor).
Machine: `taskset -c 2` pins a `cpu_core` (P-core) logical CPU on this box's hybrid PMU — `perf
stat`/`perf record` both show `cpu_core/.../u` at ~99.9%+ enabled time on that pin, with a
negligible (0–3 sample) `cpu_atom` stray bucket from the brief moment before `taskset`'s
`exec()` into the target binary finishes migrating the thread. That stray bucket is excluded
from every number below (filtered to the real, high-sample-count `cpu_core` event section).

Write this file AS YOU GO — see Method.

## R0 — the profile, on disk

Status: DONE.

### Workloads

- `wat-scripts/bench/conj-build.wat` (existing) — run at **N = 1,000,000**: `echo '1000000' |
  target/release/wat wat-scripts/bench/conj-build.wat`. Builds+sums a persistent `Vector` (N
  `conj`/append) and a persistent `List` (N `conj`/prepend). Checksum: two numbers, both
  `499999500000` (= N(N-1)/2), printed as `#perf/VecResult` / `#perf/ListResult`.
- `wat-scripts/bench/call-heavy.wat` (new, written for this arc) — `target/release/wat
  wat-scripts/bench/call-heavy.wat`, no stdin. Three fixed-size call shapes: a non-tail `fib`
  at n=27; a self-tail loop of 1,000,000 iterations whose body is a `let` of six bindings each
  reading the previous; 1,000,000 calls through a stored closure (closed over one outer
  binding). Checksum: one number, `500013696418` (`fib(27)=196418` + `tail-loop=6000000` +
  `closure-loop=500007500000`), printed as `#perf/CallHeavyResult`.

Both scripts load+typecheck under the repo's `every_wat_scripts_file_loads` lint gate (they
live under `wat-scripts/`, not `scratch-pad/`, since they are durable, re-run corpus, not
throwaway probes).

### Commands

```
mkdir -p docs/arc/2026/10/name-resolution/perf
perf record -g -o docs/arc/2026/10/name-resolution/perf/conj-build-baseline.data -- \
  taskset -c 2 target/release/wat wat-scripts/bench/conj-build.wat <<< '1000000'
perf record -g -o docs/arc/2026/10/name-resolution/perf/call-heavy-baseline.data -- \
  taskset -c 2 target/release/wat wat-scripts/bench/call-heavy.wat
perf report -i <file>.data --stdio --no-children -g none
```

`perf record -g` worked directly (no mmap failure; `-m 16` was not needed).

### conj-build.wat baseline — top 25 self-time symbols (cpu_core event, N=1,000,000)

```
 9.49%  <triomphe::arc::Arc<rpds::vector::Node<Value,ArcTK>>>::drop_slow
 6.55%  <Vec<SharedPointer<rpds::vector::Node<Value,ArcTK>,ArcTK>> as Clone>::clone
 6.50%  wat::runtime::apply_function
 4.77%  <sip::Hasher<Sip13Rounds> as Hasher>::write
 4.61%  libc cfree
 3.60%  <wat::value::environment::Environment>::lookup
 3.42%  <RandomState as BuildHasher>::hash_one::<&str>
 3.19%  libc malloc
 2.36%  core::ptr::drop_glue::<wat::value::observe::Provenance>
 2.26%  wat::runtime::eval_tail
 2.15%  <Vec<SharedPointer<Value,ArcTK>> as Clone>::clone
 1.95%  wat::runtime::eval_inner
 1.85%  <wat::value::environment::EnvBuilder>::bind_unknown_span::<String>
 1.79%  wat::numeric::arith::eval_i64_arith::<eval_i64_add::{closure#0}>
 1.70%  wat::collection::transform::__wat_intrinsic_shim_eval_vec_foldl
 1.41%  <ArcTK as SharedPointerKind>::make_mut::<rpds::vector::Node<...>>::{closure#0}
 1.33%  __rustc::__rdl_alloc
 1.15%  <HashMap<String, BoundEntry, RandomState>>::insert
 1.15%  <Arc<String>>::drop_slow
 1.10%  <WatAST as Clone>::clone
 1.09%  <RandomState as BuildHasher>::hash_one::<&String>
 1.02%  <String as Clone>::clone
 0.96%  core::ptr::drop_glue::<wat::value::environment::BoundEntry>
 0.96%  libc.so.6 0x17640d (unresolved)
 0.94%  libc.so.6 0x17652b (unresolved)
```

**Name-resolution share (conj-build): 16.4%** — sum of every self-time symbol that is the
hashing/lookup/insert path for a name (`sip::Hasher::write`, `RandomState::hash_one::<&str>`
and `::<&String>`, `Environment::lookup`, `EnvBuilder::bind_unknown_span`,
`HashMap<String,BoundEntry,_>::insert`, plus small `reserve_rehash`/`RandomState::new` tails
below the top-25 cutoff): 4.77 + 3.60 + 3.42 + 1.85 + 1.15 + 1.09 + 0.41 + 0.07 ≈ 16.4%, computed
over the FULL symbol list (not capped at 25) so small tail entries are not dropped.
`Arc<EnvCell>::drop_slow` (environment teardown, not hashing) is excluded from this number —
counted instead with malloc/free below.

**malloc/free share (conj-build): 7.8%** (`cfree` 4.61% + `malloc` 3.19%) — matches the old
unwritten profile's "~7%" almost exactly.

conj-build is dominated by the PERSISTENT COLLECTION machinery, not name resolution: the top
two lines alone (`Arc<rpds::vector::Node>::drop_slow` 9.49%, `Vec<SharedPointer<Node>>::clone`
6.55%) are the vector's structural-sharing teardown/COW path — 16% before any hashing is
counted. `apply_function` (6.50%) is the generic call-dispatch cost shared by every workload.

### call-heavy.wat baseline — top 25 self-time symbols (cpu_core event)

```
 5.94%  <sip::Hasher<Sip13Rounds> as Hasher>::write
 5.50%  <wat::value::environment::Environment>::lookup
 5.28%  <RandomState as BuildHasher>::hash_one::<&str>
 4.71%  wat::numeric::arith::eval_i64_arith::<eval_i64_add::{closure#0}>
 4.22%  core::ptr::drop_glue::<wat::value::observe::Provenance>
 4.16%  wat::runtime::eval_inner
 4.14%  libc cfree
 3.40%  wat::runtime::apply_function
 3.32%  <WatAST as Clone>::clone
 3.23%  libc malloc
 2.96%  core::ptr::drop_glue::<wat_reader::ast::WatAST>
 2.71%  wat::runtime::eval_tail
 2.68%  <String as Clone>::clone
 1.98%  wat::runtime::eval_list
 1.55%  <Arc<wat::value::environment::EnvCell>>::drop_slow
 1.54%  wat::numeric::arith::eval_i64_arith::<eval_i64_sub::{closure#0}>
 1.35%  wat::runtime::__wat_special_form_tail_eval_let_tail
 1.27%  libc.so.6 0x17640d (unresolved)
 1.20%  <HashMap<String, BoundEntry, RandomState>>::insert
 1.19%  __rustc::__rdl_alloc
 1.16%  wat::runtime::bind_let_binding
 1.14%  <RandomState as BuildHasher>::hash_one::<&String>
 1.05%  wat::runtime::eval_compare::<eval_i64_eq::{closure#0}>
 1.00%  <WatAST as Clone>::clone (second call site)
 0.97%  libc.so.6 0x1764e4 (unresolved)
```

(Three stray `cpu_atom` samples — 0.0004% of the 72K total — landed on `taskset`/`ld-linux`/
`libc` during the pre-`exec` window; excluded as noise, not part of the top 25 above.)

**Name-resolution share (call-heavy): 20.3%** — same symbol set as conj-build, over the FULL
list: 5.94 + 5.50 + 5.28 + 1.20 + 1.16 + 1.14 + 0.86 (`EnvBuilder::bind_unknown_span`) + 0.69
(`reserve_rehash`) + 0.06 (`RandomState::new`) ≈ 20.3%. `Arc<EnvCell>::drop_slow` (1.55%,
environment teardown) again excluded — counted with malloc/free.

**malloc/free share (call-heavy): 7.4%** (`cfree` 4.14% + `malloc` 3.23%).

call-heavy is the better of the two name-resolution witnesses: `sip::Hasher::write` +
`Environment::lookup` + `RandomState::hash_one` alone are the top 3 self-time symbols (16.7%),
ahead of every arithmetic or call-dispatch cost.

### Reading against the brief's unwritten precedent

The never-written-down profile claimed "~25–35% of time in name resolution by string hashing."
Measured here: **16.4% (conj-build) / 20.3% (call-heavy)** — real, and the single/second-
largest cost category in both workloads, but BELOW the old estimate's range. Both numbers are
well clear of STOP-3's 10% floor, so the premise holds and R1 proceeds. (The gap to 25–35% is
plausibly the earlier profile having been taken on a workload or build that spent relatively
less time in the persistent-collection and `Provenance`/`WatAST` clone machinery that dominates
these two — an honest "measured now, higher than 10%, lower than the old recollection,"
not a refutation of the earlier number, since that number was never captured to compare.)

### perf stat baseline — instructions:u, cycles:u (taskset -c 2, 3 runs each)

Command: `perf stat -e instructions:u,cycles:u -- taskset -c 2 target/release/wat <script>
[<stdin>]`.

**conj-build (N=1,000,000):**

| run | instructions:u | cycles:u | elapsed |
|---|---|---|---|
| 1 | 32,190,267,034 | 14,624,004,575 | 13.43s |
| 2 | 32,103,905,877 | 15,075,009,130 | 14.60s |
| 3 | 32,129,333,082 | 15,395,068,697 | 15.36s |

Noise floor: instructions spread **0.27%** (min 32,103,905,877 / max 32,190,267,034 / mean
32,141,168,664); cycles spread **5.13%** (min 14,624,004,575 / max 15,395,068,697 / mean
15,031,360,801). Checksums: `499999500000` / `499999500000`, all 3 runs, matching the
functional run above.

**call-heavy:**

| run | instructions:u | cycles:u | elapsed |
|---|---|---|---|
| 1 | 60,874,086,710 | 26,419,312,249 | 26.65s |
| 2 | 61,088,338,427 | 24,641,732,710 | 22.66s |
| 3 | 61,286,166,826 | 24,782,359,161 | 18.80s |

Noise floor: instructions spread **0.67%** (min 60,874,086,710 / max 61,286,166,826 / mean
61,082,863,988); cycles spread **7.03%** (min 24,641,732,710 / max 26,419,312,249 / mean
25,281,134,707). Checksum: `500013696418`, all 3 runs.

Instructions:u is the stable metric (<1% spread both workloads); cycles:u carries real
scheduling/cache noise (5–7% spread) — any R1 comparison leans on instructions:u first, cycles:u
read only against its own noise floor.

## Gate 1 — baseline floor

Status: **RED. Executor stopped here, before R1, per "a red test is never baseline" / "do not
re-run a red."**

Run: `nohup env NEXTEST_TEST_THREADS=4 cargo nextest run --release >
/var/tmp/wat-rs-names-logs/floor-baseline.log 2>&1 &`, on this clone's HEAD (`732f357d4`, the
brief commit itself — no edits made, R1 not started). Waited to completion via `timeout 590
tail --pid=<PID> -f /dev/null`, repeated (~26 minutes total, matching the brief's estimate).

**Result: 5405 tests run: 5404 passed (6 slow), 1 failed, 22 skipped — 1522.875s.** The brief's
Gate 1 states the unmodified-branch floor is 5,405 of 5,405 at `e67f9da99`. This clone, two
commits later (`732f357d4`, docs-only in between), is NOT clean: one test failed.

**The exact arm, captured whole, verbatim, not re-run:**

```
FAIL [   5.036s] (2718/5405) wat::kernel test::deftest_wat_tests_core_core_seq_walkers_reductions_2arity_on_empty_vector_raises
  stdout ───

    running 1 test
    test test::deftest_wat_tests_core_core_seq_walkers_reductions_2arity_on_empty_vector_raises ... FAILED

    failures:

    failures:
        test::deftest_wat_tests_core_core_seq_walkers_reductions_2arity_on_empty_vector_raises

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 566 filtered out; finished in 5.00s

  stderr ───

    thread 'test::deftest_wat_tests_core_core_seq_walkers_reductions_2arity_on_empty_vector_raises' (3485205) panicked at /var/tmp/wat-rs-names/tests/kernel/test.rs:17:1:
    deftest_wat_tests_core_core_seq_walkers_reductions_2arity_on_empty_vector_raises: exceeded time-limit of 5000ms — deftest :wat-tests::core::core-seq-walkers::reductions-2arity-on-empty-vector-raises at /var/tmp/wat-rs-names/wat-tests/core/core-seq-walkers.wat:281:1 (test thread leaked — process exit will reap)
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**The arm:** a hardcoded 5000ms wall-clock deadline on the wat-side deftest
`:wat-tests::core::core-seq-walkers::reductions-2arity-on-empty-vector-raises`
(`wat-tests/core/core-seq-walkers.wat:281`), enforced by the Rust harness at
`tests/kernel/test.rs:17`. This is a DEADLINE failure, not an assertion/golden mismatch — the
same class the-little-wat-floor-threads memory names ("wat-rs floor red at 16 and 8 threads
(deadlines only)"), though this run already used `NEXTEST_TEST_THREADS=4` as that memory
prescribes.

**Observed, not used to dismiss the above:** at the moment this ran, `ps aux` showed
`/home/watmin/.cache/wat-kw-007/release/wat elf/compile.wat` pinned near 99% CPU for 10+ minutes
— a DIFFERENT executor's long-running job on this shared 16-core machine (the-little-wat /
elf-compile territory, not this arc's). Per CLAUDE.md's "no test is pre-blessed" / "known
flake/timing is not a disposition" doctrine, this is reported as an observation only — it is
NOT offered here as a verdict that the deadline-exceed is contention and therefore safe to wave
through. **Per instruction, the test was NOT re-run** (re-running a red destroys the evidence);
no attempt was made to reproduce, isolate, or diagnose further, since this arc's territory is
`wat-rs` sources/tests/`wat-scripts/bench/`/this directory, not a license to chase an unrelated
floor red to ground.

**Consequence at the time:** R1 was paused here and the finding surfaced to the orchestrator,
rather than guessing at a disposition for a deadline red.

**Resolution — builder's ruling (relayed by the orchestrator):** mitigate with a relaxed
timer as a STOPGAP; another branch has already fixed the timing problem properly, and this
line is meant to lose cleanly at that branch's merge. The orchestrator's own measurement:
`reductions-2arity-on-empty-vector-raises` alone (`NEXTEST_TEST_THREADS=1`) took 1.69 / 1.90 /
1.74 / 1.78 / 1.81s across five runs — nowhere near 5000ms; it only exceeded 5000ms in the
floor run above because another job held a CPU core for the whole run, and a wall-clock
deadline counts other processes' time too, so a busy machine reads as a hang even when the
test itself is far under budget.

Applied: `crates/wat-macros/src/lib.rs:881` (now 890 after the added history paragraph)
`const DEFAULT_TIME_LIMIT_MS: u64 = 5000;` → `= 20000;`, with a dated history paragraph above
it in the style of the existing ones (2026-05-03, 2026-05-13) recording the stopgap, the
evidence, the "deadline counts other processes' time" mechanism, and that the real fix lives
on another branch and this line loses cleanly at merge. Nothing else in source or tests
referenced the old value (grepped; only historical arc docs mention `5000ms`).

Re-ran the floor on this change (not a re-run of the same red — a fresh floor run after an
actual code change): `NEXTEST_TEST_THREADS=4 cargo nextest run --release` →
**5405 tests run: 5405 passed (2 slow), 22 skipped (1243.834s). ALL GREEN.** This is now the
arc's baseline floor. Committed `15a7f024a` and pushed.

## R1 — FxHash for the interpreter's name maps

Status: DONE.

### The edit

- `src/value/environment.rs` — added `pub(crate) type BindingMap = rustc_hash::FxHashMap<String,
  BoundEntry>;` (one alias), and retyped `EnvCell.bindings` and `EnvBuilder.bindings` to it.
  `HashMap::new()` → `BindingMap::default()` at both construction sites (`Environment::new`,
  `Environment::child`). Removed the now-unused `use std::collections::HashMap;`.
- `src/value/symbol_table.rs` — added three aliases (`FunctionMap`, `UnitVariantMap`,
  `RuntimeDefValueMap`, all `rustc_hash::FxHashMap<String, _>`), and retyped the `functions`,
  `unit_variants`, `runtime_def_values` fields. `BindingMetadata` and `acronym_registry`
  (untouched by the brief) kept `std::collections::HashMap` — the import stays for those.
- `function_entry`'s return type (`std::collections::hash_map::Entry<'_, String,
  Arc<Function>>`) did NOT need a hasher type parameter: `HashMap<K,V,S>::entry()`'s `Entry`
  is generic over `K`, `V`, and an (unstable, defaulted) allocator — NOT the hasher `S` — so
  the original signature compiles unchanged against the new `FunctionMap`. Confirmed by
  letting the compiler check it: an initial guess at adding `rustc_hash::FxBuildHasher` as a
  third `Entry` type parameter failed with "`FxBuildHasher: Allocator` not satisfied" (9
  errors), which is what revealed the actual 3rd parameter's meaning; reverted to the
  original three-parameter-free signature, which built clean.
- No other file in the tree names the concrete `HashMap<...>` types for these four fields —
  all external access goes through `SymbolTable`'s private methods (`get`, `has_function`,
  `register_function`, `functions_iter`, etc.) and `Environment`/`EnvBuilder`'s public API —
  confirmed by grep before editing, so the edit is contained to the two files above.
- `cargo build --release` was clean on the first attempt after the `function_entry` fix (one
  round-trip, not a struggle).

STOP-2 does not apply: none of these four maps are persisted, hashed into an interchange
format, or compared across processes — all four are in-memory-only, and nothing walks any of
them in insertion/iteration order for externally-visible output (the brief's own contract
note: iteration order was never reliable, since `RandomState` reseeds every process).

### Re-measurement (same method as R0)

Checksums, all runs (functional run + 3×perf-record + 3×perf-stat per workload, 8 runs total
per workload): conj-build `499999500000`/`499999500000`; call-heavy `500013696418`. **Gate 3
satisfied** — byte-identical output, same as before R1.

**Instructions:u / cycles:u, 3 runs each, taskset -c 2:**

| workload | metric | R0 mean | R1 mean | reduction | R0 spread | R1 spread |
|---|---|---|---|---|---|---|
| conj-build (N=1e6) | instructions:u | 32,141,168,664 | 28,380,056,129 | **11.70%** | 0.27% | 0.06% |
| conj-build (N=1e6) | cycles:u | 15,031,360,801 | 13,976,872,988 | **7.02%** | 5.13% | 1.38% |
| call-heavy | instructions:u | 61,082,863,988 | 54,113,393,785 | **11.41%** | 0.67% | 0.67% |
| call-heavy | cycles:u | 25,281,134,707 | 23,244,187,186 | **8.06%** | 7.03% | 0.46% |

Both workloads landed inside the orchestrator's predicted 10–20% instruction reduction for
call-heavy — but conj-build's reduction (11.70%) came in essentially EQUAL to call-heavy's
(11.41%), not "less," against the prediction's "less off conj-build." Reported as measured,
against the prediction, not adjusted to fit it.

**Name-resolution share (same symbol set + full-list-sum method as R0, so the two numbers are
comparable):**

| workload | R0 share | R1 share | implied absolute-instruction reduction |
|---|---|---|---|
| conj-build | 16.4% | **11.96%** | ≈35.6% fewer instructions in the named symbol set |
| call-heavy | 20.3% | **17.17%** | ≈25.1% fewer instructions in the named symbol set |

(Absolute reduction = `1 - (R1_share × R1_total) / (R0_share × R0_total)`, using the
instructions:u means above — e.g. conj-build: `1 - (0.1196×28,380,056,129)/(0.164×32,141,168,664)
≈ 1 - 3,394M/5,271M ≈ 35.6%`.) The SHARE fell less than the absolute instruction count inside
that share, because total program instructions also fell ~11.5%, so the remaining
name-resolution cost is a slightly larger slice of a markedly smaller pie.

Top 25 self-time symbols, R1, both workloads (cpu_core event, clean single-section reports —
no hybrid-PMU stray samples this time):

**conj-build R1:**
```
 9.99%  <triomphe::arc::Arc<rpds::vector::Node<Value,ArcTK>>>::drop_slow
 7.17%  wat::runtime::apply_function
 6.48%  <Vec<SharedPointer<rpds::vector::Node<Value,ArcTK>,ArcTK>> as Clone>::clone
 5.20%  <wat::value::environment::Environment>::lookup
 4.82%  libc cfree
 3.48%  libc malloc
 2.66%  core::ptr::drop_glue::<wat::value::observe::Provenance>
 2.46%  wat::runtime::eval_tail
 2.23%  <Vec<SharedPointer<Value,ArcTK>> as Clone>::clone
 2.16%  wat::runtime::eval_inner
 2.00%  wat::numeric::arith::eval_i64_arith::<eval_i64_add::{closure#0}>
 1.91%  <wat::value::environment::EnvBuilder>::bind_unknown_span::<String>
 1.87%  wat::collection::transform::__wat_intrinsic_shim_eval_vec_foldl
 1.57%  <sip::Hasher<Sip13Rounds> as Hasher>::write
 1.46%  libc.so.6 0x17640d (unresolved)
 1.44%  <HashMap<String, BoundEntry, FxBuildHasher>>::insert   <- was RandomState pre-R1
 1.40%  <ArcTK as SharedPointerKind>::make_mut::<Node<...>>::{closure#0}
 1.33%  __rustc::__rdl_alloc
 1.27%  <RandomState as BuildHasher>::hash_one::<&str>
 1.16%  <Arc<String>>::drop_slow
 1.11%  <WatAST as Clone>::clone
 1.08%  core::ptr::drop_glue::<wat::value::environment::BoundEntry>
 0.94%  <String as Clone>::clone
 0.85%  <Value as Clone>::clone
 0.80%  libc.so.6 0xa6324 (unresolved)
```

**call-heavy R1:**
```
 7.90%  <wat::value::environment::Environment>::lookup
 5.01%  wat::numeric::arith::eval_i64_arith::<eval_i64_add::{closure#0}>
 4.43%  wat::runtime::eval_inner
 4.39%  libc cfree
 4.33%  core::ptr::drop_glue::<wat::value::observe::Provenance>
 3.62%  <WatAST as Clone>::clone
 3.55%  wat::runtime::apply_function
 3.46%  libc malloc
 3.25%  core::ptr::drop_glue::<wat_reader::ast::WatAST>
 2.78%  wat::runtime::eval_tail
 2.62%  <String as Clone>::clone
 2.32%  <sip::Hasher<Sip13Rounds> as Hasher>::write
 2.16%  <RandomState as BuildHasher>::hash_one::<&str>
 2.00%  <HashMap<String, BoundEntry, FxBuildHasher>>::insert   <- was RandomState pre-R1
 1.94%  wat::runtime::eval_list
 1.80%  wat::numeric::arith::eval_i64_arith::<eval_i64_sub::{closure#0}>
 1.58%  libc.so.6 0x17640d (unresolved)
 1.55%  <Arc<wat::value::environment::EnvCell>>::drop_slow
 1.37%  wat::runtime::__wat_special_form_tail_eval_let_tail
 1.26%  __rustc::__rdl_alloc
 1.13%  libc.so.6 0x1764e4 (unresolved)
 1.12%  wat::runtime::eval_compare::<eval_i64_eq::{closure#0}>
 1.07%  <WatAST as Clone>::clone (2nd call site)
 1.06%  wat::runtime::bind_let_binding
 1.02%  core::ptr::drop_glue::<wat::value::environment::BoundEntry>
```

**Confirmed working:** `<HashMap<String, BoundEntry, FxBuildHasher>>::insert` — the BINDING
map's own `insert` (the write side of `EnvBuilder`/`Environment`) is now labeled
`rustc_hash::FxBuildHasher`, not `std::hash::random::RandomState` — direct symbol-level proof
the hasher swap took effect on the hot path, not just in source.

**What did NOT go away:** `sip::Hasher::write` and `RandomState::hash_one::<&str>` are still
present post-R1 (1.57%/1.27% conj-build; 2.32%/2.16% call-heavy) — SHRUNK from R0
(4.77%/3.42% and 5.94%/5.28%) but not zeroed. Investigated via `perf report -g graph,0,caller`:
the callers mostly resolve to unsymbolized small-integer addresses (`0x1`, `0x3b`, …) — this
binary's tail-call/dispatch-heavy interpreter loop loses frame-pointer-based unwind
information at some call sites, so perf cannot name the exact caller. The only OTHER
std-`RandomState`-keyed maps still live and touched at these sample counts are `SymbolTable`'s
check-time registries (`types: Option<Arc<TypeEnv>>`'s `HashMap<String, TypeExpr/TypeDef/
TypeScheme, RandomState>`, `acronym_registry`, `binding_metadata`) — all OUT OF THIS ARC'S
SCOPE (the brief named only `functions`/`unit_variants`/`runtime_def_values`/`Environment`
bindings) and all showing ≤0.01% individually, i.e. negligible at these sample counts; they do
NOT explain the 1.27–2.32% residual. The residual is most likely the FxHash-vs-SipHash
INLINING ASYMMETRY named in R2 below, not an un-converted map.

### Gates after R1

- **Gate 2** (floor, same result): `NEXTEST_TEST_THREADS=4 cargo nextest run --release` →
  **5405 tests run: 5405 passed (4 slow), 22 skipped (1621.419s). ALL GREEN** — same as the
  post-stopgap baseline. CONFIRMED.
- **Gate 3** (same checksums before/after): CONFIRMED above — every run, both workloads,
  identical to R0.

Committed as a single row (code + re-measurement) once Gate 2 came back green.

## R2 — the next cost, named, not changed

Status: DONE. Per the brief's prompt — "the parent walk, the Provenance construction, the
value clone, the string compare" — read against R1's profile:

**1. The string compare: gone as a visible cost.** Pre-R1, a `HashMap<String,_,RandomState>`
lookup/insert showed as distinct `sip::Hasher::write` + `RandomState::hash_one` frames because
SipHash's computation is too large to inline into the caller. Post-R1, `FxBuildHasher`'s own
`hash_one` is itself nearly invisible (0.11–0.16%, both workloads) — small enough that LLVM
inlines the whole hash-then-probe-then-compare sequence (hash, bucket probe, and the
collision-resolving `==`) directly into `Environment::lookup` and the `BoundEntry` map's
`insert`. The string compare did not get faster in isolation so much as it STOPPED BEING ITS
OWN LINE ITEM — it is now folded into whichever caller holds the map operation.

**2. `Environment::lookup` is now the single largest OR near-largest named symbol in both
workloads** (5.20% conj-build, **7.90% call-heavy — the #1 symbol outright**), up from
3.60%/5.50% pre-R1, even though the SAME function's absolute instruction count likely fell (it
now contains the formerly-separate FxHash work, per point 1) — Amdahl's law on a shrunk pie,
not new work. Its own logic (`src/value/environment.rs:200–222`), unchanged by R1, is:
```rust
if let Some(entry) = self.inner.bindings.get(name) {              // now FxHash — cheap
    let value = entry.value.value().clone();                       // the "value clone"
    let provenance = match entry.value.provenance().clone() {      // <- clone ALWAYS happens
        Provenance::RuntimeBuilt { producer, call_span } => {
            Provenance::RuntimeBuilt { producer, call_span }       // re-wrapped, not reused
        }
        _ => {
            Provenance::SymbolBound {                               // <- clone just discarded
                binding_span: entry.binding_span.clone(),           // the "Provenance construction"
                head_span: head_span.clone(),
            }
        }
    };
    return Some(TrackedValue::new(value, provenance));
}
self.inner.parent.as_ref().and_then(|p| p.lookup(name, head_span)) // the "parent walk"
```

**3. The Provenance construction, named with numbers:** `entry.value.provenance().clone()` on
line 203 ALWAYS clones the stored provenance — then, on every arm except `RuntimeBuilt`
(`Unknown`/`Literal`/`SymbolBound`, which is every binding a `let` or function-param creates;
`RuntimeBuilt` only comes from producer functions like `keyword/from-name`), that clone is
immediately discarded and a NEW `Provenance::SymbolBound` is built from two FRESH
`Span::clone()` calls. `Span` is `Arc<String>`-backed (cheap — an atomic refcount bump, no
allocation — confirmed by reading `crates/wat-reader/src/span.rs`'s own doc comment: "Stored as
`Arc<String>` so spans clone cheaply"), so no single clone is expensive — but the PATTERN is a
guaranteed clone-then-discard on the common path, done once per `let`-binding/param lookup.
This shows up directly as `core::ptr::drop_glue::<Provenance>` self-time: **2.66% (conj-build)
/ 4.33% (call-heavy)** post-R1 (barely changed from R0's 2.36%/4.22% — R1 did nothing to this
cost, as expected, since it is untouched by the hasher swap). The cheap fix named, not done:
match on `entry.value.provenance()` BY REFERENCE first (no clone), and only clone in the
`RuntimeBuilt` arm (where today's clone is actually needed) — skip the intermediate clone
entirely in the `Unknown`/`Literal`/`SymbolBound` arms, since their old value is about to be
replaced regardless.

**4. The value clone (`entry.value.value().clone()`): inlined, not separately visible, and
workload-dependent.** For call-heavy (i64s and one closure Value, both `Arc`-cheap-or-`Copy`)
this is NOT a likely dominant cost. For conj-build, the looked-up accumulator is the growing
persistent `Vector`/`List` itself — but THAT clone is the already-separately-visible
`triomphe::Arc::drop_slow` (9.99%) / `Vec<SharedPointer<Node>>::clone` (6.48%) pair at the top
of conj-build's profile, which is the persistent-COLLECTION arc's territory (2026-10
persistent-vector-and-list), not a name-resolution cost — `Environment::lookup` only clones the
`Arc`-wrapped handle to the Vector, not its contents; the expensive work is the Vector's OWN
structural-sharing machinery on the subsequent `conj`, unrelated to how the binding was found.

**5. The parent walk: not separately measurable in this profile, named as a gap.** Both
benchmark workloads build SHALLOW environment chains — `tail-loop`/`closure-loop` rebuild a
flat one-level `Environment` each iteration (TCO discards the parent on each tail call; no
chain grows), and `fib`'s non-tail recursion nests up to 27 Rust call frames but each frame's
own lookup only walks to its immediate parent (the function's own single enclosing scope), not
a chain of 27. `Environment::lookup`'s recursive `self.inner.parent.as_ref().and_then(...)`
call (line 222) is a Rust-level tail call (LLVM may or may not fold it into a loop; not
confirmed either way here) — but with chain depth ≈1–2 in both workloads, its cost is folded
into `Environment::lookup`'s own self-time indistinguishably from everything else in the
function, and neither benchmark exercises a DEEP chain to isolate it. **Named as the gap R0/R1
cannot close**: a THIRD workload with deliberately deep lexical nesting (e.g. N nested `let`s
each binding one new name, looking up the OUTERMOST one from the INNERMOST scope) would be
needed to give the parent walk its own visible cost, separate from a one-hop lookup.

**The map for the next strike**, per the brief's own framing: `Environment::lookup` itself —
specifically the clone-then-discard `Provenance` pattern (cheap to fix, named in point 3) —
is now the better-evidenced NEAR-term target than a full interned-symbol or
slot-resolution redesign; the latter (resolving a local to a slot ONCE instead of re-looking-up
by name every reference) would also eliminate the now-dominant `Environment::lookup` cost
category wholesale, including the unmeasured parent-walk gap from point 5, and remains the
larger, correctly-named LONGER-term strike the brief anticipated.

## STOP triggers fired

None of STOP-1/STOP-2/STOP-3 fired at any point. STOP-3 does not apply (both workloads'
name-resolution share — 16.4%/20.3% pre-R1, 11.96%/17.17% post-R1 — stayed above the 10%
floor throughout).
