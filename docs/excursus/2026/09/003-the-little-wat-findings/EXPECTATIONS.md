# EXPECTATIONS — STONE 003.1

Written **before** the strike, so the result cannot move the goalposts.
Scored against my own re-run, never the executor's report.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the board holds its measured verdicts | `cargo nextest run --release -E 'test(/little_wat_findings/)'` | Summary: all tests passed, 0 failed |
| 2 | the vacuity rule is enforced | same run | `no_row_can_be_vacuous` present and passing |
| 3 | ⭐ the gate CAN fail | mutate one row's `expect_run_rc`, re-run | Summary reports that row FAILED, naming the finding id |
| 4 | …and recovers | restore, re-run | Summary: all passed |
| 5 | the meta-gate accepts it | `cargo nextest run --release -E 'test(/non_vacuity/)'` | passes — the `NON-VACUITY` marker is seen and its assert is within 12 lines |
| 6 | the banked probe is untouched | `git diff --stat 68e7de0d3 -- tests/diagnostics/` | **empty** |
| 7 | no `src/` moved | `git diff --stat 68e7de0d3 -- src/ wat/` | **empty** |
| 8 | the floor | `scripts/floor.sh`, read `.floor/latest/clean.log`'s `Summary` | 0 failed. ⚠ the count RISES by the rows added — a rise is expected, a FAILURE is not |

⛔ **Row 8 is read from the `Summary` line, never from a piped exit code.** A `cargo nextest … | tail`
returns `tail`'s status. This session already saw the harness print `[exited with code 0]` over a
run whose Summary said **2 failed**.

## Runtime prediction

Writing: 30–50 min. ⚠ **Build cost dominates and is easy to under-call:** a cold release build of
the test harness on this host measured **3m 15s**, and `cargo build --release` alone **1m 07s**.
Budget **four separate nextest runs** (green, mutated, restored, floor) — the floor is ~8 min on
6,011+ tests. Total 60–90 min, most of it compiling.

⛔ **Do not poll for a build with `pgrep -f 'cargo …'` — the pattern matches its OWN command line
and the loop never exits.** That cost this session an hour of spinning processes the builder had
to kill by hand. Run the build in the foreground, or background it once and read its output file.

## Trap doors, named in advance

1. **`main` moves under us.** Another session is striking `main` (255.13 drawn 13:04 today). Any
   row can legitimately flip between the draw and the landing. STOP-1 covers it: a flip is a
   finding, never a number to adjust.
2. **A fixture stops being a specimen.** `probes` here pin *defects*. If someone "fixes" a fixture
   to make a red go green, the board silently stops measuring. Every fixture carries a header
   comment saying so; row 6 checks the banked one was not touched.
3. ⚠ **F-083's spelling.** Its program as written in `recheck.sh` uses `HolographicLru::new`, which
   `main` has RETIRED. Written that way, the fixture fails at CHECK and the row reads `(1, 3)` —
   a plausible, wrong verdict that looks like a STRICT finding. The retirement is exactly what
   blinded their own instrument to `?`.
4. **The floor count.** It RISES with the added rows. A rise is not a regression; a FAILURE is.
   Do not let a changed total read as either success or breakage on its own.
5. ⚠ **`no_row_can_be_vacuous` may be the only test that can never fail.** It asserts over a
   constant table, so on a board where no row is `(0,0)` it is vacuous itself. F-083 IS `(0,0)`,
   so it has a live subject today — but if F-083 ever leaves the board, that test must be
   re-derived or removed. Named now so it is not discovered as a surprise later.
