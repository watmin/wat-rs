# PERAGRARE — Cast Report (TARGET 4) — **FINDINGS**
## Instrument: `tests/lint/every_wat_bad_fixture_actually_fails.rs`

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> ⭐ The census it proposed is committed at `tests/lint/peragrare-bad-census.sh`; the orchestrator
> re-ran all three pins and the full report independently before recording this.

## Normative block

```
ANCHOR: PASSED — populated pin (clean, owner-ignored) = 3 members
        empty pin (nonexistent-outcome, *) = 0 (a value no axis takes)
        moving pin: one synthetic fixture per non-default axis value, 7 of 7 landed where placed
INSTRUMENT: for each .wat.bad, ask whether startup_from_file(rel) errs (the filename's implicit
        claim); a clean (Ok) start is tolerated only if the file's rune:lint(bad-is-banked)
        declaration is well-formed (known category, reason >=24 chars, owner field) AND names a
        test that exists under tests/ and is still #[ignore]d — else the gate fails.
CENSUS: tests/lint/peragrare-bad-census.sh (proposed path — argued below), committed beside
        the instrument it counts
CORPUS: 268 files = 268 members | 0 never offered | 0 dropped
        membership: a fixture is a member iff it is a *.wat.bad under ROOTS — check_shard()
        loops over every discovered file with no pre-filter, so corpus == members exactly
AXES: 2, both derived from the comparison —
        AXIS1 startup outcome (2: err | clean)
            <- line 244's `.is_err()` is the primary branch the whole gate hinges on
        AXIS2 declaration state (7: absent | bad-category | short-reason | no-owner-field |
                                  owner-absent | owner-live | owner-ignored)
            <- lines 251-301's sequential validity checks on the exemption; each is a distinct
               way the gate's verdict flips for a clean fixture
      not modelled: the fixture's own `;;`-comment-claimed error KIND/law (129 of 268 name one).
      DELIBERATELY REJECTED, not merely cut: changing which error kind fires while staying an
      Err cannot flip this comparison's verdict (still `.is_err()`=true) — it fails the axis
      litmus test (step 2) and is domain colour for THIS instrument, not a grid axis.
CELLS IN GRID:   14 = read 1 | hollow 0 | empty 6 | exempt 7 (cell-unconstructible 7)
FINDINGS:         6 = unvisited 6 (L1) + hollow 0 (L2) + silently-dropped 0 (L1)
                        + instrument-inert 0 + census-void 0
EMPTY, NOT FILED: 0
HYPOTHESES:       6 filed = confirmed 0 | refuted 0 | unchecked 6
COST: ~70 min - 15 to read the instrument and state its comparison, 10 to derive axes (and
      reject "error kind" after the litmus test), 10 to re-derive every handed-down number,
      15 to build+debug the census (mawk's `close` is a reserved builtin — cost a rewrite),
      10 to run and fix the three pins (a bad moving-pin probe self-triggered owner-absent),
      10 to write findings.
VERDICT: the corpus does NOT span the instrument's discrimination space
```

## Findings

**`(clean, absent)`** — **unvisited** (0 of 268 members)
hides: a regression that makes the "no declaration" branch spuriously pass (`declaration_on` false-matching, or the `let Some(...) else { failures.push… }` at lines 251-256 weakened/inverted) would let a truly-clean, undeclared `.wat.bad` sail through silently — no real fixture currently reaches this branch. **Unchecked** — would require executing `startup_from_file`, forbidden in this read-only cast.

**`(clean, bad-category)`** — **unvisited** (0 of 268 members)
hides: a regression in `DECLARED_CATEGORIES.contains(&cat)` (line 260) — e.g. a second category added without updating this call, or the check silently loosened — is never caught, because the only category the entire corpus ever writes is `bad-is-banked` itself. **Unchecked**.

**`(clean, short-reason)`** — **unvisited** (0 of 268 members)
hides: a regression in the `MIN_REASON_CHARS` enforcement (line 268, off-by-one or comparison flipped) is never caught, because all 3 real reasons are far longer than the 24-char floor and none sits near the boundary. **Unchecked**.

**`(clean, no-owner-field)`** — **unvisited** (0 of 268 members)
hides: a regression in `owner_in` (line 142, e.g. matching a decoy `banked-by` occurring elsewhere, or the needle string drifting between writer-convention and reader) is never caught, since every real declaration already carries a well-formed owner field. **Unchecked**.

**`(clean, owner-absent)`** — **unvisited** (0 of 268 members)
hides: a regression that resolves a nonexistent owner name to some *other* real test (a stale `sources` cache bleeding across fixtures in the same shard via `get_or_insert_with`, or a substring rather than exact-`fn` match in `owner_state_in`) would never be caught, because no real fixture currently names an owner that doesn't exist. **Unchecked**.

**`(clean, owner-live)`** — **unvisited** (0 of 268 members)
hides: the gate's own headline promise — *"the day arc 255 lands and the test is un-ignored, this gate goes RED"* (lines 51-53) — has never fired against a real fixture. A bug in `owner_state_in`'s upward attribute scan (lines 190-203, e.g. missing a multi-line attribute or stopping one line early) would silently let a stale exemption survive un-caught, because none of the 3 real banked owners has ever transitioned from `Ignored` to `Live` while this test ran. **Unchecked** — this is the highest-value gap: it is the exact self-clearing mechanism the header cites as this gate's reason for being "checked, both halves."

**EXEMPT** `(err, *)` — **7 cells** — `cell-unconstructible`: `if startup_from_file(rel).is_err() { continue; }` at lines 244-246 skips every declaration read entirely when a fixture errs — no combination of (err, ⟨declaration-state⟩) has a reachable control-flow path. Rune at `tests/lint/every_wat_bad_fixture_actually_fails.rs:244-246` (a control-flow fact, cited rather than an in-repo `;; rune:` line since this is Rust, not `.wat`).

Visited, non-finding cell: **`(clean, owner-ignored)`** — **read** (3 members: `tests/types/probe_diag_typealias_leniency_check.wat.bad`, `tests/wat_lang/probe_undefined_builtin_resolves_bogus.wat.bad`, `tests/wat_lang/probe_undefined_builtin_resolves_wrong_leaf.wat.bad`). Verified by reading both owner `.rs` files (`tests/types/probe_diag_typealias_leniency.rs:16-17`, `tests/wat_lang/probe_undefined_builtin_resolves.rs:17-18,31-32`): all three owners exist and are genuinely `#[ignore]`d, and `check_shard`'s exhaustive `match owner_state(...)` (lines 290-301) computes this state for real against these three files rather than merely defaulting to it.

## The census script — path argument

Proposed path: **`tests/lint/peragrare-bad-census.sh`** — argued rather than the brief's default `tests/rete/`: the corpus here is `tests/**/*.wat.bad` repo-wide (only 19 of 268 sit under `tests/rete/`), so "beside the corpus it counts" has no single directory home; the instrument itself lives at `tests/lint/every_wat_bad_fixture_actually_fails.rs`, and that gate's own header names its nearest sibling (`tests/lint/docs_wat_loads_or_declares_why_not.rs`) as living in the same directory — `tests/lint/` is where this repo already keeps corpus-wide walk-and-declare gates.

*(The full script content was emitted in the return and is committed at that path.)*

## Deltas from every figure in the brief

| Figure | Brief | Re-derived | Delta |
|---|---|---|---|
| Total `.wat.bad` under the 3 roots | 268 | 268 | 0 |
| Under `tests/` | 268 | 268 | 0 |
| Under `wat-scripts/` | 0 | 0 (dir exists, non-empty, 0 `.wat.bad`) | 0 |
| Under `docs/` | 0 | 0 (dir exists, non-empty, 0 `.wat.bad`) | 0 |
| Fixtures naming an intent-comment | 129 of 268 | 129 | 0 |
| `tests/rete/*.wat.bad` | 19 | 19 | 0 |
| `rune:lint(bad-is-banked)` fixtures | 3 (implied by header) | 3, exact filenames confirmed, all owners confirmed `#[ignore]`d | 0 |

No handed-down number moved. One number the brief didn't give and I had to derive myself: the (err, clean) split is **265 err / 3 clean** — grounded in the corpus grep (only 3 `rune:lint(` lines exist anywhere in the 268 `.wat.bad` files), not in execution (see the disclosed AXIS1-inference limitation).

⛔ **One thing I explicitly disagreed with in the brief's own framing:** the ⭐⭐ tension ("the comparison reads one bit… 129 of 268 name an intent it never checks") is real as an *observation* but I did **not** admit "claimed error kind" as a grid axis — it fails the step-2 litmus test (changing it, while staying an `Err`, cannot flip the verdict), so treating it as a grid axis would have been exactly the domain-colour mis-cast the ward opens by warning against. The genuine, comparison-derived thinness is narrower but still real: the exemption *state machine* (7 branches) has driven only 1 of its 7 states through a live fixture, ever — most consequentially the self-clearing `owner-live` branch that is this gate's own advertised reason for existing.

**FINDINGS**
