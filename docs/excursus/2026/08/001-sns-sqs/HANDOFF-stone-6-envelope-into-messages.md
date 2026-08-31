# HANDOFF → grok — excursus 001 stone 6: `Envelope` into `:messages`

Branch `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-6-envelope-into-messages.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-6-envelope-into-messages.md`

**Small stone, and it takes the floor back to green.** Stone 5 widened the surface guard, and
`wat-scripts/queue/sqs.wat` now correctly refuses to freeze. This is the fix it asks for.

Move the `:queue::Envelope` `defrecord` (`sqs.wat:36`, beside the surface) **into
`:queue::Queue`'s `:messages`**.

**It is a PURE MOVE — measured.** A message may keep a non-prefixed name: `:p::Item` inside
`:messages` without renaming to `:p::Src::Item` type-checks clean, and moving `Envelope` on a
scratch copy of the real file gave `--check = 0`. **No rename, no call-site churn.**

★ **The gate is the grep precedent's standard, applied to a move** — *"the counts are the proof
it moved intact."* The queue's summary must be **byte-identical** afterwards:

```
./target/release/wat wat-scripts/queue/sqs.wat   →   "bound=x;r1=a,b;r2=c;r3=;redel=b"
```

★ **And row 8 is the one that could be missed:** both `repro/*.wat` must STILL fail `--check`.
The obvious way to make the queue freeze is to move `Envelope`; a *wrong* way is to weaken the
guard. If either reproduction goes green, stone 5 was undone to make stone 6 pass.

⛔ **Do not touch `wat-scripts/fanout/circuit.wat`.** It carries stone 4's foreign-read
workaround, written when `Envelope` was unreachable — now unnecessary and possibly wrong.
Nothing in the floor runs it. That is stone 7 (the re-attempt of the proof). **If you notice
what the workaround now does, write it in the SCORE so stone 7 starts informed. Change nothing.**

Floor is **5126 with 2 reds, both the queue.** This stone must take it to **FLOOR=0**.
Verify in the FOREGROUND; read the Summary line.
