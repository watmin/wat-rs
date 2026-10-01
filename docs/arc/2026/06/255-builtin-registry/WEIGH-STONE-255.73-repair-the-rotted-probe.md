# WEIGH — STONE 255.73: repair the rotted probe, and make it run — ACCEPTED

**Executor: a Sonnet subagent, commit `9a54f673c`.** Weighed by the orchestrator on 2026-10-01.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor at `9a54f673c` | `.floor/2026-10-01T01-32-01Z`: **6236 passed / 24 skipped**, exit 0 (6235 + this stone's test) |
| the new test on that floor | `PASS wat::process probe_arc170_s3b_astsplice::probe_s3b_astsplice_runs_and_prints_6_10` |
| clippy | `cargo clippy --release --all-targets -- -D warnings`, exit 0 |
| the probe, run directly | `./target/release/wat wat-scripts/probes/arc-170/probe-s3b-astsplice.wat` → `"6 10"`, rc 0 |
| the census's sharpest row | `probe-compound-upcast.wat` reproduced: rc 2, `panicked at src/value/value.rs:913:37 … Value::RustOpaque is not atomizable; … the predicate has drifted` |
| probe count | `git ls-files 'wat-scripts/probes/**/*.wat'` → 83 |

## What landed, read in the diff

- **The claim is kept:** the probe still proves 259 S3b Blocker A's derive-and-splice (a work-fn's types reflected off
  `fn-forms`, spliced into a shipped runner's `Peer`/`self-peer` types, spawned, round-tripped), with the same
  `EXPECT "6 10"`.
- **Line 59's angle-bracket name** is now the type form `(Head :- [args])`, spliced from the type nodes already in scope.
  There is no string round trip.
- **The `Peer` slot order changed.** The old string had `Peer<(i64,arg),(i64,ret)>`; the new form is
  `(Peer :- [(i64,ret) (i64,arg)])`. The agent called this send-first. **Checked against the precedent:**
  `wat/bracket.wat:425` is `runner-self-kw (Peer :- [~sp-out ~sp-in])` with `sp-out = (Tuple :- [i64 ~ret-ty])`. So
  the repair matches the precedent. ⚠ This probe's arg and return are both `i64`, so its run **cannot discriminate**
  the order. The precedent is the evidence; the run is not.
- **A second defect** was reached for the first time once line 59 stopped raising: the runner bare-bound `(recv prn)` as
  the payload, and its `send` match lacked `SendOutcome.Stopped`. It now matches on `RecvOutcome`, as `dial-runner` does.
- **The wire-in:** `tests/process/probe_arc170_s3b_astsplice.rs` asserts rc 0 and stdout exactly `"6 10"\n`. A future rot
  of this file is now a floor red.

## The census (the measurement for the builder)

**83 probes; 82 are run by nothing** (only the loader gate type-checks them). Run today: **21 crash, 61 exit 0**. The
SCORE has the full table. Classified by the orchestrator from that table:

| class | count | probes |
|---|---|---|
| Rust panic, a drifted invariant | 1 | `probe-compound-upcast.wat` (reproduced above) |
| Rust panic `disconnected` (`src/freeze.rs:1542`) | 1 | `arc-278/s2s-revoke-probe.wat` |
| this stone's class: bare `RecvOutcome` + missing `Stopped` | 5 | `probe-generic-shipped`, `probe-s1-fn-forms`, `probe-s1-named`, `probe-s3-process-runner`, `probe-s3c-rendezvous` |
| `ast-name` refuses the type node (the line-59 class) | 3 | `probe-c1-ast-shape`, `probe-c1-plain-fnforms-shape`, `probe-m1-argcount` |
| red by design, by their own docs/names (not verified further) | 2 | `probe-fnforms-keyword-err`, `probe-s3b-crux-fnforms-closure` |
| a fragment with no `:user::main` | 1 | `w3-n-dial-runner.wat` |
| other wat-level failures, each needing a read | 8 | the rest of the table |

**And the root:** only **21 of 83** probes declare an `EXPECT` header, so for 62 of them there is no written claim a
runner could check. The 61 that exit 0 today are "not crashing", not "proving what they say".

## Verdict

Accepted, pushed. The rot class (a `.wat` probe that is type-checked and never run) is open across 82 files; what to do
about it is the builder's ruling.
