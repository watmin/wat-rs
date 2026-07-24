# BRIEF — the `connect'` OUTCOME WALL (peer-lifecycle Strike 4 — the LAST peer wall)

> **The work in one paragraph.** `:wat::kernel::connect'` returns a bare `Peer'<S,R>` and RAISES on its
> *handleable* failures (ECONNREFUSED / no listener, identity-reject, peer_cred + socket-wrap io). Per the
> peer-lifecycle LAW — a handleable failure is a matchable enum; raise is reserved for must-never-happen —
> `connect'` returns `:wat::kernel::ConnectOutcome<S,R>` (Impure, parametric, mirrors `AcceptOutcome`/
> `RecvOutcome` — `Connected` holds a live `Peer'`). This is the **exact twin of `accept'` (Strike 3,
> `2976d887`)** — copy that strike's shape. **Big sweep** (`connect'` is common — grep ~161 sites; the rider
> re-scouts via the CHECKER, not a grep — the recv' lesson). This is the LAST peer-lifecycle wall — landing
> it makes recv'/send'/poll'/close'/accept'/connect' all whole.

## The shape — RULED (four-Q + grounding), do not re-fork
```clojure
;; mirror AcceptOutcome<R,S> (types.rs, arc 278 Strike 3) — PARAMETRIC + Impure (Connected holds a live Peer').
(:wat::core::defenum :wat::kernel::ConnectOutcome<S,R> :wat::enum::Impure
  :Connected [peer <- :wat::kernel::Peer'<S,R>]  ;; dialed + admitted (success)
  :Refused   [cause <- :wat::kernel::Failure]     ;; ECONNREFUSED / no listener / rendezvous gone — RETRYABLE transport
  :Rejected  [cause <- :wat::kernel::Failure]     ;; OnlyThisPeer identity check failed (server pid/euid ≠ minter) — NOT retryable
  :Failed    [cause <- :wat::kernel::Failure])    ;; peer_cred read / socket-wrap io error
```
Note the arg order `<S,R>` — connect's current return is `Peer'<S,R>` (send-type first), the MIRROR of accept's
`<R,S>`. `Rejected` **fires here** (unlike accept', where it was cut) — the client dials once and the server's
identity failing is a caller-visible outcome. `Failed` added beyond the DESIGN's 3 variants (the io errors that
are neither transport-refused nor identity-rejected), exactly as accept' needed one. Named-per-kind (R52):
Refused (retry) / Rejected (don't — wrong process) / Failed (io — abort/log) are handled distinctly.

## Read in order (the rooms)
1. **The `accept'` strike `2976d887`** — `git show 2976d887` — the twin; copy its shape (AcceptFail-style inner
   Result, `accept_outcome_*` builders, infer + must-use). This whole strike mirrors it.
2. `src/types.rs` — the `AcceptOutcome<R,S>` `register_builtin` (Impure, parametric) — the exemplar to mirror.
3. `src/kernel/address.rs:148-212` — `SocketAddress::connect` (PROCESS tier) — the raise sites (the disposition
   table below): `:160` connect_addr→Refused, `:185` connect_admits→Rejected, `:176/:202` io→Failed, `:153`
   malformed addr→STOP.
4. **the THREAD-tier connect** — find the crossbeam-rendezvous `connect()` impl (the sibling of `SocketAddress::
   connect`; grep `fn connect(` in `src/kernel/address.rs`) — ground ITS failure set too (rendezvous gone →
   Refused; decode/io → Failed). Both tiers must return the outcome.
5. `src/kernel/address.rs:284` — `connect_as_value` (the conversion seam; wraps `connect()`, returns Peer-as-Value
   today) — build the `ConnectOutcome` here (mirror `accept_as_value`).
6. `src/runtime.rs:20957` — `eval_connect_prime` (arity + address-type-mismatch raises STAY; delegates to
   `connect_as_value`).
7. `src/check.rs:11124` — `infer_connect_prime` (returns `Peer'<S,R>` today → make it `ConnectOutcome<S,R>`).
8. `src/check.rs` — `MUST_USE_PARAMETRIC_HEADS` (`["wat::kernel::RecvOutcome", "wat::spawn::ServiceEvent",
   "wat::kernel::AcceptOutcome"]`) — **add `"wat::kernel::ConnectOutcome"`**.

## The disposition (both `connect()` impls + `connect_as_value`)
Mirror accept': change `connect()` to return `Result<Result<Peer, ConnectFail>, EvalBreak>` (outer Err =
must-never-happen raise; inner Err = handleable), and `connect_as_value` builds the outcome value. `ConnectFail`
carries the kind: `{Refused(String), Rejected(String), Failed(String)}`. Add builders
`connect_outcome_{connected,refused,rejected,failed}` in `runtime.rs` beside `accept_outcome_*`. Use the canonical
`message_only_failure` for the cause (NOT a hand-rolled `struct-new` Failure — R57). Mapping:

| current raise | tier | → |
|---|---|---|
| `connect_addr` fail (ECONNREFUSED) `address.rs:160` | process | `Refused[cause]` |
| `!connect_admits` (identity) `address.rs:185` | process | `Rejected[cause]` |
| `peer_cred` read `:176` · `wrap socket stream` `:202` | process | `Failed[cause]` |
| thread-tier rendezvous-gone / decode | thread | `Refused` (gone) / `Failed` (decode io) — ground it |
| `from_abstract_name` malformed `:153` | process | **STOP-3** (substrate-minted addr; must-never-happen vs Failed) |
| arity / address-type-mismatch (`eval_connect_prime`) | — | **STAY raises** (must-never-happen) |

## The sweep (checker-scouted — NOT a grep) — the BIG one
`connect'` is common (~161 grep hits — many are comments/docs; the LIVE call sites are what matter). After
`infer_connect_prime` returns `ConnectOutcome`, build `target/release/wat` and run the floor / `--check` to find
EVERY site that now faces an unfaced outcome; face each: `(match (connect' addr) (ConnectOutcome::Connected p) …
(ConnectOutcome::Refused c …) (ConnectOutcome::Rejected c …) (ConnectOutcome::Failed c …))`. Per-site: a gone
server where connect is fatal → `assertion-failed!`; in a dial-retry loop → back off. **Atomic** — no green state
where connect' returns the outcome but a site drops it. The grep undercounts (embedded-in-`forms` sites won't
show at top-level `--check`, the accept' lesson) — trust the CHECKER + the floor, not the grep.

## The probe (RED-first)
`tests/comms/probe_arc278_connect_outcome_wall.{rs,wat}` (mirror `probe_arc278_accept_outcome_wall`):
- happy dial → `Connected[peer]` (peer asserted live); dial a dead/no-listener address → `Refused`.
- if cheaply reachable: an identity-mismatch → `Rejected` (the client-side gate is testable — a server that isn't
  the minter); io `Failed` via the eval mapping if not cheaply reachable (no faking — the accept'/close' precedent).
- structural `Value::Enum` asserts, never a loose `Debug`-string contains. RED before, GREEN after.

## STOP triggers (rejection criteria — halt + surface)
- **STOP-1:** the checker-scout finds connect' sites in NON-test PRODUCTION/stdlib `:wat::` wat — surface the full
  list before sweeping (a stdlib caller changes the blast radius).
- **STOP-2:** `connect()`'s errors can't be cleanly split Refused/Rejected/Failed (a single opaque error lumps
  them) — STOP, report the error surface (maybe fewer variants).
- **STOP-3:** the `from_abstract_name` malformed-address raise (`:153`) — ground it: the address is substrate-minted
  (a capability), so a malformed one is likely an in-process substrate BUG → **stays a raise** (must-never-happen).
  If you find it can carry wire/adversarial data → `Failed`. Do not silently pick — surface the call.

## Weigh (the orchestrator re-runs; do NOT trust the report)
- the RED probe: RED before, GREEN after.
- **the floor: `cargo nextest run --release`, read the Summary line** (never a piped exit). Expected 4219/0 + the
  new probe. Any OTHER new RED = a swallow site the scout missed → STOP-1. **The unused-span lint MUST stay green**
  (if connect' adds an ignored `_span`, it needs a rune — the lint will catch it).
- content-integrity: the diff is types.rs + address.rs (both connect impls + connect_as_value) + runtime.rs
  (builders) + check.rs (infer + must-use) + the faced sweep sites + the new probe. Nothing else moved. Do NOT
  touch the recv'/send'/poll'/close'/accept' walls.

## Copy for shape
The `accept'` strike (`2976d887`) — this is its exact twin. `BRIEF-accept-outcome-wall.md` for the full pattern.
