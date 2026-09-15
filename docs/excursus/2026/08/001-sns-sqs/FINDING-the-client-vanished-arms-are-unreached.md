# FINDING — a dying client does NOT kill the service, and `ServiceEvent::Lost` has no known producer

**Found 2026-09-15**, answering the builder's *"what's the next unexpected failure?.. what else doesn't
have chaos in it?"* — and **refuting the orchestrator's own claim** before it became a stone. The
builder's instruction was *"draw the probe first - prove it kills the service."* It does not.

## The claim under test, and it was a READ not a measurement

`wat/service.wat:2549`, the serve loop's `ServiceEvent::Lost` arm:

```wat
((:wat::spawn::ServiceEvent::Lost idx cause)
  (:wat::core::do
    (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) …)   ;; NEVER RETURNS
    (~serve-name self l (:wat::seq::remove-at selectables idx) next-id state))) ;; DEAD CODE
```

Its own comment states the opposite intent: *"surface the reason on the honest loud sink (**stderr**)
BEFORE evicting + continuing to serve … this arm's contract here is simply: do not DROP it."*
`assertion-failed!` is documented `@ret :T never returns`, and `eprintln` — the primitive the comment
describes — is used for exactly that purpose at `service.wat:3522`/`:3524`. So the arm as written kills
the service where the comment says it should log and continue.

**That reading is still true. What is false is the consequence I drew from it.**

## The measurement

`wat-scripts/scratch-pad/probe-a-dying-client-kills-the-service.wat`. A plain victim service; a client
service that dials it, pings it (so the victim holds a live accepted peer), then **raises in an internal
arm** — reachable because D1-a wrapped op-handler bodies only, and exactly what the publisher's `-run`
did before `the-publisher-gives-up-in-time`:

```
a-before =Ok      non-vacuity — the victim answers
cli-armed=yes     the doomed client dialed, pinged, armed its death 50 ms out
a-after  =Ok      ⭐ the victim STILL SERVES the original connection
b-fresh  =Ok      ⭐ and a fresh client connects fine
```

⭐ **The deduction is clean:** the victim survived ⇒ the `assertion-failed!` at `:2549` did **not**
execute ⇒ a dying client process yields `ServiceEvent::Closed` (EOF → evict → recur, the *correct* arm
at `:2539`), **not** `Lost`. `Lost` requires an abnormal read error; a dying process just closes its fd.

## ⛔ So `ServiceEvent::Lost` has TWO documented producers and NEITHER has been observed

| claimed producer | where | observed |
|---|---|---|
| a frame over `:max-frame-bytes` → `RecvError::FrameTooLarge` → `ServiceEvent::Lost` | `wat/service.wat:576` | ⛔ `probe-frame-cap-severs-one-conn.wat` measured *"B's separate connection kept working throughout — **the service was alive the whole time**"* |
| a client dying abnormally | this FINDING's reading | ⛔ measured above: the service survives |

**Neither is a measurement of `Lost` firing.** The arm is live code with a fatal body, and nothing in
this tree is known to reach it.

⛔⛔ **WHICH KIND OF UNREACHABLE IS UNANSWERED, and that is the point.** The breadcrumb's standing rule:
*"no mechanism yet"* is a gap worth ranking; *"no mechanism by construction"* is a wall holding. This
arm has not been classified on that axis, so **it must not be ranked as work** until it is. Conflating
the two is how arc 259's *"the user never holds the rope"* came to sit above real work.

## What IS still true and unmeasured: there is no client-side chaos at all

```
every knob we own is CALLEE-side:   drop-recv · drop-ack · drop-check · drop-mark
                                    store-drop-reply · store-die · disrupt · delay
client-side injectors:              ZERO
counters for "my client vanished":  ZERO  (neither :2539 nor :2549 is counted anywhere)
```

The whole campaign makes **services** misbehave and watches **clients** cope. The mirror — make a client
misbehave, watch the service — has never been built. ⚠ And one route is closed by ruling, not by
absence: **userland cannot kill a lineage** (`close` is `:wat::kernel::`-restricted, arc 259 S2d).

⭑ But note what this probe just showed: the mirror's most obvious defect **is not there**. A service
survives a client's abnormal death, silently and correctly. So a client-side injector is worth building
for *coverage*, not to chase a known bug — and it should be ranked as such.

## The orchestrator's error, recorded

I read `assertion-failed!` in a fatal position and asserted the consequence — *"kill any worker mid-call
and its queue goes down with it"* — without an instrument. Fifth wrong claim this session, same species:
**a structure quoted after reading one token of it.** ⭑ The difference this time is that it cost a
15-minute probe instead of a strike, and only because the builder said *prove it first*.
→ `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`
