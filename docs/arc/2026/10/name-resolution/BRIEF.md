# Name resolution — measured first, then the hasher

## Why

wat-rs's interpreter is the slow half of every program the-little-wat bootstraps through. The persistent-collections
swap (`docs/arc/2026/10/persistent-vector-and-list/`) took one measured cost out: stage 0 fell 1,971 s → 1,665 s, about
15%, byte-identical output. A perf profile taken before that swap put **~25–35% of time in name resolution by string
hashing** and ~7% in malloc/free. That profile was never written down, so this arc's first row re-takes it, on disk.

What the code does today (read 2026-10-03, branch `the-little-wat` at `e67f9da99`):

- `src/value/environment.rs:148–260`. An `Environment` is `Arc<EnvCell>` holding a `std::collections::HashMap<String,
  BoundEntry>` (SipHash, `RandomState`) and a parent. `EnvBuilder` builds one per `let` / call. `lookup` hashes
  `name`, walks parents on a miss, and on a hit clones the value AND builds a fresh `Provenance::SymbolBound` (two
  `Span` clones).
- `src/value/symbol_table.rs:32–110`. `functions`, `unit_variants` and `runtime_def_values` are `HashMap<String, …>` on
  SipHash, and a call's head resolves through `functions.get(path)` (`:281`).
- The precedent: arc 278 moved the rete fire path's internal maps to `rustc_hash` FxHash (`Cargo.toml:124–127`,
  DESIGN-STONE-setup-fxhash). The dependency is already in the tree.

## The contract — unobservable to wat programs

Same values, same output, same diagnostics. Only the cost changes. Iteration order of these maps cannot be relied on
today, because `RandomState` reseeds every process. FxHash makes it fixed, not different-in-kind.

## The rows

- **R0 — the profile, on disk.** Build release in your own clone. Two wat-rs-only workloads:
  - `wat-scripts/bench/conj-build.wat`, which exists.
  - A new `wat-scripts/bench/call-heavy.wat`: a non-tail `fib` at 27; a self-tail loop of 1,000,000 iterations whose
    body is a `let` of six bindings that read one another; and 1,000,000 calls through a closure. It prints one checksum.

  Run each under `perf record -g` (use `-m 16` if mmap fails), and write the top 25 self-time symbols and the
  name-resolution share into `SCORE.md`, with the command. Then baseline `perf stat -e instructions:u,cycles:u`,
  `taskset -c 2`, three runs each, giving each workload's noise floor.
- **R1 — FxHash for the interpreter's name maps.** `Environment`/`EnvBuilder` bindings, and `SymbolTable`'s `functions`,
  `unit_variants` and `runtime_def_values`, all become `rustc_hash::FxHashMap`. Use one type alias per map, so a
  later change of hasher is one line. Re-measure both workloads exactly as in R0.
- **R2 — the next cost, named, not changed.** From R1's profile, say what name resolution costs NOW and where it goes:
  the parent walk, the `Provenance` construction, the value clone, the string compare. That is the map for the next
  strike, such as interned symbols or resolving a local to a slot once. Write it in `SCORE.md` with the numbers.

## Gates

1. Baseline on the unmodified branch: `cargo build --release`, then the floor `NEXTEST_TEST_THREADS=4 cargo nextest run
   --release`, which is 5,405 of 5,405 at `e67f9da99`.
2. After R1: the floor with the same result. A red test is never baseline.
3. Both workloads print the same checksum before and after.

## STOP triggers

- **STOP-1** — a test's EXPECTED output changes, golden or assert. The swap would be observable; report which, and
  why.
- **STOP-2** — a map in scope is persisted, hashed into an interchange format, or compared across processes. Say which.
  It keeps SipHash, as arc 278 kept its persisted hashes.
- **STOP-3** — R0's profile shows name resolution under 10% on both workloads. The premise is wrong; report the
  profile and stop before R1.

## Method

- **Where you work.** Work in your OWN clone, `/var/tmp/wat-rs-names`, with its own `target/`.
  `git clone /home/watmin/Work/holon/wat-rs /var/tmp/wat-rs-names`, then check out `the-little-wat` and set the push
  remote to GitHub's URL from the shared checkout's `git remote -v`. The shared checkout's binary is in use by another
  executor's runs, so it is never rebuilt from here.
- **Your territory.** This arc is wat-rs only: wat-rs sources, wat-rs tests, `wat-scripts/bench/`, and this directory.
- **Running and checking.**
  - `timeout -s KILL` on every run.
  - Never read an exit code through a pipe.
  - Never re-run a red.
  - Assert every text replacement.
- **Committing.** Commit each green row on `the-little-wat` with a `feat(name-resolution): …` / `bench(name-resolution):
  …` message, and PUSH after every commit (GitHub is the disaster-recovery site).
- **Write `SCORE.md` here AS YOU GO.** Executors get cut off; the SCORE is how a cut-off run resumes.

**Prediction (the orchestrator's, written before the strike):** R1 takes 10–20% of instructions off `call-heavy` and
less off `conj-build`. If R0 confirms the 25–35% share, R2 names the parent walk and `Provenance` construction as the
larger remainder.
