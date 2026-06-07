# Stone 252.1 — the rune-aware coverage gate (the "wat-cov" layer)

**Status:** BUILT + PROVEN 2026-06-06. Shipped: `scripts/coverage-gate.sh` (managed runner) +
`scripts/coverage_rune_check.py` (rune-aware analyzer). Baseline established (TOTAL 62.75%); the
gate emits the **100%-or-runed work-list: 2179 uncovered-not-runed lines / 411 blocks / 30 warded
files**, worst-first (`check/error` 385, `collection/eval` 342, `value/value` 165, the `*/error.rs`
cluster). Stone 252.2 drives that list to 100%-or-runed; Stone 252.3 wards the gate tooling + ships
the convention to datamancy.

This DESIGN is written against EMPIRICALLY PROVEN mechanics (not memory) — the proving was done
first, this session, including a DEAD END (hand-rolled show-env) that the managed-mode engine
replaced. The failure modes below were discovered, not imagined.

---

## What is proven (this session, 2026-06-06) — and the dead end that taught it

1. **DEAD END (do not repeat): the hand-rolled `show-env` approach mis-manages profraw.**
   `cargo llvm-cov show-env --export-prefix` + run-your-own-way + a SEPARATE `cargo llvm-cov
   report` *seems* to work but does not, reliably: (a) a dev-configured `show-env` cannot
   correlate `--release` profraw (the leak-safe runner uses `--release`) → mixed-profile profraw
   don't merge → `comms/*` read 0.00%; (b) even single-profile, `clean` + manual `report` showed
   STALE profdata (byte-identical 40188-missed across different runs). The lesson: do NOT hand-roll
   the env + report; let cargo-llvm-cov MANAGE the profraw lifecycle.

2. **THE ENGINE: cargo-llvm-cov MANAGED mode (`--no-report` accumulate → one `report`).**
   `cargo llvm-cov clean` once, then `cargo llvm-cov --no-report ... <tier>` for each leak-safe
   tier (it builds instrumented + runs + writes profraw, all profile-consistent + state-managed),
   then ONE `cargo llvm-cov report --release --lcov`. This correlates EVERY tier correctly.
   PROVEN: `comms/*` 0.00% → **52/67/65%** (mod/process/thread) once the comms tests were homed +
   run under managed accumulation; whole-tree TOTAL **62.75%** (real). cargo-llvm-cov 0.8.7.

3. **LCOV, not JSON, for the rune check.** `report --lcov` is line-based (`DA:<line>,<count>`;
   count 0 = uncovered) — a clean fit for line-based `// rune:coverage` comments. Far simpler to
   parse than the segment-JSON. The gate emits LCOV and the analyzer keys on it.

---

## The pipeline (the gate's runner — SHIPPED as `scripts/coverage-gate.sh`)

```bash
cargo llvm-cov clean --workspace                       # managed: reset profraw state
cargo llvm-cov --no-report --release -p wat --lib      # tier 1: lib (in-src unit tests)
cargo llvm-cov --no-report --release -p wat --test test  # tier 2: the wat-corpus (deftest demos)
cargo llvm-cov --no-report --release -p wat \           # tier 3: homed [[test]] groups (auto-discovered)
    --test nursery --test collection --test comms --test function --test macros --test types
cargo llvm-cov report --release --lcov --output-path target/coverage.lcov   # merge → LCOV
python3 scripts/coverage_rune_check.py target/coverage.lcov                 # rune-aware warded-home check
```

Leaky `#[ignore]`'d process tests are EXCLUDED (their non-ignored siblings already drive comms to
~52-67%); a genuinely leaky-only path earns `rune:coverage(proves-elsewhere)` citing the contained
`integration-run.sh` run. NEVER `--workspace` (proc-leak).

- **NEVER `--workspace`** for the run (proc-leak the recovery doc bans). The leak-contained
  tier 3 runner (`integration-run.sh`, arc 245.7) is the only safe way to run the process class;
  it already uses setsid + timeout + `pkill -s` containment. The gate REUSES it (DRY — do not
  re-implement the proven containment).
- `--all` includes the leaky-signal tier so `comms/*` (only exercised by spawn/fork/hermetic
  tests) gets measured. Containment backstops it.
- profraw `%p` makes each process (incl. spawned children) write a unique file; report merges all.

CAVEAT (honest): children reaped by `pkill -s` before their atexit flush lose their profraw.
This affects only LEAKED/hung children; normally-completing test children flush first. Quantify
once measured; if material for a comms region, that region is a `proves-elsewhere` candidate.

---

## The rune-aware gate (the part cargo-llvm-cov can't do)

cargo-llvm-cov reports uncovered regions but knows NOTHING of our runes. The gate adds:

1. Run the pipeline above → `report --release --json`.
2. For each uncovered region, map → `file:line` (the JSON carries segment/region coords; exact
   shape is the next empirical step — inspect the 0.8.7 `--json` schema before coding the parser,
   per FM-2-bis: parse against the real schema, not an imagined one).
3. **SCOPE FILTER — warded homes only.** The gate judges ONLY the 12 warded homes
   (`value/ function/ check/ types/ collection/ macros/ scope/ comms/ remedy/ argspec/ rust_deps/`).
   The flat monolith (`runtime.rs` 28k, `check.rs`, the unwarded `src/*.rs`) is OUT OF SCOPE
   until migrated — it has no vigilatum stamp to defend. (This is why TOTAL 51% is irrelevant;
   the gate never reads the monolith.)
4. For each uncovered region IN a warded home: PASS iff the region's line carries a
   `// rune:coverage(<category>) — <reason>` (or is on/adjacent per a placement rule TBD at build).
5. Report: uncovered-AND-not-runed = FINDINGS (the work); runed = the exemption list (excusare's input).
   Exit non-zero iff any warded-home region is uncovered-and-not-runed.

---

## Scope: warded homes only (real baseline buckets, release pipeline pending)

From the dev lib+corpus baseline (release-unified numbers land from the bg run; shape holds):

- **Well-covered** (wards that got testing attention): `scope/*` 100%, `remedy/*` 97–100%,
  `macros/{eval,expand,parse,registry}` ~85–87%, `function/infer` 87.86%, `check/env` 87.42%,
  `rust_deps/marshal` 88.67%. → near-clean; a few runes.
- **Integration-elsewhere** (rises with tier 3, or `proves-elsewhere`): `comms/*` (0% pre-tier-3),
  parts of `collection/*`, `value/observe`. → tier 3 decides; residue → `proves-elsewhere`.
- **Genuinely thin — the work** (loudest: every `*/error.rs`): `check/error` 5.87%,
  `value/value` 10.50%, `types/error` 19.77%, `value/signal` 20.35%, `types/defstruct` 22.64%,
  `argspec/error` 31.11%. → mix of `defensive`/`unreachable` runes (error/Display paths) AND
  genuine under-testing → the Shape-A in-src `#[cfg(test)]` unit tests (the test-reorg arc).

The error-path pattern is the signal: error rendering/Display is classic `defensive`/`unreachable`
rune territory AND a real test gap. The gate forces the file-by-file decision.

---

## The vigilatum third axis

`vigilatum` asserts L1+L2=0 + clippy-0. This stone adds **coverage: 100%-or-runed**. A stamped
home means audited AND clean AND exercised. A stamp that can silently lose coverage lies
(connects arc 250 vigilatum-integrity — the gate is another self-enforcement axis). value/'s
stamp (`dce4253b`) predates the gate; re-stamping with the coverage axis is part of 252.2.

---

## Deliverables

- `scripts/coverage-gate.sh` (the wat-cov layer): the pipeline + `--json` parse + warded-home
  scope filter + rune check. Mirrors `green-gate.sh`'s style (arc 239).
- Possibly parameterize `integration-run.sh` if the dev-vs-release profile ever needs to differ
  from its hardcoded `--release` (currently aligned, so no change needed — note it as a coupling).
- The rune placement rule (exact-line vs adjacent-comment) — settle at build against the `--json`
  region coords.

## The chain within 252

252.1 (this — build the gate) → 252.2 (drive 100%-or-runed across the 12 homes; value/'s ~13
`unreachable!` arms = first `rune:coverage(unreachable)` exemplars; re-stamp with coverage axis)
→ 252.3 (ward the gate tooling + datamancy ship: excusare override-list + convention doc +
intueri-name the category words & the gate-tool — human-gated publish).

## Cross-references

- `../../../COVERAGE-RUNE.md` — the convention (categories, triad, doctrine).
- `DESIGN.md` — the arc blueprint.
- `scripts/integration-run.sh` — the leak-contained runner (arc 245.7) the gate reuses.
- `scripts/green-gate.sh` — style precedent (arc 239).
- The test-reorg (Shape A: co-locate `#[cfg(test)]` unit tests + home-group `tests/<home>/`) is a
  SEPARATE later arc, gate-first because the gate keys on `src/` coverage + runes and is
  therefore invariant to `tests/` layout. The gate's per-home numbers drive that reorg's file→home mapping.
