# BRIEF — 278: a liveness bound's only job is to catch a hang

> Builder, 2026-08-15: *"let's get these tests more breathing room and prove it."*
>
> Baseline: HEAD `bc277511` **plus uncommitted work** — 26 Wave-B1 goldens, 26 lifted `#[ignore]`s,
> and the arc-198 span fix. Floor **4560 run / 4559 passed / 1 failed / 128 skipped**, clippy 0.
> **That 1 red is this brief's subject.**

## WHAT HAPPENED

`probe_arc278_partial_frame_residue::probe_sender_send_leaves_headless_partial_frame_on_shutdown`
went red on a full floor with:

```
tests/comms/probe_arc278_partial_frame_residue.rs:287:9
  the child did not report within 3s of SIGTERM — the blocked send never woke; this is the
  poll-arm-missing RED state probe_arc278_send_poll_arm.rs already covers, not this probe's subject.
```

**Measured, in isolation, 5 runs:** the same wait completes in **9.236 ms**. Budget 3000 ms — a
**325× margin**. The test passes 5/5 alone and failed once under a 45-binary parallel floor.

**The defect is not the IPC.** Nothing implicates it: 9.2 ms typical, 5/5 green isolated. The defect is
that **a 3-second wall-clock bound on another OS process's scheduling cannot distinguish its own
subject from CPU contention.** The panic message admits it — the condition that fired is explicitly
*"not this probe's subject."* A probe that can go red for a cause it disclaims is not isolated.

`mora`: *time is I/O; it arrives as an fd-event or it doesn't arrive honestly.* The wait itself is
fine — it arrives on a channel. The **bound** is the guess.

## ⛔ THE LOAD-BEARING DISTINCTION — three roles, and only ONE gets raised

A blanket raise breaks this test. Classify every value before touching it:

| role | what it is | action |
|---|---|---|
| **LIVENESS BOUND** | "if this doesn't arrive, something is stuck" | **RAISE** — only a hang may trip it |
| **WINDOW** | *creates* an observable condition for a fixed period | **DO NOT RAISE** — it is the scenario |
| **NEGATIVE ASSERTION** | proves absence by waiting ("did NOT return within X") | **COUPLED** — see below |

### The inventory, classified

**`tests/comms/probe_arc278_partial_frame_residue.rs`**
- `:182` `sleep(3s)` — **WINDOW.** The CHILD holds its write fd open so the parent can observe the
  "writer fd still open" case, then exits. This is the scenario, not a wait. **Do not raise it to give
  breathing room** — it only slows the test.
- `:284` `recv_timeout(3s)` — **LIVENESS.** The failing one. Measured 9.236 ms. ← raise
- `:356` `recv_timeout(1500ms)` — **NEGATIVE ASSERTION** ("recv did NOT return within 1.5s while the
  writer was still alive"). ⛔ **COUPLED TO `:182`**: it must complete inside the child's open-fd
  window or the premise evaporates. If you change either, you change both, and you must say why.
- `:373` `Instant::now() + 6s` with `:378` `sleep(20ms)` poll — **LIVENESS** (waiting for child exit).
  ← raise
- `:392` `recv_timeout(2s)` — **LIVENESS** (after the writer closes, recv must return). ← raise

**`tests/comms/probe_arc278_send_poll_arm.rs`**
- `:227` `recv_timeout(3s)` — **LIVENESS** ← raise
- `:243` `now + 3s` with `:248` `sleep(10ms)` poll — **LIVENESS** ← raise

**`tests/comms/probe_select_flood_no_deadlock.rs`**
- `:66` `arm_watchdog(10s)` — **LIVENESS** (already a watchdog) ← raise
- `:160` `sleep(timeout)` — classify it before deciding. Report what it is.

**`tests/comms/process.rs`**
- `:309` `sleep(50ms)` — classify it. If it is a settle-and-hope, that is a finding to report, not
  necessarily to fix here.

**Verify this inventory against the disk before acting** — every number in this arc's briefs has been
wrong at least once.

## THE CHANGE

Each **LIVENESS** bound goes to a value only a hang can trip. **60s** is the suggested floor; justify
anything different. Beside each, record in a comment:

```rust
// LIVENESS BOUND — only a hang may trip this. Measured typical: 9.24ms (isolated, 5 runs,
// 2026-08-15). 60s is ~6500x that, so a red here means STUCK, never "the box was busy".
// A bound that competes with the scheduler cannot tell its subject from CPU contention.
```

The measured typical is the point: it turns "why 60?" into arithmetic instead of taste.

**Do NOT change any WINDOW.** **Do NOT decouple the negative assertion from its window.**

## ⛔ THE PROOF — this is what "prove it" means, and a raise alone does not do it

A bound raised until it never fires is not a fixed bound, it is a deleted one. **Every raised liveness
bound must be shown to still catch a real hang.**

1. **Negative control, per file** (both files with liveness bounds): induce a genuine hang — e.g.
   temporarily prevent the child from reporting — rebuild, run, and confirm the bound **still fires**
   with its message. Then revert and confirm green. `git diff` must show no residue.
   Without this the change proves nothing (`[[feedback_a_green_test_can_prove_nothing]]`).
2. **The subject still passes.** All assertions about partial-frame residue, fill counts, and the
   writer-open/closed cases must be unchanged and still asserted. **This strike changes bounds, not
   subjects.** If an assertion has to weaken, that is STOP-2.
3. **The window coupling holds.** If `:182`/`:356` moved, show the negative assertion still completes
   inside the child's open-fd window.
4. **The floor goes green** — `4560 run / 4560 passed / 128 skipped`, and the previously-red test is
   green **for the right reason** (its subject's assertions ran and passed, not because a bound got
   loose enough to skip them).
5. **Report the measured typical for every bound you raised**, from an actual run.

## STOP TRIGGERS

- **STOP-1 — a bound cannot be made to fire in the negative control.** Then it is decorative. Report;
  do not ship it.
- **STOP-2 — a subject assertion would have to weaken to reach green.** Never. Report.
- **STOP-3 — the red reproduces in ISOLATION.** Then it is not contention and this brief's premise is
  wrong. Capture verbatim and STOP — that would be a real IPC liveness defect.
- **STOP-4 — a value you cannot classify** as liveness / window / negative-assertion. Report it rather
  than guessing; misclassifying is how this strike breaks the test.

## BLAST RADIUS

`tests/comms/probe_arc278_partial_frame_residue.rs`,
`tests/comms/probe_arc278_send_poll_arm.rs`, `tests/comms/probe_select_flood_no_deadlock.rs`,
`tests/comms/process.rs`. **No `src/` changes. No `.wat` corpus changes.** Do not touch the
uncommitted Wave-B1 goldens or the arc-198 span fix.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D warnings`
(0), then `scripts/floor.sh` — read the **Summary line**, never a piped exit code. Expect
`1 failed → 0 failed` at `4560`.

**On any red you did not intend: do NOT re-run.** Copy the whole stdout+stderr block **verbatim** —
never a `| head` window — name the exact assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you.** ⛔ **Run every build and test in the FOREGROUND and
block on it. Do NOT use `run_in_background`. Do NOT set a Monitor. Do NOT poll and stop.** THREE riders
on this arc died exactly that way. If you are about to wait for a notification, you are about to die —
run it in the foreground instead.

Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. **The tree holds uncommitted work — 26 Wave-B1
goldens, 26 lifted ignores, the arc-198 span fix. Do not revert it, do not re-ignore anything.** Leave
your work uncommitted. Never `git commit`/`push`/`stash`/`revert`/`checkout --`; `stash@{0}` holds
unrelated work.

## REPORT

- the classified inventory: every wall-clock value, its role, raised or not, and why
- **the negative control per file, both directions**, with the message the induced hang produced
- the measured typical beside each raised bound
- confirmation that no subject assertion changed
- the floor Summary line verbatim with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.**
