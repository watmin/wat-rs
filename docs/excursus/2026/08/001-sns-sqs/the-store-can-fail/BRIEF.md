# BRIEF — the store can fail

**Read `DESIGN.md` beside this first.** It carries the one thing that makes this stone honest (the fault
must **lose the reply**, never skip the call), the probed 10 s cost, and the contract decision.

## The work, in one paragraph

Six arms in `sqs.wat` handle a store call that returns `Lost`/`Closed`/`TimedOut`, and **none has ever
executed** because nothing can fail a store call. Build a userland service that satisfies
`:wat::query::Store`, forwards every op to the real store, and at a configurable rate **destroys its own
reply after the real write has landed**. The queue then genuinely cannot know whether the write happened —
which is the condition those six arms exist for. No stdlib change, no Rust.

## The rooms

| where | why you are going there |
|---|---|
| `wat-scripts/queue/sqs.wat:201` | `store-addr <- (Address :- [Store::Op Store::Reply])`, and `connect` at `:248`/`:251`. ⭑ **This is why a proxy works**: the queue takes an address, so it can be pointed at you with no edit. |
| `wat-scripts/queue/sqs.wat:780 :814 :845` | the `queue.send` store-put arms — `Lost` / `Closed` / `TimedOut`. Your targets. Read their comment: *"Do not claim Accepted n — the put is unknowable."* |
| `wat-scripts/queue/sqs.wat:1215 :1248 :1279` | the `queue.ack` store-delete arms, same three. |
| `wat-scripts/queue/sqs.wat:202–203` + `:864`, `:1079` | ⭑ **THE PROVEN SHAPE — read before writing.** `drop-recv-bp`/`drop-ack-bp`: a rate on the Record, a seed, and a suppression that **does the work and then returns `None` instead of `Some Reply`**. You are building the same thing one tier down. Copy its structure, including where it counts. |
| `wat/query.wat` `defsurface :wat::query::Store` | the six features you must forward: `ensure-schema · put · delete · count-index · scan · scan-index`. **All six.** |
| `wat/query/sqlite-store.wat:418–433` | a worked `:satisfies`-mode impl of these ops — the shape of a handler that returns a `Store::Reply`. |
| `wat-scripts/fanout/circuit.wat:2930`, `:2952`, `:2989` | where the store is started and its address handed to the queue Records. This is where a harness would insert you. |
| `wat-scripts/scratch-pad/probe-deadline-ms-is-callee-derived.wat` | my probe for the 10 s cost. Do not re-derive it; do not try to shorten the wait with `:deadline-ms` on your proxy — **measured inert**. |

## Implementation sketch

A `defservice` satisfying `Store`, whose `:durable` carries the real store's address and the two rates +
seed, and whose `:ephemeral` carries the connected peer and the counters.

```
;; per op, the same three steps:
resp  = (:wat::query::Store/<op> real-store <Request…>)   ;; FORWARD — the write really lands
hit?  = <dice roll from the seed, against store-drop-reply-bp>
reply = (:wat::core::if hit?
          (:wat::core::None)                              ;; SUPPRESS — count HERE, at the suppression
          (:wat::core::Some (<Store::Reply::…> resp)))     ;; PASS THROUGH — byte-for-byte
```

⚠ **The sketch is a convenience; the declarations on disk are the contract.** `Store`'s feature
signatures and `Outcome::Continue`'s arity are what you match — two of my sketches have been wrong in
this campaign and the executor was right to follow the disk both times.

`store-die-bp` is the same roll, but after forwarding the proxy **exits** instead of replying — that
yields `Lost`/`Closed` promptly rather than after 10 s.

## ⛔ Two things to get right or the stone is worthless

1. **Forward first, suppress second.** If you roll the dice *before* forwarding and skip the call, the
   write did **not** land and you have built the opposite of §2d. The arm would fire on a state it was
   never written for.
2. **Count at the suppression site, not at the dice roll.** This campaign shipped an injector whose
   counter cannot distinguish *never fired* from *fired and did nothing* — `disrupt-hits`, still broken at
   `ec3ea95c8` for exactly this reason. Emit `drops-fired` where the `None` is produced.

## ⭑⭑ The proof — make each of the three arms fire and say which

The deliverable is not "the proxy exists". It is:

> **each of `Lost`, `Closed`, `TimedOut` observed on a store call from the queue**, with the arm's own
> output captured verbatim.

⚠ **Those six arms `assertion-failed!` today** — they are among the 61 placeholders. **So firing one kills
the tier, and that is the expected, correct result of this stone.** Capture what each raise prints; that
text is the product. Do **not** migrate the arms — that is the next ruling.

Size the run for the cost: **10 s per `drop-reply` fault** (probed). A small `n` with one or two faults,
not `n=2000` at a live rate.

## Verify

- `./scripts/floor.sh`, read the **Summary line** → **5237 passed, 0 FAIL**. Your new `.wat` lives under
  `wat-scripts/`, so `every_wat_scripts_file_loads` type-checks it — a rotted proxy reddens the floor.
- ⛔ Never a piped exit code (a type-error run this session reported `$?` = 0 through a `| head`; true
  exit **3**, on stderr).
- `cargo nextest run --release --no-run` — the build does not compile tests.
- `cargo clippy --release --workspace --all-targets -- -D warnings` → 0.
- **Unfaulted pass-through is invisible:** with both rates at 0, the existing happy path must be unchanged
  — `circuit.wat 2000 4 3 8192 true 1000` → `distinct=8000;dup=0`. ⚠ Run it **without** the proxy inserted;
  `rt-store` through a proxy is meaningless (trap-door 5).
- `git diff --stat -- src/ wat/` must be **EMPTY**.
- Everything heavy through `./scripts/capped.sh --limit 8g`.

## STOP triggers

1. **STOP-1 — if a proxy cannot satisfy `Store` for some feature**, STOP and name the feature and the
   obstacle. Do not stub it: a proxy that stubs `scan` passes a shallow probe and corrupts a real run.
2. **STOP-2 — if forwarding-then-suppressing is not expressible** (e.g. the handler cannot both call out
   and return `None`), STOP and report the shape. Do **not** fall back to skipping the call — that inverts
   the condition under test.
3. **STOP-3 — do not touch `wat/query.wat`, `wat/query/mem.wat`, `wat/query/sqlite-store.wat`, or `src/`.**
   A chaos knob in a production stdlib type is rejected in DESIGN §Out of scope. If you conclude the proxy
   route cannot work, STOP and say why rather than reaching for the stdlib.
4. **STOP-4 — do not migrate the six `assertion-failed!` arms.** They are supposed to raise. If you find
   yourself making the tier survive, you are doing the next stone.
5. **STOP-5 — if `:deadline-ms` turns out to work somewhere** and you can shorten the 10 s, say where and
   how you measured it. My probe says a callee's is inert; a *caller*-side deadline is unmeasured and would
   be a genuine improvement — but report it, do not assume it.

## Shape to copy

`wat-scripts/queue/sqs.wat`'s `drop-recv-bp` path for the injector structure, and
`the-crash-surface-is-enumerated/FINDING-the-crash-surface.md` §5 item 1 for what this stone is for.
For the SCORE: `the-store-says-what-it-deleted/SCORE.md`.
