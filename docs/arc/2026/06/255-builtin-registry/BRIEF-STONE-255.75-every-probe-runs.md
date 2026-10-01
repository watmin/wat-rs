# BRIEF — STONE 255.75: every probe runs — the gate, and the twenty probes that fail today

**Drawn 2026-10-01 against `main` @ `a63500e04`.** **Executor: a Sonnet subagent.** A strike: one gate test, twenty probe
dispositions. Commit locally on `main` (`git add -- <paths>`, never `-A`); **do not push**. Your final message is your
report.

## The ruling (builder, 2026-10-01): E3

**A probe asserts its own claim in wat. The floor runs every probe and requires exit 0. A probe that must fail is a
`.wat.bad` with a driven test naming its error.** So a probe nothing runs cannot exist, and "exit 0" means "the claim
held". This stone lands the gate and the twenty probes that do not exit 0 today. The next stone (255.76) writes
assertions into the 39 probes that only print.

## The measurement (orchestrator, at `a63500e04`, `./target/release/wat <probe>`, 10 s timeout, stdin `/dev/null`)

83 probes under `wat-scripts/probes/**/*.wat`; all 83 together take 52 s sequentially, none timed out. 63 exit 0.
**These 20 do not** (rc, file); 255.73's SCORE § 4 has each one's first error:

```
2 arc-170/probe-bracket-cause.wat            1 arc-170/probe-c1-ast-shape.wat
1 arc-170/probe-c1-plain-fnforms-shape.wat   1 arc-170/probe-cap2-peer-pid.wat
2 arc-170/probe-child-inherits-defns.wat     1 arc-170/probe-defclause-discriminate.wat
1 arc-170/probe-fnforms-keyword-err.wat      2 arc-170/probe-generic-shipped.wat
1 arc-170/probe-m1-addr-roundtrip.wat        1 arc-170/probe-m1-argcount.wat
2 arc-170/probe-m1-cf-norevoke.wat           2 arc-170/probe-m1-fix-revoke.wat
2 arc-170/probe-s1-fn-forms.wat              1 arc-170/probe-s1-impure-gate.wat
2 arc-170/probe-s1-named.wat                 2 arc-170/probe-s3-process-runner.wat
1 arc-170/probe-s3b-crux-fnforms-closure.wat 2 arc-170/probe-s3c-rendezvous.wat
4 arc-170/w3-n-dial-runner.wat               2 arc-278/s2s-revoke-probe.wat
```

Known so far: five share 255.73's `RecvOutcome` bare-bind + missing `SendOutcome.Stopped` class (`generic-shipped`,
`s1-fn-forms`, `s1-named`, `s3-process-runner`, `s3c-rendezvous`; 255.73 repaired the same class in
`probe-s3b-astsplice.wat`, copy that repair). Three hit the `ast-name` refusal that 255.73's line-59 repair answered
(`c1-ast-shape`, `c1-plain-fnforms-shape`, `m1-argcount`). `crux-fnforms-closure` is named red by design in
`docs/arc/2026/06/259-forced-hand/NOTE-S3b-blockers-and-resolution.md`, though its header says `EXPECT "6 10"`.
`fix-revoke`'s header expects a raise. `s2s-revoke-probe.wat`'s header says its outcome depends on a race.
`w3-n-dial-runner.wat` has no `:user::main`.

## The work

1. **Per probe, recover its claim** from its header and the arc doc that names it (`git grep -l <basename> -- docs/`), then
   give it exactly one disposition, with the evidence in the SCORE's table (probe · claim in one line · disposition · why):
   - **repair**: the claim still holds in today's language; fix the retired forms the run reaches (as 255.73 did), and
     add assertions so the probe checks its claim itself (`:wat::test::assert-eq` or `assertion-failed!` on data, never
     on a string rendering). It must exit 0.
   - **negative**: the probe exists to show a refusal or a gap (red by design). Rename it `.wat.bad`, and a driven Rust
     test asserts its error kind and names (255.74's `tests/types/probe_arc255_74_key_must_be_data.rs` is the shape).
     Correct a header that says otherwise.
   - **retire**: its claim is superseded (the thing it probed no longer exists, or a later stone made it moot). `git rm`
     it; the SCORE row names what superseded it (commit or doc).
   - **fragment**: a file that is not a program (no `:user::main`) and something else loads: say what loads it; move it
     out of `probes/` beside its consumer, or retire it if nothing does.
2. **The gate.** A driven test runs every `wat-scripts/probes/**/*.wat` (not `.wat.bad`) with the release binary and
   requires exit 0, **one test per probe** so a red names the probe and nextest runs them in parallel (the
   `tests/process/` `build.rs` registration, or a generated list: say which), each with a timeout. A new probe is picked
   up with no edit. Mutation-prove it once (break one probe, see its test red, restore). The two existing driven probe
   tests (`probe_arc170_s3b_astsplice.rs`, `probe_arc255_74_compound_upcast_runs.rs`) stay.

## Gates

| what | how | expected |
|---|---|---|
| every probe | the new gate | all pass; the probe count stated |
| release floor | `scripts/floor.sh`, **one run at a time, in the foreground, nothing else running** | all passed; the count against 6257 at `2ffcdfa3e`, plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change (the gate finding a probe, a renamed fixture, the loader gate): capture it
  **verbatim** from `.floor/<stamp>/`, cure it, run a **new** floor. Never re-run unchanged code for a green. The prior
  green floor is `.floor/2026-10-01T05-33-41Z` (6257/6257).
- **STOP-1:** a probe whose claim depends on a race or on timing, and cannot be made deterministic by ordering it (do the
  revoke before the connect, wait on an ack the code already sends). Never retry it, and never hide it as a `.wat.bad`.
  Report the race with its evidence and STOP. `s2s-revoke-probe.wat` is the known candidate: its header says caller2's
  refusal depends on a revoke landing before caller2's connect. The orchestrator ran it five times: byte-identical
  output each time (`"echo:hi"`, `"revoke-midlife-ok"`, then a wat `AssertionFailure` `"disconnected"` raised in
  `:user::main`, rc 2), so it is deterministic today but racy by design. Repair it by ordering if the code allows (and
  say how); otherwise this STOP.
- **STOP-2:** a probe fails because of a substrate defect (a Rust panic, a wrong result from a builtin, a refusal the
  language should not make), not because the probe is stale. Quote it verbatim and STOP; that is its own stone, as
  255.74 was.
- **STOP-3:** a claim you cannot recover from the header and docs. List it and STOP.
- A STOP means STOP. If one STOP fires, finish every other probe's disposition first, then stop before the gate.

## Doctrine

`holon/CLAUDE.md` binds you. These are named individual files (edit them directly and say so; a codemod is for
corpus-wide migrations). Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a number,
file:line or example you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.75-every-probe-runs.md` beside this brief, commit it, **do not push**.
