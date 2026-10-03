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

**Consequence for this arc:** R1 was not started. Gate 1 requires a clean (5405/5405) baseline
before an after-R1 floor run can be compared meaningfully — a red baseline makes Gate 2's "same
result" comparison uninterpretable (a post-R1 red could be R1's fault or this pre-existing one,
and nothing here can tell them apart without re-running, which is foreclosed). R0 above stands
on its own (perf profiles, checksums, instructions/cycles, all independent of the floor) and is
unaffected by this gate's status.

## R1 — FxHash for the interpreter's name maps

Status: NOT STARTED. Blocked behind Gate 1 (see above) — the brief's own gate requires a clean
baseline floor before R1's "re-measure... exactly as in R0" can be meaningfully compared, and
before R1's own gate 2 (floor after R1, same result) can be judged against a known-good
reference rather than an already-red one.

## R2 — the next cost, named

Status: NOT STARTED (depends on R1's post-change profile).

## STOP triggers fired

None of STOP-1/STOP-2/STOP-3 fired. STOP-3 does not apply (both workloads' name-resolution
share, 16.4%/20.3%, is above the 10% floor). The blocker above is a GATE failure (Gate 1, the
baseline floor), not one of the brief's three named STOP conditions — surfaced here because the
brief says "the floor... which is 5,405 of 5,405" and "a red test is never baseline," and
because CLAUDE.md (injected, overriding default behavior) forbids re-running a red to see if it
goes away.
