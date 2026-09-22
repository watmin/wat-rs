# DESIGN — arc 301: wat-rs answers `the-little-wat`

**Opened 2026-09-22 on `reason/little-wat-findings`, off `main` @ `600abe8c3`.**
Host: `reason`. ⛔ `main` is LIVE — 255.13 was drawn at 13:04 the same day — which is why this
work is on a branch and not on main.

## Where the findings come from

`~/work/holon/the-little-wat` (cloned beside wat-rs, its `audit.sh` expects exactly that layout)
is Friedman's *Little* books plus a dozen more suites worked through in wat **to find where wat
breaks**. `FINDINGS.md` is 14,315 lines: **172 distinct findings**, 6 deliberate refusals, 183
clean-port results, each with a repro among **364 probes**, every diagnostic quoted verbatim.
Its own rule: *"Nothing is claimed from wat-rs's docs. Every 'wat can't' is a probe that ran."*

⛔ **It is a SNAPSHOT, stamped `2026-09-15, wat-rs a3218644d`.** Its own header says so. Individual
entries carry **no status field**. The only automated OPEN/FIXED signal is `tools/recheck.sh`,
which covers **11 of 172**. Nothing in that repo can tell a cured finding from a live one at our HEAD.

## Measured at HEAD, not inherited

Run on `main` @ `600abe8c3` this session, binary freshly built:

| finding | `--check` | run | shape |
|---|---|---|---|
| F-031 `length` on a String | 0 | 1 | LENIENT |
| F-045 `first` of an empty Vector | 0 | 1 | LENIENT |
| F-090 `rational/to-f64` on a collapse | 0 | 1 | LENIENT |
| F-058 `PersistentMap` nested bracketed type | 1 | 3 | STRICT |
| F-080 `filterv` on a `PersistentVector` | 1 | 3 | STRICT |
| F-088 `:wat::stream::collect` | 1 | 3 | STRICT (absent name) |
| F-093 clj-spelled wrong-typed argument | 1 | 3 | ✅ **CURED** — was (0,0); 8c, `bc93125aa` |
| F-083 `HolographicLru` re-put | 0 | 0 | ⛔ WRONG-ANSWER |

Also measured: `doc-names` **86 of 203** taught-and-rejected, up from their recorded **81**;
exactly five names flipped EXISTS→RETIRED, all `Type::method` → `Type/method`
(`Bytes::from-hex`, `Bytes::to-hex`, `HandlePool::{finish,new,pop}`).

## ⛔ Why the INSTRUMENT before any FIX — derived, not preferred

**Nearly every one of these cures is a small language-design RULING, not a mechanical fix.**
F-045 appears in their relay under **Fix** (*"first and rest die on an empty collection"*) AND
under **Extend** (*"first/rest total, like last"*) — refuse at the checker, or return an Option
and change every caller. Those are opposite languages. F-031 is the same question for `length`.

An orchestrator cannot brief that. A board can be built with no rulings at all, it makes every
later ruling measurable, and it pins F-093 so 8c cannot silently regress.

## What 301.1 builds

A **table-driven gate** over the program-level findings. One row:

```
(id, fixture, expect_check_rc, expect_run_rc, expect_stderr_fragment, expect_stdout, shape)
```

Adding a finding is adding a row; the whole ledger is legible in one place.

### The ONE contract decision

⛔ **THE DRIVER IS THE BINARY, TWICE — `wat --check F` then `wat F`.**

`tests/lint/every_wat_bad_fixture_actually_fails.rs` records that the binary and the in-process
`startup_from_file` give **OPPOSITE verdicts on the same files**, and that a prior strike was
withdrawn for choosing wrong. Every finding here is a claim about what a user experiences running
`wat foo.wat`, and the in-process driver **cannot express the second half of the pair at all** —
it does not evaluate. Ported onto it, this entire class would read as *absent*.

### The structural invariant — a vacuous row must be UNREPRESENTABLE

⛔ **A row asserting `(0, 0)` with no `expect_stdout` can never fail.** It is a row that measures
nothing while looking like coverage. The gate REFUSES such a row — not by convention, by an
assertion over the table itself. This is the F-083 trap generalised: exit codes cannot see a
wrong value, so a finding whose whole content is a wrong value must pin the value.

No pinned row-count. A pinned length is right for a quarantine that must SHRINK; this board must
GROW, and a count would cap coverage downward while looking like rigour.

### Relationship to the banked probe

`tests/diagnostics/probe_arc301_silent_failure_pair.rs` (`68e7de0d3`) stays. It is the **depth**
example — it alone asserts the death happens for F-031's *own* reason (a `RuntimeError` naming
`:wat::core::length`), because `assert_ne!(rc, 0)` is satisfied by any startup failure at all.
The board is **breadth**. Deliberate, not duplication.

## Out of scope — REJECTED, not deferred

- **F-085** (USER-GUIDE uuid note), **F-087** (86/203 doc names), **F-089** (CLI `--help`),
  **F-096** (defrecord/defstruct read ratio). Three *other* instruments — a doc audit, a CLI
  surface probe, a benchmark. Not this gate, and not this strike.
- **Any cure.** 301.1 changes no `src/`. A board that moves the number under its own instrument
  cannot be trusted to have measured anything.
- **Extending `the-little-wat`'s own `tools/recheck.sh`.** That repo is being actively pushed by
  another session (8 commits on 2026-09-22). Committing there is a collision, not a strike.
