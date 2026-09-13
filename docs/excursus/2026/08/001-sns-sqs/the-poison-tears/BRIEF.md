# BRIEF — the poison tears

**Read `DESIGN.md` beside this first.** It carries the measured fact the whole fix rests on (an oversized
frame **does** tear the sender's connection), the contract decision, and the invariant that is the proof.

## The work, in one paragraph

`disrupt-bp` is not a chaos knob at any rate: the poison it sends produces `RecvOutcome::Malformed`, the
`tore?` predicate only admits `lost`/`closed`, and the `Malformed` arm **raises**, so a fired poison kills
the worker. Make `Malformed` a tear — return a tear marker instead of raising, admit it in `tore?`, and
correct the comment that claims the counter means lost/closed. Then `disrupt-hits` can increment, and a run
at a firing rate completes.

## The rooms — all in `wat-scripts/fanout/circuit.wat`

| where | why you are going there |
|---|---|
| `:385–392` | the `disrupt-draws` / `disrupt-fires` / `disrupt-hits` comment block. ⭑ `:387` claims *"the poisoned call came back **lost/closed**: it TORE"* — **this comment is the bug's documentation.** Correct it to say what `tore?` will now mean. Read `:388–391` too: it records `draws=256, hits=0` and says *"that question is why this field exists"* — `disrupt-fires` was added for exactly this and is already right. |
| `:528` | `hit? (< bp rate)` — the dice roll. **Do not touch.** |
| `:531–534` | the pad: `10 chars × 200` = 2000 bytes, against `:max-frame-bytes 256` at `:102`. This is why the frame is rejected. |
| `:538–543` | ⭑ **THE ROOM.** The poisoned-call match. `Message→"message"`, `Lost→"lost"`, `Closed→"closed"`, `Stopped→raise`, `TimedOut→"lost"`, and **`Malformed→assertion-failed!` — the arm.** |
| `:544` | `tore? (or (= poisoned "lost") (= poisoned "closed"))` — admit the new marker here. |
| `:545–549` | the redial on `tore?`. ⭑ Already correct; it is the right response to a torn connection. ⚠ Its own raise at `:547` (*"redial seen failed — peer is dead, not a broken pipe"*) **stays** — a dead service is not a momentary failure. |
| `:550–552` | `hits'` — increments on `tore?`. Already correct; it will simply start firing. |
| `:553–555` | `points'` — also gated on `tore?`; it will start recording draw numbers. Expected, not a bug. |
| `:574–576` | `disrupt-fires`, counted on `hit?` where the poison is SENT. **Do not touch** — that separation is what exposed this defect. |
| `wat-scripts/scratch-pad/probe-crash-surface-frame-cap.wat` | ⭑ the evidence. Run it: `a-big=Malformed`, `b-other=Message/Ok`, `a-again=Closed`. The service lives; the sender's connection is dead. |

## Implementation sketch

```
;; :543 — was: (Malformed _cause) → assertion-failed! "… UNMIGRATED PLACEHOLDER"
((:wat::kernel::RecvOutcome::Malformed _cause) "malformed")

;; :544
tore? (:wat::core::or (:wat::core::= poisoned "lost")
        (:wat::core::or (:wat::core::= poisoned "closed")
                        (:wat::core::= poisoned "malformed")))
```

⚠ **The sketch is a convenience; the disk is the contract.** Check how `or` arity works here and match the
file's existing style — two of my sketches have been wrong in this campaign and the executor was right to
follow the declarations both times.

## ⭑⭑ The proof — an equality, not an eyeball

Run the circuit at a rate that actually fires and show **both**:

1. **`disrupt-fires == disrupt-hits`, both `> 0`.** The poison deterministically yields `Malformed` and
   `Malformed` always tears, so the two counters must agree exactly. Before this stone `hits` is pinned at
   **0** while `fires` can be positive — that gap is the bug.
2. **The run COMPLETES.** The FINDING measured `disrupt-bp=10000` stalling with `arrived=0`, polls=601,
   because the worker died on the placeholder. After the fix a firing rate must finish with **`dup=0`**.

Report the phase line verbatim for both a firing rate and a zero rate. ⚠ Expect the firing run to be
**slower** (a redial per fire) — report the wall clock, do not hide it.

## Verify

- `./scripts/floor.sh`, read the **Summary line** → **5237 passed, 0 FAIL**, no `ARM.txt`. `circuit.wat` is
  in the `every_wat_scripts_file_loads` gate, so a type error reddens the floor.
- ⛔ **Never a piped exit code** — a type-error run this session reported `$?` = 0 through a `| head`; its
  true exit was **3**, on stderr.
- `cargo nextest run --release --no-run` — the build does not compile tests.
- `cargo clippy --release --workspace --all-targets -- -D warnings` → 0.
- The **unperturbed** happy path unchanged: `circuit.wat 2000 4 3 8192 true 1000` → `distinct=8000;dup=0`.
- The shipped chaos line still green: `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` → exit 0.
- `git diff --stat -- src/ wat/` must be **EMPTY** — this is a `wat-scripts/` change only.
- Everything heavy through `./scripts/capped.sh --limit 8g`.

## STOP triggers

1. **STOP-1 — if `Malformed` turns out NOT to tear the connection** in your own run of the frame-cap probe,
   STOP. The whole fix rests on `a-again=Closed`; if you cannot reproduce it, the redial is wrong and I want
   to know before it ships.
2. **STOP-2 — do not touch `disrupt-fires` or `hit?`.** Counted at the send, deliberately. If you believe
   they are wrong, STOP and say so; do not adjust them to make an equality come out.
3. **STOP-3 — do not migrate any other placeholder arm.** This stone is **one** arm. If a firing run trips a
   *different* `assertion-failed!`, STOP and report which one and what it printed — that is a finding, and
   the 60 remaining arms are the builder's to sequence.
4. **STOP-4 — if `fires == hits` does NOT hold after the fix**, STOP and report the actual pair with the
   `disrupt-points` string. Do not widen `tore?` further to force the equality; a disagreement means
   something other than the poison tore a connection.
5. **STOP-5 — if a firing rate still cannot complete**, STOP and name what stalls, with the phase line. The
   worker surviving the poison is the point; if it survives and the circuit still cannot fill, that is a
   second defect and not this stone's.

## Shape to copy

`the-store-can-fail/SCORE.md` — the last stone that made a never-executed path fire and captured what it
did. And `the-crash-surface-is-enumerated/FINDING-the-crash-surface.md` §3 for how the poison's real
outcome was established.
