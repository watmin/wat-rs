# BRIEF — STONE 301.1: the findings board

**Drawn 2026-09-22 against `reason/little-wat-findings` @ `68e7de0d3`** (off `main` @ `600abe8c3`).
Read `DESIGN.md` beside this first — it holds the measured table and the one contract decision.

## The work, in one paragraph

Build one table-driven gate in `tests/lint/` that drives the **binary, twice** over a fixture per
finding from the sibling repo `the-little-wat`, and asserts the `(check rc, run rc)` pair measured
at HEAD. Eight rows, listed below with their measured verdicts. The gate changes **no `src/`**: it
records what is true today so that every later cure announces itself by flipping a row.

## Read in order, and why you are being sent there

1. **`tests/diagnostics/probe_arc301_silent_failure_pair.rs`** — the banked probe, committed at
   `68e7de0d3`. **Copy its `fixture()` and `rc()` helpers verbatim.** Its module doc holds the
   driver decision and its reasoning; do not re-derive them. It also shows the shape of a failure
   message that tells the reader what to DO when the row flips.
2. **`tests/diagnostics/probe_arc301_silent_failure_pair__f031_length_on_string.wat`** — the
   fixture shape: a header comment saying *why this file is a specimen and must not be "fixed"*.
   Every fixture you write carries one.
3. **`tests/lint/every_walking_gate_declares_non_vacuity.rs:45-50`** — ⛔ **THIS GATE WILL JUDGE
   YOURS.** Any `tests/lint/*.rs` containing `Command::new` is in its population and must carry
   either a `NON-VACUITY` comment with an assertion **within 12 lines below it**
   (`ASSERT_WINDOW`), or a `rune:lint(vacuity-guard) — <reason>` of at least 40 characters naming
   a mechanism. Write the in-code form: a table that silently became empty must red.
4. **`tests/lint/diagnostic_output_is_deterministic.rs:196`** — the house `const TABLE: &[(…)]`
   style. Take the table shape. ⛔ **Do NOT take its pinned length** — that is right for a
   quarantine that must shrink and wrong for a board that must grow (`DESIGN.md` says why).
5. **`tests/lint/every_wat_bad_fixture_actually_fails.rs:1-35`** — why these fixtures are `.wat`
   and never `.wat.bad`: `.wat.bad` asserts failure at **startup**, and six of our eight rows
   start up clean. `tests/` is excluded from `every_ungated_wat_checks` (that file's
   `GATED_PREFIXES`, line 29), so a fixture that dies at eval is safe there — verified, not assumed.

## The eight rows, measured this session on `main` @ `600abe8c3`

| id | fixture content | check | run | shape |
|---|---|---|---|---|
| F-031 | `(:wat::core::length "abc")` | 0 | 1 | LENIENT |
| F-045 | `(:wat::core::first (:wat::core::Vector :- [:wat::core::i64]))` | 0 | 1 | LENIENT |
| F-090 | `(:wat::rational::to-f64 (:wat::rational::+ 1/2 1/2))` | 0 | 1 | LENIENT |
| F-058 | `(:wat::core::PersistentMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::core::i64])])` | 1 | 3 | STRICT |
| F-080 | `(:wat::core::filterv :u::odd? (:wat::core::PersistentVector :- [:wat::core::i64] 1 2 3))` | 1 | 3 | STRICT |
| F-088 | `(:wat::stream::collect 1)` | 1 | 3 | STRICT (absent) |
| F-093 | `(wat.core/defn bad/ident [n :- wat.type/i64] :- wat.type/i64 n)` called with `"a String"` | 1 | 3 | **CURED** |
| F-083 | `HolographicLru/new` → `put k v1` → `put k v2` → `len` | 0 | 0 | **WRONG-ANSWER**, stdout `0` |

The exact programs are in `~/work/holon/the-little-wat/tools/recheck.sh` lines 33–90. ⚠ **F-083's
is written there in a spelling `main` RETIRED** (`HolographicLru::new` → `HolographicLru/new`);
use the current spelling — that retirement is what turned its verdict into a `?`.

## Implementation sketch

```rust
/// (id, fixture stem, expect_check_rc, expect_run_rc, stderr_fragment, expect_stdout, shape)
const BOARD: &[(&str, &str, i32, i32, Option<&str>, Option<&str>, Shape)] = &[ … ];

#[test] fn every_row_holds_its_measured_verdict() { … }

/// ⛔ A row asserting (0, 0) with no expect_stdout can never fail.
#[test] fn no_row_can_be_vacuous() {
    for r in BOARD { if r.2 == 0 && r.3 == 0 { assert!(r.5.is_some(), "…"); } }
}
```

## Blast radius

`tests/lint/` only — one `.rs`, eight `.wat` fixtures, and the arc doc. **No `src/`. No `wat/`.
No change to the committed probe.** No new dependencies, no new types outside that file.

## STOP triggers

1. **A measured verdict disagrees with the table above.** Do NOT adjust the table to match and do
   NOT adjust the fixture to restore the number. STOP and report both verdicts with the fixture
   you ran — `main` moves under this branch, and a genuine flip is a finding worth more than the
   stone.
2. **A row would be `(0, 0)` and you cannot capture a stable stdout for it.** STOP. That row
   measures nothing; report it rather than landing it.
3. **Any gate in `tests/lint/` goes red that you did not add.** STOP and report the whole
   untruncated failure block, the test name and the arm. Do not re-run first.
4. **The cure for a finding becomes tempting.** Out of scope by design (`DESIGN.md`): a board that
   moves the number under its own instrument has measured nothing. Report the idea; land no `src/`.

## Prior comparable result

`68e7de0d3`'s commit message is the shape to copy for your own: what was measured, on what commit,
what the mutation proof was, and what the instrument CANNOT see stated plainly.

## Prove the gate

⛔ **A gate that has never failed is not a gate.** Mutate one row's expected rc, show it RED,
restore it, show it GREEN — and quote the `Summary` line for all three.
