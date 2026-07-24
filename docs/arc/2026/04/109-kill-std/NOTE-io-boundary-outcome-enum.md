# NOTE (arc 109 vocabulary) — every FAILING IO boundary interfaces via a matchable OUTCOME ENUM; a world-fault failure is a value you face, never a raise

> **⚠ REFINEMENT PENDING (2026-07-23, builder) — the ENTROPIC third property.** As first written this
> note said "every IO boundary." That over-reaches. There is a **third purity property** the builder
> named **entropic** — an op that reads the *environment/entropy source* (measuring **time**; a **UUID
> from randomness**) is IO-touching but **non-deterministic** (so NOT Pure), **resource-less**, and
> **cannot world-fault** (the clock always returns a time; the RNG always returns bytes) — a *"pure
> syscall."* An **entropic** op does **NOT** return an outcome enum (there is no `Failed`/`Closed` to
> face). So the law is: every **FAILING** IO boundary (holds/reaches a resource that can fail
> independently — socket, file, peer, process) returns an Outcome enum. The pure / entropic /
> impure-failing **trichotomy** is under a grounded arc-corpus sweep (is "entropic" novel, or already
> the `deterministic?` axis from arc-278 Stone 6a?) before formalization; this note is updated with the
> grounded distinction when that lands. Everywhere below, read "IO boundary" as "**failing** IO
> boundary"; entropic ops are governed by the (forthcoming) purity-property treatment, not by this wall.

**Filed 2026-07-23 (builder directive, mid arc-278 the peer-lifecycle OUTCOME WALLS).** The
no-hidden-failures crusade (arc 278, R41→R57) has been converting the peer/comms verbs one by one
from *raising* their failures to *returning* them as matchable enums. Walling the sixth verb, the
builder named the shape underneath all of it and made it **permanent doctrine for ALL IO ops** —
this note is that doctrine, homed in 109's kill-std / stdio-contract lineage.

## The contract (firm — builder's words, 2026-07-23)

> *"the entire crusade we've been on for annihilating masked errors … this is starting to take
> shape … IO boundaries must have an enum to interface with … this is the pattern we need for
> /all IO ops/."*

**Every IO op — every point where wat touches a resource the program does not control (a peer, a
socket, a file, a process, a channel, the wire, the clock) — interfaces through a matchable
`Outcome` enum: one success variant + failure variants named *per kind*, and the enum is
`must-use`.** No IO op raises its handleable failures; no caller may drop the outcome.

## The law under it — the axis the whole crusade sorts: WHOSE FAULT is the failure?

- **The world's fault** — the peer crashed, the socket severed, the child was signaled, ECONNREFUSED,
  the disk filled, EOF, `WouldBlock`. The program is *correct*; the outside failed independently of
  it. This is a **handleable runtime condition** → a **matchable enum variant** the caller faces
  (retry / fallback / log / abort — the caller's choice, forced by the type).
- **The program's fault** — wrong arity, a type mismatch, a double-close, a call to a retired/
  restricted verb, a broken internal invariant. This is a **must-never-happen** bug → an
  **uncatchable raise** (`panic_any`/`EvalBreak`, structured-exit; halts loud, is never hidden, is
  never caught past the reader).

An IO boundary is *precisely* where world-fault failures live — so an IO boundary returns an enum
**by this law**. A raise at an IO boundary *for a world-fault condition* is the mask this crusade
annihilates (R53 the raise that flees past the reader; R57 the send-side twin).

## Both axes of masking are closed at the boundary — this is WHAT the enum buys

1. **Cannot FLEE** — the outcome is a *value*, not a control-flow raise, so the failure surfaces to
   the caller's own site (located at the caller's `match`), never unwinding past whoever should read
   it (R53 `VERBO MEO CAPTVS`).
2. **Cannot HIDE** — the outcome is `must-use`, so a dropped/`_`-swallowed outcome is a *compile
   error* in every discard door (`do`-non-final, `let`-`_`) (R55 `REVOLVTIONE, NVLLA LARVA` / R57
   `IGNORANTIAM DELEMVS`).

A world-boundary failure can therefore neither escape nor be swallowed. That is the whole point.

## The named-per-kind rule (R52 / the io-budgets doctrine)

The failure variants are split *by how the caller handles them*, not lumped under one `Failure`.
`Refused` (retryable transport) is a distinct variant from `Rejected` (non-retryable identity);
`Closed` (clean, no cause) is distinct from `Lost`/`Failed` (abnormal, carries a structured
`Failure` cause). A variant that would never be constructed is **cut** (fails Honest — e.g. `accept'`'s
`Rejected` was cut because the security gate bounces internally, never returning a reject). The
cause, where carried, is a **structured `Failure` record** (EDN), never a flat/prompt-inject String.

## Proven instances (grounded at HEAD, arc 278 — the peer/comms boundary)

| IO op | Outcome enum | commit |
|---|---|---|
| `recv'` | `RecvOutcome<O> {Message, Closed, Lost[cause]}` | R53 (`ee522630`) |
| `send'` / `try-send'` | `SendOutcome {Sent, Closed, Lost[cause]}` / `TrySendOutcome {+WouldBlock}` | R57 (`53bdfb0a`/`186ffb91`) |
| `poll'` / `select'` | `ServiceEvent {Message, Closed, Lost, Malformed, Rejected}` (must-use gated) | `4c087e27` |
| `close'` | `CloseOutcome {Closed[exit], Signaled[signal], Failed[cause]}` | `e7868da4` |
| `accept'` | `AcceptOutcome<R,S> {Accepted[peer], Closed, Failed[cause]}` | `2976d887` |
| `connect'` | `ConnectOutcome<S,R> {Connected[peer], Refused[cause], Rejected[cause]}` | (Strike 4, next) |

Each: a success variant + world-fault variants named per kind; parametric + `Impure` when a variant
holds a live resource (a `Peer'`), non-parametric + `Pure` otherwise; `must-use` via
`MUST_USE_TYPES` / `MUST_USE_PARAMETRIC_HEADS`.

## The mandate — ALL IO ops, and the extirpare ladder for the ones that don't yet conform

The comms boundary is walled. The doctrine now **governs every other IO boundary**, and the
non-conforming ones are debt to convert (each a stone when drawn):

- **File IO** (`src/io.rs` — `read`/`read_all`/`read_line`/`write`/`flush`/`temp-file`/`temp-dir`,
  and the `WatReadable`/`WatWriteable` traits): today return `Result<_, RuntimeError>` — a *raise*
  for world-fault conditions (disk full, permission, EOF, closed handle). → convert to an `IoOutcome`
  family (e.g. read → `{Read[bytes], Eof, Closed, Failed[cause]}`).
- **The store / sqlite boundary** (`:wat::query::Store` / `:wat::sqlite'`): a query/put against a
  backend is an IO op; its transient/constraint/fatal conditions are world-fault → an outcome enum,
  not a raise (arc 278 had a `sqlite'::Error` enum on the recovery axis — the direction; ground its
  current state before converting).
- **stdio** (`readln'`/`println`/`eprintln`): EOF/disconnect on `readln'`, a broken pipe on
  `println` are world-fault → an outcome. (`eprintln` is the *dying declaration* channel — see
  `NOTE-edn-only-rust-stdio-enforcement.md`; a terminal write is a different axis, but a
  *recoverable* stdio failure is an outcome.)
- **The clock / timers**, and every **future transport** (localhost TCP, mTLS, separate hosts —
  the `A FILO AD VSVM` wire-to-app horizon): a transport is a networked file handle; its failures
  are world-fault → the *same* outcome-enum shape, transport-general, so networking is a later swap,
  not a redesign.

Ladder: **convention → a lint that flags a raise at an IO boundary → the shape where an IO op
*cannot* return a bare `Result<_, RuntimeError>` for a world-fault condition** (the top rung — the
raise-at-an-IO-boundary becomes unrepresentable, the way the outcome walls made a swallowed peer
failure unrepresentable). The arc-277 raise-abuse rete-lint (tracked) is the natural seed of the
middle rung.

## Why this lives in 109's vocabulary

109 is the kill-std / IO-substrate lineage — wat owning its own channels and their contracts (the
stdio triangle, the EDN-only stdio note above). The outcome-enum boundary is that lineage's
**error contract**: the shape by which every wat IO op refuses to lie about a world-fault failure.
Filed here permanently per builder direction; the peer set is proven; the non-comms boundaries
convert as their stones are drawn. It is the same substrate `A FILO AD VSVM` needs — *react to
every failure at every boundary, crash on none, miss none.* The enum is how the boundary refuses
to lie.
