# One selectable-set primitive

**A substrate stone, and the one that unblocks the bracket work.** Excursus `001-sns-sqs`.

Builder, on being shown that bracket's `select` and the service's `poll` classify a decode failure
differently: *"is brackets not just the same thing? service is many clients to one server, brackets
is one client to many servers... i still don't see why this is demanding two impls."*

**It does not.** This stone is that answer, built.

## The sentence

> **`select(peers)` is `poll(∅, ∅, peers)`. There is one waiting discipline, not two.**

## The evidence, measured 2026-09-19

Both verbs match the **same** substrate type — `SelectOutcome<T>` (`src/comms/mod.rs:507`), whose
whole surface is `Recv { index, result }` · `Shutdown` · `Listener`. And the wat-level interfaces
are *already* the right shape:

```rust
#[wat_intrinsic(":wat::kernel::poll")]   fn eval_poll_prime(self_peer, listener, peers, …)
#[wat_intrinsic(":wat::kernel::select")] fn eval_peer_select_prime(peers, …)
```

`self_peer` and `listener` are **separate named arguments, not positional members**. So the two
verbs differ only in whether those two are supplied. ⭐ Everything else follows from set membership:

| | set | extra variants that follow |
|---|---|---|
| `poll` (N:1, a server) | self-peer, listener, clients | `Admin` — literally `if index.0 == 0`; `Connection` — a listener is present |
| `select` (1:N, a client) | peers only | none |

And `select`'s own body says it is the same match with holes punched in it:

```rust
SelectOutcome::Listener => unreachable!("process-tier 1-arg select has no listener arm")
SelectOutcome::Listener => unreachable!("thread-tier Peer Select has no listener arm")
// No self-peer / listener — select is peers-only.
```

⛔ **THE DUPLICATION HAS ALREADY COST A CORRECTNESS BUG, and it is the one this excursus is about.**
The two copies classify an undecodable frame differently:

| impl | decode failure becomes |
|---|---|
| `poll` | `Malformed` — *alive, sent garbage* |
| `select` | **`Lost`** — *dead* (`runtime.rs:27309`: "a peer whose frame will not decode is dead") |

Nobody designed that. It is drift between near-identical paths — and its consequence is that a
healthy remote runner emitting one corrupt frame is reported **dead**, which is exactly the
conflation `a-momentary-failure-is-not-fatal` exists to undo. ⭐ **One code path makes that
divergence unrepresentable.**

Consumers today: `:wat::kernel::select` **11** call sites (2 stdlib — `service.wat`, `bracket.wat`);
`:wat::kernel::poll` **7** (1 stdlib). `ServiceEvent::Lost` is matched in **12** files.

## The work

### 1. ⭐ The drift table — exhaustive, and it IS the stone

For every `SelectOutcome` variant × every tier (thread, process) × both impls: what `ServiceEvent`
does it produce? ⛔ **Measured from the code, one row per cell.** The decode-failure row is the one
already known; **find the others before unifying**, because a unification that silently adopts one
side's behaviour for a cell nobody compared is how a second bug of this class ships.

⚠ **Permit the null loudly:** if the impls turn out to diverge somewhere that is *load-bearing* —
a case where the service genuinely needs behaviour a pool must not have — then the answer is "two
impls are warranted, here is the cell that warrants them", and **that is a full delivery.** Do not
unify through a real difference to satisfy the sentence.

### 2. One engine

`select` becomes the peers-only call of the same engine. Keep both **wat verbs** — they are the
honest surface for the two shapes, and 18 call sites depend on them. ⛔ **Do not change either
verb's wat signature**; this is substrate, not surface.

### 3. The variant set follows from the inputs

With one engine, a peers-only call cannot construct `Admin` or `Connection` — not by an
`unreachable!()`, but because nothing in its set can produce them. ⚠ Whether that can be expressed
in the TYPE (so bracket stops being forced to write four impossible arms) is a real question:
answer it, and if the answer is "not without a narrower event type", say so and leave it — that is
the next stone, not this one.

### 4. Do not regress the serve loop

`poll` is the service's hot loop (`service.wat:2478`). Measure before/after on an existing service
benchmark, and say what you measured.

## Scope wall

⛔ Do **not** fix bracket's four dead arms (the parked stone owns them). Do **not** build the
transport resend (its own stone — see below). Do **not** change `select`'s or `poll`'s wat
signatures. Do **not** widen `ServiceEvent`.

## What this unblocks, and the ordering

1. **This stone** — one engine, uniform failure classification.
2. **The transport resend** — lock-step (`bounded(1)` at every tier, `bounded(N)` refused by
   four-questions) means a receiver that sees a bad decode can ask the still-blocked sender to
   resend. Absorb the transient half below, and `Malformed` reaching a consumer is *genuinely*
   REPORT-FINAL — the governing contract becomes true by construction.
3. ⏸ **`a-dead-runner-loses-one-item-not-the-run` — PARKED behind (1).** Its re-dispatch design is
   sound, but it is written against a `Lost` that today means both "died" and "sent garbage". Land
   (1) and its RETRY arm gets a `Lost` that means one thing.

## The four questions

- **Obvious** — ✅ duals do not need two engines, the interfaces already differ only by two
  arguments, and the drift has already produced a bug.
- **Simple** — ⚠ **the weakest.** `poll` is large and is the service hot loop; `select` has two
  `unreachable!()` arms that become real. This is surgery on a live path, not a one-liner. Say so.
- **Honest** — ✅ the drift table is the proof, and row 1's null permits "two impls ARE warranted"
  if a cell says so.
- **Good UX** — ✅ one failure classification for every selectable set, at any distance.
