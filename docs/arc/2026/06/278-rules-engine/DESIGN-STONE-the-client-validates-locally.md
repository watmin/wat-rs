# DESIGN-STONE — the client validates locally, and a bounced request never waits for a reply

> **Status: RULED 2026-08-03, unbuilt.** Found by a deadlock, not by looking.
> Builder: *"we'll impose the byte limit at send time to bounce a bad req locally (think… aws sdk
> client side request validations) and then… if the req is locally bounced… we must not attempt to
> recv on a send that never happened."*
>
> Supersedes the transport-level frame check that `DESIGN-STONE-send-mirrors-recv.md` originally
> proposed. **That check was at the wrong layer** and is deleted from that stone, not relocated
> within it.

## How this was found

A rider building the send-mirror added a pre-write frame check to `comms::process::Sender::send`
and the floor produced **one** regression: `probe_arc278_service_max_frame_bytes::large_foo_accepts_
a_600kib_request` **hung**. Its forensics were exact — `/proc/<pid>/task/*/wchan` showed a worker
parked in `io_cqring_wait` (a recv) and main in `futex_do_wait` joining it, while the send had
returned *immediately* with a refusal and never entered the write loop at all.

So the hang was not in the mechanism being built. It was a caller **waiting for a reply to a
request that was never sent.**

## The two halves

### 1. Validate locally, against the number the client is entitled to know

There are **two** limits and conflating them is what produced the wrong check:

| limit | declared on | who can see it | role |
|---|---|---|---|
| **`:max-request-bytes`** | the **surface**, per op | **both sides** — the client compiles against the surface | the CONTRACT |
| **`:max-frame-bytes`** (FOO) | the **defservice**, per service | server only | the DEPLOYMENT's ceiling, may be tighter |

`tests/services/probe_arc278_service_max_frame_bytes.wat` has both, and they can disagree by
design: the surface declares `put … :max-request-bytes 1048576`, `bigfoo` sets FOO to 1 MiB, and
`smallfoo` sets FOO to **4 KiB** — a deployment stricter than the contract.

**A client checks the contract, never the deployment.** It cannot predict FOO and must not try;
that path stays a server-side dismissal surfacing as `Lost` naming `max-frame-bytes` (already
asserted at that file's line 83). This is the builder's earlier ruling made precise: *the client
must be well behaved* = honour the declared contract; *bad clients are dismissed* = FOO catches
liars and stricter deployments.

**The number already exists and is already in the client's scope.** `build_op_budget_constants`
(`src/types.rs:3041`, called at `:3132`) emits one `(def :<Surface>::<OP>-MAX-REQUEST-BYTES <n>)`
per serviceable op — at **defsurface** registration, for any surface with `nature == Peer`. It is
**surface-scoped, not service-scoped**, so anyone holding the surface holds the constant. Nothing
needs negotiating, threading, or putting on the wire.

Its own comment scopes it too narrowly — *"so `serve-op-arms` can reference the budget by
keyword"* — one consumer imagined, and the constant handed to that consumer only. **Fourth
instance today of a number that exists, is correctly computed, and reaches one side of a pair.**

### 2. A bounced request returns — it does not fall through to `recv`

Validation and dispatch are **one operation**, exactly as in an SDK: botocore validates parameter
constraints and raises before any HTTP call, so the await-response code is never reached. Today the
generated method is two independent statements and nothing couples them, so you can wait for a
reply to a request that does not exist.

## The site — `wat/service.wat:1427`, inside `op-methods`

```clojure
;; arc 278 the send'-outcome wall — a send-then-recv': the
;; recv' right below faces Lost/Closed as a real outcome, so
;; this send' just needs to proceed regardless (faced, not `_`-swallowed).
[~discard-sym (:wat::core::match (:wat::kernel::send c (~op-variant-kw req))
                (:wat::kernel::SendOutcome::Sent   nil)
                (:wat::kernel::SendOutcome::Closed nil)
                ((:wat::kernel::SendOutcome::Lost _c) nil))
 ~r-sym (:wat::kernel::recv c)]
```

`discard-sym` is literally `(symbol-node "_")`. All three arms return `nil`. Then it recvs
unconditionally.

### ★ The comment is a sound argument with an unstated precondition

*"the recv' right below faces Lost/Closed as a real outcome, so this send' just needs to proceed
regardless."*

**That was true when written.** Every way a send could fail was also visible to the reader — peer
gone, peer closed — so proceeding was safe: the recv would report the same death. The precondition
is *"send failures are always peer-visible,"* and it is nowhere stated.

A **locally-refused** send breaks it. Nothing touched the wire, the peer is alive and well and has
no idea it was meant to answer, and the recv waits forever. The deadlock is not ignorance; it is a
correct argument whose premise changed underneath it, with no one re-checking the comment that
rested on it.

### ★★ And this is why must-use did not save us

`SendOutcome` **is** must-use. The gate fired correctly and was satisfied — the value *was* matched.

**A match whose arms are all identical is a discard that passes every wall we have.** Must-use can
force you to *look* at a value; it cannot force you to *act differently* on it. This is the fifth
and most sophisticated face of one problem this arc keeps meeting:

```
_cause                      discard by name              (R55 mask #4)
let _ =                     discard by binding           (R59 — the stop protocol never ran)
_sig                        discard by slipping an exact-match gate   (#67)
Err(_)                      discard by pattern           (the send cause)
match with identical arms   discard that SATISFIES the gate           ← this
```

The first four are greppable. This one **looks like diligence**, and it was written by someone who
knew about the gate and was explicitly trying to honour it (*"faced, not `_`-swallowed"*).

## ★★ WHERE IT APPLIES — the budget is a property of the WIRE

> Builder, ruling: *"the budget is a property of the wire — process tier only (eventually our N-loci
> for networked peers)… the distinction is 'shared memory or not'… so threads are never going to hit
> the limit, only processes and remote peers."*

**This is not a process-tier special case. It is an axis**, and it is the one the record already
names: *the real split is SHARED-MEMORY vs A WIRE, not thread-vs-process — build wire concerns
transport-general so networking is a later swap* (`[[project_aws_on_a_single_computer_then_networking]]`).

Grounded, and the substrate already carries the discriminant **under the right name**:

```rust
PeerTx::Thread(tx) => tx.send(value)      // shared memory — a crossbeam Value handoff. NO encoding.
PeerTx::Socket(tx) => tx.send(wire)       // THE WIRE — String, "to_wire() is a raw passthrough"
```

It is `Socket`, not `Process`. So the rule is **"the budget applies wherever `send_wire` is the
path"** — which covers process peers today and networked peers the day they land, with no second
edit. Do NOT branch on `locus == process`; branch on whether there is a wire.

### Why this is correctness, not thrift

The thread tier **never encodes at all** — the value crosses in-memory as a Rust value. There is no
frame, no pipe, no 64 KiB buffer, and no bytes. Imposing a byte budget there would:

1. **Measure a thing that does not exist.** `:max-request-bytes` is a wire concept; a shared-memory
   handoff has no wire representation to be too large for.
2. **Force a full serialization onto the one path whose entire point is not serializing** — not a
   2× cost, but zero-to-full on every call.

And on the wire tier the cost is **nil**: the encode already happens, so the client measures the
same string it is about to hand to `send_wire`. One encode, as it always was.

*(An earlier draft of this stone worried about "two encodes." That was wrong in both directions —
the wire tier encodes once either way, and the thread tier encodes never. The builder's "we just
write edn to the wire like normal?" was the correct instinct; the real question was never cost, it
was whether a frame exists at all.)*

## The strike — and it needs no new type

`defservice` **MANDATES** `RequestTooLarge{bytes <- i64, cap <- i64}` on every response enum
(`types.rs:1081`; required shape at `:2686`; a test at `:5733` proves an enum lacking it is a
located error). So the local refusal returns **the same answer the server would have sent**:

```
op-method(c, req):
  measure the encoded req against <Surface>::<OP>-MAX-REQUEST-BYTES
  over  -> RecvOutcome::Message(<Op>Response::RequestTooLarge{bytes, cap})   ; no send, no recv
  under -> send -> ACT on the outcome (never three arms of nil) -> recv
```

**The caller's match does not change.** Whether the refusal came from local validation or from the
service is invisible — the SDK property exactly. Locality is an optimisation, not new API surface.
And because the variant is mandatory, this holds for every op on every surface, with no fallback.

The send arm must now discriminate: a send that did not land returns the honest outcome rather than
falling through. Do not replace one uniform match with another.

## STOPs

- **⛔ Do not check FOO on the client.** `:max-frame-bytes` is the deployment's, unknowable to a
  dialer, and `smallfoo` proves it can be stricter than the contract. Check `:max-request-bytes`.
- **⛔ Do not add a budget to `comms::Sender`.** The transport knows the frame and not the op, so it
  can never hold the right number. This is the correction that produced this stone.
- **⛔ Do not mint a new outcome for the local refusal.** `RequestTooLarge` is mandatory and is the
  answer.
- **⛔ Do not leave the send arm uniform.** Three arms returning `nil` is what caused the deadlock;
  reproducing it with different arms that all still `recv` changes nothing.
- **⛔ Do not delete the server-side FOO rejection.** Belt and braces: the client honours the
  contract, FOO defends against liars and stricter deployments.

## Open

**Should a uniform-arm match over a must-use outcome be a lint?** It would have caught this before
the deadlock existed, and it is the only one of the five discard faces no existing wall can see.
Not proposed here — it needs its own grounding (how many legitimate uniform matches exist; the
serve loop's own `client gone → keep serving` arms are plausibly correct and identical).
