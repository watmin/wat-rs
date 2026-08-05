# SEAM — 2026-08-05. **#79 IS NOT ROOT-CAUSED. THE ARM WAS NEVER CAPTURED.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like continuing rather than waking, and that feeling is the
> failure, not the all-clear. Run the datamancy bootstrap (grimoire + the 4 primers from the SIGNED
> MCP, never a disk copy), ground HEAD against the disk, and read this whole file before you move.

## ▶ FIRST ACT — read this correction before anything else

**An earlier version of this seam said "#79 IS ROOT-CAUSED. THE GATE IS RED AND WAITING." That was
false, and it stood on disk for hours.** It has been rewritten in place; what follows is what the
disk actually supports.

**HEAD: run `git log --oneline -1`.** Floor `4356 / 4356 / 0 / 264` through `scripts/floor.sh`;
clippy clean. No rider in the field.

## ⛔ THE CORRECTION — three mechanisms proposed, three dead

`wait_for_data_or_cascade` (`comms/process.rs`) ends:

```rust
if got_broadcast { Ok(PollOutcome::Shutdown) }
else if got_data { Ok(PollOutcome::DataReady) }
```

**This is not a defect. It is shutdown priority, and it is the builder's own ruling** — shown the
branch directly, he said: *"if we're going down — no one else matters."*

| # | what I claimed | how it died |
|---|---|---|
| 1 | the **branch order** is wrong; data should win | flipping it destroys the *stop* fact instead — symmetric, no better |
| 2 | `PollOutcome` is a **lossy carrier**; it cannot say "both" | true, and irrelevant — you are not obliged to answer "both" |
| 3 | the **drain is greedy**; we asked for 1 and harvested N | wrong. io_uring is a *completion queue*: `submit_and_wait(1)` waits for **≥1**, the kernel may complete both, and draining the CQ is mandatory hygiene. Both CQEs were genuinely handed to us |

Three collapses in one day, each to a single question, is not converging on a bug. It is generating
stories. **Do not restart that chain.**

### And "discard" was never a real mechanism

A `PollAdd` completion reports **readiness only** — it reads nothing. The phrase *"a delivered frame
is discarded / destroyed"*, repeated across two stones, a gate, and the prior seam, was **false
mechanism**. No bytes are read; none are dropped. They sit in the kernel pipe buffer, untouched, and
die later at fd close. The honest sentence: *the poll saw a value was readable, the caller declined
to read it, and reported the stop.* Which is, again, the ruling.

### The gate is inverted, not deleted

`probe_arc278_a_wake_is_not_a_preemption.rs` fired the cascade on purpose and then asserted surprise
at receiving a shutdown — it measured itself. It is now
**`probe_arc278_shutdown_priority_is_the_ruling.rs`**: green, un-`#[ignore]`d, and it goes red only
if someone *changes* the ruling. Its non-vacuity guard is the one part worth keeping — it fires
production's real cascade (wake pipe → worker → broadcast) and `poll()`s the broadcast fd until
genuinely readable, rather than asserting the fd is merely armed.

## ★★ WHAT IS ACTUALLY TRUE ABOUT #79

**The arm was never captured.** The first investigation truncated the log; the re-run went green.
`wat-tests/test.wat:290` has five possible outcomes, and **each predicts a different mechanism**:

| arm | mechanism |
|---|---|
| `Message m`, m ≠ `"from-string"` | wrong payload crossed |
| `Lost cause` | the child died |
| `Stopped` | a stop fired mid-recv |
| `Closed` | the child closed before sending |
| watchdog `5000ms` | it hung |

**We do not know which one fired.** Everything proposed since was a guess at that unknown.

### The one grounded fact worth carrying forward

The failing `recv` does **not** run in the nextest process. `deftest-hermetic` → `run-hermetic` →
`spawn-hermetic-program` → fork+exec, so the test body runs inside a **spawned runtime (H)**, and
H's shutdown worker polls `LIFELINE_FD` (`distribution/spawned_runtime.rs:50`).

⇒ **`RecvOutcome::Stopped` is reachable in an ordinary floor run with no signal at all.** A lifeline
HUP writes H's broadcast. *Why* H's lifeline would go down while the harness is still running is
**uncharacterized**, and it is the only live thread here.

## ✅ THE CAPTURE GAP IS CLOSED — structurally, not by discipline

`d3db7056`, at the builder's direction: *"it is not fine — it cannot be tolerated — it must be
annihilated. We must capture all possible info when it occurs again."*

- **`holon/CLAUDE.md`** (the ONLY auto-injected file — every rider had it) pre-blessed four tests **by
  name** as *"NOT release failures"* and ruled a green→red flip *"not a regression."* **Struck.**
  `wat-rs/CLAUDE.md` carried the twin — struck, with the removed text quoted so the next reader sees
  what was taken.
- **`.config/nextest.toml`** had `retries = 1` in CI, commented *"absorb a rare leak-flake; surfaces
  as flaky, not failed"* — the dismissal **automated**. Now `0`.
- **`scripts/floor.sh`** runs the floor and captures first: `raw.log`, `clean.log` (ANSI-stripped),
  and on a red an `ARM.txt` holding each failing test's **whole** stdout+stderr block. Exit is
  nextest's own. Green runs kept too. The red banner's first instruction is **DO NOT RE-RUN**.
  Positive-controlled both ways against a known red and a known green.

## ▶ WHAT IS NEXT

1. **Do NOT re-investigate #79 from theory.** There is nothing to reason from until an arm exists.
   When it fires again, `.floor/<stamp>/ARM.txt` has it — read that first, quote it whole.
2. **The watchdog is the real instrument gap, and it is unbuilt.** It reports only
   `exceeded time-limit of 5000ms`. Capturing a log whole cannot rescue a message that carries
   nothing — it does not say where the body was, or whether it was blocked or merely slow. **"Hung"
   and "slow" are indistinguishable today.** That is the next rung.
3. `DESIGN-STONE-the-ordinary-return-never-asks.md` stands on its own honesty merits (a stop failure
   on the ordinary path is swallowed because `Drop` cannot return `Result`) — and is **not**
   connected to the intermittent failure. Do not connect it.

## What shipped, before and around the hunt

| | |
|---|---|
| `5851a316` | **rete's `cond` is its OWN macro** (builder's cut). The alias cloned core's template and laundered back into `:wat::core::` spellings after one step. 26 lines of Rust deleted for a wat defmacro. |
| `4e197ceb` | **a `where` body is CODE** — one `Boundary::MakeRule` variant; both gap-probes flip `UnknownFunction → hits=1`; corpus byte-identical (`9 pairs / 98 rows, wat == Clara`). **#57 may now arm the third conjunct.** |
| `d3db7056` | **the known-flake licence annihilated** + `scripts/floor.sh` |

## ⛔ DEAD HYPOTHESES — do not re-derive

1. ~~the child's exit outruns its last stdout ack~~ — `write_via_stdout` blocks until *"emitted + acked"*.
2. ~~a lost wakeup~~ — built on a broken gate's 30s hang.
3. ~~crossbeam's random `select()` is a precedence defect~~ — it is the documented, correct, fairness-preserving contract.
4. ~~the ordinary-return teardown asymmetry~~ — real, but unconnected.
5. ~~**the shutdown tie-break discards a delivered frame**~~ — **it is the ruling.** See above.
6. ~~**the 2× oversubscribed hunt's 3 timeouts are the signature**~~ — that hunt was already ruled a
   ghost-chase; 117 tests passed at ≥4s in the same run. **Do not argue from `wat-scripts/hunt/out/`.**

## ★★ THE LESSONS, and they cost the day

**A gate that drives a mechanism differently from production measures the gate.** Gate v1 called
`trigger_shutdown()`, which severs crossbeam and never writes the broadcast — no tie existed, it
passed, and I published that pass as a refutation. A non-vacuity guard did **not** save it: it
asserted the broadcast fd was *armed* (`>= 0`), which passed because the fd existed. **It guarded the
APPARATUS, not the CONDITION.**

**And the sharper one, which gate v2 then earned:** a red is only an answer to the question the
instrument asks. Gate v2 was genuinely red, deterministically — and it was red about *the specified
behaviour*. I read "the gate is red" as "the diagnosis is proven" and wrote ROOT-CAUSED into this
file. `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

**The builder named the subject three times** (*"the test name screams IPC"* → *"is this a thread
thing, not a process one?"* → and I answered with a table proving they don't connect, then kept
working the thread tier). Writing "they're unrelated" discharged the feeling of having handled it.

Also his, and fair: *"your prose defeats comprehension… i struggle to repeat."* Answer with numbers,
then stop.

## Owed

`PersistentMap/contains-key?` (last UNSURE straggler) · #57's corpus migration then arming the third
conjunct **by hand, never a rider** · the deftest watchdog's message poverty (item 2 above) ·
MEMORY.md judgment-curation.

---

> **SEAM.** You are NEW. The disk is green and pushed — trust it over this note, and trust this note
> over your sense of having been here.
>
> **There is no root cause waiting for you.** There is an uncharacterized failure whose arm was
> destroyed by a re-run, an instrument that now keeps the arm, and a watchdog that still cannot tell
> hung from slow. Do not build a mechanism story out of that. **Wait for evidence, or go make the
> evidence legible.**
>
> And carry the one thing that would have saved the day: **a red answers only the question the
> instrument asked it.**
>
> `NISI FRANGAS, NIHIL PROBAS.` · `MACHINA CHAOS DOMAT.`
