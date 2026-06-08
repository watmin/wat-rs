# SCORE — Stone 6.3: fork.rs dies — the process family rehomes

**Mode B flight, Mode A stone.** Sonnet executed the lift correctly but
VIOLATED THE ENVELOPE on the comms gate (ran it bare — no setsid/timeout),
hung on a pre-existing race, burned its remaining budget waiting, and ended
with a truncated report ("All four deleted"). The orchestrator completed the
scoring. The lift itself is sound.

## Scorecard (every row = orchestrator's own re-run/read)

| # | Row | Result |
|---|-----|--------|
| 1 | Gate-probe 63 2/2 GREEN; all four flat files GONE | ✓ own runs |
| 2 | Home: process/{mod 60, clone 541, child 289, handle 129, verbs 1566, stdio 141} = **2726** (fresh wc; flat was 2815) — the intueri-cast layout executed | ✓ read |
| 3 | Visibility widenings: 7 pub(crate)/pub(super) in-home (the allowed class) | ✓ own grep |
| 4 | lib 943/0/1 · nursery 865/4/4 (4 = parked-255; the 63 gate +2) · alpha 12/0 · check --all-targets 0 · clippy-in-home 0 | ✓ own runs |
| 5 | Process binaries ENVELOPED: channel_pipes 23/0 · gamma 5/0 · hermetic 2/0 · **comms 50/0/8 in 0.28s** | ✓ own runs |
| 6 | FULL CORPUS **661/0/54** (grew 649→661 — the runner re-included rehomed binaries), histogram all-zero | ✓ own run |
| 7 | The lift is LIVE in the wild: the hang's gdb stacks showed `wat::process::clone::spawn_lifelined` + `wat::process::child::run_in_fork` frames — the home's paths on a real call stack | ✓ builder's gdb capture |

## The live catch — THE FORK-ZOMBIE SHUTDOWN INFRA (diagnosed, not fixed here)

The comms gate hang was diagnosed LIVE (builder + orchestrator, gdb +
/proc): `shutdown_cascade_memory` — the arc-253 Slice-B detector — hangs
IN-SUITE because an earlier test inits the shutdown infra in the PARENT;
the clone3 child inherits `SHUTDOWN_RX = Some` but NOT the worker thread;
`init_shutdown_signal`'s guard (runtime.rs:233) no-ops; SIGTERM's wake byte
has no reader; the blocked recv never wakes. **Passes alone, hangs in-suite
— deterministic, not a race.** Pre-existing (5.1-era exposure; the lift
never touched the shutdown machinery — verified by the chain's file set).

THE CLASS: OnceLock'd global infrastructure with an attendant worker thread
does not survive fork — the state survives, the thread doesn't, and the
idempotence guard turns rebirth into a lie. Post-Slice-8 every production
parent is multithreaded (the service trio), so the class is architectural.

Disposition: both cascade detectors `#[ignore]`'d with the full attested
diagnosis + the 6.4 citation (un-ignored by 6.4). **Stone 6.4 — THE REBIRTH
GATE — drawn next**: pid-aware guard (the guard can no longer lie) + rebirth
wired into `child_post_fork_init_preserving` + fork+exec banked as the
top-rung arc. Hard-blocks the stability-100 soak (#207).

## Flight discipline findings
- **Envelope violation (Mode B)**: the brief specified `setsid timeout 120`
  for every process binary; sonnet ran comms bare — which converted a
  would-be 120s timeout diagnostic into an eternal hang the builder had to
  hand-kill. The envelope exists for exactly this; the violation cost the
  flight its remaining budget.
- The truncated report meant the orchestrator self-served the gate evidence
  — all rows above are orchestrator-run regardless (the standing doctrine).

## Slice 6 standing
6.1 ✓ (typed_channel dead) · 6.2 ✓ (corpses purged) · 6.3 ✓ (the family
home; **zero flat concurrency files remain in src/**) · NEXT: 6.4 (the
rebirth gate) → 6.w (ward channel/ + process/ + touch-audits).
