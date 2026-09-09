# SCORE — closing cell `(record, absent, derived, leading, na)`: `accum-lead-derived`

Executing strike per `DESIGN.md`. Written as-I-go per house rule.

## Re-derivation, before any edit

Read `DESIGN.md` in full, then read `wat-rs/CLAUDE.md` in full (its own doctrine reaches no other
way — floor discipline, no-known-flakes, the scratch-`.wat` location, the wat-fix codemod rule).
Then read the two parents side by side, as instructed:

- `accum-over-derived.wat`/`.clj` — `(record, absent, derived, nonleading, na)`. `Seed` anchors
  the accumulate (`[Seed] [?n <- (acc/count) :from [Step]]`), so the accumulate has a parent
  condition. `Step(k) :- Step(k-1)` for `k` in `[1,depth]` makes `Step` itself derived within the
  same ruleset — the `A=derived` mechanism.
- `leading-exists.wat` — `(record, absent, none, leading, na)`. A leading (parentless) `:exists`
  over `Wind`, a plain seeded fact never derived — the `L=leading` mechanism, proven only where
  the witnessed type is NOT derived.

DESIGN names the new fixture as their intersection: drop `Seed` entirely so the accumulate becomes
the rule's **only** condition (leading), over `Step`, which the same ruleset still derives.
Verified the "sole accumulate condition in a rule" shape is syntactically legal before writing
anything new, by reading `where-accum-lead.wat` (`count-zero`/`count-three`: an accumulate as the
only LHS condition, plus a same-token `:where` guard) and `accum-lead-rule-cascade.wat` (`busy-rule`:
an accumulate with no parent, "no left fact").

## A correction found while re-deriving DESIGN's own claim

DESIGN says fixture discovery needs **no runner registration** — citing
`check-grid-three-way.sh:123`'s glob. Read that script in full before trusting the claim: the glob
at line 123 is real, but two lines later (135–145) the **same script** hard-fails ALL axes if a
discovered `.wat` has no row in its `SIZES` map. So `check-grid-three-way.sh` DOES need a
registration — just not a "list of runners," a `SIZES["accum-lead-derived"]="9"` entry. Added it
(alongside the existing rows, alphabetical).

More materially: `tests/rete/wat_scripts_grid_axes_live.rs` (`SIZED_AXES`) and
`tests/rete/wat_scripts_grid_port_check.rs` (`CORRECTNESS_SIZES`) **both walk
`wat-scripts/perf/grid/*.wat` and assert exact SET equality against a hardcoded array** — neither
is mentioned in DESIGN's "what done means" checklist, but both are genuinely part of the floor
(`cargo nextest run --release`), and both would have gone instantly, loudly RED the moment
`accum-lead-derived.wat` landed on disk with no matching row (that IS the liveness gate DESIGN
names — "a stub reddens it" — but a **non-stub, unregistered** fixture reds it too, for a
different, purely-administrative reason: the exact-set assertion, not non-vacuity). Added a row to
each, matched to the file's own documented liveness size (`[4]`, non-vacuous justification
mirroring `accum-over-derived`'s liveness row) and correctness size (`[9]`, expected count `10`,
derivation re-stated from the `.wat` header, not read off a run).

This is a correction to my own first reading of DESIGN, not to the builder's: DESIGN's checklist
("Registered in `peragrare-census.sh`'s `EXPECTED_FIXTURES` and its coordinate table") is silent on
these two Rust arrays, and skipping them would have shipped a floor-breaking change while
technically satisfying every line DESIGN wrote down.

## The fixture

`wat-scripts/perf/grid/accum-lead-derived.wat` + `.clj` (Clara twin, static — same
`gen-<axis>.sh`-vs-static-`.clj` convention as `accum-over-derived.clj`, so a correctness-only axis
cannot be dragged onto the perf ladder by `run-all.sh:81-99`'s discovery).

Shape: `Step(0)` seeded; `Step(k) :- Step(k-1)` for `k` in `[1,depth]`; **`Tally(n) :- [?n <-
(acc::count) :from Step]`, LEADING — its only condition, no anchor, no join.** `:derived` is
sorted-NOT-deduped `enc(0,k,0)` per derived `Step` level plus `enc(1,0,n)` per `Tally`. Depth 9
(matching `accum-over-derived`'s stated-choice reasoning: past the probe's two intermediate
states). Expected count = depth + 1 = 10 — re-derived from the formula, not read off a run.

## Registration — everywhere the disk-walking gates require it

- `wat-scripts/perf/grid/peragrare-census.sh`: `EXPECTED_FIXTURES` (17 now) and the coordinate
  table row `accum-lead-derived record absent derived leading na`.
- `wat-scripts/perf/grid/check-grid-three-way.sh`: `SIZES[accum-lead-derived]="9"`.
- `tests/rete/wat_scripts_grid_axes_live.rs`: `SIZED_AXES` row, size `[4]` (liveness; correctness
  size 9 is separately dialed).
- `tests/rete/wat_scripts_grid_port_check.rs`: `CORRECTNESS_SIZES` row, size `[9]`, expected `10`.
- **Not added to `run-all.sh`'s `ORDER`** — the perf ladder is a different population (DESIGN's
  pin honored).

## Census — before and after, both runs

**Before** (ran prior to writing any file):

```
== VISITED CELLS ==
  (record,absent,none,none,not-consumed) = 2  [negation strat-neg]
  (userfn,absent,none,none,consumed) = 1  [userfn-head]
  (record,present,none,none,na) = 1  [retract-multiplicity]
  (record,absent,none,none,na) = 5  [asym-join deep-cascade fanout node-share parametric-erasure]
  (record,absent,none,leading,na) = 1  [leading-exists]
  (record,absent,derived,nonleading,na) = 1  [accum-over-derived]
  (record,absent,base,leading,na) = 1  [accum-lead-rule-cascade]
  (record,absent,base,nonleading,na) = 3  [accum min-finding user-reduce]
  (record,absent,none,none,consumed) = 1  [neg-consumer]
total cells = 108 ; visited = 9 ; empty = 99
members = 16
```

**After** (post-registration):

```
== VISITED CELLS ==
  (record,absent,none,none,not-consumed) = 2  [negation strat-neg]
  (userfn,absent,none,none,consumed) = 1  [userfn-head]
  (record,present,none,none,na) = 1  [retract-multiplicity]
  (record,absent,none,none,na) = 5  [asym-join deep-cascade fanout node-share parametric-erasure]
  (record,absent,none,leading,na) = 1  [leading-exists]
  (record,absent,derived,nonleading,na) = 1  [accum-over-derived]
  (record,absent,base,leading,na) = 1  [accum-lead-rule-cascade]
  (record,absent,base,nonleading,na) = 3  [accum min-finding user-reduce]
  (record,absent,none,none,consumed) = 1  [neg-consumer]
  (record,absent,derived,leading,na) = 1  [accum-lead-derived]
total cells = 108 ; visited = 10 ; empty = 98
members = 17
```

**The move is exactly what was added, nothing else**: `visited` 9→10 (+1, the target cell only),
`empty` 99→98, `members` 16→17. `total cells` unchanged (108 — no axis widened). Every pre-existing
row is byte-identical between the two runs.

`--verify` (mechanical presence/absence facts against the live corpus, all 17 fixtures):

```
peragrare-census --verify: all 17 fixtures' mechanical facts agree with the table.
```

`--pins` (populated pin still 5, empty pin still 0, moving pin still 8 of 8) — unchanged and still
`PASSED` on all three; not reprinted here since none of the three pins names this cell.

## ⛔ THE THREE-WAY — the risk this strike exists to measure

Ran `wat-scripts/perf/grid/check-grid-three-way.sh accum-lead-derived` (isolated, correctness size
`[9]`, one JVM):

```
accum-lead-derived   clara=10   native=10   oracle=10    ALL THREE MATCH
grid-three-way: 1 axis/axes, all AGREED — Clara == oracle == native (3s)
```

The script only prints full sets on a mismatch, so the sets themselves were captured separately,
verbatim, from each engine directly (not summarized):

**wat native + oracle**, `echo '[9]' | ./target/release/wat wat-scripts/perf/grid/accum-lead-derived.wat`:

```
#grid/Result {:axis "accum-lead-derived" :size #wat.core/PersistentVector [9] :derived #wat.core/PersistentVector [1000000000 2000000000 3000000000 4000000000 5000000000 6000000000 7000000000 8000000000 9000000000 1000000000000010] :native-ns 231693 :oracle-derived #wat.core/PersistentVector [1000000000 2000000000 3000000000 4000000000 5000000000 6000000000 7000000000 8000000000 9000000000 1000000000000010] :oracle-ns 123225441}
```

**Clara 0.24.0** (staged exactly as `check-grid-three-way.sh` does — file renamed
`accum_lead_derived.clj`, `require`d, `-main "9"` invoked):

```
#grid/Result {:axis "accum-lead-derived" :size [9] :derived [1000000000 2000000000 3000000000 4000000000 5000000000 6000000000 7000000000 8000000000 9000000000 1000000000000010] :clara-ns 5222668}
```

**All three `:derived` sets are byte-for-byte identical**: `[1000000000 2000000000 3000000000
4000000000 5000000000 6000000000 7000000000 8000000000 9000000000 1000000000000010]` — 9 derived
`Step` levels (`enc(0,k,0)` for `k` in `[1,9]`) plus one `Tally` (`enc(1,0,10)`, i.e. `n=10`). No
leaked intermediate tally (`accum-over-derived`'s failure shape) and no per-round duplicate
(`leading-exists`'s failure shape) in any of the three engines.

**Per the pin, this is not a formality passed — it is the risk that was measured and came back
negative.** The two cures (accumulate-over-derived supersession, and leading-non-monotonic
re-emission) DO compose at this cell, at this size, today. That is a real (if narrow) result, not
an assumption: the fixture was driven, not merely written, and the three engines were compared as
values, not counts (`3` elements is not the check; the exact multiset is).

## Floor

Ran `./scripts/floor.sh` in the foreground (not backgrounded), once, clean on the first pass — no
red to capture. Before that, ran three narrower `cargo nextest` selections foreground-sequentially
(checking `pgrep -af 'cargo|nextest'` clear before each) to catch a cheaper red first, all green:

```
PASS wat::rete wat_scripts_grid_port_check::every_grid_axis_native_matches_its_oracle
PASS wat::rete wat_scripts_grid_axes_live::grid_axes_run_and_derive_nonvacuously
PASS wat::rete wat_scripts_grid_axes_live::spec_equals_native_on_every_where_family
Summary [  35.284s] 3 tests run: 3 passed, 5506 skipped
```

```
Summary [ 162.352s] 292 tests run: 292 passed, 5217 skipped   (wat::lint — includes
  every_wat_scripts_file_loads_on_the_current_runtime and
  every_rete_name_in_wat_scripts_code_resolves, the two gates wat-rs/CLAUDE.md names as reading
  every .wat under wat-scripts/)
```

Full floor, read from `.floor/latest/clean.log`:

```
Summary [ 451.150s] 5490 tests run: 5490 passed (1 slow), 19 skipped
```

**5490 — exactly the pinned number.** No new `#[test]` function was added anywhere; the change is
two new corpus files plus rows added to four already-existing arrays that existing tests already
iterate. `.floor/2026-09-09T08-21-11Z/` (symlinked as `.floor/latest`) holds the untruncated log.

## What the fixture cost — time and judgement calls

**Judgement calls, named:**

1. **Whether to keep `Seed` as a vestigial record or drop it.** Dropped it — DESIGN's own framing
   ("Seed is DROPPED entirely… leading, exactly like accum-lead-rule-cascade's… no parent
   condition") is the tightest reading of "their intersection," and a vestigial unused record
   would have obscured which mechanism is actually under test.
2. **Whether the new fixture needed registration beyond what DESIGN's checklist names.** It did
   (see "A correction found" above) — `SIZED_AXES` and `CORRECTNESS_SIZES` are both walked and
   asserted-equal against disk by tests that run on every floor. This was found by reading the two
   test files in full before writing the `.wat`, not discovered by a red.
3. **What liveness size to pick for `SIZED_AXES`.** Reused `accum-over-derived`'s liveness size
   (`depth=4`, 5 elements) rather than inventing a new one — same shape, same non-vacuity argument,
   and running it directly (`echo '[4]' | wat …`) confirmed 5 elements before committing the row.
4. **Whether to trust the three-way script's summary line or capture full sets by hand.** Captured
   full sets by hand from each of the three engines directly (native, oracle, Clara) rather than
   relying on `check-grid-three-way.sh`'s own report — the script only prints sets on a MISMATCH,
   and the task's own pin says the disagreement (if any) is the deliverable, so the agreement case
   needed its own verbatim evidence, not just a summary count.

**Cost, concretely (not a stopwatch total — this is what was actually necessary, once):**

- One `.wat` file (~185 lines, mostly boilerplate mirrored from `accum-over-derived.wat` — the
  session/query/encoding scaffolding is identical across every sized grid axis) and one `.clj` twin
  (~85 lines, same relationship to `accum-over-derived.clj`).
- Four registration edits (`peragrare-census.sh` ×2 sites, `check-grid-three-way.sh` ×1,
  `wat_scripts_grid_axes_live.rs` ×1, `wat_scripts_grid_port_check.rs` ×1 — five sites, four
  files), none mechanical enough to skip reading the surrounding array for placement/style.
  **This registration surface (5 sites across 4 files) is bigger than DESIGN's 2-site checklist
  implies**, and is the single most expensive thing this cell taught: DESIGN's own "what a fixture
  costs" section undercounts by exactly the two Rust arrays.
  Fixed cost, does not grow with the fixture's content.
- One `cargo build --release --bin wat` (~53s) plus one full `cargo test --no-run` compile inside
  the first targeted `nextest` invocation (~2m46s, unavoidable — nothing had built the test
  binaries yet this session) — a fixed tax paid once, not per-cell.
  Cost to check the actual mechanism (JIT/compile warmup + three fires per engine): under 10s.
- Total wall time across the whole strike (build + edits + three targeted `nextest` runs + one
  full floor) was on the order of 15–20 minutes; the floor run itself (`451.150s` ≈ 7.5 min) is the
  dominant term and is fixed regardless of how many cells this closes.

## Are the other four cells mechanical from here, or does each carry its own unknown?

**Not uniformly mechanical — the registration surface is now known and IS mechanical (same 5
sites, same shape, for any of the four); the CONTENT is not.**

What this cell proved cheap and repeatable:
- The Clara-staging convention (rename `-` to `_`, `require`, drive `-main`).
- The registration surface — now that all 5 sites are named, a future cell does not need to
  rediscover `wat_scripts_grid_axes_live.rs`/`wat_scripts_grid_port_check.rs` by reading two ~500
  and ~650-line files; this SCORE names them directly.
- The pattern of "take two existing single-axis fixtures, keep one's derivation-witness shape,
  drop or add exactly the condition that changes L/R/H/N."

What each of the remaining four still carries as its own unknown, read against their own
descriptions in DESIGN:

- **Cell 1** (userfn-head LHS accumulating over a same-ruleset-derived type): combines `H=userfn`
  with `A=derived` — `userfn-head.wat`'s stratification is by PRODUCED type, not fn name, and
  whether that still holds when the type it accumulates over is ALSO derived (two derived-type
  dependencies in one ruleset) is not something this cell's result speaks to at all.
- **Cell 2** (duplicate-retract feeding a leading accumulate): `R=present` combined with
  `L=leading` — `retract-multiplicity`'s remove-one/remove-all semantics have never been driven
  against a LEADING node (no parent to anchor which token the retract's supersession targets
  against). This is a genuinely different composition than cell 3 (which combined `A` and `L`, not
  `R` and `L`), and nothing here derisks it.
- **Cell 4** (a positive consumer downstream of a leading gate): `neg-consumer`'s positive-consumer
  shape has only ever sat downstream of a NON-leading negation. Whether a positive rule can even
  correctly consume output from a leading gate's re-evaluation semantics is untested by this cell.
- **Cell 5** (duplicate-retract of the accumulate's own source): `R=present` combined with
  `A=derived` (or `base`) on the retracted side specifically — distinct from both cell 2 and this
  cell, and closer in shape to `accum-over-derived` than to `accum-lead-rule-cascade`.

**So: the SCAFFOLDING cost (registration, Clara staging, three-way driving) is now a known
quantity and genuinely cheap for all four. The MECHANISM each combination might expose is not
derisked by this result** — this cell's agreement is evidence about ONE compound pressure
(derived-source × leading), not about the other four pairings, which combine different axis pairs
entirely (R×L, H×A, N×L, R×A). DESIGN's own five-cell list is not a single hypothesis tested once;
it is five distinct hypotheses that happen to share a test harness.

## What I did NOT do

- Did **not** build any of the other four cells (cell 1, 2, 4, 5) — the pin is explicit and this
  report is what "judge the remaining four" is supposed to be judged against.
- Did **not** add `accum-lead-derived` to `run-all.sh`'s `ORDER` — DESIGN rules this out
  explicitly (wrong population: 11 dialed perf axes vs the census's differential fixtures).
- Did **not** widen the census's five axes (H/R/A/L/N) or touch its `TABLE`/`AGGREGATION` logic
  beyond adding one row — DESIGN rules this out ("The grid is what it is").
- Did **not** re-run any test after a red — there was no red to react to; every gate went green on
  its first run, including the full floor.
- Did **not** tune the fixture after seeing the three-way result. The fixture (depth=9, the exact
  LHS shape) was fixed before the first `check-grid-three-way.sh` invocation and was not touched
  afterward — the agreement above is what the first run reported, not a converged-upon one.
- Did **not** modify `accum-over-derived.wat`, `leading-exists.wat`, `accum-lead-rule-cascade.wat`,
  or any other existing fixture — read-only reference reads only.

---

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_018ntHDMRNCKDKNr2gVfzXmP
