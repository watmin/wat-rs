# DESIGN — `:wat::kernel::Peer'` as a matchable ENUM (the parent-handle unification)

> **Status: DRAWN (2026-07-24), builder-directed.** *"should we have a wat.kernel/Peer who contains
> Thread and Process… the caller of spawn-program' gets back a kernel peer' who is then matched in an
> enum to run a func for that kind of peer… draw the peer' enum stone."* The `spawn-process` de-prime
> surfaced the need; this stone is the foundation the de-prime should be done AGAINST.
>
> **⚠ NAMING UNSETTLED — the enum name is NOT `Peer'` and NOT `Child'`; RE-CAST intueri on the far side
> with the client/server + remote-generalization constraint.** The four-questions shape (below) is
> RATIFIED (matchable enum, opaque Impure per-variant payloads, common ops transport-blind, kind-specifics
> matched, `LociDiedError` pattern). Only the NAMES are open, and here is the trail:
> - intueri (cast 2026-07-24) returned **`:wat::kernel::Child'`** (variants `Thread`/`Process`; accessors
>   `child-pid'`/`child-wait'`; `Peer'` left as the self-peer) — reasoning: the parent handle is *custodial*
>   (pid/pdeathsig/lifeline/reap), not a symmetric peer, matching `std::process::Child`.
> - **The builder CORRECTED it: `Child'` fails the REMOTE case.** Over a wire (uds / localhost tcp / remote
>   mTLS) there is **no parent/child** — you don't fork a remote host, you *connect* to it; nothing to reap.
>   The substrate prefers **client/server** vocabulary. So the custodial ops (pid/wait/kill) are
>   **LOCAL-FORK-variant-specific** (`Thread`/`Process` carry them; a `Remote` variant never does — exactly
>   what per-variant kind-specifics capture), and the enum's NAME must tell the truth across ALL transports
>   (the local end's handle to a spawned-or-connected **server**), not the local-fork truth `Child'` names.
> - **Far-side re-cast:** intueri on the enum name + variant names + accessor verbs under: *client/server
>   vocabulary; the handle generalizes thread→process→uds→tcp→mtls→remote; custodial verbs live only on the
>   local-fork variants.* Candidates to weigh (not decided): the enum as a `Server'`/`Connection'`/`Client'`-
>   family noun; variants `Thread`/`Process`/`Uds`/`Tcp`/`Remote`. FQDN = zero collision risk (the builder's
>   point) — so pick the truest names freely. NOTE: replace the `Peer'`/`Child'` working names in the
>   user-forms below with the re-cast result before building.

## Why (how the de-prime surfaced it)

The `spawn-process` de-prime hit a wall: `spawn-program'` returns a **transport-specific parent handle**
per clause — thread → `:wat::kernel::Thread'<R,S>`, process → `:wat::kernel::Process'<I,O>` (an opaque
`RustOpaque(ProcessPeerBundle)`). ~10 caller tests broke because their `.rs` harnesses field-poked the
*old* concrete `Process` struct (`fields[3]`→`Forked`→exit-code); the opaque `Process'` has no such
shape, and there is **no way to `match` the peer's kind** to reach kind-specific data (a process's pid
for the lifeline/pdeathsig tests). Migrating each caller to the transport-specific handle is fighting a
missing unification — and doing it twice (once to `Process'`, once to the eventual unified peer) is waste.

## What already exists (grounded)

- **The SELF-peer is already unified.** Inside a worker, `[self <- :wat::kernel::Peer'<S,R>]` is the one
  transport-blind self-peer (`spawn.rs:130,551`); `send'`/`recv'` take `Peer'` (`spawn.rs:650`).
- **The PARENT handle is NOT.** `spawn-program'` hands back `Thread'<R,S>` / `Process'<I,O>` — and via
  arc-293 (`Nature::Peer`'s `root_keyword` = `:wat::kernel::Peer'`, "every aggregate registers
  `:Name <: root_keyword()`"), **`Thread'` and `Process'` are ALREADY subtypes of `Peer'`.** So
  `send'`/`recv'`/`recv-all'` already accept the parent handles transport-blind (the 9 green de-prime
  migrations prove it). What's missing is the ability to **`match` the kind** for kind-specific behavior.
- **The payloads (what each variant must carry):**
  - `Process`: `ProcessPeerBundle` (`spawn.rs:268`) — `peer: Process<String,String>` (the **Pidfd** →
    pid + `wait`, the channels), `err: Receiver<String>` (the crash channel), `_lifeline_w: OwnedFd`
    (the lifeline). Kind-specifics: **pid / wait / pdeathsig / lifeline** (all PROCESS-specific — a
    remote wire has no local pid).
  - `Thread`: the thread parent handle (shared-memory channels; no fork, no pid). Kind-specifics: the
    shared-memory nature (a thread worker may carry impure captures via `ThreadSelfPeer'`).

## The design — `Peer'<I,O>` becomes a matchable sum

Turn the `Peer'` subtype-*top* into a matchable **enum** whose variants ARE the loci kinds. This is the
`LociDiedError` pattern applied to peers: one type every caller can exhaustively `match`, where a new
transport is a new variant the checker forces every kind-match to handle (the R52 ablaze / verbosity-
is-the-shield).

```clojure
;; :wat::kernel::Peer'<I,O> — the ONE parent handle spawn-program' returns, matchable by kind.
;; I = what the parent sends; O = what the parent receives (send=I, recv=O — grounded orientation).
(:wat::core::defenum :wat::kernel::Peer'<I,O>
  (Thread  [<thread parent-handle payload — shared-memory channels>])
  (Process [<process parent-handle payload — pid/wait/lifeline + channels>]))
  ;; …future WIRE kinds (Uds, Tcp, Remote) arrive as new variants — the checker
  ;; then lights up every kind-match that doesn't handle them (the shield).
```

### The two op classes (this is the crux — reconciles transport-blind with per-kind)
- **Common ops NEVER match** — `send'` / `recv'` / `recv-all'` take `Peer'<I,O>` and dispatch on the
  variant **internally**; the caller stays transport-blind (the "never `if process`" for wire ops).
- **Kind-specific ops DO match** — a caller that needs a process's pid (lifeline/pdeathsig) writes
  `(match peer ((Process p) …p's pid/wait…) ((Thread t) …))`; honest exactly where the transports
  genuinely differ. A new transport forces the match to grow an arm (the shield).

### The user-forms (the UX)
```clojure
;; spawn — returns the ONE Peer' enum regardless of locus:
(:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::process) (forms …))]
  ;; common: transport-blind, no match
  (:wat::kernel::send' peer req)
  (:wat::core::match (:wat::kernel::recv' peer)
    ((:wat::kernel::RecvOutcome::Message v) …) …)
  ;; kind-specific: match the peer kind
  (:wat::core::match peer
    ((:wat::kernel::Peer'::Process p) (:my::supervise-pid p))
    ((:wat::kernel::Peer'::Thread  t) (:my::supervise-thread t))))
```

## Four questions (the shape — decided)
- **Obvious?** **YES** — "a peer is a Thread or a Process (or a future wire kind); match it to reach
  what differs." One enum, the loci named.
- **Simple?** **YES** — one type; common ops on the whole, kind-specifics on the variant. Replaces the
  two-parent-handle-types + implicit-subtype-only model with one matchable sum.
- **Honest?** **YES** — a new transport is a NEW VARIANT the checker forces every kind-match to handle
  (no silent `if process` gap; the shield). No opaque you can't ask "which kind are you?".
- **Good UX?** **YES** — common path is match-free (transport-blind); kind-specific path is an explicit,
  exhaustive match. The wrong path (ignoring a new transport's kind-specifics) is uncompilable.

## Open sub-decisions (ground/rule at draw-out)
1. **Variant payloads' visibility.** Are `Thread`/`Process` payloads opaque (the caller matches to *pass
   them to kernel funcs* like `pid`/`wait`) or do they expose fields? Lean: **opaque payloads** + kernel
   accessor funcs (`(:wat::kernel::…/pid p)`), matching the RustOpaque model — the caller matches to
   get the right-kind opaque, then calls kind-specific kernel verbs. GROUND against how the lifeline/
   pdeathsig tests need the pid.
2. **`Peer'` self-peer vs parent-handle: one enum or two?** The SELF-peer (`[self <- Peer'<S,R>]`) is
   already unified/opaque; the PARENT handle is what's becoming the enum. Decide whether the self-peer
   is the same `Peer'` enum (a worker matching its own kind — probably never needed) or the enum is
   specifically the parent handle and the self-peer stays the opaque. Lean: **the enum is the parent
   handle**; the self-peer stays the transport-blind opaque (a worker doesn't inspect its own locus).
   This means the NAME may need care (is the parent enum `Peer'`, or e.g. a distinct parent-handle
   noun?) — **cast intueri at draw-out.**
3. **`ThreadSelfPeer'` relationship** — the impure-capture escape hatch (`spawn.wat:267`) — stays as the
   Thread worker's self-param; unaffected (it's the self-peer side, not the parent enum).
4. **Nature::Peer** — does the enum replace the `Nature::Peer` opaque model, or coexist? An enum with
   opaque per-variant payloads (the `LociDiedError` shape) likely *is* the clean replacement.

## The strike scope (ablaze-driven, big — a foundation stone)
1. Register `:wat::kernel::Peer'<I,O>` as an enum (Thread/Process variants; opaque payloads) in `types.rs`.
2. `spawn-program'` (`wat/spawn.wat` defclause) — both clauses return the unified `Peer'<I,O>`
   (wrapping the thread/process payload in the matching variant); `eval_kernel_spawn_thread_prime` /
   `_spawn_process_prime` (`spawn.rs`) construct the variant.
3. `send'`/`recv'`/`recv-all'` — dispatch on the variant internally (they already accept `Peer'`; make
   them match the variant to reach the transport's send/recv).
4. Kind-specific kernel accessors (`pid`/`wait` on the Process variant) for the lifeline/pdeathsig tests.
5. The de-prime callers (the ~10 redesign + 3 lifecycle + the Arc-sharing `counter-service-N3`) migrate
   to the `Peer'` enum — the redesigns match `Process`→pid; the common ones `send'`/`recv'`; the
   Arc-sharing is against the one `Peer'` type. The 2 subject-gone (`t7`, `arc208`) still annihilate.
6. R52 ablaze: the re-type screams every producer/consumer; weigh 4217/0.

## Dependency / sequence
This stone is the FOUNDATION the `spawn-process` de-prime is done against (build it FIRST, migrate the
callers to `Peer'` — NOT to the transport-specific `Process'`). The 9 mechanical migrations already on
disk are forward-compatible (they use `send'`/`recv'`/`recv-all'` on the `Peer'` supertype); their
`Process'` return-annotations become `Peer'` when this lands. Kin: `LociDiedError` (the enum-every-caller-
handles pattern), the arc-209 "unify the two under the flat-sea" TODO (`spawn.rs:98/112`), the
shared-memory-vs-wire axis ([[project_aws_on_a_single_computer_then_networking]]).
