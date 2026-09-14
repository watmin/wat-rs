# BRIEF — the topic can be slow

**Read `DESIGN.md` beside this first.** It carries the one contract decision (counter at the delay
site, not the dice roll), five trap-doors, and four things deliberately out of scope — including why
the internal-arm wall is NOT bundled with this.

## The work, in one paragraph

Give `:demo::topic` two new durable fields, `delay-bp` and `delay-ms`, plus a seed. Before the topic
replies, roll; on a hit, park on a timer channel for `delay-ms` and increment `delays-fired` **at the
park**, then reply normally. Wire both through `run-with` and one appended argv slot, defaulting to 0
so every existing invocation is byte-identical. The deliverable is an instrument: with the topic slowed
past the publisher's 10 s generated deadline, the publisher's `TimedOut` arm (`circuit.wat:2270`) fires
**on purpose** for the first time.

## The rooms

| where | why you are going there |
|---|---|
| `wat-scripts/scratch-pad/probe-a-slow-peer-desyncs-the-next-call.wat` | ⭑ THE PROVEN MECHANISM. A forked process-tier handler parking on `(recv (after PeerKind::thread (Milliseconds n) :done))`, **inlined** because a child cannot see parent helpers. Copy this shape exactly. |
| `wat-scripts/query/faulting-store.wat:117`–`:130` | ⭑ THE PROVEN INJECTOR SHAPE — a rate, a seeded roll, and the counter incremented where the fault is produced. This is the file that got `disrupt-hits`' mistake right. |
| `wat-scripts/topic/sns-fanout.wat:69` `:demo::topic` | the service. Its `:durable` gains the two knobs + seed. |
| `wat-scripts/topic/sns-fanout.wat:252` | `:demo::topic-worker`'s `disrupt-rate-bp` — the sibling knob already on this file, for naming and placement. |
| `wat-scripts/fanout/circuit.wat:2950` | ⛔ the PER-TIER SEED comment — read it before choosing a seed. One shared seed made all four sub tiers fail on identical calls. |
| `wat-scripts/fanout/circuit.wat:3635` | the usage string. 16 argv slots in use; yours is **17**, appended, 0-defaulting like its neighbours. |
| `wat-scripts/fanout/circuit.wat:3662` | the literal `1` for `p`, one line above — ⚠ **do not touch it.** Exposing `p` is out of scope. |
| `wat-scripts/fanout/circuit.wat:2270` | the arm this instrument is built to reach: `"recv: timed out — the peer is alive and silent"`. You are not changing it. You are making it fire. |
| `wat-scripts/queue/sqs.wat:2082` · `sns-fanout.wat:659` | the two existing `await-timer-ms` helpers — *parent-side*. They show the idiom; they are **not callable from a forked child**. |

## Implementation sketch

```wat
;; on :demo::topic's :durable — the knobs, defaulting to 0
delay-bp     <- :wat::core::i64
delay-ms     <- :wat::core::i64
delay-seed   <- :wat::core::i64
delays-fired <- :wat::core::i64     ;; incremented AT THE PARK
delay-draws  <- :wat::core::i64     ;; incremented at the ROLL

;; in the reply path, before the Outcome is returned
(:wat::core::if (:wat::core::and (:wat::i64::> delay-bp 0) (:wat::i64::< roll delay-bp))
  ;; INLINED timer park — mora-legal, and a forked child cannot see a parent helper
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds delay-ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    …every RecvOutcome arm, all nil — this wait cannot fail meaningfully…)
  nil)
```

Report `delay-draws` and `delays-fired` on the topic's existing stats line, beside the effective rate,
the way every other injector in this harness prints its effective rate next to its observed one.

## Blast radius

`wat-scripts/topic/sns-fanout.wat` and `wat-scripts/fanout/circuit.wat`. **Expected 0 changes to
`.rs`, goldens, or `wat/`** — confirm that rather than inheriting it.

## STOP triggers

1. **STOP-1 — if `delay-bp 0` is not byte-identical to today**, STOP. That is the whole safety of this
   stone: every existing invocation, every floor expectation, the happy path. If a zero-rate run
   differs in any reported number, the injector is in the wrong place.
2. **STOP-2 — if the park cannot be inlined** into the service form and needs a parent helper, STOP and
   say so. That is a real substrate limit and it changes the design, it does not get worked around.
3. **STOP-3 — if the slow proof needs more than ~15 s**, STOP and report the number before adding it
   anywhere near the floor. The terminate wall is 30 s and a test timed out at 40 s under contention
   this week. A scratch-pad run reported in the SCORE is the expected home for the slow proof.
4. **STOP-4 — do NOT touch `circuit.wat:3662` (`p`), the backoff policy, or `circuit.wat:2270`.**
   Making that arm *fire* is the deliverable; changing what it *does* is a later stone, and doing it
   here would convert a loud crash into a silent desync.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run`.
- `./scripts/floor.sh`, read the **Summary line**, never a piped exit code.
- ⭑ **Byte-identical control (STOP-1):** `./scripts/capped.sh --limit 8g ./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000`
  → `distinct=8000;dup=0`, and every reported counter unchanged from today.
- ⭑ **The relation, at full rate:** `delays-fired == delay-draws` when `delay-bp` is 10000. Holds at
  every n and every delay-ms — a relation, not a threshold.
- ⭑ **The slow proof (scratch-pad, reported in the SCORE):** a `delay-ms` above the publisher's 10 s
  generated deadline must reach `circuit.wat:2270` and kill the publisher with
  *"recv: timed out — the peer is alive and silent"*. **That death is the PASS** — it is the crash we
  have observed once by accident and could never reproduce.
- ⭑ **A negative control:** at `delay-bp 0`, `delays-fired` must be 0 **and** `delay-draws` must be 0 —
  a counter that ticks with the injector disarmed is measuring the wrong thing.

## Shape to copy

`docs/excursus/2026/08/001-sns-sqs/the-store-can-fail/` — the last injector drawn on this harness, and
the source of `faulting-store.wat`'s counter placement. And `the-chaos-gate-has-teeth/` for how a
floor harness proves an injector is not inert.
