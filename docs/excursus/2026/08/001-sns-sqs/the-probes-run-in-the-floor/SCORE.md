# SCORE — the probes run in the floor

**Struck 2026-09-15**, on `sns-sqs` at `6682ed8be`. Stone 2 of 2; it spends stone 1
(`the-handshake-deadline-is-injectable`) to make the silent-child rows affordable. Did not commit.
Two files added (`tests/probes/`), one `Cargo.toml` entry, two behaviour edits to one `.wat` plus a
header note on each of the four probes. **No `src/`, no `wat/`.**

⛔ The headline is that **five lifecycle invariants, each previously driven ONCE by hand, now
re-drive on every floor as five separately-named tests.** It is NOT "the probes run in the floor":
**392 scratch-pad probes are still parsed-only** (recounted below — the number went *up* since the
DESIGN, because the corpus grew faster than this stone guarded it).

## ⛔⛔ THE FIRST FLOOR WAS RED, AND THE RED WAS MINE — reported first, not last

```
     Summary [ 557.580s] 5257 tests run: 5256 passed (9 slow), 1 failed, 22 skipped
```

`./scripts/floor.sh`, **`.floor/2026-09-16T05-43-33Z/`**, `exit=100`, **`ARM.txt` PRESENT**.
The failing arm, named exactly:

```
FAIL [   5.398s] (3272/5257) wat::probes scratch_pad_manifest::two_faults_before_stop_are_faced_at_all_four_surfaces

⛔ MANIFEST ROW FAILED — two_faults_before_stop_are_faced_at_all_four_surfaces
exit 0 was as expected, but 1 of 5 MARKERS ARE ABSENT from stdout+stderr:
           MISSING  "\"stop     =GaveUp waited-ms=0 last=a second Status::Faulted arrived"
An exit code is not an invariant. Each marker names something only the passing world prints.

row      two_faults_before_stop_are_faced_at_all_four_surfaces
probe    wat-scripts/scratch-pad/probe-two-faults-before-stop.wat
argv     []
env      []  (WAT_STARTUP_HANDSHAKE_DEADLINE_MS is cleared unless listed)
timeout  20000 ms (SIGKILL)
expected exit 0
markers  ["\"control=Message\"", "\"stop     =GaveUp waited-ms=0 last=a second Status::Faulted arrived", "\"hibernate=GaveUp waited-ms=0 last=a second Status::Faulted arrived", "\"grant    =GaveUp waited-ms=0 last=a second Status::Faulted arrived", "\"revoke   =GaveUp waited-ms=0 last=a second Status::Faulted arrived"]

What the world this row fences against printed instead:
  pre-stone (HEAD's wat/service.wat, rebuilt and driven as that SCORE's negative control) the owner DIED on a recoverable error: `AssertionFailure … "defservice stop: expected Status::Stopped"`, with the word `Faulted` nowhere in it. It printed no GaveUp line at all, and exited non-zero.

── the WHOLE captured output follows, both pipes, untruncated ──
── stdout (634 bytes) ──
"control=Message"
"stop-boom1=Lost"
"stop-boom2=Lost"
"stop     =GaveUp waited-ms=1 last=a second Status::Faulted arrived — the fault stream outran the one-level drain"
"hib-boom1=Lost"
"hib-boom2=Lost"
"hibernate=GaveUp waited-ms=0 last=a second Status::Faulted arrived — the fault stream outran the one-level drain"
"grant-boom1=Lost"
"grant-boom2=Lost"
"grant    =GaveUp waited-ms=0 last=a second Status::Faulted arrived — the fault stream outran the one-level drain"
"revoke-boom1=Lost"
"revoke-boom2=Lost"
"revoke   =GaveUp waited-ms=0 last=a second Status::Faulted arrived — the fault stream outran the one-level drain"

── stderr (0 bytes) ──

── end of captured output ──
```

⭐ **`waited-ms=1`.** One millisecond. Every invariant this row exists to guard HELD — four
`GaveUp`s, the right `last=` mechanism on each, exit 0 — and the row went red on an **integer
millisecond count** that is 0 every time the probe is run by hand and 1 when 5257 tests share the
box. **The marker was the defect, not the substrate.** `waited-ms` is read from the method's own
`t0`, so `waited-ms=0` asserts that two recv round-trips complete inside 1 ms under full floor
contention — a threshold on a clock, which is precisely what trap-door 1 forbids (*"assert the
invariant, never the statistic"*). Trap-door 2 was right that `waited-ms=0` **is** a real value in
today's output; it is just not an invariant.

⛔ **This red is not re-run away. It is the finding.** The fix was to the MARKER (the four now read
`"stop     =GaveUp` etc. plus the full `last=…` mechanism string, with no elapsed time anywhere),
and the tree was then re-weighed — that is a second weighing of a *changed* tree, not a retry of the
same one. All three floors are below; the red one is quoted whole.

⚠ **AND THE INSTRUMENT NEARLY HID IT FROM ME.** I launched floor 1 as
`./scripts/floor.sh 2>&1 | tail -25` in the background; the harness reported the job as **"exit code
0"** — because a pipeline's status is `tail`'s. The Summary line says `1 failed` and `ARM.txt` is on
disk. This is the exact failure `holon/CLAUDE.md` warns about in two places (*"read the Summary line
— never a piped exit code"*), and it happened on this stone, to me, while I was writing a runner
whose whole purpose is to stop exactly this class of silence. Floors 2 and 3 were launched with **no
pipe**, and every floor here is read from its own `clean.log` Summary line.

## THREE FLOORS, all reported — the second is green and is NOT the record

```
floor 1   Summary [ 557.580s] 5257 tests run: 5256 passed (9 slow), 1 failed, 22 skipped
          .floor/2026-09-16T05-43-33Z/ · exit=100 · ARM.txt PRESENT · ⛔ RED, quoted whole above
floor 2   Summary [ 557.045s] 5257 tests run: 5257 passed (9 slow), 22 skipped
          .floor/2026-09-16T05-56-56Z/ · exit=0 · NO ARM.txt · green, but see below
floor 3   Summary [ 559.222s] 5258 tests run: 5258 passed (9 slow), 22 skipped
          .floor/2026-09-16T06-11-38Z/ · exit=0 · NO ARM.txt · 559.222 s, inside the 540.4–561.5 s band · THE RECORD
```

**Floor 1 → 2**: exactly one file changed — `tests/probes/scratch_pad_manifest.rs`, md5
`7ee10b744d68474cfbc98eaf55fdda6f` → `3a3e61f2f88fa8c767cb1cdc0d6175ab` (the `waited-ms` markers).
Verified by timestamp, not asserted: floor 1 started 05:43:33Z and every other file in the blast
radius is older (`Cargo.toml` 05:33Z, `tests/probes/mod.rs` 05:31Z, the probe `.wat` 05:28Z; the
`wat/` and `src/` files are stone 1's, 04:18–04:35Z).

**Floor 2 → 3, and why floor 2 is not the record.** While floor 2 was running I found that
`probe-a-silent-child-cannot-hang-launch.wat`'s header still claimed the file *"is never run
there"* — false as of this stone — and closed the sibling hazard for all four probes (next section).
Those are comment-only `.wat` edits, so floor 2's **outcome** is unaffected; its **attribution** is
not, because `wat-scripts/` is inside the `every_wat_scripts_file_loads` gate, which *is*
substantially the whole floor. A green attached to a tree that no longer exists is not a green — the
rule both sibling SCOREs applied to themselves. Floor 3 also carries a **seventh** test
(`every_manifest_probe_says_it_is_in_the_floor`), so its count is **5258**, not 5257. The only thing
edited after floor 3 started is **this document**, which no gate reads.

`cargo build --release`: clean. Clippy `--release --workspace --all-targets`: **0** warnings, 0
errors. `cargo nextest run --release --no-run`: clean.
Circuit happy path (the `~/work/BREADCRUMB.md` FRESHNESS-PROBE one-liner, verbatim, one line, 23.5 s):
`timeout=yes;discarded=yes;redial=Connected;retry-on=fresh` · `distinct=8000;dup=0` ·
`pub-exh-last=none` · `bp-delay=0`.

## The ten rows

| # | what | result |
|---|---|---|
| 1 | ⭑⭑ Each seed row is its own nextest test, by name | ✅ **7 names quoted from the run** (5 rows + two fences on the runner) |
| 2 | ⭑⭑ Markers unique to the PASSING world | ✅ per-row table below; the "failing world" also lives **in the code**, as a `failing_world` field |
| 3 | ⭑⭑ A deliberately broken row FAILS, driven | ✅ **DRIVEN THREE TIMES** — twice unplanned (a SCORE line that does not reproduce; the red floor above) and once deliberately, whole output each time |
| 4 | ⭑ A hanging probe is a FAIL, not a hang | ✅ **DRIVEN**: SIGKILL at 1.505 s, FAIL carrying the partial capture. SIGTERM re-measured useless (60.6 s) on this box |
| 5 | ⭑ A missing/renamed path is a FAIL | ✅ **DRIVEN**: red in 0.005 s, before any spawn |
| 6 | ⭑⭑ Every row's expectation came from a SCORE | ✅ 5/5 cited with the SCORE's own line — ⚠ **and two of those lines did not survive contention**; both corrections are below |
| 7 | ⭑⭑ Floor, cost as a BAND | ✅ green (floor 3, the record; floor 1 RED is reported above); added cost **not separable from the band** — 7.2 s of test time, 2.6 s of wall, inside the ~15 s budget. No STOP |
| 8 | ⭑ No flaky assertion added | ✅ **earned, not claimed**: two markers removed for being statistics, each after going red |
| 9 | clippy + `--no-run` | ✅ 0 / clean |
| 10 | ⛔ What is still unguarded is stated | ✅ **392 parsed-only**, and three of this session's invariants still have no row, each with its reason |

---

## Row 1 — the seven test names, quoted from the run

```
        PASS [   0.004s] (1/7) wat::probes scratch_pad_manifest::every_manifest_probe_says_it_is_in_the_floor
        PASS [   0.005s] (2/7) wat::probes scratch_pad_manifest::every_manifest_row_has_its_own_test
        PASS [   0.526s] (3/7) wat::probes scratch_pad_manifest::a_dead_runner_names_the_item_it_orphaned
        PASS [   0.747s] (4/7) wat::probes scratch_pad_manifest::a_silent_child_cannot_hang_launch_on_the_process_tier
        PASS [   1.008s] (5/7) wat::probes scratch_pad_manifest::a_handler_raise_does_not_kill_the_service_for_everyone
        PASS [   2.355s] (6/7) wat::probes scratch_pad_manifest::two_faults_before_stop_are_faced_at_all_four_surfaces
        PASS [   2.556s] (7/7) wat::probes scratch_pad_manifest::a_silent_child_cannot_hang_launch_on_the_thread_tier
────────────
     Summary [   2.557s] 7 tests run: 7 passed, 0 skipped
```

Five are the manifest rows; two are fences on the runner itself
(`every_manifest_row_has_its_own_test`, `every_manifest_probe_says_it_is_in_the_floor`).

`tests/probes/` is a new group: `Cargo.toml` gains `[[test]] name = "probes"`, `tests/probes/mod.rs`
is the usual `include!(concat!(env!("OUT_DIR"), "/probes_mods.rs"))` stub, and `build.rs` generates
the module list on its own. ⚠ One gotcha worth recording: `build.rs` only declares
`rerun-if-changed` for group dirs it **already** knows, so a brand-new group does not appear until
`build.rs` itself is touched (`touch build.rs` once; the first attempt failed with *"couldn't read
…/out/probes_mods.rs"*). It is its own group rather than a file in `tests/services/` because a row
here is neither a service nor a module: it is a whole `wat` **process** run from the outside, and
`binary_id(wat::probes)` is then something the Summary and a `-E` filter can name.

⛔ **One test per row — with a fence that keeps it that way.**
`every_manifest_row_has_its_own_test` reads this file's own source (`include_str!`) and fails if any
`MANIFEST` row has no `fn <name>(` of its own; it also refuses duplicate row names (`row()` returns
the first match, so a duplicate would leave a row permanently dead). Add a row, forget the test, and
the manifest would otherwise *read* as a guard while nothing ran it. Keyed on the property, not on a
count, so renumbering cannot satisfy it.

## ⭑ THE SIBLING HAZARD I FOUND WHILE WRITING THIS UP — gated, not annotated four times

Four `.wat` files just changed status: they read as scratch probes, because until this stone that is
what they were. Someone relabels a `println` or tunes a number, the floor goes red somewhere they
were not looking, and the only link between the two is a path inside a panic message they have to go
and find. Worse, one of them had a header that was now **false**:
`probe-a-silent-child-cannot-hang-launch.wat` said *"This file is only PARSED and TYPE-CHECKED … it
is never run there, so the 60 s park costs the floor nothing."* Both clauses stopped being true the
moment this stone landed.

So all four probes now carry a header note naming the runner and the row — and the note is **gated,
not trusted**: `every_manifest_probe_says_it_is_in_the_floor` requires every `MANIFEST` row's probe
to name `tests/probes/scratch_pad_manifest.rs` **in its first 60 lines** (where an editor lands, not
buried at the bottom). Gated on the property rather than on a list, so an annotation cannot decay
into decoration and a new row cannot ship without one. ⭑ This is the *fix the sibling or gate the
class* move: four sites, one fence, and the fence is what makes the next row carry its own warning.
The notes also record the two removed markers where the next editor will actually read them (the
`waited-ms` and `runner N` rulings), and the handler-raise note states outright that **the label
padding is part of the contract now** (`"a-boom   "`, `"b-after  "`).

⭑ **The gate is DRIVEN, not merely written** — a fence nobody has seen fail is a claim. One row's
`path` was temporarily pointed at a real but unannotated probe
(`probe-the-handshake-deadline-is-injectable.wat`), and it went red in 0.004 s:

```
thread 'scratch_pad_manifest::every_manifest_probe_says_it_is_in_the_floor' panicked at
tests/probes/scratch_pad_manifest.rs:624:5:
these probes are RUN BY THE FLOOR but their first 60 lines do not say so:
  wat-scripts/scratch-pad/probe-the-handshake-deadline-is-injectable.wat
Add a note naming `tests/probes/scratch_pad_manifest.rs` and the row, so the next person to edit
the probe learns from the FILE that a change there can redden the floor — not from a panic message
after the fact.
```

Restored from a byte-identical copy (md5 `01bc69cba4542ec56ef7977953001dae` before and after,
`grep -c` for the temporary path → 0) and the group re-run green before the floor was weighed.

⚠ **Cost of finding this late: a third floor.** `wat-scripts/` is inside the load gate, so those
four comment edits invalidated the attribution of the floor that was running while I made them. It
is reported above rather than hidden, and floor 3 is the record.

## Rows 2 & 6 — the five rows: markers, the failing world, the SCORE line

Markers are substrings of **stdout+stderr combined**; all are required.

### 1. `two_faults_before_stop_are_faced_at_all_four_surfaces` — exit **0**, 2.4 s

| | |
|---|---|
| probe | `wat-scripts/scratch-pad/probe-two-faults-before-stop.wat` |
| markers | `"control=Message"` · `"stop     =GaveUp` · `"hibernate=GaveUp` · `"grant    =GaveUp` · `"revoke   =GaveUp` · `last=a second Status::Faulted arrived — the fault stream outran the one-level drain` |
| SCORE | `every-status-arm-is-named/SCORE.md`, *"⭐ Row 9 — DRIVEN, on all four sites, with a negative control"*: `"stop     =GaveUp waited-ms=0 last=a second Status::Faulted arrived — the fault stream outran the one-level drain"`, and its `hibernate`/`grant`/`revoke` triplets |
| ⭑ the failing world | that SCORE's own negative control, driven on a binary rebuilt from HEAD's `wat/service.wat`: the owner **died** — `AssertionFailure … "defservice stop: expected Status::Stopped"`, the word `Faulted` nowhere in it, exit non-zero, and **no `GaveUp` line printed at all** |

`"control=Message"` is the non-vacuity control — the service answered a normal request before
anything raised, so an all-failures run cannot read as the invariant. The `last=…` string is the
mechanism: a `GaveUp` for any *other* reason fails this row. ⛔ `waited-ms` is absent by ruling; the
red floor above is why.

### 2. `a_silent_child_cannot_hang_launch_on_the_thread_tier` — exit **2**, 2.5 s

| | |
|---|---|
| probe | `wat-scripts/scratch-pad/probe-a-silent-child-cannot-hang-launch.wat`, `env WAT_STARTUP_HANDSHAKE_DEADLINE_MS=200` |
| markers | `"sht: tier=thread"` · `recv: timed out — the peer is alive and silent` · `:wat::spawn::ThreadOpts/launch` |
| SCORE | `the-handshake-deadline-is-injectable/SCORE.md`, *"⭐ Rows 2 and 3 — DRIVEN, four wall-clocks"*: `thread, =200   +000.657 AssertionFailure "recv: timed out — the peer is alive and silent" / frame :wat::spawn::ThreadOpts/launch    Δ = 0.201 s` |
| ⭑ the failing world | the pre-stone launcher awaited readiness on a **bare** `recv`, so this probe **HUNG**: no arm, no exit, and not stoppable by SIGTERM (125 s ignored, measured by the predecessor; re-measured here at 60.6 s). The row's timeout is what turns that world into a FAIL instead of a wedged floor |

### 3. `a_silent_child_cannot_hang_launch_on_the_process_tier` — exit **2**, 0.7 s

| | |
|---|---|
| probe | the same file with `argv[2] = "process"`, `env WAT_STARTUP_HANDSHAKE_DEADLINE_MS=200` |
| markers | `"sht: tier=process"` · `recv: timed out — the peer is alive and silent` · `:wat::spawn::ProcessOpts/launch` |
| SCORE | same SCORE: `process, =200   +000.669 AssertionFailure … frame :wat::spawn::ProcessOpts/launch    Δ = 0.207 s` |
| ⭑ the failing world | as the thread row (a hang). ⭑ And had the tier word not reached the probe, **the frame here would read `ThreadOpts/launch`** — which is why the frame is a marker and not decoration: it is the one string that tells these two rows apart |

### 4. `a_dead_runner_names_the_item_it_orphaned` — exit **2**, 0.5 s

| | |
|---|---|
| probe | `wat-scripts/scratch-pad/probe-a-dead-runner-orphans-its-item.wat` |
| markers | `"clean=[0 2 4 6 8 10 12 14]"` · `crashed holding item 3:` |
| SCORE | `a-dead-runner-names-its-orphan/SCORE.md`, *"⭑⭑ THE HEADLINE — item 3, still a raise"* (five runs, every one): `clean=[0 2 4 6 8 10 12 14]` / `exit=2` / `bracket collect-loop: runner 0 crashed holding item 3: …` |
| ⭑ the failing world | `bracket collect-loop: runner 0 crashed: …` — the same raise, the same exit 2, **with no item named** (that SCORE quotes it as *"Today:"*). A row asserting only the raise or the exit code would pass on that anonymous world, which IS the defect the stone removed |

⛔ **The runner INDEX is not in the marker, and that is a correction to that SCORE** — row 3 below.

### 5. `a_handler_raise_does_not_kill_the_service_for_everyone` — exit **0**, 1.0 s

| | |
|---|---|
| probe | `wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat` |
| markers | `"a-control=Ok"` · `"a-boom   =Lost"` · `"b-after  =Ok"` |
| SCORE | `a-service-stops-instead-of-crashing/SCORE.md`, *"⭑ THE HEADLINE — the innocent client is served"*: `a-control=Ok` / `a-boom   =Lost` / `b-after  =Ok` |
| ⭑ the failing world | `b-dial=connect-REFUSED` — the dead service could not even be dialled, so `b-after  =Ok` was never printed. That line is what this probe's own v2 recorded in that SCORE, and the `b-dial` label is still emitted by the probe's own connect-failure arm (`:72`), so the failing world's line is reachable from **today's** file, not only from a memory of it |

`"a-control=Ok"` is the non-vacuity control; `"a-boom   =Lost"` is the raiser's own severed conn,
which is expected and not the defect.

## ⛔ ROW 3 — DRIVEN THREE TIMES, AND TWICE I DID NOT PLAN IT

### 3a. A SCORE line that does not reproduce under contention (`runner 0`)

The dead-runner row was first written with the SCORE's line copied verbatim —
`bracket collect-loop: runner 0 crashed holding item 3:`. **It went red on its very first nextest
run**, whole output printed by the runner:

```
⛔ MANIFEST ROW FAILED — a_dead_runner_names_the_item_it_orphaned
exit 2 was as expected, but 1 of 2 MARKERS ARE ABSENT from stdout+stderr:
           MISSING  "bracket collect-loop: runner 0 crashed holding item 3:"
…
── stdout (48 bytes) ──
"clean=[0 2 4 6 8 10 12 14]"
"subject-starting"

── stderr (3672 bytes) ──
#wat.kernel/AssertionFailure {… :message "runner: dying on VALUE 13, which is INDEX 3" …}
#wat.kernel/AssertionFailure {… :message "bracket collect-loop: runner 1 crashed holding item 3: …
```

**`runner 1`, not `runner 0`.** Then measured rather than assumed:

```
10 sequential runs   →  10 × "runner 0 crashed holding item 3"
12 CONCURRENT runs   →   6 × "runner 0",  6 × "runner 1"      (the ITEM was 3 in all 22)
```

⭑ The index is a **scheduling statistic that the floor's own parallelism perturbs**; that SCORE's
*"five runs, every one"* were five **uncontended** runs. The item is the stone's invariant, the
runner position is which worker happened to take it. Marker now: `crashed holding item 3:`.

### 3b. The red floor (`waited-ms=0`) — quoted whole at the top of this document

Same species, one layer subtler: not a scheduler but a **clock**, and it survived a
`binary_id(wat::probes)`-only run before the full floor caught it. Recorded at the top because a red
floor is reported before anything else, and because the pair makes the rule concrete: **uncontended
stability is what makes a statistic look like an invariant.**

### 3c. The deliberate break — three temporary rows, one run, four independent reds

Also a demonstration of row 1's point: a per-row test does not hide its siblings.

```
    Starting 9 tests across 1 binary (46 binaries skipped)
        PASS [   0.005s] (1/9) wat::probes scratch_pad_manifest::every_manifest_row_has_its_own_test
        FAIL [   0.005s] (2/9) wat::probes scratch_pad_manifest::temp_control_row_5_a_missing_path_is_a_failure
        PASS [   0.527s] (3/9) wat::probes scratch_pad_manifest::a_dead_runner_names_the_item_it_orphaned
        PASS [   0.968s] (4/9) wat::probes scratch_pad_manifest::a_silent_child_cannot_hang_launch_on_the_process_tier
        FAIL [   1.052s] (5/9) wat::probes scratch_pad_manifest::a_handler_raise_does_not_kill_the_service_for_everyone
        FAIL [   1.515s] (6/9) wat::probes scratch_pad_manifest::temp_control_row_4_a_parked_probe_is_sigkilled_and_failed
        PASS [   2.395s] (7/9) wat::probes scratch_pad_manifest::two_faults_before_stop_are_faced_at_all_four_surfaces
        FAIL [   2.438s] (8/9) wat::probes scratch_pad_manifest::temp_control_row_3b_a_wrong_exit_code_is_a_failure
        PASS [   2.759s] (9/9) wat::probes scratch_pad_manifest::a_silent_child_cannot_hang_launch_on_the_thread_tier
────────────
     Summary [   2.759s] 9 tests run: 5 passed, 4 failed, 0 skipped
```

**The broken marker** — `"b-after  =Ok"` temporarily replaced with
`"b-after  =TEMPORARY-BREAK-FOR-EXPECTATIONS-ROW-3"`. Whole block, verbatim, including the probe's
whole (correct) output:

```
⛔ MANIFEST ROW FAILED — a_handler_raise_does_not_kill_the_service_for_everyone
exit 0 was as expected, but 1 of 3 MARKERS ARE ABSENT from stdout+stderr:
           MISSING  "\"b-after  =TEMPORARY-BREAK-FOR-EXPECTATIONS-ROW-3\""
An exit code is not an invariant. Each marker names something only the passing world prints.

row      a_handler_raise_does_not_kill_the_service_for_everyone
probe    wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat
argv     []
env      []  (WAT_STARTUP_HANDSHAKE_DEADLINE_MS is cleared unless listed)
timeout  20000 ms (SIGKILL)
expected exit 0
markers  ["\"a-control=Ok\"", "\"a-boom   =Lost\"", "\"b-after  =TEMPORARY-BREAK-FOR-EXPECTATIONS-ROW-3\""]

What the world this row fences against printed instead:
  `b-dial=connect-REFUSED` — the dead service could not even be dialled, so `b-after  =Ok` was never printed (that exact line is the failure this probe's own v2 recorded in the SCORE; the label `b-dial` is printed by the probe's own connect-failure arm, which is still in the file).

── the WHOLE captured output follows, both pipes, untruncated ──
── stdout (47 bytes) ──
"a-control=Ok"
"a-boom   =Lost"
"b-after  =Ok"

── stderr (0 bytes) ──

── end of captured output ──
```

**A lying exit code** (`expect_exit: 42` on a probe that exits 0) is the other half, because a
marker check that is never reached would hide a changed exit:

```
⛔ MANIFEST ROW FAILED — temp_control_row_3b_a_wrong_exit_code_is_a_failure
EXITED 0, expected 42. The exit code alone never proves a row (see the markers), but a changed one
means the probe no longer does what the manifest says it does.
```

All three temporary rows and the broken marker were then **restored from a byte-identical copy**
(md5 `7ee10b744d68474cfbc98eaf55fdda6f` before the controls and after the restore;
`grep -c temp_control` → 0, `grep -c TEMPORARY-BREAK` → 0) and the group re-run green before
anything else was weighed.

## ⭑ ROW 4 — the timeout SIGKILLs, and SIGTERM was re-measured to be useless here

The temporary row pointed at the **same** silent-child probe with the deadline **unset**, so the
launcher genuinely waits the full default 30 000 ms, against a row timeout of 1 500 ms:

```
⛔ MANIFEST ROW FAILED — temp_control_row_4_a_parked_probe_is_sigkilled_and_failed
TIMED OUT after 1.505 s and was SIGKILLed. A timeout is a FAIL, never a skip — and whatever the
probe had printed before the kill is below, which is the only evidence this run will ever produce.

row      temp_control_row_4_a_parked_probe_is_sigkilled_and_failed
probe    wat-scripts/scratch-pad/probe-a-silent-child-cannot-hang-launch.wat
argv     []
env      []  (WAT_STARTUP_HANDSHAKE_DEADLINE_MS is cleared unless listed)
timeout  1500 ms (SIGKILL)
expected exit 2
markers  ["recv: timed out — the peer is alive and silent"]

What the world this row fences against printed instead:
  TEMPORARY CONTROL — nothing; this row exists to be killed.

── the WHOLE captured output follows, both pipes, untruncated ──
── stdout (104 bytes) ──
"sht: tier=thread"
"sht: starting a service whose :init parks (10x the handshake deadline, capped 60s)"

── stderr (0 bytes) ──

── end of captured output ──
```

⭑ **Note what is inside that capture: the two lines the probe HAD printed before the kill.** A skip
would have reported nothing, and a `head`/`tail` window would have made the absence of the third
line unfalsifiable.

**SIGTERM is not a bound — re-measured on this box rather than inherited** (the 125 s figure is the
predecessor's). Same parked probe, deadline unset:

```
timeout          1 s  (SIGTERM)  →  exit=124, WALL 60.560 s   ← the signal was IGNORED for 59.5 s;
                                                                the process ended on its own park
timeout -s KILL  1 s  (SIGKILL)  →  exit=137, WALL  1.008 s
```

So `Child::kill()` (SIGKILL on Unix) in `run_row` is not a stylistic choice: SIGTERM would have left
the row wedged until nextest's own 30 s kill, with no arm.

⚠ **One mechanism the runner needs because of this, stated because it is subtle.** The two pipe
readers are joined with a **bounded** grace (2 000 ms), never an unbounded join: on the process tier
a forked grandchild can outlive its parent holding the same pipe, and an unbounded join there would
convert a fast FAIL back into a hang. Draining on threads at all is also not decoration — the
dead-runner probe emits ~3.7 KB of EDN and a probe that outgrew the pipe buffer would otherwise
block on write while the runner blocked on wait.

## Row 5 — a missing path is red before anything spawns

```
MANIFEST row "temp_control_row_5_a_missing_path_is_a_failure": probe not found at
/home/john/work/holon/wat-rs/wat-scripts/scratch-pad/probe-this-file-does-not-exist.wat
A manifest row whose path does not exist is a FAILURE, not a skip — a renamed or deleted probe
must not silently stop being tested. Fix the path in MANIFEST
(tests/probes/scratch_pad_manifest.rs) or remove the row deliberately.
```

0.005 s, before any spawn (trap-door 4).

## ⭐ Trap-door 0 — how the deadline is set, and one step further than asked

`WAT_STARTUP_HANDSHAKE_DEADLINE_MS=200` appears **only** in the two silent-child rows' `env` field,
applied to those two `Command`s. It is nowhere in `.config/nextest.toml`, nowhere in
`scripts/floor.sh`, and never exported. ⭑ And `run_row` goes further than the trap-door asked:
it calls **`cmd.env_remove("WAT_STARTUP_HANDSHAKE_DEADLINE_MS")` for every row** before applying
that row's own env, so an ambient export in the operator's shell cannot silently change what a row
means (a row expecting a 30 s park would otherwise quietly become a different experiment). The
floors above are the evidence nothing leaked: 5258 tests, including every `deftest` and
`deftest-hermetic` in the corpus — all of which would starve at 200 ms (**300 starves, 500 does
not**, stone 1's bisection).

## ⭐ How BOTH TIERS are driven without hand-editing `:locus`, and the two probe edits

The task named this as a thing to decide and state. The probe's `:locus` **was** a source literal
flipped by hand between runs; two floor rows cannot do that. Decided: **the tier is an argv word.**

```
./target/release/wat …probe-a-silent-child-cannot-hang-launch.wat            → thread
./target/release/wat …probe-a-silent-child-cannot-hang-launch.wat  process   → process
```

- The probe reads `argv[2]` through `(:wat::runtime::argv)`.
  `wat-scripts/scratch-pad/probe-execve-argv-cow-leak.wat:23` is the cited exemplar for reading it,
  and `tests/cli/wat_cli__argv_passthrough.wat` is the contract (argv[0] = binary, argv[1] = the
  entry file, argv[2..] = the caller's words, unedited). A `:wat::core::length` guard precedes the
  `nth` because `nth` **raises** out of range and a bare `wat <file>` has no argv[2].
  ⚠ `:wat::vector::contains?` would have been shorter and **does not type-check** — driven, not
  guessed: `:wat::vector::contains?: parameter #1 expects (:wat::core::PersistentVector :- [:?749]);
  got (:wat::core::Vector :- [:wat::core::String])`.
- The two `start` calls are written out **per branch** rather than selecting a locus value first:
  `ThreadOpts` and `ProcessOpts` are distinct types with distinct `:wat::spawn::Locus` impls, so
  there is no single expression that is either one. Both branches evaluate to `nil`.
- The tier is **echoed** (`"sht: tier=…"`) so a captured log says which arm it drove, and the raised
  **frame** says it independently — `:wat::spawn::ThreadOpts/launch` vs `ProcessOpts/launch`,
  confirmed by name on both tiers (`grep -o ':symbol "[^"]*"'` → also
  `:sht::slow/start$impl-thread` / `$impl-process`), not inferred.

⭐ **The second edit is what made the thread row affordable, and it is not cosmetic.** The park in
`:init` was the literal `60000`. On the **thread** tier the parked child thread is still **joined at
interpreter teardown**, so the park is the whole process's wall-clock no matter when the arm fires —
a 60 s row, twice nextest's own kill. The park is now
`min(60000, 10 × (:wat::program::startup-handshake-deadline-ms))`, evaluated in the child's own
frame:

| deadline | park | arm fires | process gone by |
|---|---|---|---|
| unset (30000) | 60000 | Δ30.001 s thread / Δ30.008 s process | **60.4 s** thread, 30.5 s process |
| `=200` | 2000 | Δ0.201 s thread / Δ0.207 s process | **2.49 s** thread, 0.69 s process |

⭑ **The top row of that table is a driven regression control on my own edit**: at the default
deadline the derived park is exactly 60000, i.e. byte-identical behaviour to the literal it
replaced, and both tiers still arm at Δ30 s with the thread process lingering to 60.4 s — the same
numbers stone 1 recorded. (Timestamped per output line with the recipe in the probe's header; the
process wall-clock is the trap both stone 1's executor and the orchestrator fell into.) The park
must OUTLAST the deadline — otherwise the child's own recv times out first and the launcher sees
`Lost`, not `TimedOut` — and `10×` keeps a margin an order of magnitude wider than the old `2×`
while making the injected case cheap. On the process tier the nullary is read in the **forked
child**, which is sound because stone 1 row 4A drove the value crossing the fork four times.

⚠ Both edits are to a `.wat` file and both used the **Edit tool**. No python, no sed. The rule has
no carve-out for scratch files — the sibling stone's executor recorded a violation of exactly this,
and the orchestrator let it stand as a violation.

## Row 7 — the cost, as a BAND, with the arithmetic

```
band (same-code, this box, the sibling SCOREs)     540.4 – 561.5 s     (width 21.1 s)
floor 3 — THE RECORD (5258 tests)                  559.222 s
floor 2 — green, 5257 tests                        557.045 s
floor 1 — RED, 5257 tests                          557.580 s
```

**The added wall-clock is not separable from the band.** The group measured alone is **2.557 s of
wall**; its seven tests **sum to 7.202 s** of test time (0.004 + 0.005 + 0.526 + 0.747 + 1.008 +
2.355 + 2.556), and nextest runs them concurrently with everything else. The band's own width is
21.1 s — roughly 3× the group's total test time — so any claim finer than "inside the band" would be
measuring noise and calling it cost. ⭑ The structural reason it is nearly free: the
`every_wat_scripts_file_loads_on_the_current_runtime` gate is ~98 % of the floor's wall-clock (its
own `.config/nextest.toml` note says so; 300 s warn / 600 s kill, `priority = 100`), so seven tests
totalling 7 s land *inside* that gate's window instead of after it.

**Against the DESIGN's ~15 s budget: 7.2 s of test time, 2.6 s of wall. No STOP.**

⚠ **Headroom, stated because a row that times out under load is worse than no row.** Under a full
floor the same rows measured 0.008 / 1.133 / 1.379 / 2.267 / 3.197 / 5.519 s — a **1.4×–2.3×**
contention factor, well under this box's 3.5×–4.4× band for CPU-heavy cohorts
(`.config/nextest.toml`'s rete note). Worst row projected at 4.4× is ~11 s: inside nextest's default
15 s slow-warn, far inside its 30 s kill, and far inside each row's own 20 000 ms SIGKILL. **No
`.config/nextest.toml` override was added**, deliberately — the rows fit the default budget, and
this repo files a widened deadline as a stopgap, not a fix.

## Row 8 — no flaky assertion, and the two that were REMOVED to make that true

Every marker is a **message or a computed value**; never a count, a rate, an elapsed time, or a
scheduler outcome:

| removed | how it was caught |
|---|---|
| `runner 0` (dead-runner row) | **red on its first nextest run**; 6 of 12 concurrent runs print `runner 1`, 10 of 10 sequential print `runner 0` |
| `waited-ms=0` (two-faults row) | **red on the first full floor**; the probe printed `waited-ms=1` for `stop` only, every hand-run prints 0 |

Kept, each defensible: `"clean=[0 2 4 6 8 10 12 14]"` is the **exact return value** of a
deterministic map (not a count of anything); `"control=Message"` / `"a-control=Ok"` are non-vacuity
controls; every other marker is a substrate message string or a frame symbol. Nothing asserts `>`,
`<`, a draw, a rate or a duration. `probe_chaos_gate_has_teeth`'s header is the cited precedent
(*"Do not assert `fires > 0` at 200 bp"*, *"Do not assert a draw count"*).

## Row 10 — ⛔ WHAT IS STILL UNGUARDED

**Recounted on today's tree with my own instrument** — a name scan, whose output is printed whole so
the number is falsifiable (`count_parsed_only.py`, in the session scratchpad; read-only, it never
writes):

```
*.wat under wat-scripts/scratch-pad/ (recursive)           432     (418 top-level + 14 in subdirs)
NAMED by anything under tests/ or src/                      40     (36 before this stone)
PARSED-ONLY — load gate only, executed by nothing          392     (396 before this stone)
```

⚠ **The DESIGN's 416 / 34 / 382 does not reproduce, and the reason is growth, not a wrong count.**
416 is a **top-level** count from when the stone was drawn; top-level is 418 today (stone 1 added
two probes) and the recursive universe is 432. So the honest answer to *"how many of the 382 remain
parsed-only"*: **392 do. This stone moved 4 probes out of that set, and the set grew by more than 4
while the excursus was running.** ⚠ What the instrument cannot see: it matches a basename as a
**substring**, so `probe2.wat` counts as "named" because longer filenames under `tests/` contain
that string — at least one of the 40 is a false positive, which makes **392 a lower bound**. It also
cannot see a path assembled at runtime, a name mentioned only in a doc or shell script, or a name in
dead code under `tests/`.

**Three of this session's own invariants still have NO row, each for a stated reason:**

1. ⛔ **The trap-door demonstration** (`the-handshake-deadline-is-injectable` row 5:
   `owner-recv-loop=GaveUp waited-ms=10000 last=TimedOut` while the knob is at 200 ms). Driven, in a
   SCORE, admissible by the rule — **excluded on COST**: it burns 10 000 ms of `budget-ms` by
   design, 10.47 s measured alone. At this box's contention band that projects 17–46 s, which
   crosses nextest's 30 s kill; a row that can time out under load is worse than no row, and the fix
   is not a widened deadline. **Open, and cheap**: `owner-recv-loop` takes `budget-ms` as a
   parameter, so a probe calling it with a small budget would be a fast row — but that has not been
   driven, so writing the row now would be a guess with a test around it.
2. ⛔ **The value crossing the fork** (`the-handshake-deadline-is-injectable` row 4A, driven at
   500/1500/5000 ms). Excluded as **structurally flaky**: the probe reports through
   `:wat::test::spawn-hermetic-program`, one of the two sites where this deadline is the child's
   **entire runtime budget**. A row there must pick a number small enough to be the point and large
   enough for a forked child to boot the runtime under contention — and the measured starvation
   threshold is 300–500 ms **uncontended**. That is a race by construction (trap-door 1).
3. ⛔ **The child's own `TimedOut` arm** (stone 1 row 4C) — **never driven at all**, so no row is
   admissible under the DESIGN's rule. Unchanged and still ABSENT: `ProcessOpts/launch` sends the
   startup ship immediately after `spawn-program` returns, so the only deadline that loses that race
   is shorter than IPC delivery.

**Also unguarded, and out of scope by the DESIGN's own words:** every new chaos injector (close a
peer mid-send, delay or drop a startup ship, refuse a ring), the two `RecvOutcome` wildcards
`every-status-arm-is-named` found-and-did-not-fix, and the other 392 probes. Message-level chaos
(`drop-*-bp`, `inbox-cap`, `delay-bp`, `chaos-bp`, …) is **already** on the floor and this stone
neither adds to nor weakens it.

## What this stone does NOT do

- **It does not run the corpus.** Five rows, four probes, by the DESIGN's rule; the manifest grows
  one driven SCORE at a time. `MANIFEST` is explicit and must never become a glob — many of the 392
  park 60–120 s deliberately, some were refuted, some are obsolete.
- **It does not delete a dead probe.** Tempting while reading 432 files; deletion needs the builder.
- **It does not supervise a timed-out child.** Unchanged from both predecessors: the runner SIGKILLs
  the `wat` it started, and on the process tier a forked grandchild can still outlive its parent for
  the remainder of its own park. `ps -eo etimes,args | grep release/wat` after every run in this
  stone: nothing left behind (the only `wat` processes on this box are two `wat --mcp` servers,
  present throughout, and there were no strays before or after either floor).
- **It does not use `WAT_RUNTIME_BIN`.** The DESIGN names it as the precedent for locating the
  binary; the runner uses `env!("CARGO_BIN_EXE_wat")` (the `tests/cli/wat_cli.rs` precedent)
  instead, and that is a **correction worth stating**: `WAT_RUNTIME_BIN` exists because a cargo test
  binary's own image cannot serve as a spawned runtime (`src/process/exec_plan.rs`), whereas here the
  spawned thing IS the real `wat` CLI, which holds its own image fd from CLI entry
  (`ImageSource::HeldSelf`). Driven: the process-tier row forks a child successfully with the
  variable unset.
- **No `wat-fix` codemod.** Two bespoke edits in ONE `.wat` file (a park expression, and a `main`
  that branches on argv) — no uniform rule to record, so a codemod would be two special cases. The
  Edit tool per site, as the two sibling stones did for nine and five.

## Blast radius

```
 M Cargo.toml                                                            +11   ([[test]] probes + why)
 M wat-scripts/scratch-pad/probe-a-silent-child-cannot-hang-launch.wat    (park derived; tier from argv; header)
 M wat-scripts/scratch-pad/probe-two-faults-before-stop.wat               (header note only)
 M wat-scripts/scratch-pad/probe-a-dead-runner-orphans-its-item.wat       (header note only)
 M wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat    (header note only)
?? tests/probes/mod.rs                                                   (the include! stub)
?? tests/probes/scratch_pad_manifest.rs                                  (manifest + runner + 7 tests)
?? docs/excursus/2026/08/001-sns-sqs/the-probes-run-in-the-floor/SCORE.md (this file)
```

Only ONE of the four `.wat` files changed behaviour (the silent-child probe: the derived park and
the argv tier); the other three gained a header note and nothing else.

**No `src/`. No `wat/`.** `touch build.rs` was needed once to make the new test group appear (the
build script only declares `rerun-if-changed` for groups it already knows); `build.rs` itself is
unmodified.

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-15 — all rows hold

```
floor    Summary [ 558.291s] 5258 tests run: 5258 passed (9 slow), 22 skipped
         .floor/2026-09-16T06-28-36Z/ · exit=0 · NO ARM.txt · inside the 540.4–561.5 s band
clippy   0 · circuit distinct=8000;dup=0
group    7 tests PASS, 2.84 s wall on my own run (executor measured 2.557 s)
```

| row | the orchestrator's own result |
|---|---|
| 1 | ✅ 7 tests, each named in the Summary: `every_manifest_row_has_its_own_test` · `every_manifest_probe_says_it_is_in_the_floor` · `a_dead_runner_names_the_item_it_orphaned` · `a_silent_child_cannot_hang_launch_on_the_{thread,process}_tier` · `a_handler_raise_does_not_kill_the_service_for_everyone` · `two_faults_before_stop_are_faced_at_all_four_surfaces`. |
| 3 | ✅ **re-driven**: broke `control=Message` → `FAIL` naming `MISSING "\"control=ThisWorldDoesNotExist\""`, whole output printed. Restored, md5 verified. |
| 5 | ✅ **re-driven**: row pointed at a nonexistent probe → **FAIL in 0.004 s**, before any spawn, with an explicit "not a skip" message. Restored, md5 verified. |
| 7 | ✅ 558.291 s, inside the band; the group's 2.84 s is **below the band's own 21.1 s width**, so the added cost is not separable from noise. |
| 10 | ✅ **recounted independently**: `find wat-scripts/scratch-pad -name '*.wat'` = **432**, named by `tests/` or `src/` = **40**, **parsed-only = 392**. The DESIGN's 416/382 was a TOP-LEVEL count (418 top-level today) — the executor's correction is exact. |

### ⭐ THE FIRST FLOOR WAS RED, AND THAT IS THE BEST THING IN THIS STONE

`.floor/2026-09-16T05-43-33Z/` is **still on disk with its `ARM.txt`** — verified by the orchestrator:
`FLOOR RED`, `exit=100`, `Summary [ 557.580s] 5257 tests run: 5256 passed (9 slow), 1 failed`. The
executor captured it, named the arm, fixed the defect and re-weighed. It did **not** re-run and quote a
green.

⛔ **The defect was the MARKER, not the invariant** — and it is the INVERSE of this stone's own
trap-door 2. That trap-door says *"`waited-ms=0` is a real value — do not write a marker that assumes a
nonzero elapsed time."* The marker assumed a **zero** one: `waited-ms=0` is true on every hand-run and
became **`waited-ms=1`** under a loaded floor. ★ **The rule is therefore stronger than it was written: a
marker must not encode elapsed time in EITHER direction.** Markers now carry no timing at all.

⚠ **And the harness reported success while the floor was red.** The executor launched floor 1 as
`floor.sh | tail -25`; the tool result said *"exit code 0"* — the **pipe's** status — while the Summary
line said `1 failed`. That is the trap `CLAUDE.md` names in bold, hit *while building a runner against
exactly that class of silence*. Read the Summary line. Always.

### Two choices the executor made that were not in the brief, and both are right

1. **Tier is `argv[2]`, not a `:locus` source edit.** Both silent-child rows drive from one unmodified
   probe — the orchestrator's own grading of stone 1 had been hand-editing that literal per run.
2. ⭐ **`run_row` `env_remove`s `WAT_STARTUP_HANDSHAKE_DEADLINE_MS` for every row that does not set it.**
   Trap-door 0 is enforced **structurally** rather than by discipline: a future row cannot inherit a
   suite-wide value even if someone exports one.

It also closed a sibling hazard unasked: the probe headers still claimed *"this file is only PARSED …
never run there"*, which stopped being true the moment the runner existed. All four now name the runner,
**gated** by `every_manifest_probe_says_it_is_in_the_floor`, driven red against an unannotated probe.

### An independent re-measurement of the session's oldest number

`timeout 1` → **60.560 s** (SIGTERM ignored); `timeout -s KILL 1` → **1.008 s**. The "a blocked `wat`
ignores SIGTERM" finding, carried since 2026-09-15 morning, is now confirmed by a second measurer.
