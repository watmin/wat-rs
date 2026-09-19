# A dead runner loses one item, not the run

> ⏸⏸ **PARKED 2026-09-19, behind `../one-selectable-set-primitive/`.** Builder's ruling: bracket's
> `select` and the service's `poll` are the 1:N and N:1 faces of one waiting discipline and do not
> warrant two implementations. This stone's RETRY arm is written against a `Lost` that today means
> **both** "the runner died" and "the runner sent garbage" — a drift between the two copies, not a
> design. Land the unification first and this stone's `Lost` means one thing. **Do not strike this
> until that one is scored.**

**The crusade's real target in `bracket.wat`.** Excursus `001-sns-sqs`. Read first:

- `../a-momentary-failure-is-not-fatal/DESIGN.md` — **drawn 2026-09-10, still UNSTRUCK.** The
  governing taxonomy (RETRY / REPORT-FINAL / REPORT-GONE). This stone is its first real consumer.
- `../the-bracket-runs-on-the-queue/SCORE.md` + its regrade — and `632335c55`, which found that
  the ten arms that stone targeted are **dead**, and that the live ones are elsewhere.
- `tests/kernel/probe_bare_recv_outcome_surface.rs` — the reachability pins this rests on.

## The sentence

> **One runner dying should cost one item, not the whole bracket.**

## What is actually there — measured 2026-09-19, not assumed

`:wat::bracket::collect-loop` (`wat/bracket.wat:620`) drives the pool. It returns
`(Vector :- [(Tuple :- [:wat::core::i64 O])])` and matches **eight** `ServiceEvent` arms from
`(:wat::kernel::select peers)`. **Seven of the eight raise.**

| arm | disposition today | emitters in the select path |
|---|---|---:|
| `Message` | continue — the only non-raising arm | 3 |
| `Closed` | ⛔ `assertion-failed!` | 4 |
| `Lost` | ⛔ `assertion-failed!` | 2 |
| `Malformed` | ⛔ `assertion-failed!` | 1 |
| `Rejected` | ⛔ `assertion-failed!` | 1 |
| `Shutdown` | ⛔ `assertion-failed!` | 5 |
| `Connection` | ⛔ `assertion-failed!` | 2 |
| `Admin` | ⛔ `assertion-failed!` | 2 |

⭐ **Every one of the eight variants is constructed by the select path** (counted in
`src/runtime.rs`'s two `SELECT_EVENT_TYPE` impls). So these are not exhaustiveness filler — unlike
the ten `RecvOutcome` arms the previous stone chased, which genuinely cannot fire. **Whether each is
reachable *in a bracket runner pool specifically* is a different and unmeasured question** — see
row 1. `Connection` / `Admin` are service-shaped events and may be structurally impossible here;
their own panic strings hint at it (*"select has no self-peer"*).

⭐ **And the recovery data already exists.** `holding` is a per-runner vector of the item index each
runner is currently working (`holding-set`, `holding-phrase`, `bracket.wat:565`/`:578`). It exists
today **only so the panic message can say what was lost**. The same fact is what makes re-dispatch
possible: when runner *k* dies, `holding[k]` is the item to hand to a survivor.

## The work

### 1. ⛔ CORRECTED 2026-09-19 — THE DISPOSITION MUST BE TRANSPORT-INDEPENDENT

This row first asked "can this arm reach a bracket pool *over local IPC*?", planning to bless the
unreachable ones with `assertion-failed!`. **That is the wrong question, and the builder's ruling
says why:**

> *"brackets is meant to be used with networked hosts — the ipc is a stand in for that — we are
> doing chaos work on the ipc to show that we can recover from there as these are what networking
> will induce."*

`scratch/WAT-NETWORK.md` is the corroborating record: `RemoteProgram`, bounded wire buffers,
*"if a remote node is overloaded"*. `RemoteOpts` is the deliberately-uncut third `spawn-program`
key. **A bracket's transport is a network; IPC is the local stand-in.** So "local IPC cannot produce
this" is *irrelevant* to how the arm must behave — it would bake a crash into exactly the case the
work exists for.

**The rule this row now applies:**

> An arm may keep `assertion-failed!` **only if its impossibility follows from the PROTOCOL SHAPE**
> — a runner pool has no listener and no admin channel, so `Connection` / `Admin` mean the substrate
> handed us someone else's event, an invariant violation at any distance. It may **never** assert
> because of the transport's locality.

Sort the seven on that axis, with evidence:

| | |
|---|---|
| **protocol-impossible** (assert is honest, at any distance) | candidates: `Connection`, `Admin` — prove it from what a runner peer *is* |
| **transport facts** (a network produces these routinely) | `Closed`, `Lost`, `Malformed`, `Rejected`, `Shutdown` — all in scope for row 2 |

⚠ Emitted-somewhere ≠ reachable-here still holds as a caution against the previous stone's error —
but the fix is not to narrow to today's transport, it is to classify by **why** an arm can occur.

### 1b. ⛔ MEASURED AFTER THE FIRST CORRECTION — bracket can receive FOUR of the eight, and
### `select` COLLAPSES A DECODE FAILURE INTO `Lost`

`ServiceEvent` is ONE enum serving TWO select verbs with disjoint variant sets:

| verb | impl | builds |
|---|---|---|
| `:wat::kernel::select` — **bracket's** | `eval_peer_select_values` | `Message` `Closed` `Lost` `Shutdown` |
| the service `poll` — bracket calls it **0** times | `eval_poll_prime` | + `Admin` `Connection` `Malformed` `Rejected` |

So **four** of bracket's eight arms are dead, not two — `Malformed` and `Rejected` among them. ⛔ The
first draft of this stone called `ServiceEvent::Malformed` "the live ungraceful arm"; **it is not
live for bracket either.** Pinned now by `kernel_select_builds_only_four_serviceevent_variants`.

⭐ **And here is why only the service has `Malformed`, which is the builder's question and the
answer matters more than the classification:** the two verbs make different TRUST assumptions.
`select` calls `decode_trusted_wire`, and on a decode failure emits **`Lost`** —
`src/runtime.rs:27309` says it outright:

> *"A peer whose frame will not decode is **dead** (recv:25047), not a live reason-free"*

`poll` decodes **untrusted client** messages, so it keeps "alive but sent garbage" (`Malformed`)
separate from "gone" (`Lost`). **`select` has no such concept — it collapses garbled into dead.**

⛔⛔ **THAT COLLAPSE POISONS ROW 2, AND IT IS AN ORDERING CONSTRAINT ON THIS STONE.** Over IPC a
trusted wire is near enough. Over a network it is false: a healthy remote runner that emits one
corrupt frame is reported **dead**. So a naive RETRY-on-`Lost` would re-dispatch on decode failures
too — and for a deterministic encode bug that is **re-dispatch until the wall clock expires**,
violating the governing DESIGN's one contract decision (*"`Malformed` is never retried"*) — violated
not by the handler but by the transport collapsing the distinction **before the handler can see
it**.

#### ⛔ CORRECTED AGAIN — I OVERSTATED THIS. Builder: *"why is bracket special here? if the wire
#### screws up a transmission they cannot be made to retry."*

Two things I had wrong.

**1. Bracket is not special as a VICTIM.** The collapse is in `select`, one tier below, and every
consumer inherits it: `:wat::kernel::select` has **11 call sites — 2 in the stdlib**
(`wat/service.wat`, `wat/bracket.wat`) and 9 in `wat-scripts/`; `ServiceEvent::Lost` is matched
across **12 files**. Writing this up as bracket's problem pointed the fix at the wrong tier.
**`select` needs the `Malformed` that `poll` already has** — that is a substrate stone, the
`a-momentary-failure-is-not-fatal` 1a/1b shape applied to `ServiceEvent` instead of `RecvOutcome`,
and it is the one that serves all 11 callers.

**2. ⛔ MY CLAIM THAT A CORRUPTED TRANSMISSION CANNOT BE RE-SENT IS WRONG.** Builder:

> *"there's zero reason why a network can't implement such a thing via a network protocol — we can
> observe something is wrong and have the caller resend it. everything is lock step in wat. the
> producer will not produce until it's been told the receiver can receive it. the threads do this,
> the processes do it, the network will too — it doesn't exist yet, that doesn't mean it can't be
> made lockstep too."*

**Verified in the substrate, and it is a designed invariant rather than an accident:**

```
src/comms/mod.rs:54     Thread tier: crossbeam `bounded(1)` — structurally capacity-1
src/comms/mod.rs:68     There is no `bounded(N)` factory at any tier
src/comms/thread.rs:27  no `bounded(N)` factory (retired by four-questions)
src/comms/thread.rs:44  backpressure shape is the same: substrate refuses to absorb work
```

Capacity-1 everywhere. **The sender is blocked *inside* the send until the receiver takes the
value — it has not moved on, and it still holds the value.** So on a decode failure the receiver
can ask for a resend, and a remote tier that keeps the same discipline can do the same. I imported
a fire-and-forget model that this substrate explicitly refuses.

⭐⭐ **AND THAT RESOLVES THE TAXONOMY HOLE I RAISED IN ROW 2 — I had the retry at the wrong layer.**
Put the resend in the TRANSPORT, where lock-step makes it possible, and the layering comes out
clean:

| fault | who handles it | what the consumer sees |
|---|---|---|
| wire corrupted the frame | **the transport** — receiver observes the bad decode, asks the still-blocked sender to resend | nothing; it never surfaces |
| the sender encoded garbage | nobody can fix it — a resend reproduces it byte for byte | `Malformed` — genuinely **REPORT-FINAL** |
| the peer died | the transport cannot help | `Closed` / `Lost` → bracket re-dispatches the work |

So `a-momentary-failure-is-not-fatal`'s contract — *"`Malformed` is never retried"* — is **not** a
statement that networking breaks. It becomes **true by construction**, once the transient half is
absorbed below by a resend. ⚠ My earlier note that "`Malformed` is two facts wearing one name" was
right about the ambiguity and wrong about the remedy: the fix is not a richer variant at the
consumer, it is a resend at the transport so only the deterministic fact ever gets there.

⭐ **And THAT is the one sense in which bracket IS special — as the RECOVERER, not the victim.** It
owns the work queue, so `holding[idx]` tells it which task to re-run. A generic `select` consumer
has no such handle and can only report. So:

| | can retry the transmission | can re-run the work |
|---|---|---|
| any `select` consumer | **no** — the frame is gone | **no** — it does not know what produced it |
| **bracket** | no | ⭐ **yes** — `holding[idx]` |

**Which softens my "do not build re-dispatch on a collapsed `Lost`" to something narrower and
true:** for bracket the collapse does not block the ACT, because *runner died* and *runner sent
garbage* both mean "this item's result did not arrive" and both are answered by re-dispatch. What
the collapse costs is (a) the REPORT — bracket cannot tell the caller which happened — and (b) the
protection against a **deterministic** encode fault, where every runner reproduces the same
undecodable reply and re-dispatch spins until the bound.

⛔ **So the wall-clock bound is not a nicety here, it is the only thing standing between a
deterministic encode bug and an infinite re-dispatch loop.** Grade it as load-bearing, and make the
give-up report name that it could not tell death from garbling.

⚠ **And note what the lock-step finding does NOT change for this stone:** a transport-level resend
does not exist yet, so today a wire fault still reaches bracket as `Lost`. The bound stays
load-bearing until the resend lands. What changes is the ORDER of the fix — the substrate stone
(`select` gains `Malformed`, and the transport resends the transient half) is the one that makes
bracket's `Malformed` handling honest, and it should be drawn as the lock-step design it is, not
as a richer enum bolted onto a fire-and-forget assumption.

⚠ The `Malformed`-is-two-facts note in row 2 applies to `poll`'s `Malformed`. `select` has a
THIRD problem: it has no `Malformed` at all.

### 2. Place each reachable arm in the taxonomy

| | |
|---|---|
| **RETRY** | `Closed` / `Lost` — the runner is gone, but **its item is known** (`holding[idx]`). Re-dispatch to a surviving runner. This is the stone's whole point. |
| **REPORT, FINAL** | `Malformed` — a reply that did not decode. ⚠⚠ **AND HERE THE GOVERNING TAXONOMY HAS A HOLE THAT ONLY THE NETWORKING LENS EXPOSES.** It classifies `Malformed` as REPORT-FINAL because *"deterministic — retrying reproduces it exactly; WE are the defect."* That is true of a LOCAL decode failure (a type mismatch, the sender encoded garbage). **Over a network it is not:** a corrupted frame is a transmission fault, and retrying may well succeed. So `Malformed` is really two facts wearing one name — *the sender encoded garbage* (deterministic, REPORT-FINAL) and *the wire damaged it* (momentary, RETRY) — and the transport cannot tell them apart from the decode error alone. ⛔ **Do not silently pick one.** Report which fact today's `ServiceEvent::Malformed` actually carries, and if it cannot distinguish them, say so — that is a finding against `a-momentary-failure-is-not-fatal`, not a decision for this stone to make. |
| **REPORT, GONE** | no runners left to re-dispatch to |

Bounded by **wall clock, never an attempt count**, and the report names which bound it hit — the
governing DESIGN's ruling #1, and the same rule the queue runner already follows.

### 3. ⛔ The surface question — name it, do not smuggle it

`(:wat::bracket::map …)` returns `(Vector :- [O])`. There is **no room for a per-item failure
value**, which is why every arm raises. That constraint already bit the previous stone: its row 2
(surface byte-identical) and row 3 (arms become values) **could not both be satisfied**, and nobody
noticed until the work was done.

So this stone's scope is deliberately drawn to **not need** the surface change:

- **RETRY loses nothing** — a re-dispatched item still produces its `O`, so `(Vector :- [O])` is
  still exactly right. ⭐ **This is why `Closed`/`Lost` are the target and not `Malformed`.**
- If, after row 1, the only honest treatment of a reachable arm needs a value the surface cannot
  carry, ⛔ **STOP AND REPORT IT.** Do not widen `map`'s return type in this stone. An outcome-typed
  bracket surface is a separate stone with a builder ruling attached, because it changes every
  caller.

### 4. A control that can redden

⛔ Judged by mutation, and the bar is the one `632335c55` set after the last control failed it:
**kill the mechanism and show the test go red.** Concretely: a bracket over N items where one runner
is killed mid-flight must (a) still return all N results, and (b) go RED if the re-dispatch is
removed. A test that only passes proves nothing.

⚠ **Non-vacuity:** prove the runner actually died in the passing run (the fault fired), the way the
queue control asserts `recv-drops>0`. A green because nothing broke is the failure mode here.

## Scope wall

⛔ Do not touch the ten dead `RecvOutcome` arms — `probe_bare_recv_outcome_surface.rs` pins them as
unreachable and explains why. Do not strike stone 1b. Do not change `map` / `each` signatures. Do
not touch the queue path (`0d8bada7b`, `dac31ac03`).

## The four questions

- **Obvious** — yes, and more so under the networking frame: a worker dying should not lose the
  other 999 items, the pool already records what the dead worker held, and a network will kill
  workers as routine weather rather than as an exceptional event.
- **Simple** — ⚠ genuinely unclear until row 1. Re-dispatch inside an existing accumulator loop is
  small; a re-dispatch that must also re-establish a dead peer may not be. Score it honestly.
- **Honest** — the RETRY arms are made recoverable and the rest are **classified and stated**, not
  silently left raising. Row 1 is what keeps this from being a guess.
- **Good UX** — `(map locus items work-fn)` unchanged; a dead runner costs a re-dispatch instead of
  the run.
