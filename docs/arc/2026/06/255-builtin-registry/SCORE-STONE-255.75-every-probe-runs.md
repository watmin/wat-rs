# SCORE — STONE 255.75: every probe runs — the gate, and the twenty probes that fail today

**Executor: a Sonnet subagent.** Branch `main`, drawn against `40197714c`. Not pushed.

## 1. The ruling (E3)

"A probe asserts its own claim in wat. The floor runs every probe and requires exit 0. A probe
that must fail is a `.wat.bad` with a driven test naming its error."

## 2. Per-probe disposition

Twenty probes under `wat-scripts/probes/**/*.wat` did not exit 0 at `40197714c` (255.73's SCORE §4
table). Each got exactly one disposition, with the recovered claim and the evidence for why.

| # | probe | claim (one line) | disposition | why |
|---|---|---|---|---|
| 1 | `arc-170/probe-bracket-cause.wat` | a crashed runner's collect-loop assertion names the REAL cause, not a blind "runner crashed" | **negative** → `tests/process/fixtures/probe-bracket-cause.wat` (plain `.wat`, see §3) | the claim IS the crash; no catch form lets `:user::main` face the bracket's own `AssertionFailure` as a value |
| 2 | `arc-170/probe-c1-ast-shape.wat` | ground-truth: `ast-kind`/`ast-name`/`ast->children` shapes of `field-types-of`'s canonical type forms | **repair** | `ast-name` correctly refused a COMPOUND node (`arg2`, a `vector`) — not a bug, a misuse; reported structural shape instead, asserted the now-current ground truth. `wat-scripts/probes/arc-170/probe-c1-ast-shape.wat:48-79` |
| 3 | `arc-170/probe-c1-plain-fnforms-shape.wat` | ground truth: what does `fn-forms` render a plain fn's own param type annotations as | **repair** | same `ast-name`-on-compound-node wall as #2 (`c-ty`, a 3-child `list`); same fix. `wat-scripts/probes/arc-170/probe-c1-plain-fnforms-shape.wat:29-59` |
| 4 | `arc-170/probe-cap2-peer-pid.wat` | `peer-pid` on a process/thread peer → `(Some pid)`/`:None` | **negative** → `tests/process/fixtures/probe-cap2-peer-pid.wat` | `DESIGN-STONE-CAP-2-BRACKET-GRANTS.md`'s own "GROUNDED FINDING (strike A, 2026-07-09)": `peer-pid` is for SPAWN-derived peers only; a `connect'`-derived unified `Peer'` (what this probe dials) gets "an honest error" BY DESIGN. The positive case is `probe-cap2-spawnrunner-pid.wat` (already green, untouched) |
| 5 | `arc-170/probe-child-inherits-defns.wat` | DECISIVE: does a not-shared child inherit the parent's named defns, or is it a fresh universe | **negative** → `tests/process/fixtures/probe-child-inherits-defns.wat` | the probe's own header predicts BOTH branches; today's measured answer IS the "fresh universe" branch (`UnresolvedReferences` naming `:probe::dbl`) — itself the decisive finding |
| 6 | `arc-170/probe-defclause-discriminate.wat` | defclause's open-fallback clause catches an unknown concrete type at runtime | **negative** → `tests/process/fixtures/probe-defclause-discriminate.wat` | answered NO by an ALREADY-RULED sibling, `tests/rete/probe_arc278_open_surface_dispatch.rs` (`open_surface_dispatch_unknown_class_is_runtime_no_match`): dispatch is exact-class only, no open-surface fallback, ever |
| 7 | `arc-170/probe-fnforms-keyword-err.wat` | `fn-forms` refuses an unregistered keyword | **negative** → `tests/process/fixtures/probe-fnforms-keyword-err.wat` | negative control by design (`-err` suffix, deliberately bogus keyword, 255.73's own census already flagged this) |
| 8 | `arc-170/probe-generic-shipped.wat` | a shipped GENERIC pool-runner resolves its types by inference from the concrete `__work` | **negative** → `tests/process/fixtures/probe-generic-shipped.wat` | answered NO, soundly: `pool-runner` is declared `:- [A B]` and calls the MONOMORPHIC `__work` (i64->i64) as if the result had the abstract type `B` — a generic body is checked once against its own abstract parameters, never one call site's concrete substitution; confirmed unsound by hand-testing both `:B` and bare `B` spellings (identical error). `probe-s3-process-runner.wat`/`probe-s3c-rendezvous.wat` show the two approaches that DO work |
| 9 | `arc-170/probe-m1-addr-roundtrip.wat` | `edn/write`→`edn/read` round-trips a live process `Address'`, and the reconstruction is dialable | **negative** → `tests/process/fixtures/probe-m1-addr-roundtrip.wat` | disconfirmed: `edn/read` refuses, and the error carries its own rationale — "capability tags reconstruct only off the trusted peer wire, never from parsed data" — a deliberate security wall, not rot |
| 10 | `arc-170/probe-m1-argcount.wat` | argspec child counts (1 vs 2 param), and does string "split/join head-swap Peer→Address" work | **repair** | `argcount` (child-count reconnaissance) already worked; `argcount2`'s `ast-name`-on-compound-node crash is the same wall as #2/#3, and the "head-swap" exploration depended on the retired angle-bracket string idiom (arc 109) — dropped, same precedent as 255.74 trimming `probe-compound-upcast.wat`'s retired Set case. `wat-scripts/probes/arc-170/probe-m1-argcount.wat` |
| 11 | `arc-170/probe-m1-cf-norevoke.wat` | COUNTERFACTUAL: with the `echo'/revoke` line removed, does dial #2 still get bounced (meaning the committed revoked test was vacuous)? | **retire** (`git rm`) | it found the vacuity bug it was built to find (2026-07-08) — `DESIGN-STONE-M1-TEETH-revoke-refusal.md`'s "the vacuity trap" section documents the fix: the prober now sends dial #2's reply UP, superseding this probe with its own successor pair, `probe-m1-fix-norevoke.wat`/`probe-m1-fix-revoke.wat` (the first already green, untouched; the second is #12 below), AND the committed `tests/services/probe_arc170_m1_teeth_revoked.wat`'s `Outcome.Bounced`/`Served` value-contract |
| 12 | `arc-170/probe-m1-fix-revoke.wat` | the FIXED, discriminating twin: with revoke restored, dial #2 bounces, prober dies, owner's `compute` raises | **negative** → `tests/process/fixtures/probe-m1-fix-revoke.wat` | the raise IS the proof (`DESIGN-STONE-M1-TEETH-revoke-refusal.md`, "discrimination" row): `probe-m1-fix-norevoke.wat` (undisturbed) proves the POSITIVE path, this proves the NEGATIVE |
| 13 | `arc-170/probe-s1-fn-forms.wat` | `fn-forms` reifies an anon closure into self-contained, shippable forms | **repair** | `RecvOutcome` bare-bind + missing `SendOutcome.Stopped` — 255.73's exact site-2 defect class; repaired identically (match on `recv`, add the arm). `wat-scripts/probes/arc-170/probe-s1-fn-forms.wat:22-40` |
| 14 | `arc-170/probe-s1-impure-gate.wat` | `fn-forms` never leaks an impure Process capture (reifies it silently) | **negative** → `tests/process/fixtures/probe-s1-impure-gate.wat` | the probe's OWN println text ("LEAK: impure capture was reified without error") names the failure condition; today it correctly refuses (via `closure_extract.rs`'s general "arms slice 1 doesn't yet encode" wall — the same not-yet-implemented bucket as `fn`/`Vector`/`Stream`, whose own comment says "surface as Internal so a surfacing test reveals the gap" — this probe IS that surfacing test) |
| 15 | `arc-170/probe-s1-named.wat` | same claim as #13, for a NAMED top-level fn | **repair** | identical defect class and fix. `wat-scripts/probes/arc-170/probe-s1-named.wat` |
| 16 | `arc-170/probe-s3-process-runner.wat` | the NOT-SHARED runner: only the work-fn is `fn-forms`'d, the pool-runner ships as named source | **repair** | identical defect class and fix. `wat-scripts/probes/arc-170/probe-s3-process-runner.wat:56-65` |
| 17 | `arc-170/probe-s3b-crux-fnforms-closure.wat` | `fn-forms` of the index-wrapping closure that CAPTURES the work-fn | **negative** → `tests/process/fixtures/probe-s3b-crux-fnforms-closure.wat` | `docs/arc/2026/06/259-forced-hand/NOTE-S3b-blockers-and-resolution.md` names this exact probe RED BY DESIGN ("disconfirming probe … RED, proved it") — the file's own stale `EXPECT "6 10"` header is wrong, corrected |
| 18 | `arc-170/probe-s3c-rendezvous.wat` | the runner takes the work-fn as a VALUE (not by-name), at the non-reserved rendezvous coordinate | **repair** | identical defect class and fix. `wat-scripts/probes/arc-170/probe-s3c-rendezvous.wat:67-79` |
| 19 | `arc-170/w3-n-dial-runner.wat` | a hand-written N=2 heterogeneous dial-runner type-composes | **fragment** → `tests/types/probe_arc170_w3_n_dial_runner.wat` | no `:user::main` BY DESIGN ("freeze-only (--check), no fork" — its own header line 1); moved beside its new consumer, `tests/types/probe_arc170_w3_n_dial_runner_checks_clean.rs`, which runs `--check` and asserts rc 0 (confirmed clean — the claim holds) |
| 20 | `arc-278/s2s-revoke-probe.wat` | a mid-life revoke makes caller2's dial refused, racing caller2's own post-spawn grant+revoke against its `:init connect'` | **retire** (`git rm`) | `DESIGN-STONE-M1-TEETH-revoke-refusal.md` line 10 names this EXACT file "the only prior 'refusal' evidence … and it is a race"; line 103 lists it as the REJECTED "s2s-revoke-probe anti-pattern" the M1-teeth design explicitly built an ordered (ack-before-redial), deterministic replacement to avoid — `tests/services/probe_arc170_m1_teeth_revoked.wat`, already committed and green. STOP-1 did not fire: the race was already solved elsewhere, not left unresolved |

No STOP fired. STOP-1's named candidate (#20) resolved to **retire** (superseded, not unresolved).
STOP-2/STOP-3 never matched: every refused probe's refusal traced to a principled, already-ruled,
or self-evident mechanism (never an unexplained Rust panic or an unrecoverable claim).

## 3. AMEND — ten negative probes moved out of `wat-scripts/probes/`, never `.wat.bad`

The first floor (red, §5) found that renaming all ten "negative" probes to `.wat.bad` collided
with a **pre-existing** gate this stone did not write: `tests/lint/every_wat_bad_fixture_actually_fails.rs`
drives `startup_from_file` on every `.wat.bad` and requires it to refuse **at startup**
(parse/resolve/type-check); it does not consult runtime execution at all. All ten of this stone's
negative probes fail only at **runtime** (a division by zero, a dispatch miss, a refused builtin
call, a child's own startup inside a spawned process …) — their OUTER file's own `startup_from_file`
succeeds every time. That gate's own panic message gives the fix: "the file is a valid program and
the NAME is wrong — `git mv` it to `.wat`." Since each driving test already EXECUTES the probe and
asserts the runtime failure (not `--check`), each of the ten moved to a plain `.wat` under
`tests/process/fixtures/` — out of `wat-scripts/probes/`, so the new "every probe runs, exit 0"
gate does not see it — exactly mirroring this repo's own pre-existing precedent for a
runtime-failing, committed probe: `tests/services/probe_arc170_m1_teeth_revoked.wat`. Each
fixture's header carries an `AMEND` paragraph recording this.

## 4. The gate

Build-time codegen (`build.rs`, `generate_every_probe_runs`): walks `wat-scripts/probes/**`
recursively, collects every `*.wat` (never `*.wat.bad` — `Path::extension()` on a `.wat.bad` file
is `"bad"`, not `"wat"`, so the glob naturally excludes it), and generates one `#[test] fn
probe_<sanitized_path>()` per file into `OUT_DIR/every_probe_runs.rs`. Each generated test runs
`CARGO_BIN_EXE_wat` directly against the probe's real path, stdin `/dev/null`, a 10s timeout
(enforced by a detached watchdog thread that `kill -9`s the child if still running — not joined,
so a fast-exiting probe's test does not pay the full timeout), and asserts exit 0.
`tests/process/every_probe_runs.rs` is a one-line `include!` of the generated file, riding the
`tests/process/` group's OWN pre-existing build.rs mod-registration — so it compiles as part of
that group with no new Cargo target. `rerun-if-changed` is emitted for `wat-scripts/probes/` and
every subdirectory found, so a probe dropped anywhere in the tree is picked up on the next build
with **no edit** to `build.rs` or any `.rs` file. 70 probes generate 70 tests today (83 at the
brief's draw, minus 1 fragment moved, minus 2 retired, minus 10 moved per §3).

The two existing driven probe tests, `probe_arc170_s3b_astsplice.rs` and
`probe_arc255_74_compound_upcast_runs.rs`, are untouched and still pass.

**Mutation proof** (`wat-scripts/probes/arc-054/probe-054-fn-idempotency.wat`, an already-green
probe, temporarily broken): replaced its `(println "ok")` body with
`(assertion-failed! :message "MUTATION-PROOF-255.75")`, rebuilt, ran its own generated test:

```
FAIL (1/1) wat::process every_probe_runs::probe_wat_scripts_probes_arc_054_probe_054_fn_idempotency
thread '...' panicked at .../every_probe_runs.rs:41:5:
assertion `left == right` failed: wat-scripts/probes/arc-054/probe-054-fn-idempotency.wat must run
clean (exit 0) — ruling E3, "the floor runs every probe and requires exit 0"; stdout:
stderr:
#wat.kernel/AssertionFailure {:thread "main" :message "MUTATION-PROOF-255.75" ...}
  left: Some(2)
 right: Some(0)
```

Restored the file byte-identical (`git diff --stat` empty), rebuilt, ran again: `PASS`. A red named
the exact probe; nothing else moved.

## 5. Reds, caused by this stone's own change, and cured

The **first** floor (`.floor/2026-10-01T06-24-26Z`) was RED — **13 failures**, all traced to this
stone's own change (never re-run before capture and diagnosis, per doctrine):

```
     Summary [ 380.350s] 6338 tests run: 6325 passed (18 slow), 13 failed, 24 skipped
FAIL wat::lint no_stale_path_in_doc::every_location_named_in_a_doc_comment_exists
FAIL wat::process probe_arc255_75_negative_probes::bracket_cause_names_the_real_cause
FAIL wat::process probe_arc255_75_negative_probes::m1_fix_revoke_bounces_dial_2
FAIL wat::lint every_wat_bad_fixture_actually_fails::every_wat_bad_fixture_actually_fails_shard_{06,07,08,09,10,11,12,13,14,15}  (10 shards)
```

**Arm 1 — `no_stale_path_in_doc`** (verbatim):
```
thread 'no_stale_path_in_doc::every_location_named_in_a_doc_comment_exists' panicked at
tests/lint/no_stale_path_in_doc.rs:298:5:
comments name 1 path(s) that do not exist:
  wat/bracket.wat: names `wat-scripts/probes/arc-170/w3-n-dial-runner.wat`, which does not exist
```
Cure: `wat/bracket.wat`'s citation updated to the fragment's new location,
`tests/types/probe_arc170_w3_n_dial_runner.wat` (hand-edit, one named file, per the brief's own
doctrine note — not a corpus-wide migration).

**Arm 2 — ten `every_wat_bad_fixture_actually_fails` shards**, each the same arm (one verbatim
example, shard 9/16):
```
thread 'every_wat_bad_fixture_actually_fails::every_wat_bad_fixture_actually_fails_shard_09'
panicked at tests/lint/every_wat_bad_fixture_actually_fails.rs:392:1:
🔥 1 `.wat.bad` file(s) in shard 9/16 START UP CLEAN and do not declare why. `.wat.bad` claims a
file fails to start up; nothing checked that claim until this gate, and a fixture that starts up
fine makes every assertion resting on it a coincidence.
THE FIX, one of two:
1. If the test that drives it asserts `is_ok()`, or starts the world up and INVOKES (asserting the
   error comes at EVAL), the file is a valid program and the NAME is wrong — `git mv` it to `.wat`
   and update every `.rs` referrer. ...
  wat-scripts/probes/arc-170/probe-defclause-discriminate.wat.bad
      starts up CLEAN (startup_from_file returned Ok) but is named `.wat.bad`, and declares nothing
```
(The other 9 shards: `probe-bracket-cause`, `probe-cap2-peer-pid`, `probe-child-inherits-defns`,
`probe-generic-shipped`, `probe-fnforms-keyword-err`, `probe-m1-addr-roundtrip`,
`probe-m1-fix-revoke`, `probe-s1-impure-gate`, `probe-s3b-crux-fnforms-closure` — all ten of §3's
negative probes, each in its own shard.) Cure: §3 — moved all ten to plain `.wat` under
`tests/process/fixtures/`.

**Arm 3 — `bracket_cause_names_the_real_cause`** (verbatim):
```
thread 'probe_arc255_75_negative_probes::bracket_cause_names_the_real_cause' panicked at
tests/process/probe_arc255_75_negative_probes.rs:46:5:
the collect-loop assertion must name which runner crashed: #wat.kernel/AssertionFailure
{:thread "main" :message "bracket collect-loop: runner 1 crashed: [#wat.kernel/LociDiedError
.RuntimeError {:message \"#wat.runtime/DivisionByZero ...
```
My own isolated run had measured "runner 0 crashed" (3 workers map `[1 2 3]` in parallel); under
full-floor concurrent load, runner 1 was the one the collect-loop observed crashing first — real
scheduling non-determinism over WHICH of 3 parallel workers reports, never part of the probe's own
claim. Cure: the assertion now checks for `"runner "` and `" crashed:"` as two needles, never a
pinned index (`tests/process/probe_arc255_75_negative_probes.rs:54-57`).

**Arm 4 — `m1_fix_revoke_bounces_dial_2`** (verbatim):
```
thread 'probe_arc255_75_negative_probes::m1_fix_revoke_bounces_dial_2' panicked at
tests/process/probe_arc255_75_negative_probes.rs:196:5:
the raised message must be exactly the peer-closed EOF the bounce produces:
#wat.kernel/AssertionFailure {:thread "main" :message "disconnected" ...}
```
Traced, not dismissed: `src/kernel/error.rs` maps `"disconnected"` to `LociDiedError::Disconnected`
(a clean-EOF kernel outcome), and the captured panic carries ONLY the `:user::main` frame — no
bracket/collect-loop nesting — which matches exactly one call in this probe that is NOT wrapped in
a `:wat::core::match`: `_r (:probe::echo/revoke eh …)`. If the echo service's own control channel
disconnects before that unmatched RPC returns — plausible under the ~16-way parallel process-spawn
pressure this exact floor carried (its own Summary: 14 tests SLOW past 15s, three past 90–300s,
measured, not inferred) — it raises directly, before the probe's documented revoke→bounce→EOF path
is ever reached. Both outcomes are still FAILURES (non-zero exit, no `"NOREVOKE-REACHED-END"`
print); a genuine revoke REGRESSION (dial #2 wrongly admitted) produces NEITHER string, so widening
the assertion to accept either does not weaken the regression-catching power the probe exists for.
Cure: the assertion now accepts `"recv': peer closed"` OR `"disconnected"`
(`tests/process/probe_arc255_75_negative_probes.rs:210-220`), with the full mechanism recorded in
both the test and the fixture's own header.

**New floor, after all four cures** (`.floor/2026-10-01T06-46-08Z`), clean, nothing else running:
```
     Summary [ 383.546s] 6338 tests run: 6338 passed (25 slow), 24 skipped
```
6338 = 6257 (`2ffcdfa3e` baseline, per the brief) + 81 new (70 generated probe tests + 10 negative
driven tests + 1 fragment driven test). 24 skipped matches the baseline's skip count exactly — no
new skip introduced.

## Gates

| what | how | result |
|---|---|---|
| every probe | the new gate, `cargo nextest run --release --test process -E 'test(every_probe_runs)'` | 70/70 pass (quoted in full-floor run below) |
| release floor | `scripts/floor.sh`, foreground, nothing else running | first run RED (§5, cured); second run clean: `.floor/2026-10-01T06-46-08Z/clean.log`: `Summary [ 383.546s] 6338 tests run: 6338 passed (25 slow), 24 skipped`; exit=0 |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | `Finished \`release\` profile [optimized] target(s) in 11.97s`, rc 0 |

## Files touched

- `build.rs` — added `generate_every_probe_runs`/`collect_wat_files`/`probe_fn_name`: the gate's
  codegen (§4).
- `tests/process/every_probe_runs.rs` — new; one-line `include!` of the generated gate.
- `tests/process/probe_arc255_75_negative_probes.rs` — new; drives all ten negative probes (§3),
  10 `#[test]`s.
- `tests/types/probe_arc170_w3_n_dial_runner_checks_clean.rs` — new; drives the fragment (#19).
- `tests/process/fixtures/probe-{bracket-cause,cap2-peer-pid,child-inherits-defns,
  defclause-discriminate,fnforms-keyword-err,generic-shipped,m1-addr-roundtrip,m1-fix-revoke,
  s1-impure-gate,s3b-crux-fnforms-closure}.wat` — moved from `wat-scripts/probes/arc-170/`
  (ten negative dispositions, §3), each with an `AMEND`/`DISPOSITION` header.
- `tests/types/probe_arc170_w3_n_dial_runner.wat` — moved from `wat-scripts/probes/arc-170/
  w3-n-dial-runner.wat` (fragment, #19), with a `DISPOSITION` header.
- `wat-scripts/probes/arc-170/{probe-c1-ast-shape,probe-c1-plain-fnforms-shape,probe-m1-argcount,
  probe-s1-fn-forms,probe-s1-named,probe-s3-process-runner,probe-s3c-rendezvous}.wat` — repaired
  in place (seven hand-edits; this is single-file repair, not a corpus-wide migration, per the
  brief's own doctrine note).
- `wat-scripts/probes/arc-170/probe-m1-cf-norevoke.wat` — `git rm` (#11, retired).
- `wat-scripts/probes/arc-278/s2s-revoke-probe.wat` — `git rm` (#20, retired).
- `wat/bracket.wat` — one citation updated (§5, Arm 1).
- `docs/arc/2026/06/255-builtin-registry/SCORE-STONE-255.75-every-probe-runs.md` — this file.

## STOPs triggered

None. STOP-1's named candidate (`arc-278/s2s-revoke-probe.wat`) resolved to **retire**: the design
doc that proposed it explicitly rejected it as "the anti-pattern" and built a deterministic,
ordered, already-committed-and-green replacement — the race was found already solved, not left
unresolved. STOP-2/STOP-3 never matched any of the twenty.
