# SCORE — the four remaining compound cells

Executing strike per `DESIGN.md`. Written as-I-go per house rule (drafted while the targeted
`nextest` selection ran in the background; the floor section below is filled in after it and the
full floor both completed).

## Re-derivation, before any edit

Read `DESIGN.md` in full, then `wat-rs/CLAUDE.md` in full (floor discipline, no-known-flakes,
scratch-`.wat` location, wat-fix codemod rule — none of it reaches a rider any other way). Then
read `strike-grid-first-compound-cell/SCORE.md` in full, the worked precedent for this exact kind
of cell, and copied its method: read both parents for each cell side by side, build the fixture as
their intersection with everything else held at a parent's default, register at all five sites
before running anything, drive all three engines and capture verbatim output (not just the
three-way script's summary line), census before/after, then the full floor.

Read every fixture named as a parent for one of the four cells: `userfn-head.wat`/`.clj`,
`accum-over-derived.wat`/`.clj`, `accum-lead-derived.wat`, `retract-multiplicity.wat`/`.clj`,
`leading-exists.wat`, `accum-lead-rule-cascade.wat`/`.clj`, `neg-consumer.wat`, and
`where-accum-lead.wat`/`where-exists.wat` for two syntax precedents (a leading accumulate as a
rule's sole condition; a leading `:exists` producing a derived fact via `:then`, `where-exists.wat`'s
`lead-wind` rule).

## The four cells, what each crosses, and how each was built

### Cell 1 — `userfn-accum-derived` — head-kind × accum-from

`accum-over-derived.wat`'s EXACT shape (Seed anchors `[?n <- acc::count :from Step]`;
`Step(k):-Step(k-1)` makes Step derived within the ruleset), with the tally rule's `:then` head
changed from a bare `(:aod::Tally ?n)` construction to a call `(:cad::mk-tally ?n)` on a user fn —
userfn-head's exact `(mk-rate ?k)` idiom (a bound var in, one record out, no compute). Coordinate:
`(userfn, absent, derived, nonleading, na)`. Depth 9 (matching both parents' stated choice).
Expected count = depth+1 = 10, unchanged from accum-over-derived's own number since `mk-tally` is a
pure 1:1 wrapper.

### Cell 2 — `retract-lead-accum` — retract × accum-position (leading)

`accum-lead-rule-cascade.wat`'s EXACT shape (a LEADING `[?n <- acc::count :from Reading]` — no
parent condition — then joined to `Anchor(k)`, plus an inert `Link` cascade forcing extra fixpoint
rounds), with `Reading(0)` duplicated (retract-multiplicity's exact technique: one extra seed of
only the retracted key) and then ONE copy retracted before re-fire. Coordinate:
`(record, present, base, leading, na)`. Size `[3 2 3]`, matching accum-lead-rule-cascade's own
stated choice. Expected: pre-retract Reading population = items+1; remove-one returns it to
items=3, so `n`=3 and Busy count = anchors = 2 (constancy in depth is still the assertion, exactly
accum-lead-rule-cascade's own).

### Cell 4 — `leading-neg-consumer` — consumer × accum-position (leading)

leading-exists's exact Wind/inert-S1..S6-cascade shape, with a NEW three-stage positive chain
sitting where leading-exists's own direct query sat: `Signal(loc) :- (exists Wind(loc))` (LEADING,
no parent — `where-exists.wat`'s `lead-wind` rule is the syntax precedent for a leading `:exists`
with a `:then` head, since accum-lead-derived and leading-exists never combine `:exists` with a
`:then` head that itself gets consumed further) → `Ok(loc) :- Signal(loc), NOT Bad(loc)` (Bad never
seeded, vacuous — neg-consumer's own `Ok` shape) → `Final(loc) :- Ok(loc), Tag(loc)` (neg-consumer's
own positive-consumer subject, verbatim). Coordinate: `(record, absent, none, leading, consumed)`.
Items=20 (leading-exists's own correctness size). Expected: exactly `[0..items)`, one Final per loc.

### Cell 5 — `retract-accum-derived` — retract × accum-from (derived)

`accum-over-derived.wat`'s EXACT shape UNCHANGED, with `Step(0)` — the accumulate's own `:from`
source AND the cascade's root — duplicated (retract-multiplicity's technique) and then ONE copy
retracted before re-fire. Coordinate: `(record, present, derived, nonleading, na)`. Depth 9.
PREDICTED expected count = depth+1 = 10 (the canonical accum-over-derived number) under correct
remove-one / truth-maintenance semantics — stated explicitly in the fixture's own header as the
HYPOTHESIS under test, not a settled fact, since duplicating the cascade's own root and letting it
propagate through `Step(1..depth)` before the retract is exactly the compound risk this cell exists
to measure (does a retract on a type ALSO being derived correctly narrow to the one retracted
token, and does any transient over-derivation get cleaned up).

**A build-time type error was found and fixed before any run.** `retract-accum-derived.wat`'s first
draft built `seed-facts` by conjing `Seed` then `Step` onto a bare `(:wat::core::PersistentVector)`
literal; the homogeneous-PV inference locked the accumulator's element type to `:rad::Seed` from
the first `conj` and refused the second (`:rad::Step`) at check time — 2 `TypeMismatch` errors,
`wat-scripts/perf/grid/retract-accum-derived.wat:123-124`. accum-over-derived.wat's own
`empty-records` helper (`(:wat::core::PersistentVector :- [:wat::core::Record])`, explicitly typed
before the first `conj`) is exactly the guard against this — I had read it, and still wrote the bug
on the new file; adding `:rad::empty-records` and conjing onto THAT fixed it. Verified fixed by
running the file, not by re-reading it.

## Registration — everywhere the disk-walking gates require it (all 4 cells, all 5 sites)

- `wat-scripts/perf/grid/<name>.wat` + `.clj` — all four pairs written above.
- `wat-scripts/perf/grid/peragrare-census.sh`:
  - `EXPECTED_FIXTURES` — 4 new stems added (21 total).
  - The coordinate `TABLE` — 4 new rows (see cell descriptions above for each 5-tuple).
  - **A correction found in the census's own mechanical `verify_fixture`, beyond the four
    registration sites DESIGN + the precedent's SCORE both name.** Its `H=userfn` check was
    hardcoded to grep for USERFN-HEAD'S OWN literal snippet
    (`':then [(:ufh::mk-rate ?k)]'`) regardless of which fixture's row was being verified — true by
    accident while userfn-head was the only `H=userfn` row, and would have wrongly reported drift
    (or, worse, silently passed for the wrong reason if some OTHER file happened to contain that
    exact substring) the moment a second `H=userfn` fixture existed with a different fn name, which
    cell 1 is. Generalized to a per-name `USERFN_SNIPPET` associative array
    (`userfn-head` → its own snippet, `userfn-accum-derived` → `:cad::mk-tally ?n`), looked up by
    `$name` inside `verify_fixture`. Found by reading the function in full before adding a second
    `H=userfn` row, not discovered by a red.
- `wat-scripts/perf/grid/check-grid-three-way.sh`: `SIZES` rows for all 4
  (`leading-neg-consumer="20"`, `retract-accum-derived="9"`, `retract-lead-accum="3 2 3"`,
  `userfn-accum-derived="9"`).
- `tests/rete/wat_scripts_grid_axes_live.rs`: `SIZED_AXES` rows for all 4, liveness sizes
  `[3]`/`[4]`/`[2 1 2]`/`[4]` respectively, each run by hand before being committed to the array
  (see "Liveness sizes, driven" below).
- `tests/rete/wat_scripts_grid_port_check.rs`: `CORRECTNESS_SIZES` rows for all 4, sizes
  `[9]`/`[20]`/`[3 2 3]`/`[9]`, expected counts `10`/`20`/`2`/`10`.
- **Not added to `run-all.sh`'s `ORDER`** — DESIGN's pin honored; the perf ladder is a different
  population.

## Liveness sizes, driven (not read off a run — confirmed BY one)

```
$ echo '[4]' | ./target/release/wat wat-scripts/perf/grid/userfn-accum-derived.wat
:derived [1000000000 2000000000 3000000000 4000000000 1000000000000005]   (5 elements)
$ echo '[2 1 2]' | ./target/release/wat wat-scripts/perf/grid/retract-lead-accum.wat
:derived [2]   (1 element)
$ echo '[3]' | ./target/release/wat wat-scripts/perf/grid/leading-neg-consumer.wat
:derived [0 1 2]   (3 elements)
$ echo '[4]' | ./target/release/wat wat-scripts/perf/grid/retract-accum-derived.wat
:derived [1000000000 2000000000 3000000000 4000000000 1000000000000005]   (5 elements)
```

All four non-empty, matching `SIZED_AXES`'s stated justifications.

## Census — before and after, both runs

**Before** (matches `strike-grid-first-compound-cell/SCORE.md`'s own "after" — the state this
strike started from):

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

**After** (post-registration, all 4 cells landed):

```
== VISITED CELLS ==
  (record,absent,none,leading,consumed) = 1  [leading-neg-consumer]
  (record,absent,none,none,not-consumed) = 2  [negation strat-neg]
  (record,present,base,leading,na) = 1  [retract-lead-accum]
  (userfn,absent,none,none,consumed) = 1  [userfn-head]
  (record,present,none,none,na) = 1  [retract-multiplicity]
  (record,absent,none,none,na) = 5  [asym-join deep-cascade fanout node-share parametric-erasure]
  (record,absent,none,leading,na) = 1  [leading-exists]
  (record,absent,derived,nonleading,na) = 1  [accum-over-derived]
  (record,absent,base,leading,na) = 1  [accum-lead-rule-cascade]
  (record,present,derived,nonleading,na) = 1  [retract-accum-derived]
  (userfn,absent,derived,nonleading,na) = 1  [userfn-accum-derived]
  (record,absent,base,nonleading,na) = 3  [accum min-finding user-reduce]
  (record,absent,none,none,consumed) = 1  [neg-consumer]
  (record,absent,derived,leading,na) = 1  [accum-lead-derived]
total cells = 108 ; visited = 14 ; empty = 94
members = 21
```

**Movement: visited 10→14 (+4, exactly the four target cells), empty 98→94 (-4), members 17→21
(+4). `total cells` unchanged at 108.** Matches the DESIGN-stated expectation
("visited 10→14, empty 98→94, members 17→21") exactly. Every pre-existing row is byte-identical
between the two runs.

`--verify` (mechanical presence/absence facts against the live corpus, all 21 fixtures):

```
peragrare-census --verify: all 21 fixtures' mechanical facts agree with the table.
```

`--pins` (all three PASSED, unchanged shape — none of the three pins names any of these four
cells; the moving pin's 8-of-8 synthetic rows still land):

```
-- populated pin: (record,absent,none,none,na) --
   count = 5 (expect 5: asym-join deep-cascade fanout node-share parametric-erasure)
   PASSED
-- empty pin: H=lambda (a value no axis takes) --
   count = 0 (expect 0, and not one of the 108 grid cells since H has no such value)
   PASSED
-- moving pin: one synthetic row per non-default axis value (8 = (2-1)+(2-1)+(3-1)+(3-1)+(3-1)) --
   [H-userfn] -> (userfn,absent,none,none,na) landed, count>=1 : OK
   [R-present] -> (record,present,none,none,na) landed, count>=1 : OK
   [A-base] -> (record,absent,base,none,na) landed, count>=1 : OK
   [A-derived] -> (record,absent,derived,none,na) landed, count>=1 : OK
   [L-leading] -> (record,absent,none,leading,na) landed, count>=1 : OK
   [L-nonleading] -> (record,absent,none,nonleading,na) landed, count>=1 : OK
   [N-not-consumed] -> (record,absent,none,none,not-consumed) landed, count>=1 : OK
   [N-consumed] -> (record,absent,none,none,consumed) landed, count>=1 : OK
   moved 8 of 8
   PASSED
```

## ⛔ THE THREE-WAY — the risk this strike exists to measure, all four cells

Ran `check-grid-three-way.sh <name>` for each cell individually (isolated, one JVM each), THEN
captured the full `:derived`/Clara sets by hand from each of the three engines directly (native +
oracle from one `wat` invocation at the correctness size; Clara staged exactly as the script does
— file renamed `snake_case.clj`, `require`d, `-main` invoked with the size args) — not relying on
the three-way script's own summary line, since it only prints full sets on a MISMATCH and the
pin's whole point is that the AGREEMENT case needs its own verbatim evidence too.

### Cell 1 — `userfn-accum-derived`, size `[9]`

```
$ bash check-grid-three-way.sh userfn-accum-derived
userfn-accum-derived clara=10   native=10   oracle=10    ALL THREE MATCH
grid-three-way: 1 axis/axes, all AGREED — Clara == oracle == native (4s)
```

wat native + oracle, `echo '[9]' | ./target/release/wat wat-scripts/perf/grid/userfn-accum-derived.wat`:

```
:derived        [1000000000 2000000000 3000000000 4000000000 5000000000 6000000000 7000000000 8000000000 9000000000 1000000000000010]
:oracle-derived [1000000000 2000000000 3000000000 4000000000 5000000000 6000000000 7000000000 8000000000 9000000000 1000000000000010]
```

Clara 0.24.0, `require`d + `(-main "9")`:

```
:derived [1000000000 2000000000 3000000000 4000000000 5000000000 6000000000 7000000000 8000000000 9000000000 1000000000000010]
```

**All three byte-for-byte identical.** 9 derived Step levels (`enc(0,k,0)` for k in [1,9]) plus one
Tally (`enc(1,0,10)`) — the SAME 10-element multiset accum-over-derived itself produces, now
constructed through a user-fn `:then` head. **AGREE.**

### Cell 2 — `retract-lead-accum`, size `[3 2 3]`

```
$ bash check-grid-three-way.sh retract-lead-accum
retract-lead-accum   clara=2    native=2    oracle=2     ALL THREE MATCH
grid-three-way: 1 axis/axes, all AGREED — Clara == oracle == native (3s)
```

wat native + oracle, `echo '[3 2 3]' | ./target/release/wat wat-scripts/perf/grid/retract-lead-accum.wat`:

```
:derived        [3 1000000000000003]
:oracle-derived [3 1000000000000003]
```

Clara 0.24.0, `(-main "3" "2" "3")`:

```
:derived [3 1000000000000003]
```

**All three byte-for-byte identical.** `enc(0,3)=3` and `enc(1,3)=1000000000000003` — anchors=2
Busy facts, both encoding post-retract `n=3=items` (the duplicate Reading(0) correctly removed by
remove-one against a LEADING accumulate, no parent token to anchor against). **AGREE.**

### Cell 4 — `leading-neg-consumer`, size `[20]`

```
$ bash check-grid-three-way.sh leading-neg-consumer
leading-neg-consumer clara=20   native=20   oracle=20    ALL THREE MATCH
grid-three-way: 1 axis/axes, all AGREED — Clara == oracle == native (4s)
```

wat native + oracle, `echo '[20]' | ./target/release/wat wat-scripts/perf/grid/leading-neg-consumer.wat`:

```
:derived        [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19]
:oracle-derived [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19]
```

Clara 0.24.0, `(-main "20")`:

```
:derived [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19]
```

**All three byte-for-byte identical** — exactly `[0..20)`, one Final per loc. No per-round
duplication (leading-exists's own failure shape) survived propagating through TWO further
positive-consumer stages (Ok, Final), and no stratum-placement drop (neg-consumer's own pre-cure
failure shape) either. **AGREE.**

### Cell 5 — `retract-accum-derived`, size `[9]`

```
$ bash check-grid-three-way.sh retract-accum-derived
retract-accum-derived clara=10   native=10   oracle=10    ALL THREE MATCH
grid-three-way: 1 axis/axes, all AGREED — Clara == oracle == native (4s)
```

wat native + oracle, `echo '[9]' | ./target/release/wat wat-scripts/perf/grid/retract-accum-derived.wat`:

```
:derived        [1000000000 2000000000 3000000000 4000000000 5000000000 6000000000 7000000000 8000000000 9000000000 1000000000000010]
:oracle-derived [1000000000 2000000000 3000000000 4000000000 5000000000 6000000000 7000000000 8000000000 9000000000 1000000000000010]
```

Clara 0.24.0, `(-main "9")`:

```
:derived [1000000000 2000000000 3000000000 4000000000 5000000000 6000000000 7000000000 8000000000 9000000000 1000000000000010]
```

**All three byte-for-byte identical**, and matching the PREDICTED value stated in the fixture's own
header BEFORE this run (depth+1=10) — the population correctly converges to accum-over-derived's
own canonical state: duplicating the accumulate's own derived-cascade root and then retracting one
copy of it does not leave a stale intermediate Step, a leaked Tally, or a surviving duplicate.
**AGREE.**

## ⭐ All four cells AGREE — the summary the pin asks for

**0 of 4 disagreed.** Every one of the four axis-pair crossings DESIGN drew — `H×A`, `R×L`, `N×L`,
`R×A` — produced byte-for-byte identical `:derived` sets across Clara 0.24.0, the wat oracle, and
wat native, at the stated correctness size, driven once each and not re-run.

Per the pin, this is not four formalities passed — it is four independent measurements of whether a
cure proven under one pressure holds under a second, DIFFERENT pressure each time (no two of these
four share the same axis pair, and none shares its pair with the first cell's `A×L`). All five
compound cells the grid names (the first cell plus these four) now agree. That is five results, not
a trend extrapolated from one — each is its own experiment and is reported as such.

## Floor

Ran targeted `cargo nextest` selections FOREGROUND-SEQUENTIALLY first (checking
`pgrep -af 'cargo|nextest'` clear before each — one exceeded the 120s tool timeout and moved to
background automatically; waited for its own completion notification rather than launching
anything else concurrently), to catch a cheaper red before paying the full floor's cost:

```
PASS wat::rete wat_scripts_grid_port_check::every_grid_axis_native_matches_its_oracle
PASS wat::rete wat_scripts_grid_axes_live::grid_axes_run_and_derive_nonvacuously
PASS wat::rete wat_scripts_grid_axes_live::spec_equals_native_on_every_where_family
Summary [  35.415s] 3 tests run: 3 passed, 5506 skipped
```

```
Summary [ 137.466s] 24 tests run: 24 passed, 5485 skipped   (wat::lint — includes
  every_wat_scripts_file_loads_on_the_current_runtime and
  every_rete_name_in_wat_scripts_code_resolves, the two gates wat-rs/CLAUDE.md names as reading
  every .wat under wat-scripts/ — plus the rete-name classifier's own 18 unit tests)
```

Then ran `./scripts/floor.sh` in the FOREGROUND (not backgrounded — checked `pgrep -af
'cargo|nextest'` clear immediately before), once, clean on the first pass — no red to capture, at
any point in this strike. Read from `.floor/latest/clean.log`:

```
     Summary [ 456.109s] 5490 tests run: 5490 passed (2 slow), 19 skipped
```

**5490 — exactly the pinned number.** No new `#[test]` function was added anywhere; the change is
eight new corpus files (four `.wat` + four `.clj`) plus rows added to five already-existing arrays
(`EXPECTED_FIXTURES`, the census `TABLE`, `USERFN_SNIPPET`, `SIZES`, `SIZED_AXES`,
`CORRECTNESS_SIZES` — six, counting the new `USERFN_SNIPPET` table) that existing tests already
iterate. `.floor/2026-09-09T08-55-14Z/` (symlinked as `.floor/latest`) holds the untruncated log.

## What I did NOT do

- Did **not** tune any fixture after seeing a three-way result. Every fixture's shape (records,
  rules, seed data, retract target, correctness size) was fixed and written into the `.wat`/`.clj`
  pair, with its expected-count formula stated in the header, BEFORE the first
  `check-grid-three-way.sh` invocation for that cell. The one thing changed after a run was the
  `retract-accum-derived.wat` TYPE ERROR fix (a `TypeMismatch` at check time, before any engine
  produced an ANSWER to compare) — not a tuning of the derived-set shape.
- Did **not** re-run any test after a red. There was no red at any point in this strike — the type
  error in cell 5 was a compile/check-time failure caught before a single `:derived` value existed
  to compare, not a three-way mismatch.
- Did **not** add any of the four new fixtures to `run-all.sh`'s `ORDER` — DESIGN's pin honored.
- Did **not** widen the census's five axes or touch its `TABLE`/`AGGREGATION` logic beyond adding
  four rows and the `USERFN_SNIPPET` generalization (itself a correction to a pre-existing bug in
  the mechanical verifier, not a widening of what it checks).
- Did **not** modify any existing fixture (`accum-over-derived.wat`, `accum-lead-rule-cascade.wat`,
  `leading-exists.wat`, `neg-consumer.wat`, `retract-multiplicity.wat`, `userfn-head.wat`, or any
  `where-*.wat`) — read-only reference reads only.
- Did **not** find any cell to be semantically unconstructible. All four of DESIGN's remaining
  cells were built as designed; 0 were declared `cell-unconstructible`.

---

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_018ntHDMRNCKDKNr2gVfzXmP
