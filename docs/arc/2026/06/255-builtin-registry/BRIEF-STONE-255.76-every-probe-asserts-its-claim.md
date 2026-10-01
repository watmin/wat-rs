# BRIEF — STONE 255.76: every probe asserts its claim — the thirty-nine that only print

**Drawn 2026-10-01 against `main` @ `519035e69`.** **Executor: a Sonnet subagent.** A strike over 39 named probe files and
their headers; no `src/` change expected. Commit locally on `main` (`git add -- <paths>`, never `-A`); **do not push**.
Your final message is your report.

## The ruling (builder, 2026-10-01): E3

A probe asserts its own claim in wat; the floor runs every probe and requires exit 0 (255.75 landed that gate:
`build.rs` → `tests/process/every_probe_runs.rs`, one test per probe). **Exit 0 must mean "the claim held".** Today 39
probes under the gate contain no assertion: they print, and exit 0 means only "did not crash".

## The 39 (orchestrator, at `519035e69`: no `assert-eq` / `assertion-failed!` / `:wat::test::assert`; number = arc docs naming it)

```
arc-054/probe-054-fn-idempotency.wat (2)            arc-170/probe-fnforms-keyword.wat (0)
arc-170/probe-reason-downcast.wat (3)               arc-170/probe-c2-narrow-2param-plainrecord.wat (1)
arc-170/probe-fnforms-shape.wat (2)                 arc-170/probe-recordtype-fixed.wat (1)
arc-170/probe-c2-narrow-multisurface.wat (2)        arc-170/probe-freeze-narrow.wat (1)
arc-170/probe-s2-runner-count.wat (2)               arc-170/probe-c2-nonparam-baseline.wat (1)
arc-170/probe-kwargs-struct.wat (1)                 arc-170/probe-s3-bracket-loci.wat (5)
arc-170/probe-cap2-isolate.wat (4)                  arc-170/probe-locus1-generic-surface-method.wat (3)
arc-170/probe-s3a-select-peer.wat (1)               arc-170/probe-cap2-process-grantpath.wat (7)
arc-170/probe-locus2-abstract-surface-dispatch.wat (1)  arc-170/probe-s3b-extract.wat (2)
arc-170/probe-cap2-spawnrunner-pid.wat (2)          arc-170/probe-m1-apply-generic.wat (1)
arc-170/probe-strikeB-fields.wat (3)                arc-170/probe-compound-upcast.wat (13)
arc-170/probe-m1-arity.wat (1)                      arc-170/probe-thread-only.wat (1)
arc-170/probe-defclause-open-arg.wat (2)            arc-170/probe-m1-erase-only.wat (3)
arc-170/probe-trivial.wat (0)                       arc-170/probe-defclause-real-shape.wat (4)
arc-170/probe-m1-service-pid.wat (4)                arc-170/probe-type-splice.wat (0)
arc-170/probe-deporder.wat (0)                      arc-170/probe-mod-in-macro.wat (1)
arc-170/root-gapA.wat (2)                           arc-170/probe-edn.wat (3)
arc-170/probe-nested-vector-of-tuples.wat (5)       arc-293/s3-probe-struct-satisfies-nature-struct.wat (1)
arc-170/probe-edn2.wat (0)                          arc-170/probe-process-only.wat (3)
arc-293/s4c-carrier-probe.wat (1)
```

(All paths under `wat-scripts/probes/`.) 255.75's SCORE noted that some headers carry dated or stale `EXPECT` prose
(`probe-strikeB-fields.wat` still says "RED before B" though it exits 0 today; read each header against what the
probe prints today).

## The work

For **each** of the 39, recover its claim (its header, and `git grep -l <basename> -- docs/`), then give it exactly one
disposition, recorded in the SCORE's table (probe · the claim in one line · disposition · the assertion or the reason):

- **assert**: the claim is about a value the probe computes. Add the assertion on **data**
  (`:wat::test::assert-eq` on the value itself, or a `match` whose wrong arms call `assertion-failed!`), never on a
  printed or rendered string. A value that cannot be pinned (a pid, a timestamp, an address) is asserted by its shape
  (`Option.Some`, a positive `i64`, the variant), not its digits. The print may stay.
- **check-claim**: the claim is that a program **type-checks or starts up** (the header says `--check`, "freezes",
  "COMPILES", "is accepted"). Exit 0 already is the claim; add no ceremonial assertion. Make the header say so in one line
  (`;; CLAIM (exit 0): <the claim>`), so the next reader knows exit 0 is the proof.
- **retire**: the claim is superseded (what it probed no longer exists, or a later stone or test owns it). `git rm` it;
  the SCORE row names what superseded it (commit, test or doc).

Every kept probe's header ends with a one-line `;; CLAIM:` (or `;; CLAIM (exit 0):`) stating what it proves, and any
stale `EXPECT` prose in it is corrected or removed. The five with no doc (`probe-fnforms-keyword`, `probe-trivial`,
`probe-type-splice`, `probe-deporder`, `probe-edn2`) get their claim from their own text or are retired.

**Prove the assertions bite:** for at least five `assert` probes (say which), change the asserted expectation, see the
probe's gate test go red by name, restore.

## Gates

| what | how | expected |
|---|---|---|
| every probe | `cargo nextest run --release -E 'test(/every_probe_runs/)'` | all pass; the count stated (70 minus retired) |
| no print-only probe left | the orchestrator's search above, re-run | every remaining hit is a `check-claim` with its `CLAIM (exit 0)` line |
| release floor | `scripts/floor.sh`, **one run at a time, in the foreground, nothing else running** | all passed; the count against 6337 at `82af2f167`, minus retired probes |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change (a new assertion that fails, a retired probe still referenced by a doc-path
  lint): capture it **verbatim** from `.floor/<stamp>/`, then decide: if the assertion encodes the claim and the code
  disagrees, that is **STOP-1**, not a cure. If the assertion was wrong, fix it and run a **new** floor. Never re-run
  unchanged code for a green. The prior green floor is `.floor/2026-10-01T07-24-44Z` (6337/6337).
- **STOP-1:** an assertion that states the probe's recovered claim fails: the language no longer does what the probe says
  it does. Quote it verbatim and STOP on that probe; finish the others first.
- **STOP-2:** a claim whose value differs between runs (anything beyond a pid/time/address you assert by shape). Never
  retry it; report it with both outputs and STOP on that probe.
- **STOP-3:** a claim you cannot recover from the header, the code and the docs. List it and STOP on that probe.
- A STOP means STOP. If one fires, finish every other probe, then stop before the floor.

## Doctrine

`holon/CLAUDE.md` binds you. These are 39 named files, each a different hand-written assertion: edit them directly and say
so (a codemod is for one structural rewrite across many files). Equality is data equality, never strings. Capture `rc=$?`
on the next statement. Never wait with `pgrep -f`. Run every build, floor and clippy in the foreground. Never write a
number, file:line or example you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.76-every-probe-asserts-its-claim.md` beside this brief, commit it, **do not push**.
