# BRIEF — STONE 255.78: the floor never reads a clock to decide

**Drawn 2026-10-01 against local `main` @ `628260fd9`** (255.77, held unpushed: this stone lands first, then one floor for
both). **Executor: a Sonnet subagent, working solo** (it runs the floor; pass `timeout: 600000` on that Bash call, or the
harness moves it to the background after 120 s). A strike in `src/rete/kernel/tests/`, `benches/`, `Cargo.toml` and one
lint. Commit locally on `main` (`git add -- <paths>`, never `-A`); **do not push**. Your final message is your report.

## Why

The orchestrator's floor at `628260fd9` went red on `rete::kernel::tests::harvest_cost::harvest_wrap_split`
(`.floor/2026-10-01T09-41-45Z`):

```
panicked at src/rete/kernel/tests/harvest_cost.rs:337:5:
combined harvest (16.81 ms) is not accounted for by scan (1.06 ms) + wrap (5.54 ms) — the apportionment this test reports
no longer adds up, so one of the three closures is measuring something other than what its name says
```

The same commit passed it twelve minutes earlier (`.floor/2026-10-01T09-29-58Z`). **The builder (2026-10-01):** *"i
strongly dislike timing based on a single machine's capabilities … W1 attacking a time property makes sense."* Ruled
**T1: no verdict on the floor depends on a clock.** Each claim gets a deterministic witness; a claim that is only about
speed leaves the floor for `benches/` (`harness = false`, prints, never gates; precedent: `benches/binding_repr.rs`,
relocated from this same test directory, and the `Cargo.toml:261-265` note on why a bench is structurally not an
`#[ignore]`).

## The census (a `mora` cast plus a timing-verdict census, read-only; the orchestrator re-read every row below)

`mora` proper found nothing: no `sleep`, no `*_timeout`, no snapshot in these nine files; every clock read is after the
fact. The seven assertions whose verdict depends on a measured duration:

| gate | claim | the witness to build |
|---|---|---|
| `harvest_cost.rs:337` `harvest_wrap_split` | the scan/wrap split accounts for the combined pass | the combined closure performs exactly the filter matches and `PMap::from_pairs` calls the two halves do (counts) |
| `gather_probe_cost.rs:422` `drop_memories_cost_split` | `clear()` of the fire-scoped structures stays O(1) | the same operation/allocation count at two different N |
| `fanout_cost.rs:328` `fanout_per_call_alpha_census` | production dominates hash-join on a fan-out workload | the per-phase counts the census already collects (mark pairs or operations), production ≥ 2× hash-join |
| `node_share_cost.rs:822` `node_share_where_cost_decomposition` | the (token×tid) set-probe loop is the majority rung of the taken branch | its probe count, `tokens × tids`, against the other rungs' counts |
| `accum_cost.rs:241` `accum_fire_phase_census` (fold < 25 ms) | the fold has not regressed to the 68.49 ms mechanism | find that mechanism in `DESIGN-STONE-accum-fold-the-wall` (`git ls-files 'docs/**/*fold-the-wall*'`) and count it |
| `accum_cost.rs:250` (snapshot < 1 ms) | the snapshot `DESIGN-STONE-gather-no-snapshot` removed has not returned | a snapshot-build count of 0 (find or add the counter) |
| `binding_repr_bench.rs:500` `token_bindings_representation_dominance` | design premise: at cardinality 1 an array scan reads faster than a hash trie | none: a pure speed claim. Moves to `benches/` (beside its relocated siblings), printed, not asserted |

Not in scope: the ~30 liveness checks (`x > 0.0`, "the loop never ran"): they fail only if a loop did not run, which no
load changes. Leave them.

## The work

1. **For each of the first six rows:** recover the claim from the test's own comments and its design stone, build the
   deterministic witness, and replace the clock comparison with it. The timing **printout** may stay (information). If a
   witness needs a counter the engine does not have, add the narrowest one (test-only where it can be, `cfg(test)` or the
   existing instrumentation the census uses), and say exactly what. If a claim turns out to have no deterministic witness
   after reading its design stone, it goes with row 7 (below) and the SCORE says why.
2. **Row 7, and any row that joins it:** move the measurement into `benches/` (a new `[[bench]]` or the existing
   `binding_repr` one), printing, with no assertion on the time; the floor keeps any non-timing assertion the test had.
3. **The wall:** a lint (under `tests/lint/`, the style of its neighbours) that fails when an assertion in
   `src/**/tests/**` or `tests/**` compares a value derived from a clock (`Instant`, `elapsed`, `ns_per_iter`,
   `elapsed_ns`, …) against anything other than the liveness form `> 0` / `> 0.0`. It may be syntactic, but say what it can
   and cannot see. **Mutation-prove it:** restore one deleted timing comparison, see the lint red, remove it.
4. **Mutation-prove each new witness** (at least the six rows): break what it counts (e.g. drop a phase from the combined
   closure), see the test red by name, restore.

## Gates

| what | how | expected |
|---|---|---|
| the lint | its own test, plus the mutation in item 3 | green; red on the mutation |
| benches build | `cargo bench --no-run` (or `cargo build --release --benches`) | rc 0 |
| release floor | `scripts/floor.sh`, **in the foreground, `timeout: 600000`**, nothing else running | all passed; the count against 6339 at `628260fd9` (`.floor/2026-10-01T09-29-58Z`), ± the tests you move or add |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change (a new witness that fails, the lint finding a site outside the seven): capture it
  **verbatim** from `.floor/<stamp>/`, then decide: a witness that encodes the claim and fails is **STOP-1**, not a cure;
  the lint finding another timing verdict is work (convert it the same way, and list it). Never re-run unchanged code for
  a green.
- **STOP-1:** a deterministic witness, built to state a row's claim, fails: the engine no longer does what the test
  claimed. Quote it and STOP on that row; finish the others.
- **STOP-2:** a witness needs a counter in the engine's non-test code path that would change its behaviour or its cost
  (anything beyond a `cfg(test)` or an existing instrument). Describe it and STOP on that row.
- **STOP-3:** a timing verdict found outside `src/rete/kernel/tests/` by the lint in a file whose claim you cannot recover.
  List it and STOP on it.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you: no known flake; a red is a red. Capture `rc=$?` on the next statement. Never wait with
`pgrep -f`. Run every build, floor and clippy in the foreground and block on it. Never write a number, file:line or
example you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.78-the-floor-never-reads-a-clock.md` beside this brief, commit it, **do not push**.
