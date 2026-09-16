# DESIGN — the probes run in the floor

**Drawn 2026-09-15**, builder-directed: *"manifest runner then"* — his answer to *"do we have chaos
tooling to exercise these paths we've been adding?"* **Stone 2 of 2**; depends on
`the-handshake-deadline-is-injectable`. **NOT STRUCK.**

## The measurement that produced this stone

```
wat-scripts/scratch-pad/*.wat                                416
named by anything in tests/ or src/                           34
PARSED-ONLY — type-checked by the load gate, never executed  382
```

**All three probes written today are in the 382.** So is every driven measurement this session: the
handshake `TimedOut` at both tiers, the blocked `send`, the two-faults `GaveUp`. Each was proven **once,
by hand, on one box**, and then went unguarded. That is R59 `NISI FRANGAS, NIHIL PROBAS` turned on our own
work — a green number nothing depends on is a claim — and it is why a CI red sat for **three days**
(`a-deadline-does-not-cost-a-ring`): nothing we added was watching.

⚠ **What the floor DOES cover, stated so this stone is not oversold.** Message-level chaos is real and
re-runs every time: `drop-check-bp` · `drop-mark-bp` · `drop-recv-bp` · `drop-ack-bp` · `inbox-cap` ·
`inbox-vis-ms` · `delay-bp`/`delay-ms` · `publish-ceiling-ms` · `chaos-bp` (the umbrella rate that fills
whichever drop knobs are unset), ridden by `probe_chaos_gate_has_teeth`, `probe_ex001_fanout`, the
`probe_queue_*` family, `probe_severed_reaches_the_client`. **The gap is LIFECYCLE-level**, which is where
every arm added today lives: a silent child at spawn, a peer that closes mid-send, two faults queued ahead
of a stop ack, a kernel refusing a ring.

## What this stone is

A rust test that reads an **explicit manifest** and executes the probes it names, asserting on their
output. **Not a glob.** Of the 382, many park 60–120 s deliberately, some were refuted by later
measurement, some are obsolete. A runner that executes everything it finds would be slow, flaky and
would resurrect dead probes as if they were live — the graveyard the scratch-pad convention exists to
prevent, with a test attached.

Manifest row shape (EDN, house form — a record, not positional; `#fanout/Input` is the precedent):

```
{:path "wat-scripts/scratch-pad/probe-two-faults-before-stop.wat"
 :expect-exit 0
 :expect-markers ["stop     =GaveUp" "hibernate=GaveUp" "grant    =GaveUp" "revoke   =GaveUp"]
 :timeout-ms 20000
 :env {"WAT_STARTUP_HANDSHAKE_DEADLINE_MS" "200"}}
```

⭑ **`:expect-markers` are substrings, and each row must name a marker no OTHER world prints.** The
pre-stone world for that probe printed `defservice stop: expected Status::Stopped`; a row asserting only
`exit=0` would pass on a probe that had silently stopped testing anything. One marker per invariant the
probe exists to prove.

## The first rows — and the discipline for adding one

Seed with this session's invariants, each of which has a **driven** measurement already in a SCORE:

| probe | asserts | cost |
|---|---|---|
| `probe-two-faults-before-stop` | 4× `GaveUp` — a second `Faulted` is faced, not fatal | ~1 s |
| `probe-a-silent-child-cannot-hang-launch` | the handshake `TimedOut` arm fires, **both tiers** | ~0.5 s **only with stone 1**; 60 s without |
| `probe-a-dead-runner-orphans-its-item` | brackets names the orphaned item | verify |
| `probe-a-handler-raise-kills-the-service` | a raise is a graceful stop with all three parties told | verify |

⛔ **A row may only be added for a probe whose expected output was DRIVEN and recorded in a SCORE.** A row
written from reading the probe is a guess with a test around it.

## Mechanics to settle, with the reasons

- **Subprocess, not in-process.** Most of these are process-tier and spawn children; `WAT_RUNTIME_BIN` is
  the existing precedent for locating the binary from a test. In-process (`startup_from_file`, as
  `probe_queue_*` do) cannot carry per-row env or a kill-on-timeout.
- **`timeout -k` semantics are mandatory.** A blocked `wat` ignores SIGTERM — **125 s measured**. The
  runner must SIGKILL, and a row that times out is a **FAIL with the whole captured output**, never a skip.
- **Capture whole, print whole on failure.** No `head`/`tail` window
  (`[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]`).
- **One nextest test per row**, not one test looping rows: a loop stops at the first failure and hides the
  rest (`[[feedback_the_first_failure_hides_the_rest]]`), and per-row tests name themselves in the Summary.
- **Budget.** Report the added wall-clock against the 540.4–561.5 s floor band. If the seed rows cost more
  than ~15 s total, say so and let the builder rule rather than quietly growing the floor.

## Scope — explicitly NOT in this stone

- **New chaos injectors** (close a peer mid-send; delay or drop a startup ship; refuse a ring). Those are
  substrate hooks and a bigger effort; this stone only re-drives what already exists.
- **Migrating the other 378 probes.** The manifest starts small on purpose and grows by the rule above.
- **Deleting dead probes.** Tempting while reading 416 files; a separate pass, and deletion needs the
  builder.

## Trap-doors

1. ⛔ **Do not assert flaky counts.** `probe_chaos_gate_has_teeth`'s own header is the precedent: *"Do not
   assert `fires > 0` at 200 bp (≈1-in-170 flake)"*, *"Do not assert a draw count"*. Assert the
   **invariant**, never the statistic.
2. **`waited-ms=0` is a real value** in today's `GaveUp` output — do not write a marker that assumes a
   nonzero elapsed time.
3. **A probe that parks by design must not be given a short timeout and called fixed.** If a row needs
   stone 1's knob to be fast, it waits for stone 1.
4. **The runner must fail loudly if a manifest row's path does not exist**, or a renamed probe silently
   stops being tested.
