# FINDING — INTUERI on the peer-communication outcome vocabulary

**Cast 2026-09-25** as a read-only ward. The spell came from the signed datamancy channel and was embedded
verbatim. HEAD was `63e7e37bb`. The input was 255.31's measured table (`SCORE-STONE-255.31-…`), and the ward
verified every arm itself. **The orchestrator spot-checked the load-bearing claims against disk.** The
ward's full report is summarised here; its line numbers are at `63e7e37bb`.

## Orchestrator-verified

- The thread tier maps **every** send error to `SendError::Disconnected` (`src/comms/thread.rs:81`), so
  thread `SendOutcome.Stopped` can never be produced. 255.31's table marked it "true", which was wrong.
- `send_outcome_from_error` (`src/kernel/outcome.rs:170`) turns `Disconnected` into `Lost`. So **a clean
  departure is `Closed` on recv and `Lost` on send**: one fact, two names.
- ⛔ **A latent denial of service, `wat/service.wat` ~:1976.** The `ServiceEvent.Lost` arm's comment says
  *"surface the reason on … stderr BEFORE evicting + continuing to serve"*. Its body is
  `(assertion-failed! …)`, which **raises**, so the eviction and the continue never run. It is unreachable
  today only because `poll'` never builds `Lost`. If `poll'` gains a crash channel, **one client crash
  kills the service.**
- `ConnectOutcome.Refused` is documented *retryable*, and **250** consuming arms are immediately followed
  by `assertion-failed`. **Nothing retries.**

## The ward's verdict

**Level 1 (lies):**
- `ConnectOutcome.Refused` ("retryable", no producer delivers it).
- `ConnectOutcome.Rejected` (it holds two facts: identity mismatch, and an inert wire copy).
- `AcceptOutcome.Closed` (it folds a stop in; on process it is **only** a stop).
- `RecvOutcome.Closed` (own-handle closed, far end left, and FrameTooLarge).
- `RecvOutcome.Lost`:
  - it is used for live peers: decode failure, `Reply::Failed`, transport failure, FrameTooLarge;
  - **every such cause is wrapped as `LociDiedError::Panic`**, telling the reader "the peer panicked".
- `SendOutcome.Closed` (only *this handle* was closed; the corpus misreads it as "client gone").
- `SendOutcome.Lost` (a clean EPIPE).
- `ServiceEvent.Shutdown`:
  - it carries the owner's EOF, an Admin payload on the re-poll, and a substrate stop;
  - meanwhile `poll'`'s real stop is a raise.
- `ServiceEvent.Closed` (wildcards absorb crashes and FrameTooLarge).
- `ServiceEvent.Lost`'s doc (it claims ECONNRESET paths that do not exist).

**Level 2 (mumbles):** `ServiceEvent.Rejected` (it names a verdict, not the fact, and collides with
`ConnectOutcome.Rejected`), and `Connection` (the same fact as `Accepted`).

Rust helpers: `PeerDeath::Shutdown` is documented *"Nothing died"*. `PeerRecvError::Crashed` is built for
live-peer facts.

**The same fact is spelt many ways:**

| fact | spellings |
|---|---|
| FrameTooLarge | `Lost` / `Closed` / `Rejected` / `Disconnected` |
| a stop | `Stopped` / `Closed` / `Shutdown` / a raise |
| a malformed message | `Malformed` / `Lost` / a raise |
| a gone listener, an accept io error, use-after-close | a value on one verb, a raise on another |

**About a dozen stale or contradicting docs**, among them:

- `outcomes.wat:258` ("exactly three shapes", and the wrong carrier);
- `spawn.wat:176` (five events, but there are eight);
- `listener.rs:258`, `message.rs:572`, `message.rs:464`, `spawn.rs:182`;
- a phantom `SendError::FrameTooLarge` in `error.rs:560` / `peer.rs:330`.

## ⭐ The facts, derived from the arms

| # | fact | loci | one name |
|---|---|---|---|
| A | delivered / received | all | `Sent` / `Message` |
| B | a peer was admitted | all | `Accepted` (the connect side: `Connected`) |
| C | the owner sent an admin op | all | `Admin` |
| D | the far end is gone for good, **no reason observed from this vantage** | all | `Closed` |
| E | gone for good, **a death reason observed** | all | `Died [LociDiedError]` |
| F | a stop was requested; nothing died, nothing closed | all | `Stopped` |
| G | the owner released the service handle | all | `OwnerClosed` |
| H | **this** handle was already closed | all | `HandleClosed` |
| I | the transport broke with an io reason; the far end's fate is unknown | real transports | `Failed [cause]` |
| J | the far end is alive; the message did not decode | serialising loci | `Malformed [cause]` |
| K | the far end is alive; the message exceeded the frame budget | framed loci | `Oversized [cause]` |
| L | the far end answered my request with a refusal (`Reply::Failed`) | client vantage | `Rejected [cause]` |
| M | this address value is not dialable here (an inert wire copy) | thread today | `Undialable [cause]` |
| N | the answerer is not who the address names | an independent far end (later: mTLS) | `WrongPeer [cause]` |
| O | refused now, may accept later | **no current locus** (a future TCP) | `Refused [cause]`, reserved or dropped |

**D is defined by what the vantage observed, not by "clean".** Reads without a crash channel cannot claim
"clean". "Gone, no reason observed" is what every current `Closed` producer actually delivers.

**⭐ How a locus-agnostic surface carries locus-specific facts:** *a variant on the type is not a claim; a
producer constructing it is.* A thread never constructs I/J/K/N/O. A locus-blind consumer handles them
once, and on a thread those arms are simply unreachable. The builder's *"a thread should not claim
reconnect"* is broken today **only by a producer** (`address.rs:178` building `Refused`), never by the
type. Measured: the consumers of the locus-specific variants are locus-blind (`wat/service.wat:1977-2022`
handles them once).

## Proposed direction (the ward's; for the builder's ruling)

- **ConnectOutcome:** `Connected` · `Closed` · `Undialable` · `WrongPeer` · `Failed` (and `Refused`,
  reserved).
- **AcceptOutcome:** `Accepted` · `Closed` · `Stopped` · `Failed`.
- **RecvOutcome:** `Message` · `Closed` · `Died` · `Stopped` · `HandleClosed` · `Failed` · `Malformed` ·
  `Oversized` · `Rejected`.
- **SendOutcome:** `Sent` · `HandleClosed` · `Closed` · `Stopped` · `Failed`.
- **ServiceEvent:** `OwnerClosed` · `Stopped` · `Admin` · `Accepted` · `Message` · `Closed` · `Died` ·
  `Failed` · `Malformed` · `Oversized`.
- Every producing arm that would change is listed in the ward's report, by file:line.

## Unmeasured risk (the ward's)

The `PeerCrashed` sentinel is intercepted only in `Peer::recv`/`recv_wire`. The raw-select paths
(`message.rs:1191`, `:1277`, `:1745`) do not check for it. A caller that `select'`s over client
connections could read a server crash as `ServiceEvent.Message` carrying `:wat::kernel::__peer_crashed__`.
Not measured.
