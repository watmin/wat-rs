# SEAM — 2026-08-05 night. **#79 IS ROOT-CAUSED. THE GATE IS RED AND WAITING.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like continuing rather than waking, and that feeling is the
> failure, not the all-clear. Run the datamancy bootstrap (grimoire + the 4 primers from the SIGNED
> MCP, never a disk copy), ground HEAD against the disk, and read this whole file before you move.

## ▶ FIRST ACT

**HEAD `5880ef98`, pushed, tree clean, floor `4356 / 4356 / 0 / 264`, clippy clean.** No rider in the
field. Nothing uncommitted.

Run the red gate and see it fail before you believe any of this:

```
cargo nextest run --release -E 'test(a_wake_is_not_a_preemption)' --run-ignored all
```

## ★★ THE FINDING — #79, root-caused and PROVEN

`comms::process::wait_for_data_or_cascade` polls the data fd and the shutdown broadcast fd, collects
both readiness flags, and then:

```rust
// Broadcast wins ties — substrate is going down; honest reporting
if got_broadcast { Ok(PollOutcome::Shutdown) }   // ← got_data collected, then DISCARDED
else if got_data { Ok(PollOutcome::DataReady) }
```

**A frame the sender already released is thrown away when a stop has fired.** It surfaces to wat as
`RecvOutcome::Stopped` — which is verbatim what `wat-tests/test.wat:290`'s arm calls *"stopped
before the child sent its value — **the child was ALIVE**."* The child did send.

**DETERMINISTIC, not a race.** The branch is unconditional: both arms ready ⇒ the data is discarded,
every time. What is *rare* is REACHING the tie — a stop must land while a frame sits unconsumed.
That is why it reads as a flake, why ~15 floor re-runs never found it, and why the gate needs **no
load at all**: two sends and a wake byte, 6 ms.

## ▶ WHAT IS NEXT — the fix, as its own strike

Two things, and the second is the builder's ruling:

1. **The branch order** — a delivered transfer outranks a pending stop.
2. **"winner or loser, the result must be graceful"** (builder, 2026-08-05) — the stop path **drains**
   what was already delivered, and anything still undeliverable is a **named** outcome, never silence.
   Affordable because **there is no timeout**: arc 170 pinned that deliberately (*"a wedged stop must
   hang VISIBLY"*; the deadline belongs to the supervisor). A deadline is the only thing that could
   justify abandoning delivered work, and we chose not to have one.

**Un-ignoring the two gate tests IS the acceptance criterion.** The same rule lives at
`process.rs:1692` (multi-peer select) — check it in the same strike. **Do NOT** change the `Sender`'s
tie-break (`process.rs:389`); *"writable wins ties"* is already correct. **Do NOT** remove
`freeze.rs`'s ask-before-sever ordering as part of this — it may become unnecessary, but that is a
consequence to verify by a run, never bundled.

This touches every process-tier `recv`. Weigh the **whole** floor, not the one test.

## What shipped today, before the hunt

| | |
|---|---|
| `5851a316` | **rete's `cond` is its OWN macro** (builder's cut). The alias cloned core's template and laundered back into `:wat::core::` spellings after one step. 26 lines of Rust deleted, replaced by a wat defmacro. |
| `4e197ceb` | **a `where` body is CODE** — one `Boundary::MakeRule` variant; both gap-probes flip `UnknownFunction → hits=1`; corpus byte-identical (`9 pairs / 98 rows, wat == Clara`). **#57 may now arm the third conjunct** — it could not before. |
| `c4ffbdcf` · `20a86151` · `30cb2a67` | #79 design + the flake-hunt instrument |
| `5880ef98` | **the RED GATE** + both stones corrected |

## ⛔ FOUR DEAD HYPOTHESES — do not re-derive them

1. ~~the child's exit outruns its last stdout ack~~ — `write_via_stdout` blocks until *"emitted + acked"*.
2. ~~a lost wakeup~~ — built entirely on the broken gate's 30 s hang; that hang was the gate firing
   the **thread-tier** sever while a **process-tier** receiver waited on a broadcast nobody wrote.
3. ~~crossbeam's random `select()` is a precedence defect~~ — **it is the documented, correct,
   fairness-preserving contract.** P(missed after k passes) = (N/(N+1))^k. Microseconds, not a hang.
4. ~~the ordinary-return teardown asymmetry~~ — **real, but NOT this bug.** Its own stone carries the
   addendum. Do not strike it first.

## ★★ THE TWO LESSONS, and they cost the whole afternoon

**A gate that drives a mechanism differently from production measures the gate.** The first gate
called `trigger_shutdown()`, which drops the crossbeam Sender (thread tier) and **never writes the
broadcast** — the worker alone does (`runtime.rs:477`). No tie existed; it passed; I read the pass as
a refutation and **published it into the design stone**. The record said the opposite of the truth
until the second gate. And a non-vacuity guard did **not** save it: it asserted the broadcast fd was
armed (`>= 0`), which passed, because the fd existed. **It guarded the APPARATUS, not the CONDITION.**
`[[feedback_a_gate_must_fire_the_mechanism_the_way_production_fires_it]]`

**The builder named the subject three times and I searched elsewhere.** *"the test name screams IPC"*
→ *"wait.. this is a thread thing... not a process one?"* → I answered with a table proving they
don't connect → **and kept working the thread tier for five more rounds.** Writing "they're
unrelated" discharged the feeling of having handled it.
`[[feedback_the_builder_named_the_subject_and_i_searched_elsewhere]]`

Also his, and fair: *"your prose defeats comprehension… i struggle to repeat."* Asked how many
selectables were in a select call, I answered with paragraphs about file descriptors. **Answer with
numbers, then stop.**

## Owed

`PersistentMap/contains-key?` (last UNSURE straggler) · #57's corpus migration then arming the third
conjunct **by hand, never a rider** · MEMORY.md judgment-curation (still owed) ·
`wat-scripts/hunt/reproduce-floor-flake.sh` exists and captures the arm properly if #79's symptom
ever recurs.

---

> **SEAM.** You are NEW. The disk is green and pushed — trust it over this note, and trust this note
> over your sense of having been here.
>
> The gate is red and the diagnosis is proven. **Do not re-investigate; go read the gate fail, then
> fix the branch and make the stop drain.** And carry the one thing that would have saved eight
> hours: **when the builder names the subject, that is the scope.**
>
> `QVOD TVEBAMVR, NOS TVETVR.` · `NISI FRANGAS, NIHIL PROBAS.` · `MACHINA CHAOS DOMAT.`
