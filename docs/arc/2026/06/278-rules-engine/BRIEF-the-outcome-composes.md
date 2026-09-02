# BRIEF — the outcome composes

`Outcome` encodes four independent things — state, a reply to my caller, sends to other named
conns, and arms — as six hard-coded combinations. The two the queue actually needs are not among
them, so two sites in `wat-scripts/queue/sqs.wat` put a **one-millisecond timer in the message
path** to say the word *and*. Replace the six variants with a **lifecycle-only enum carrying
fields**, split the internal-arm outcome into its own type so an arm with no caller cannot write a
reply, migrate the corpus with a **wat-fix codemod**, and delete both hacks.

## Read in order

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-outcome-composes.md`** — the design, the
   cardinality argument, and THE ORDER this is item 1 of. Read it first; it rules on shape.
2. **`docs/arc/2026/06/278-rules-engine/probes/internal-arm-replies.wat`** — the probe, already run,
   with its measured result in the header. It is your worked reference for the internal-arm half and
   it is the acceptance criterion for that half. **Do not re-derive it; read what it recorded.**
3. **`wat/service.wat:72-81`** — the `Outcome` enum. This is the change.
4. **`wat/service.wat:67`** — `Alarm [after <- Duration, op <- O]`. Unchanged by this stone; read it
   so you can see that time is the only self-schedule and why that forced the hacks.
5. **`wat/service.wat:127`** — the margin stating that an internal arm gets a `SelfInvocation`,
   *"never an `Invocation` (it has no connection, so it has no `conn-id` field)"*. This is the
   precedent for splitting the outcome type: the input side already splits, the output side does not.
6. **`wat/service.wat:1666-1674`** — the three runtime guards ("an internal (-) op returned
   Outcome::Reply, but an internal op has no client to reply to…"). The probe proves these fire and
   kill the service. When `SelfOutcome` lands, the *internal* cases become unwritable — so these
   guards should **go away**, not be kept as belt-and-braces. Their deletion is the proof the wall
   moved from rung 2 to rung 3.
7. **`wat/service.wat:1746, 1752, 1763, 1804, 1891, 1902, 2059`** — the serve loop's `kernel::send`
   sites that consume an `Outcome`. These are what has to read the new fields.
8. **`wat-scripts/queue/sqs.wat:238-241`** and **`:443-444`** (with the margin at `:350-351`) — the
   two call sites this stone exists to simplify. They are the acceptance criterion, not decoration.
9. **`wat/fix.wat`**, the `⚠ BOOTSTRAP` header — read before you touch the stdlib. And
   **`wat-scripts/fixes/*.wat`** — copy one as the shape of your recorded migration.

## The sketch

Load-bearing: the cardinalities and the two types. Illustrative: names.

```wat
(:wat::core::defenum :wat::service::Outcome :- [S R O] :wat::enum::Pure
  :Continue [state <- :S
             reply <- (:wat::core::Option :- [:R])
             sends <- (:wat::core::Vector :- [(:wat::service::Directed :- [:R])])
             arms  <- (:wat::core::Vector :- [(:wat::service::Alarm :- [:O])])]
  :Stop     [state <- :S
             reply <- (:wat::core::Option :- [:R])
             sends <- (:wat::core::Vector :- [(:wat::service::Directed :- [:R])])])

(:wat::core::defenum :wat::service::SelfOutcome :- [S R O] :wat::enum::Pure
  :Continue [state <- :S
             sends <- (:wat::core::Vector :- [(:wat::service::Directed :- [:R])])
             arms  <- (:wat::core::Vector :- [(:wat::service::Alarm :- [:O])])]
  :Stop     [state <- :S
             sends <- (:wat::core::Vector :- [(:wat::service::Directed :- [:R])])])
```

- `reply` is `Option<R>`, **never a vector** — at most one reply per invocation is the protocol and
  the type is where it is enforced. This is the stone's one contract decision.
- `Stop` has **no `arms`** — future work on a terminating service is incoherent, so it gets no form.
- `Stop` **gains `sends`** — today it cannot answer parked waiters at all.
- public arm `[s ctx req] -> Outcome`; internal arm `[s ctx] -> SelfOutcome`.

The two call sites then collapse. `sqs.wat:238-241` becomes one outcome that replies `Ok` to the
sender **and** carries the waiter's `Directed` in `sends`, with no `Alarm`. `sqs.wat:443-444` sends
and re-arms in one outcome, and **`-flush-outbox` ceases to exist** — it is a whole arm that exists
only because the combination was missing.

## Blast radius — measured

**351 `Outcome::` construction sites across 149 files.** By variant: `Reply` 293, `ReplyAndArm` 20,
`NoReply` 16, `NoReplyAndArm` 13, `Stop` 5, `ReplyTo` 4. Heaviest directories: `tests/services` (54
files), `wat-scripts/scratch-pad` (22), `wat-scripts/probes/arc-170` (19).

**`src/` has ZERO matches** — `Outcome` is a wat-level enum matched by the generated serve loop, not
by Rust. **This stone should not touch `src/` at all.**

The corpus migration is a **wat-fix codemod** (`wat/fix.wat`, recorded under `wat-scripts/fixes/`).
Census first (`wat --grep <fix>.wat` prints matches unapplied), diff it, then apply with **every**
path listed. Count occurrences, not lines — the finder emits one long line. Comments are not
rewritten; prose is a separate manual pass.

## On the bootstrap

`fix.wat`'s header describes a chicken/egg with two horns: a codemod needing a **new `:wat::fix::`
verb** the frozen binary cannot see, and a **Rust checker change** that rejects the old stdlib at
freeze. **The second horn does not apply here** — there is no Rust change. Whether the first applies
depends entirely on whether your rewrite needs a verb `fix.wat` does not already have. Determine
that **before** you start editing, because it decides the whole sequence.

## STOP triggers

1. **If the codemod needs a `:wat::fix::` verb that does not exist yet — STOP.** That is the
   bootstrap dance, it changes the sequencing, and it is the orchestrator's call, not a thing to
   work around.
2. **If any `.wat` file needs a hand-edit, or python, or sed, to migrate — STOP.** 351 sites is
   exactly what the codemod exists for. A hand-edited corpus is not this stone.
3. **If `src/` needs to change — STOP and surface it.** The census says zero Rust matches; if that
   is wrong, the design is wrong and wants redrawing before code.
4. **If removing `arms` from `Stop` breaks a live caller — STOP and name it.** Do not add the field
   back; the incoherence is the finding.
5. **If the two `sqs.wat` sites do not simplify — STOP.** They are why this stone exists. If the new
   shape does not delete both `Millisecond 1` alarms and `-flush-outbox` entirely, the shape is
   wrong and reporting that is worth more than shipping it.

## Shape to copy

`SCORE-the-sane-circuit.md` for how a row is proven by **removing** the thing and requiring a
failure. `red-partial-satisfier.wat` (in `probes/`) for how a deliberately-red probe is homed and
why it never carries a rune.

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — do not
re-run, name the exact arm, surface it.

Write `SCORE-the-outcome-composes.md` when done. It will be graded by re-running.
