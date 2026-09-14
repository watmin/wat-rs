# DESIGN — a service stops instead of crashing

Builder, ruling the crusade's next frontier:
*"i'm not convinced we need services to be restarting themselves.. (yet)... i'm more convinced that
services shouldn't be allowed to crash, only gracefully shutdown...."* — and, on my framing:
*"'terminal failure' is an odd term.... we have a strong erlang influence..."*

**Rulings taken: D1-a + D1-c · D2-a · D3-a.** This stone is **D1-a**. D1-c (migrating the arms for
diagnosis quality) is a LATER stone and is out of scope here — see below.

**Drawn 2026-09-13. NOT STRUCK.**

## WHY — measured, not assumed

`wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat` (`79179f62f`):

```
a-control=Ok             non-vacuity — the service answers
a-boom   =Lost           the handler called assertion-failed!
b-dial=connect-REFUSED   an INNOCENT second client cannot even connect
```

One `assertion-failed!` inside one op handler destroys the service **for every client**. That is the
shape of all **19** `"redial failed — peer is dead"` arms (6 `circuit.wat` · 7 `sqs.wat` ·
6 `sns-fanout.wat`) and of the ~59 `Malformed` placeholders.

★ It is also, character-for-character, the arc-278 Stone 2 headline —
`probe-arc278-wire-dos-service-killed.wat`: *"A bad caller, malicious or dumb, cannot crash
anything."* Stone 2 built that wall for **one cause** (a malformed request SHAPE, guarded before the
handler runs). A handler that raises **on its own** walks straight through it. This stone generalizes
Stone 2 from one cause to all wat-level raises.

## ⛔ THE ONE CONTRACT DECISION

**An `assertion-failed!` escaping a public op handler becomes `Outcome::Faulted[cause]`. The service
CONTINUES SERVING from its PRE-OP state, the client is told via `PeerCrashed`, and the owner is told
via `Status::Faulted[cause]`.**

⚠ **This REVISES D2-a (exit through the stop path), on the builder's ruling:**
*"all of the chaos engineering work is meant to simulate networks doing networking things … we are
preparing for adding networking to wat."* In a network a peer failing mid-op is **routine**. If every
such failure makes a service gracefully *stop*, a flaky network takes the fleet down — gracefully, and
completely. Exit is also refuted by measurement: it fails this stone's own acceptance gate (v2 above).

Three halves, each load-bearing:

1. **PRE-OP state.** The handler's transition never completed, so the serve fn's own fifth parameter
   (`service.wat:2409`) is the last valid state. The failed op becomes a **no-op** — atomic-by-failure.
   The state must come from that parameter and **never** from anything the panicking body produced.
2. **Continue, not exit.** The acceptance gate is that an innocent second client can still connect.
3. **Nobody is muted.** Client via `PeerCrashed` (already), owner via `Status::Faulted`. ⛔ A service
   that silently absorbs a fault is the failure this campaign exists to remove — a crash traded for a
   silence is not a win.

### ⛔ AND THE SUB-DECISION THAT KEEPS IT HONEST: only an AssertionPayload converts

`catch_unwind` at this seam catches **two** populations:

| payload | disposition |
|---|---|
| `AssertionPayload` (`assertion-failed!` / `raise!`) — a wat-level raise | → graceful `Outcome::Stop` |
| anything else — a genuine Rust panic, i.e. a SUBSTRATE BUG | → `resume_unwind`, **unchanged** |

**Downcast before converting.** Converting an interpreter bug into a tidy shutdown would hide the one
class of failure that must stay loud. This stone does not make "crashes impossible"; it makes **a wat
raise a graceful stop** and leaves a substrate panic exactly as fatal as it is today.

## THE ROUTE — wrap the Outcome SCRUTINEE, and tell all three parties

⛔ **v3. Two earlier routes were drawn and both failed on the floor or the gate. The measured
history is kept below because the route turns on it.**

`serve-dispatch-op` already catches the panic and already broadcasts `PeerCrashed`. Move its wrap
**inward** — from the whole op-dispatch match to **each public handler body (the `Outcome` match
scrutinee)**. There the form's type IS `Outcome`, so a caught `AssertionPayload` can return
`Outcome::Faulted[cause]` and the serve loop's new arm is well-typed.

**Three parties must learn, and today only one does:**

| party | how | status |
|---|---|---|
| the **client** whose call raised | `PeerCrashed` broadcast → `Lost` | ✅ already works (measured: client-side wall tests pass) |
| the **service** | the `Faulted` arm continues with PRE-OP state | the stone |
| the **owner** (holds the lineage) | `Status::Faulted [cause]` sent up `self` | ⛔ **THE HOLE THIS VERSION CLOSES** |

⭑⭑ **The owner hole is what the floor red bought.** `recv_outcome_wall_panic_{thread,process}_admin_carries`
assert the owner observes `Lost [true]` **carrying the crash reason**. Route C's graceful exit gave it
`Closed []` — reason-free, *"the mute the recv-outcome wall killed for clients, relocated to the
owner"*. Continuing silently is no better: the owner learns nothing at all. **Both earlier variants
muted the owner**, and the DESIGN never asked, because it pinned "PeerCrashed is RETAINED" and that
covers clients only. `Status` is the owner's existing channel (`service.wat:1600`), so the fix uses
the mechanism already there.

### The measured history, kept because each route died on evidence

| route | what it did | how it died |
|---|---|---|
| **v1** `Outcome::Faulted`, wrap the whole dispatch | seam returns the new variant | **refuted on type**: `serve` is `-> nil` (`:3472`/`:3912`), the seam sits in its tail, so the Outcome never reaches the return position |
| **v2** route C — third/fourth arg, on-fault fn returning nil | graceful exit through the stop path | **measured RED**: acceptance row 1 fails (`b-dial=connect-REFUSED` — the innocent client is still refused) **and** floor 5244/2 FAIL on the owner mute. `.floor/2026-09-13T23-46-04Z/`, ARM kept |
| **v3 (this)** wrap the scrutinee + `Faulted` + `Status::Faulted` | the type holds, the client survives, the owner is told | — |

⛔ **v1's refutation was the orchestrator's error and it cost a working strike.** *"The seam cannot
return an Outcome"* is true only **at the original wrap site**. The remedy was to move the wrap, which
the executor had already done and **measured green** (`b-after=Ok`) before the refute made it discard
that work. Three claims of mine died on this stone; all three are recorded rather than quietly fixed:

| I claimed | measured | the defect |
|---|---|---|
| 388 `Outcome` matches would red | **1** arm head | a LINE count conflating match patterns with constructions |
| Rust cannot construct a parametric wat-only enum | params **erased**; `names` **carried** | a **zero** from a grep with no positive control |
| the seam cannot return an Outcome ⇒ the route is dead | true of the WRAP SITE, not the route | a shape read from the emission site's NEIGHBOURHOOD, never traced to its return position |

## Out of scope = REJECTED

- **The 19 arm migrations (D1-c).** The builder ruled D1-a **and** D1-c, in that order. Once the wall
  stands, those arms stop being load-bearing for crash-safety and become a **diagnosis-quality**
  change — a matched value says more than a raise. A separate stone, and each arm still owes its own
  reading: a template across them is the collapse-failures defect. **Not this stone.**
- **The lifecycle frames.** ⛔ This seam wraps **op-handler bodies ONLY**. `:init`, `:hibernate`,
  `:stop` and the serve loop's own frame unwind past it to `finish_forked_child` /
  `spawn_thread_peer` (that seam's own doc comment says so). So "a service cannot crash" is **NOT**
  what this stone delivers, and the SCORE must not say it does. It delivers: *a raise in an op
  handler is a graceful stop.* I have twice this week called a refusal total when it was not.
- **Self-restart / supervision.** The builder explicitly deferred it (*"not convinced we need
  services to be restarting themselves.. (yet)"*). ⚠ And it is not cheap: addresses are kernel-minted
  autobind (`src/kernel/address.rs:180` — 5 random bytes), so a restarted service comes back at a
  **different** address and every held `Address` is permanently `Refused`. Recovery needs a
  rendezvous story the tree does not have. Arc-shaped, and the builder's ruling to open.

## Blast radius, measured across all carriers

```
wat/service.wat                1 emission site (:2531) + the on-fault defn to emit
src/runtime.rs                 the Err(payload) branch (:27472-:27475) + the arity check (:27441)
src/check.rs                   infer_serve_dispatch_op (:12400) + its dispatch (:4775) — new arity
src/intrinsic/kernel/serve.rs  the #[wat_intrinsic] declaration (:216) + its derivation doc
.wat corpus                    0 — no user-visible form changes
```

## Trap-doors named up front

1. ⛔ **`wat/service.wat` is STDLIB — frozen into the binary at build time.** A Rust change ships
   alongside it, so this is the **STASH-DANCE** case. Read `wat/fix.wat`'s BOOTSTRAP / STASH-DANCE
   header before touching it; do not hand-edit your way out of a non-booting tool.
2. ⛔ **The `state` the seam forwards must be its own argument, never anything the panicking body
   produced.** That is the soundness argument: the handler's transition never completed, so the serve
   fn's fifth parameter is the last valid state. `AssertUnwindSafe` is an assertion, not a proof.
3. ⚠ **This seam runs on EVERY op dispatch.** Emit the on-fault function as a **top-level `defn`**
   (the shape `hibernate-project-def` already uses, `service.wat:760`) and pass its symbol — do NOT
   emit an inline `(fn …)` that allocates a closure per dispatch. The third bijection check cost
   5–9 % of floor time by walking at every expansion; this one would pay at every *message*.
4. ⚠ **The tail-call trampoline.** `serve`'s indefinite recursion depends on the recursive call
   staying in tail position through this wrapper (the seam's own doc says so). The returned value
   must not break that.
5. **Non-`AssertionPayload` panics must still `resume_unwind`.** See the sub-decision above.
6. **`cargo build --release` does not compile tests.** Use `cargo nextest run --release --no-run`.
7. ⚠ **Two stale citations found while drawing this, both in the code you are about to edit.**
   `src/intrinsic/kernel/serve.rs` cites `infer_serve_dispatch_op` at `check.rs:11347`; it is at
   **`:12400`**. And `src/assertion.rs`'s header describes the payload as existing so
   *"`run-sandboxed`'s `catch_unwind`"* can read it — **`run-sandboxed` is DELETED**
   (`wat/test.wat:175`). Verify every line number from the code, not from a comment. The live
   panic→value precedent is `src/host/test_runner.rs:301`.
