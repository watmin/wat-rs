# Arc 214 — Concurrency toolkit (foundations + brackets + services)

**Status:** OPEN 2026-05-18. Foundational arc. Ships the complete wat concurrency toolkit; structurally enforced; deadlocks illegal forever.

## Mission

Exit this arc with **wat's complete concurrency story** — peer-oriented user surface + Parallel-style brackets + protected-state services — all structurally enforced; one canonical path per concern; no options at any layer; users cannot make mistakes because the type system + module privacy + cascade-by-construction make wrong shape impossible to express.

**This is ONE arc, not three.** Per user direction 2026-05-18:

> *"we do it perfect now and build on top of them forever"*
> *"we exit this arc with all of our concurrency tools. we have proper OOP, proper concurrent or parallel processing (each, map) -- reduce is just a consumer on map - no sugar"*
> *"we get all the greatness of Ruby's OOP, FP and concurrency"*

Three separate arcs would have three close conditions, three INSCRIPTIONs, three opportunities to ship in a half-correct state where consumers (brackets, services) layer on foundations that aren't yet sealed. One arc, all layers, per-stone trust gates between slices, atomic discipline.

## The user-facing concurrency model

A wat program does TWO things to be concurrent:

```clojure
;; 1. Spawn a peer. One verb. Tier picks the transport. Returns Thread<I,O> / Process<I,O> / future Remote<I,O>.
(let [peer (:wat::kernel::spawn-program' :thread my-program)]   ; or :process, or future :remote
  
  ;; 2. Talk to it. Polymorphic on peer type. Same verbs regardless of transport.
  (:wat::kernel::send' peer input)        ;; send input
  (:wat::kernel::recv' peer)              ;; receive output (blocks; cascade-aware)
  (:wat::kernel::try-recv' peer)          ;; non-blocking
  (:wat::kernel::select' [peer1 peer2])   ;; fan-in
  (:wat::kernel::close' peer))            ;; signal end-of-stream
```

**That's the whole user surface.** No Sender/Receiver to juggle; no channel construction; no tier-specific verbs. The peer IS the abstraction. Whatever you do works on Thread/Process/Remote identically.

The `'` (prime) on verb names is the development convention — during arc 214 they coexist with legacy `:wat::kernel::send` / `recv` / `spawn-thread` / `spawn-process` / `spawn-program` / `fork-program`. After migration sweep retires the legacy verbs, primes rename to canonical.

## Layered architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Layer 2 — Services (:wat::services::*)                                   │
│   ServiceWithProvisioning rebuilt; OOP-as-protected-state                │
│   Uses kernel verbs internally; uses comms tier for construction         │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 1 — Brackets (:wat::brackets::*)                                   │
│   parallel-each / parallel-map (reduce composes; no sugar)               │
│   Uses kernel verbs internally; uses comms tier for shared work-channels │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 0c — Peer types (:wat::kernel::*)                                  │
│   Thread<I,O>, Process<I,O>, Remote<I,O> (future)                        │
│   IDENTICAL surface; transport-agnostic to consumer                      │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 0b — Kernel verbs (:wat::kernel::*)                                │
│   send / recv / try-recv / select / close (polymorphic; peer-oriented)   │
│   spawn-program (unified; :tier dispatch)                                │
│   Multimethod dispatch over peer types (arc 146)                         │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 0a — Comms tier primitives (:wat::comms::*)                        │
│   :wat::comms::thread::{Sender<T>, Receiver<T>, Select, pair}             │
│   :wat::comms::process::{...}                                            │
│   crossbeam underneath (thread) / io_uring underneath (process)          │
│   SUBSTRATE-INTERNAL — users never touch this layer                      │
└─────────────────────────────────────────────────────────────────────────┘
```

**Audience separation:**
- **Users** see Layer 0c (peer types) + Layer 0b (verbs) + Layers 1/2 (brackets/services)
- **Substrate authors** see Layer 0a (comms primitives) when building brackets/services internals
- **Wrong shape is structurally impossible** at every layer per Layer 0a's structural wall + Layer 0b/c's polymorphic dispatch

## Layer 0a — Comms tier primitives

Tier-specific channel infrastructure. ONE mechanism per tier; no options:

| Tier | Underlying | Wire form | Construction verb (substrate-internal wat) |
|---|---|---|---|
| `:wat::comms::thread::*` | crossbeam | T: Send + 'static | `pair` |
| `:wat::comms::process::*` | **io_uring** | T: HolonRepresentable (EDN over pipes) | `pair`, `from-inherited-fds` |
| `:wat::comms::remote::*` (future) | TBD | T: HolonRepresentable | TBD |

### Rust-side types (in `src/comms/{thread,process}.rs`)

```rust
// thread tier
pub struct Sender<T: Send + 'static> { /* private inner: crossbeam::Sender */ }
pub struct Receiver<T: Send + 'static> { /* private inner: crossbeam::Receiver */ }
pub struct Select<'a, T> { /* auto-registers SHUTDOWN_RX */ }

impl<T> Sender<T> { pub fn send(&self, t: T) -> Result<(), SendError<T>>; pub fn close(self); }
impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, RecvError>;       // cascade-aware
    pub fn try_recv(&self) -> Result<T, TryRecvError>;
    pub fn len(&self) -> usize;
}

pub fn pair<T>() -> (Sender<T>, Receiver<T>);  // capacity-1 mini-TCP (see DESIGN § "Mini-TCP at depth 1 — universal symmetry")
// process tier — IDENTICAL surface; T bound differs; io_uring underneath; pair() returns io::Result<...> (libc::pipe(2) can fail)
```

### Universe-residency + Mini-TCP at depth 1 (universal symmetry) (2026-05-19 architectural clarification)

User direction 2026-05-19: *"what wat wants is 'i want to run this program in a {thread,process} and it just works.. i can comm to it by sending data and getting data — i don't care where its hosted' / the user must choose a hosting env but the programs never know what env their in — they exist in a universe and that universe has provided a comm channel to use."*

**The discipline:** programs are universe-resident; the universe provides comm channels; the program never knows its transport. Hosting env chosen at the OUTSIDE; program inside writes `peer.send(v)` / `peer.recv()` and runs identically across tiers.

**Two-layer honesty:**

| Layer | Surface | Identical-across-tiers requirement |
|---|---|---|
| **Program-facing** (what the program sees) | Trait `CommSender<T>` + `CommReceiver<T>` + (future Slice 4) peer types `Thread<I,O>` / `Process<I,O>` / `Remote<I,O>` | **MANDATORY identical** — program code does not vary by tier |
| **Substrate-internal** (what the hosting env wires up) | Concrete `thread::Sender` / `process::Sender` etc. | Asymmetries permitted when STRUCTURALLY honest |

**Two substrate-internal asymmetries — each verified honest:**

1. **T bound:** `T: Send + 'static` (thread) vs `T: HolonRepresentable` (process). Transport requirements differ unavoidably; honest.

2. **`pair()` return type:** infallible (thread) vs `std::io::Result<...>` (process). `libc::pipe(2)` can fail; failure mode IS exposed; honest.

**Mini-TCP at depth 1 (universal symmetry)**

User direction 2026-05-19 (pre-wat-rs trading-lab convergence): *"before wat-rs existed - we were in the holon-lab-trading and build mailboxes and whatever their opposite is - we found that only ever needed a depth of 1 for everything - this forces us into a lock step that has an organic nature to it... its breathes based on system load - its dynamic but predictable.. when we had the option to send N things and then block we have massive perf hits - i think the thread comms need to be like process comms - you may only send one thing and must immediately read back - either an ack or some data - this is the only supported pattern - mini-tcp everywhere - forcing us to be locked eliminates entire categories of problems"*

**Four-questions verdict on `pub fn bounded<T>(n: usize)` (thread tier):**

- **Obvious?** NO — asymmetric with process tier (which has ONE factory, kernel-bounded pipes).
- **Simple?** NO — two factories; substrate-author choice carries semantic meaning; the meaning is "honor the discipline or violate it."
- **Honest?** NO — exposes a knob the substrate's own practice proved harmful (trading-lab: N > 1 produces massive perf hits + entire categories of problems).
- **Good UX?** NO — substrate-internal callers (brackets, services) could pick `bounded(64)` and break mini-TCP discipline; no structural guard.

**FAILS YES YES YES YES.** Factory retired.

**Four-questions verdict on `pub fn pair<T>()` returning bounded(1):**

- **Obvious?** YES — symmetric with process tier; ONE factory per tier.
- **Simple?** YES — N identical `pair()` call sites; no choice to make.
- **Honest?** YES — capacity-1 IS the mini-TCP discipline structurally enforced; senders cannot "send N then block" because send blocks at depth 1.
- **Good UX?** YES — substrate-author CANNOT pick wrong depth; lock-step by construction.

**YES YES YES YES.** `pair()` flipped from unbounded to bounded(1).

Universal symmetry restored: both tiers expose ONE factory (`pair()`) whose underlying transport enforces mini-TCP semantics structurally. Programs running in any universe see identical send/recv semantics. See § "Slice 2 forward-correction (2026-05-19) — Mini-TCP at depth 1 (universal symmetry)" at end of this DESIGN for full detail + cross-references.

**Convergence:** the universe-residency principle composes with Convergence #13 (autoscaling of correctness):
- Universe-residency (program/user layer): "programs don't know transport"
- Autoscaling-of-correctness (substrate/resource layer): "substrate manages resources reflexively; users don't pick"

Both compose into: users declare hosting env; nothing else. Programs run identically across thread/process/remote; substrate handles all the resource management invisibly. The discipline propagates up via Slice 4 (peer types absorb the substrate-internal asymmetries) + Slice 7 (brackets compose peers) + Slice 8 (services as universe-resident actors).

Cross-references: memory `project_universe_residency`; memory `project_autoscaling_correctness`; INTERSTITIAL § "2026-05-19 — Universe-residency principle + bounded() four-questions verdict"; INTERSTITIAL § "2026-05-19 (post-compaction, Slice 2 forward-correction) — Mini-TCP at depth 1: the trading-lab origin returns"; § "Slice 2 forward-correction (2026-05-19) — Mini-TCP at depth 1 (universal symmetry)" at end of this DESIGN for the inscription-archival four-questions block.

### Shared traits (in `src/comms/mod.rs`)

```rust
pub trait CommSender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>>;
    fn close(self);   // infallible — consuming self IS the close (Drop handles OS cleanup); single-close enforced by move semantics
}
pub trait CommReceiver<T> {
    fn recv(&self) -> Result<T, RecvError>;
    fn try_recv(&self) -> Result<T, TryRecvError>;
    fn len(&self) -> usize;
    fn close(self);   // infallible (same rationale as CommSender::close)
}

pub trait HolonRepresentable: Send + 'static {
    fn to_holon_ast(&self) -> HolonAST;
    fn from_holon_ast(ast: &HolonAST) -> Result<Self, WireError> where Self: Sized;
}

// NO blanket impl. `Into<HolonAST>` consumes self while `to_holon_ast` takes
// `&self`; a blanket would force `T: Clone` overhead at every send (silent
// clone tax at call sites). Manual `impl HolonRepresentable for T` per
// substrate-internal type is the honest form (see src/comms/mod.rs doc
// comment for full rationale). Future arc may revisit if a zero-cost
// reference-style conversion surfaces.

pub struct ReceiverIndex(pub usize);   // newtype over usize so SelectOutcome::Recv index can't be confused with a count

pub enum SelectOutcome<T> {
    Recv { index: ReceiverIndex, result: Result<T, RecvError> },   // named fields for read-once clarity
    Shutdown,
    SubstrateError(std::io::Error),   // io_uring / SQE / submit_and_wait failure on process-tier Select; thread tier never produces this arm
}

// Error types: SendError<T>, RecvError, TryRecvError, WireError
// (no CloseError — close is structurally infallible per move semantics)
```

### Cascade-by-construction (locked in this layer)

EVERY blocking method auto-wires the cascade:
- **Thread Receiver::recv()** → `crossbeam::select! { recv(data), recv(SHUTDOWN_RX) }`
- **Process Receiver::recv()** → io_uring multi-arm submission on `[data_fd, broadcast_fd]`; first completion wakes
- **Thread/Process Select** → auto-registers the shutdown signal as first arm
- **Sends** propagate cascade via reverse-direction EPIPE (peer-closed = peer-woke + closed)

Worker code cannot bypass the cascade. The wrapper IS the cascade.

### Dependencies

Per `scratch/DEPENDENCY-DOCTRINE.md`:

**New dep accepted:** `io-uring` crate
- Used by canonical projects: tokio-uring, glommio (Datadog), monoio
- Active maintenance (rio team; tokio-adjacent)
- Four-questions: Obvious YES (name says it) / Simple YES (focused crate) / Honest YES / Good UX YES

**Existing deps preserved:** crossbeam_channel (thread tier); wat-edn (HolonRepresentable serialization).

### Tunable — `:wat::config::set-process-tier-uring-depth!`

> **SUPERSEDED 2026-05-19.** This section's tunable was rejected by the four-questions during Stone E design. See § "Stone E forward-correction (2026-05-19) — TCO discipline + reflexive rebuild" below for the architectural reframe. Original text preserved per `feedback_inscription_immutable`.

io_uring SQ/CQ ring size per process-tier `Receiver` / `Select`:

- **Default:** 512 (power of 2; midpoint between tokio-uring's 256 and monoio's 1024)
- **Validation:** power of 2 in `[1, 4096]`; out-of-range → RuntimeError at setter site
- **Per-runtime semantics:** atomic config; read at receiver/select construction; existing rings keep construction-time size; typically called at program startup

**Parameter-tunability, not option-tangle** (per `feedback_options_are_tangle`): ONE mechanism (io_uring; canonical); ONE setter (canonical); power users tune the parameter; the chokepoint discipline is unchanged.

**Future tunables** (SQPOLL, registered buffers, linked operations) explicitly scoped OUT — progressive disclosure as concrete substrate use cases justify.

### Stone E forward-correction (2026-05-19) — TCO discipline + reflexive rebuild

Stone E's pre-implementation walk surfaced that the Tunable section above was the option-tangle pattern disguised as discipline. Inscribed forward per `feedback_inscription_immutable`.

**Four-questions verdict on `:wat::config::set-process-tier-uring-depth!`:**

- **Obvious?** NO — what does 512 ENABLE that 4 doesn't? Three sites in `src/comms/process.rs`: `uring_read_into_acc` uses 2 SQEs (one Read); `wait_for_data_or_cascade` uses 4 SQEs (two POLL_ADDs + headroom); `Select::select` uses `arm_count.next_power_of_two()`. None care about 512. The knob tunes nothing observable.
- **Simple?** NO — adds setter + atomic + bounds-validation + wiring for a value with no honest effect.
- **Honest?** NO — claims "parameter tunability" but the parameter isn't actually tunable in any meaningful sense. Capacity is determined by what the ring *does*, not by what the user *picks*.
- **Good UX?** NO — users tune it; observe no behavior change; or worse, tune to 4096 and waste kernel resources for nothing. FOOTGUN.

**FAILS YES YES YES YES.** Tunable rejected; setter not minted.

**The substrate-architectural truth — capacity is structural, not policy.** Ring capacity emerges from what the ring *does* at each layer. Every capacity at every layer can be derived from a user-visible structural declaration:

| Site | Capacity | Why |
|------|----------|-----|
| Receiver's persistent ring | 4 (covers Read + POLL_ADD pair) | Receiver's operation set is fixed for its lifetime |
| Select's persistent ring | `next_power_of_two(arm_count + 1)`; reflexive rebuild on mismatch | User declares arms via `select.recv(&rx)`; substrate matches |
| Bracket's internal Select (fan-in over N replies; future Slice 7) | `next_power_of_two(N + 1)` derived from bracket's N | User declares N positionally: `(parallel-for-each :tier N items fn)` |
| Defservice's dispatch-loop Select (over M users + broadcast; future Slice 8) | derived from Grant calls | User declares concurrency via Grant pattern |

Every capacity emerges from a user-visible declaration. Substrate computes; user never sees an io_uring entry count; user cannot pick wrong.

**The TCO discipline — FDs persist; io_urings are ephemeral frames.**

The substrate manages io_uring resources reflexively, analogous to tail-call optimization at the stack frame:

- **FDs are the stack** — persistent state; the real resource (pipe ends, `OwnedFd`); allocated once, owned by the Receiver/Sender, dropped only at the owning struct's `Drop`.
- **io_urings are the frames** — ephemeral; sized for current structural need; replaced when need changes. The kernel resource the substrate manages invisibly.

| Layer | What persists | What's replaceable |
|------|---|---|
| Receiver | `read_fd: OwnedFd` (the pipe end) | `ring: IoUring` (sized for current operations) |
| Select | `receivers: Vec<&Receiver>` (registration set) | `ring: IoUring` (sized for current arm_count) |
| Service dispatch loop (future) | user registry | the Select ring serving the N+1 arms |
| Bracket fan-in (future) | the N child Process handles | the Select ring across N replies |

**Reflexive correctness — the invariant the substrate proves at every operation.**

At every operation entry on a structure with a ring, the substrate maintains:

> **invariant:** `current_capacity == next_power_of_two(structural_need + 1)`

If the invariant holds: reuse the ring. If it doesn't (structural need grew OR shrank): rebuild the ring at the right capacity. The replacement IS the tail call — old ring drops; new ring constructs; FDs untouched; structural state untouched.

**Symmetric grow + shrink** — the substrate proves correctness by scaling DOWN when over-capacity, not just up. "Approximately correct" isn't testable; "always correct" is. Long-running services + brackets + remote layers don't stockpile over-capacity rings across hours of execution. Memory + kernel resources stay MATCHED to current need at every moment.

**The substrate proves itself reflexively.** Per `feedback_attack_foundation_cracks` + `feedback_any_defect_catastrophic`: the foundation is binary-correct or it isn't. The reflexive-rebuild discipline IS the foundation arc 214 is building toward — not "fast enough" or "correct enough" but *provably always correct by construction*. Every higher layer (brackets, services, remote) inherits "always exactly right-sized" without re-establishing the discipline.

**Why this deepens "no tunable" from option-tangle to logical incoherence:**

A global `set-uring-depth!` would say "use N forever." But N is wrong the moment the structure changes. The substrate already KNOWS the right N at every moment by inspection of its own structure. The user "knowing better" is impossible — they don't see the structure the substrate sees. The dragon dies not just because the tunable is dishonest; it dies because the tunable is logically incoherent with how io_uring is being used.

**If/when an honest tunable emerges** (SQPOLL mode actually delivers measurable benefit for a real workload; bounded channel capacity for backpressure; etc.), THAT tunable gets minted at THAT moment with its own four-questions verdict — at the right layer, not buried in a HashMap or a global setter. Per `feedback_realizations_open_directions`: don't pre-mint slots; mint when the honest need arrives.

**Stone E decomposition (revised; two stones, not three):**

- **E-1 — Receiver persistent ring (capacity 4).** Add field; helpers operate on `&self.ring`; Clone gets fresh ring; migrate 2 Receiver runes (`uring_read_into_acc` + `wait_for_data_or_cascade`) from `temperare(no-reactor)` to cold. Static-need case (Receiver's operation set is fixed). 34/34 still pass.

- **E-2 — Select persistent ring (reflexive rebuild-on-mismatch).** Add field with lazy + grow-OR-shrink-on-mismatch; Select's Read-step delegates to fired Receiver's E-1 ring; invariant `cap == next_power_of_two(arm_count + 1)` at every select() entry; migrate Select rune to cold. 34/34 still pass.

- **E-3 (originally: config tunable) DIES.** Disqualified by four-questions, not deferred.

**Cross-references for the reframe:**

- `feedback_options_are_tangle` — the pattern the tunable was; rejected here
- `feedback_inscription_immutable` — original Tunable section preserved as historical record
- `feedback_attack_foundation_cracks` + `feedback_any_defect_catastrophic` — the foundation discipline reflexive-rebuild embodies
- `feedback_realizations_open_directions` — when honest tunables emerge, mint at the right layer then
- `feedback_refuse_easy_solutions` — "grow eagerly; never shrink" was the L2 default; rejected for the L4 symmetric discipline

### The structural wall (Slice 6)

Bare crossbeam outside `src/comms/thread.rs` = compile error. Bare libc::pipe/read/write/poll/epoll/io_uring outside `src/comms/process.rs` = compile error. Mechanism: Rust module privacy + `pub(crate)` discipline; external code sees only the wrapper public API.

**Tests get crate-internal exposure for verification; users see only the chokepoint.** Per user 2026-05-18: *"hide all the guts - don't let users make mistakes .. we need whatever exposure for us to test ourselves - but users cannot be given the option to fuck up - deadlocks are illegal"*.

## Layer 0b — Kernel verbs (peer-oriented; polymorphic)

The user-facing wat verbs. **Multimethod dispatch on peer type** (per arc 146 pattern):

```
:wat::kernel::send' peer data       ;; -> :wat::core::Result<:wat::core::nil, SendError>
:wat::kernel::recv' peer            ;; -> :wat::core::Result<O, RecvError>
:wat::kernel::try-recv' peer        ;; -> :wat::core::Result<:wat::core::Option<O>, TryRecvError>
:wat::kernel::select' [peer1 peer2 ...]   ;; fan-in; cascade-aware
:wat::kernel::close' peer           ;; signal end-of-stream
```

Dispatch table:
- `peer: :wat::kernel::Thread<I,O>` → routes to Thread's input/output channels (crossbeam underneath)
- `peer: :wat::kernel::Process<I,O>` → routes to Process's input/output channels (io_uring underneath)
- `peer: :wat::kernel::Remote<I,O>` (future) → routes to remote transport
- `peer: :wat::comms::thread::Sender<T>` / `Receiver<T>` → direct channel ops (substrate-author escape hatch)
- `peer: :wat::comms::process::Sender<T>` / `Receiver<T>` → same

**Same verb. Same semantics. Different transport invisible to caller.**

### Prime convention (development naming)

Existing `:wat::kernel::send` / `recv` / `try-recv` / `select` exist with current (channel-endpoint-oriented) semantics. Arc 214 mints REVISED versions with peer-oriented semantics under primes (`send'`, `recv'`, etc.). During development, prime + legacy coexist; callers migrate from legacy to prime; legacy retires; prime renames to canonical.

```
;; During dev (Slices 1-7):
:wat::kernel::send peer-or-sender data    ;; legacy (channel-endpoint-oriented; current substrate)
:wat::kernel::send' peer data             ;; revised (peer-oriented; arc 214's new shape)

;; After Slice 5 migration sweep completes:
;; legacy :wat::kernel::send retires; :wat::kernel::send' → :wat::kernel::send (canonical reclaimed)
```

Per `feedback_inscription_immutable`: each rename is a NEW commit; git history preserves the convergence explicitly. No retroactive edits.

**Apostrophe is wat-legal** (per src/lexer.rs:166 arc 171 retired comma in favor of `'` as canonical keyword-body separator); `:wat::kernel::send'` parses as a single keyword.

## Layer 0c — Peer types

```
:wat::kernel::Thread<I,O>      ;; in-process peer; crossbeam underneath
:wat::kernel::Process<I,O>     ;; cross-process peer; io_uring underneath
:wat::kernel::Remote<I,O>      ;; future; transport TBD
```

**IDENTICAL SURFACE.** Whatever you can do with `Thread<I,O>` you can do with `Process<I,O>`. The "crossbeam direct-struct-share vs EDN-over-pipe" is implementation detail; the consumer never sees it.

### Unified spawn primitive

```
:wat::kernel::spawn-program' :tier program    ;; the ONE user-facing spawn verb
```

Where:
- `:tier` is `:thread`, `:process`, or future `:remote` — picks the transport
- `program` is a value of type `:wat::core::Fn(I) -> O` — the work the peer performs

Returns:
- `:tier = :thread` → `:wat::kernel::Thread<I,O>`
- `:tier = :process` → `:wat::kernel::Process<I,O>`
- `:tier = :remote` (future) → `:wat::kernel::Remote<I,O>`

**This is the ONLY user-facing spawn verb.** Existing `:wat::kernel::spawn-thread` / `:wat::kernel::spawn-process` / `:wat::kernel::spawn-program` / `:wat::kernel::fork-program` ALL collapse — Slice 5 migration sweep retires them as callers move to the unified `spawn-program'` form. After migration, `spawn-program'` renames to canonical `spawn-program`.

Substrate-internal Rust functions (called by the wat-level dispatcher based on `:tier`):
```rust
crate::comms::thread::spawn_program(program) -> Thread<I, O>
crate::comms::process::spawn_program(program) -> Process<I, O>
```

These are NOT directly callable from wat code (substrate-internal only); the wat-level user sees only the unified `spawn-program' :tier program`.

### Sandbox-compatibility constraint (real user-visible)

`:process` and `:remote` programs cross address-space boundaries; their closure captures must be **HolonRepresentable** (serializable via HolonAST roundtrip). The substrate enforces this at spawn time via type-checker walker (per arc 170's existing sandbox-scope discipline):

```clojure
;; Legal — captures are HolonRepresentable
(let [seed 42]
  (:wat::kernel::spawn-program' :process
    (fn [input] (+ input seed))))

;; Compile error — captures non-serializable Sender directly
;; (Sender belongs to the parent's address space; can't cross to a child process)
(let [(tx, rx) (:wat::comms::thread::pair)]
  (:wat::kernel::spawn-program' :process
    (fn [input] (:wat::kernel::send' tx input))))   ;; ILLEGAL
```

`:thread` programs can capture freely (in-memory sharing via Arc). The asymmetry is **in the workload**, not in the interface — the API surface stays uniform; the type-checker catches workload-tier mismatches at spawn time.

This is not new substrate work; it extends arc 170's existing sandbox-scope walker to handle the unified spawn-program's `:tier` parameter.

## Slice 4 forward-correction (2026-05-20) — ProgramEnv + accessor surface

Arc 215's literal-flexibility work (Stone 1: `:wat::type::Infer` minted + `{...}`/`#{...}` literal completion; Stone 2: `[...]` Vector unification + `{...}` keyword-key lift) is the prerequisite for this section. With three Clojure-style collection literals now routing through unified inference, ProgramEnv has a clean construction surface — `{:k v}` literal at the call site, HM unification at the function signature, zero parser-layer restrictions blocking the LLM-first delivery claim.

This forward-correction inscribes the design conversation that landed 2026-05-20 between arc 215 closure and Slice 4 implementation. Eight design questions resolved with four-questions verdicts. The Layer 0c surface (peer types + unified spawn) extends with ProgramEnv as a load-bearing third parameter; the new `:wat::program::Env` namespace gains its accessor verb family.

### Why a forward-correction (not a rewrite)

Per `feedback_inscription_immutable`. The original Layer 0c + Unified spawn primitive sections (above) inscribed the spawn surface before arc 215 ratified literal flexibility. With literals now Clojure-fluent, the spawn surface gains a third param (env) naturally — but rewriting the prior section would hide the design evolution. This forward-correction extends, surfaces the new verdicts explicitly, and preserves the historical record of how the design grew.

### Design verdicts — eight questions, four-questions discipline applied

#### Q1 — spawn-program signature

**Verdict:** `(:wat::kernel::spawn-program' :tier env program)` — three positional args; reads "WHERE-WITH-DOING."

Four-questions: Obvious YES (positional order matches natural-language reading); Simple YES (three args, no optional flags, no keyword args); Honest YES (each arg has a single semantic; tier picks transport, env carries config, program carries work); Good UX YES (no overloading; no sentinel forms).

The legacy alternative `(spawn-program' :tier program)` (no env) was rejected because env is load-bearing — even when empty, it should be explicit at the call site. Verbose IS honest (per `feedback_verbose_is_honest`); callers who don't need env pass `{}`.

#### Q2 — tier-config vs ProgramEnv

**Verdict:** tier-config IS ProgramEnv. One map; one truth.

Substrate-internal concerns (mTLS keys for `:remote`; pipe sizes; whatever) live in the same map as user-facing app config. The program reads its own env; the substrate reads what it needs from the same map. No split.

Four-questions: Obvious YES (one map per spawn; no "two maps that look alike but mean different things"); Simple YES (single ingestion path; single type at the signature); Honest YES (no artificial split between "what the substrate cares about" and "what the user cares about" — both are config); Good UX YES (caller writes one literal).

For `:remote` tier specifically: the function signature mandates minimum-required-keys via the arc 215 two-layer enforcement model. The literal must satisfy the signature; signature satisfaction is the contract. Example shape (Slice 4 implementation will mint the concrete signature):

```wat
;; Spawning a remote program with required mTLS config
(:wat::kernel::spawn-program' :remote
  {:client-key   (:wat::holon::Atom "...bytes...")
   :remote-url   (:wat::holon::Atom "https://...")
   :app-setting  (:wat::holon::Atom 42)}
  my-program)

;; Mandatory keys enforced at call-site via function signature unification.
;; Missing :client-key → check fails with "expected key :client-key in ProgramEnv"
```

For `:thread` and `:process` tiers: no mandatory keys; users can decorate with whatever (`{:brackets-id 7}`, `{}`, etc.).

#### Q3 — Per-tier config shape

**Verdict:** Moot per Q2. All tiers see the same env type; per-tier-specific keys live in the same map.

#### Q4 — Namespace separation

**Verdict:** Two parallel namespaces for two genuinely different kinds of env:

- **`:wat::program::Env`** — wat-level program env; type is `HashMap<:wat::core::keyword, :wat::holon::HolonAST>` at the surface; internally a `HolonAST::Bundle` (holon all the way down). **In scope for Slice 4.**
- **`:wat::process::Env`** — OS-level process env vars; type is `HashMap<:wat::core::String, :wat::core::String>`; the equivalent of `$HOME`, `$PATH`, `getenv()`/`setenv()`. **Out of scope for Slice 4; separate concern.** May share design pattern with `:wat::program::Env` (similar accessor verbs) but is distinct namespace.

The two never collide — different namespaces, different types, different semantics. Process env vars are the OS's contract; program env is wat's. Reserving the `:wat::process::Env` name now prevents future namespace squatting.

Four-questions on namespace separation: Obvious YES (the two have orthogonal semantics; conflating them would be a Level-1 lie per intueri); Simple YES (one namespace per concept); Honest YES (different types, different lifecycle, different ownership); Good UX YES (callers reach for the right namespace based on what they're talking about).

#### Q5 — Polymorphic `get` dispatch

**Verdict:** Per arc 146 dispatch mechanism. The polymorphic verb `:wat::core::get` dispatches on container type; per-type implementations live under each container's namespace.

```wat
:wat::core::get                   ;; polymorphic dispatcher
:wat::core::HashMap/get           ;; HashMap impl (existing)
:wat::core::Vector/get            ;; Vector impl (existing)
:wat::program::Env/get            ;; Env impl (new in Slice 4)
```

Callers can use the short polymorphic form (`(:wat::core::get coll key)`) or the typed explicit form (`(:wat::program::Env/get env key)`). Both work; dispatch table grows by one entry.

Sets terminal: `:wat::core::HashSet` does not implement `/get` (sets have no key→value mapping; member presence is a different verb). Dig into a set returns `None` (treated as "no further navigation possible" — matches Ruby's `dig` behavior).

#### Q6 — Error model

**Verdict:** Option<T> uniformly for `/get` and `/dig`; expect-variants panic with KeyError-flavored diagnostic; default-variants return supplied fallback.

| Verb | Returns | Use case |
|---|---|---|
| `:wat::program::Env/get env key` | `:wat::core::Option<T>` | "might be present" |
| `:wat::program::Env/expect-get env key` | `T` (panic on miss/wrong-type) | "must be there; honest crash if not" (Ruby's `fetch`) |
| `:wat::program::Env/get-default env key default` | `T` (return default on miss) | "missing is OK; here's the fallback" (Ruby's `fetch(k, default)`) |
| `:wat::program::Env/dig env path` | `:wat::core::Option<T>` | chained version |
| `:wat::program::Env/expect-dig env path` | `T` | chained panic-variant |
| `:wat::program::Env/dig-default env path default` | `T` | chained default-variant |

Six verbs; the Option accessor triad × single-vs-chained.

None on both miss AND wrong-type (Ruby semantics): keeps the API uniform; matches Ruby's `dig` exactly. If a caller needs to distinguish miss from type-mismatch, that's a future arc — mint `/try-get` returning `Result<T, KeyError>` where `KeyError ∈ {NotFound, TypeMismatch}`. **Default: Option semantics; richer Result variant minted only if a real use case emerges.**

The panic in `/expect-*` carries KeyError-flavored diagnostic: position-named per arc 138; matches arc 107's typed-expect pattern.

#### Q7 — Path terminology

**Verdict:** **`path`** — the arg name for `/dig`'s navigation parameter. A `Vector<HolonAST>` whose elements are successive lookup keys.

```wat
(:wat::program::Env/dig env path -> :wat::core::Option<T>)
```

Rationale: `path` is communicative across communities (Clojure's `get-in`, Python's dict-paths, JSON paths, dotted-notation traditions); not over-indexed to any one language; matches the chain-navigation metaphor honestly. One word, terse, honest.

If intueri pushes back at Slice 4 implementation time (the spell may find a better name when staring at the actual code), revisit; for now `path` is the working name.

#### Q7-lifecycle — ProgramEnv mutability

**Verdict:** **Frozen at spawn.** Once passed to `spawn-program'`, the env is immutable for the program's lifetime. Mutability requires respawn.

Matches arc 119's frozen-world discipline + Arc-everywhere immutability + zero-Mutex architecture. No "update env at runtime" semantic; no "mutable env handle." The env is a value, not a reference.

Four-questions: Obvious YES (immutability is the substrate's default); Simple YES (no mutation API); Honest YES (the env you read at runtime IS the env you spawned with); Good UX YES (no synchronization concerns; threads can read freely).

#### Q9 — Empty env case

**Verdict:** Empty `{}` works directly. No nil sentinel needed.

After arc 215 Stone 2:
- Literal type at construction: `HashMap<fresh_K, fresh_V>` (both type variables fresh)
- spawn-program's param type: `HashMap<:wat::core::keyword, :wat::holon::HolonAST>`
- Call-site HM unification: `fresh_K ↦ :wat::core::keyword`; `fresh_V ↦ :wat::holon::HolonAST`
- Resolved type: matches param ✓

Empty `{}` IS a valid empty env at any call site that expects `<Keyword, HolonAST>`. The arc 215 inference machinery handles type resolution; no sentinel forms needed.

```wat
;; Empty env for a thread that doesn't need config
(:wat::kernel::spawn-program' :thread {} my-program)
```

Four-questions on `{}` vs nil sentinel: `{}` wins YES×4 (Obvious + Simple + Honest + Good UX); nil sentinel loses on all four (introduces hidden semantic, requires conversion logic, lies about being a map). Reject sentinel; canonical empty env is the empty literal.

#### Q10 — spawn-program short-forms

**Verdict:** **Reject.** No `spawn-thread'` / `spawn-process'` sugar verbs. The verbose unified form `(spawn-program' :tier env program)` is canonical.

Rationale: users will rarely call `spawn-program'` directly — they'll go through `:wat::brackets::*` (parallel-each / parallel-map) for compute concurrency or `:wat::services::*` for stateful peers. The raw spawn verb is for substrate authors and edge cases. Verbose IS honest (per `feedback_verbose_is_honest`); one verb per concept matches LLM-first one-canonical-path discipline (per `project_wat_llm_first_design`).

### Implementation surface for Slice 4 stones

With verdicts inscribed, Slice 4's stones become tractable. Likely decomposition (intueri-runs-here for naming at brief time):

1. **Stone 4.1** — Mint `:wat::program::Env` type + Rust-side internal representation (HolonAST::Bundle of binds; arc 057 slice 3 supports HolonAST keys directly).
2. **Stone 4.2** — Mint the six accessor verbs (`/get`, `/expect-get`, `/get-default`, `/dig`, `/expect-dig`, `/dig-default`); wire to arc 146 polymorphic dispatch.
3. **Stone 4.3** — Mint unified `(spawn-program' :tier env program)` verb; dispatch on `:tier` to existing per-tier substrate spawns; ingest env at spawn site.
4. **Stone 4.4** — Mint polymorphic kernel verbs (`send'`, `recv'`, `try-recv'`, `select'`, `close'`); dispatch on peer type; the peer-types-already-exist work from arc 170 Stone C3 makes this mostly wiring.
5. **Stone 4.5** — Integration tests + sandbox-compatibility walker extension (per existing Layer 0c sandbox-compatibility-constraint section).

Each stone is atomic; sonnet ships clean; orchestrator scores per arc 215's calibrated pattern.

### Cross-references

- arc 215 (Stone 1 + Stone 2) — literal-flexibility prerequisite; ProgramEnv construction surface lands cleanly via `{...}` literal
- arc 057 slice 3 — `hashmap_key accepts HolonAST`; substrate-truth for ProgramEnv's surface/internal mapping
- arc 119 — frozen-world discipline; lifecycle precedent
- arc 146 — multimethod dispatch; polymorphic `get` mechanism
- arc 107 — typed-expect pattern; matches expect-get error model
- arc 138 — position-named diagnostics; KeyError flavor
- `project_universe_residency` — programs are universe-resident; tier-config (now env) is part of the universe at spawn time
- `feedback_verbose_is_honest` — sugar verbs rejected; verbose form is canonical
- `project_wat_llm_first_design` — one canonical path per task; six accessor verbs (no synonyms)
- `feedback_inscription_immutable` — this section is a forward-correction; the original Layer 0c content stays

*The literal IS the env. The signature IS the contract. The substrate IS the algebra. Everything composes.*

## Layer 1 — Brackets (wat's Parallel)

```
(:wat::brackets::parallel-each :tier N items (fn [item] ...))   ;; for-each; side effects only
(:wat::brackets::parallel-map :tier N items (fn [item] result)) ;; map; returns Vec<result>

;; Reduce composes — NO sugar primitive
(:wat::core::reduce + (:wat::brackets::parallel-map :thread 8 items job-fn))
```

**Worker functions are tier-agnostic** — they use only `:wat::kernel::*` peer-style verbs; same fn body runs in `:thread` or `:process`; bracket dispatches at construction site.

**Internals** use Layer 0a (`:wat::comms::*` tier primitives) for shared work-channels (work-stealing pattern: one shared `Sender` + N cloned `Receiver`s; bracket sends; workers pull). The work-channels are SUBSTRATE-INTERNAL to the bracket; not exposed to user code.

Both forms exist for both tiers (`:thread` + `:process`) at this arc's close. Future remote/reactor tiers extend mechanically by adding `:tier = :remote` / `:reactor` dispatch arms.

Retires `run-threads` (arc 170 D-stones) — its capability folds into `parallel-map-reduce`-style composition over `parallel-map`.

## Layer 2 — Services (wat's OOP)

Per user 2026-05-18:
> *"i rarely used objects in ruby... maybe like.... 3 classes total per app.. all it held was mutable state no one else could get"*

Services ARE that pattern. arc 203 `ServiceWithProvisioning` rebuilt on the unified peer model:

```clojure
(let [service (:wat::services::start :process my-service-program)]
  (:wat::kernel::send' service (Request/get "key1"))
  (let [response (:wat::kernel::recv' service)]
    ...))
```

A service IS a peer; you spawn it (with the service-program shape); you talk to it via the same kernel verbs. Multi-user dispatch happens INSIDE the service-program. Service users see exactly the same surface as any other peer.

Both thread-tier (in-process services; ~zero overhead) and process-tier (cross-process services; isolation; HolonRepresentable cost) variants. The user picks `:tier` at service-start.

Drops typed_send/typed_recv direct usage. The Value-layer chokepoint subsumed: `SenderInner::Crossbeam(...)` becomes `wat::comms::thread::Sender<Value>`; `SenderInner::PipeFd(...)` becomes `wat::comms::process::Sender<Value>`. Single source of truth (tier wrappers); Value-layer is a thin shim.

## Build approach — fresh files; rename at convergence

Per user 2026-05-18: *"we'll figure the long term names after it works - we need it to work and to have caller flipped over... then we do a mass refactor to use the more correct names -- the names are self evident once they implement something that bears a name"*

**Build NEW files; don't fight existing cruft.** Each slice's BRIEF is small + focused (build this clean file). Existing tests keep passing during build (no churn until migration). Per `feedback_iterative_complexity` + the arc 170 closure-blocking lesson: bundled scope confuses sonnet; we don't repeat that.

**File layout (gazed 2026-05-18):**

```
src/
├── comms/                       ← Layer 0a (Slices 1-3)
│   ├── mod.rs                   ← CommSender + CommReceiver + HolonRepresentable + errors
│   ├── thread.rs                ← thread tier (crossbeam underneath)
│   └── process.rs               ← process tier (io_uring underneath)
├── kernel/                      ← Layers 0b + 0c (Slice 4)
│   ├── mod.rs                   ← entry point
│   ├── peer.rs                  ← Thread<I,O> + Process<I,O> + Remote<I,O> (future)
│   └── spawn.rs                 ← unified spawn-program dispatcher
├── brackets.rs                  ← Layer 1 (Slice 7)
├── services.rs                  ← Layer 2 (Slice 8)
├── ... (existing flat substrate files; retire in Slice 5/6 as callers migrate)
```

**Naming rationale (gazed):**
- `comms` — communications; substrate's concern of "things that talk to each other across concurrency boundaries"; not utils/common/infra
- `kernel` — the wat substrate's privileged operations layer; matches wat-side `:wat::kernel::*` namespace
- `brackets` — wat's Parallel; Lisp/wat-cultural word for "bracket this work with concurrency"
- `services` — Ruby's protected-state-OOP pattern; plural because substrate hosts many

**Acknowledged asymmetries** (gaze-honest):
- Wat namespace `:wat::comms::thread::*` (three levels) ≠ Rust path `crate::comms::thread` (three levels) — symmetric for comms
- Wat namespace `:wat::kernel::*` (two levels) ≠ Rust path `crate::kernel::*` (two levels) — symmetric for kernel
- Rust `crate::comms::thread` vs `std::thread` cognitive collision — resolved per-file via `use ... as ...` aliases when needed

**Migration discipline** (Slices 5 + 6):
- Slice 5 caller-by-caller flips substrate sites to `crate::comms::*` + `crate::kernel::*` paths; old files (`typed_channel.rs`, parts of `runtime.rs` / `thread_io.rs` / `spawn.rs` / `fork.rs`) stay in place during migration
- Slice 6 retires unused old code AND does any final rename/reorganization; structural wall lands the final shape
- Per `feedback_inscription_immutable`: renames are NEW commits, not retroactive edits

## Slice decomposition

Nine slices, sequenced for dependency + per-stone trust gates. Each slice = ONE coherent concern. Stepping stones within each slice designed orchestrator-side; sonnet sees ONE stepping stone per work unit.

### Slice 1 — Foundation primitives (atomic; ~1 stepping stone)

Mint the trait shapes + signatures + error types in `src/comms/mod.rs`. NO implementations.

- `HolonRepresentable` trait + blanket impl (from HolonAST roundtrip)
- `CommSender<T>` / `CommReceiver<T>` traits (tier-agnostic abstraction)
- Error types: `SendError<T>` / `RecvError` / `TryRecvError` / `CloseError`
- `SelectOutcome<T>` enum
- Cascade contract documented (blocking ops MUST wake on substrate shutdown)
- API signatures defined; no impls yet
- Wire up `pub mod comms;` in `src/lib.rs`
- Smoke probe: trait compiles + `impl HolonRepresentable for String` example

### Slice 2 — Thread tier (big; ~3-4 stepping stones likely)

Implement thread tier in `src/comms/thread.rs`. NEW file; doesn't touch existing typed_channel.rs / runtime.rs / thread_io.rs / spawn.rs.

- `Sender<T: Send + 'static>` newtype; private inner
- `Receiver<T: Send + 'static>` newtype with cascade-aware `recv()` via `select! { data, SHUTDOWN_RX }`
- `try_recv()` + `len()` (non-blocking)
- `Select<T>` cascade-aware fan-in
- Factory: `pair<T>()` (capacity-1 mini-TCP; see § "Mini-TCP at depth 1 (universal symmetry)")

**FORWARD-CORRECTED 2026-05-19 (Slice 2 forward-correction stone):** original Slice 2 ship listed two factories (`pair()` unbounded + `bounded(N)` opt-in); both retired into a single `pair()` at capacity 1. Trading-lab convergence + universal symmetry with process tier. See § "Slice 2 forward-correction (2026-05-19) — Mini-TCP at depth 1 (universal symmetry)" at end of this DESIGN for the inscription-archival four-questions block + INTERSTITIAL § "2026-05-19 (post-compaction, Slice 2 forward-correction) — Mini-TCP at depth 1: the trading-lab origin returns".

- Clone impls
- `CommSender<T>` / `CommReceiver<T>` trait impls (from `comms::mod`)
- Smoke probe

### Slice 3 — Process tier (big; ~5-6 stepping stones likely)

Implement process tier in `src/comms/process.rs`. NEW file. io_uring underneath.

- Add `io-uring` crate to Cargo.toml
- Per-receiver io_uring instance setup (long-lived ring per receiver; ring size from config at construction)
- `Sender<T: HolonRepresentable>` with io_uring write submission + EPIPE-cascade
- `Receiver<T: HolonRepresentable>` with io_uring multi-arm read on [data_fd, broadcast_fd]
- `try_recv()` + `len()`
- `Select<T>` with io_uring multi-arm + auto-broadcast_fd registration
- HolonRepresentable serialization (HolonAST → EDN bytes via wat-edn)
- Manual `impl HolonRepresentable` for substrate-internal Rust types: StdInServiceEvent, SpawnOutcome, etc.
- **Config tunable:** `:wat::config::set-process-tier-uring-depth!` (default 512; range [1, 4096]; must be power of 2) **— SUPERSEDED 2026-05-19; rejected by four-questions during Stone E walk; see § "Stone E forward-correction (2026-05-19) — TCO discipline + reflexive rebuild"**
- `CommSender<T>` / `CommReceiver<T>` trait impls
- Smoke probe

### Slice 4 — Kernel layer (big; ~4-5 stepping stones likely)

Mint Layers 0b + 0c in `src/kernel/{mod,peer,spawn}.rs`. NEW files.

**Peer types** (in `src/kernel/peer.rs`):
- `Thread<I, O>` struct holding: input `comms::thread::Sender<I>` + output `comms::thread::Receiver<O>` + join handle for the spawned thread + cascade infrastructure
- `Process<I, O>` struct holding: input `comms::process::Sender<I>` + output `comms::process::Receiver<O>` + child process handle (Pidfd) + cascade infrastructure
- Wat-level type registrations: `:wat::kernel::Thread<I,O>`, `:wat::kernel::Process<I,O>`

**Spawn dispatcher** (in `src/kernel/spawn.rs`):
- `eval_kernel_spawn_program_prime` (handles `:wat::kernel::spawn-program'`); dispatches on `:tier`:
  - `:thread` → calls `crate::comms::thread::spawn_program(program)`
  - `:process` → calls `crate::comms::process::spawn_program(program)`
- Sandbox-walker integration (extends arc 170's sandbox-scope discipline to validate `:process` programs' captures)

**Polymorphic kernel verbs** (revised; primed during dev; in `src/kernel/peer.rs` or substrate dispatch module):
- `:wat::kernel::send'` — multimethod dispatch on peer type
- `:wat::kernel::recv'` — same
- `:wat::kernel::try-recv'` — same
- `:wat::kernel::select'` — same
- `:wat::kernel::close'` — same
- Each verb's Rust implementation: pattern match on the wat Value's variant (Thread / Process / Sender / Receiver / etc.); call the appropriate tier method

**Smoke probes:**
- `:thread` peer round-trip via kernel verbs
- `:process` peer round-trip via kernel verbs
- Cascade-wakes-recv (per tier)
- Sandbox walker rejects non-HolonRepresentable captures for `:process`

### Slice 5 — Migration sweep (big; ~5-7 stepping stones likely)

Caller-by-caller substrate migration. Substrate-as-teacher cascade per file.

- 5a: Migrate `:wat::kernel::send` callers from legacy (Sender arg) to `:wat::kernel::send'` (peer arg); cargo build cascades errors per call site
- 5b: Same for recv, try-recv, select
- 5c: Migrate `:wat::kernel::spawn-thread` / `spawn-process` / `spawn-program` / `fork-program` callers to unified `:wat::kernel::spawn-program' :tier ...`
- 5d: Subsume typed_send/typed_recv — Value-layer becomes thin shim over `comms::*` tier wrappers
- 5e: Migrate `:wat::kernel::Thread<R>` (one-shot join) usages to `:wat::kernel::Thread<nil, R>` (peer-shape); join becomes recv
- 5f: Migrate HandlePool to use `comms::thread::Receiver<T>::len()`
- 5g: Ship δ-1 (arc 213 dirty tree) atomically — by this point cascade-completeness is end-to-end; δ-1's hang vector is eliminated
- 5h: Retire legacy verb registrations; rename primes to canonical (`send'` → `send`; etc.)

### Slice 6 — Structural wall (atomic-ish; ~1-2 stepping stones likely)

Make bare mechanisms unreachable outside tier wrapper modules. Maximum hiding via Rust module privacy.

- Reorganize `src/` to consolidate the new structure (`src/comms/{thread,process}.rs` + `src/kernel/{peer,spawn}.rs`)
- `pub(crate)` discipline: tier internals accessible from inside the wat crate (tests, etc.) but not externally
- External code sees only public `crate::comms::*` + `crate::kernel::*` APIs
- Verify: external test attempting `use crossbeam_channel::Sender;` outside `crate::comms::thread` → compile error
- Same for libc::pipe/read/write/poll/epoll/io_uring outside `crate::comms::process`
- No build.rs scanner (the χ-3 direction was wrong; structural via Rust visibility)

### Slice 7 — Brackets (Layer 1; big; ~4-5 stepping stones likely)

Wat's Parallel in `src/brackets.rs`.

- 7a: `(parallel-each :thread N items fn)` — for-each form; smoke probe
- 7b: `(parallel-each :process N items fn)` — process tier variant; smoke probe
- 7c: `(parallel-map :thread N items fn)` — map form, returns Vec; smoke probe
- 7d: `(parallel-map :process N items fn)` — process tier variant; smoke probe
- 7e: Retire `run-threads` (arc 170 D-stones); migrate callers to `(reduce + (parallel-map ...))` composition

Worker bodies are tier-agnostic — use `:wat::kernel::*` polymorphic verbs only. Reduce composes from map (no sugar primitive).

### Slice 8 — Services (Layer 2; big; ~3-4 stepping stones likely)

ServiceWithProvisioning rebuilt in `src/services.rs`.

- 8a: Rebuild service Rust internals on `comms::*` tier wrappers (drop typed_send/typed_recv direct calls)
- 8b: Process-tier service variant on `comms::process::*`
- 8c: Tier-agnostic service worker body — service body uses `:wat::kernel::*` polymorphic verbs
- 8d: Migrate existing arc 203 consumers to the rebuilt service shape; smoke probes

### Slice 9 — INSCRIPTION (atomic; ~1 stepping stone)

Closure paperwork.

- INSCRIPTION.md
- 058 changelog row
- USER-GUIDE section (peer model + brackets + services + tier wrappers + prime convention history)
- Cross-references: arc 213 (cascade chokepoint precursor) + arc 198 (restriction discipline) + arc 203 (struct-restricted OOP) + arc 212 (newtype wall pattern) + arc 170 (run-threads retired into bracket-map composition)
- MEMORY entries for the doctrines this arc adds

## Slice dependency graph

```
Slice 1 (foundation traits)
   ├── Slice 2 (thread tier)  ─┐
   └── Slice 3 (process tier) ─┴── Slice 4 (kernel layer — peer types + verbs + spawn)
                                       └── Slice 5 (migration sweep)
                                              └── Slice 6 (structural wall)
                                                     ├── Slice 7 (brackets) ──┐
                                                     └── Slice 8 (services) ──┴── Slice 9 (INSCRIPTION)
```

**Slice 7 BEFORE Slice 8** per user 2026-05-18 ("brackets first, services second"). Per-stone trust gate between every transition.

## Per-stone trust gate discipline

Per `feedback_iterative_complexity` + the load-bearing lesson from arc 170 closure-blocking ("sonnet getting confused or doing too much work sets us back days to hours; we've been trying to close 170 for over a week"):

**Each stepping stone within a slice is ONE coherent concern; sonnet sees only that concern; orchestrator verifies SCORE before next stepping stone spawns.** No bundled scope; no "while you're there"; no scope-creep. Slow is smooth, smooth is fast.

Stepping stones designed orchestrator-side at slice-open time; not pre-decomposed at arc-DESIGN level.

## What this arc supersedes

- **arc 213 χ stones** (chokepoint completion via wrapper) — folds into Slice 2 as precursor; χ-1 + χ-2 are stepping stones the new arc builds on
- **arc 213 χ-3** (build.rs scanner direction) — historically inscribed at commit `40f9b95` but SUPERSEDED. Slice 6 structural wall via Rust module privacy replaces it.
- **arc 213 δ-1** (ChildHandleInner pidfd field) — dirty tree preserved; ships in Slice 5g atomically with cascade-completeness proof
- **arc 213 δ-2/3 + ε + ζ + η** (libc::fork closure) — continue in arc 213 separately
- **arc 170 D-stones** (run-threads bracket macro) — folds into Slice 7 as precursor
- **arc 203 ServiceWithProvisioning** — folds into Slice 8; rebuilt on peer model
- **typed_send / typed_recv** (Value-layer chokepoint) — subsumed into tier wrappers in Slice 5d
- **`:wat::kernel::spawn-thread` / `spawn-process` / `spawn-program` / `fork-program`** — all collapse into unified `:wat::kernel::spawn-program'` in Slice 5c
- **`:wat::kernel::send` / `recv` / `try-recv` / `select`** — semantics revised from channel-endpoint-oriented to peer-oriented; primes during dev, rename to canonical after migration

## What this arc explicitly does NOT do

- **Remote tier** — empty seat; designed in this DESIGN; minted when we know what remote IS (future arc)
- **Reactor tier** — empty seat; designed; minted when substrate adopts userspace async runtime (multi-arc architectural pivot; not bundled here)
- **Sync vs async substrate decision** — substrate stays threads-as-tasks; tier wrappers support both models; reactor tier addition is the trigger for async runtime conversation
- **HTTP / network / TLS / async crates** — separate concerns per DEPENDENCY-DOCTRINE; future arcs

## Discipline invariants (load-bearing for sonnet briefs)

These doctrines apply at every slice + every stepping stone:

- `feedback_options_are_tangle` — ONE canonical mechanism per concern
- `feedback_simple_is_uniform_composition` — N identical mechanical edits IS simple
- `feedback_iterative_complexity` — STOP when hitting deadlocks; build small funcs
- `feedback_no_hang_vector_in_additive_scorecard` — additive-mint stepping stones get cargo-build-clean as verification
- `feedback_defect_fix_or_panic_never_revert` — active replications stay on dirty tree
- `feedback_substrate_owns_not_callers_match` — substrate owns N-site identical setup
- `feedback_never_deadlock` — every comm site lands deliberately
- `feedback_brief_constraint_contradictions` — BRIEFs MUST NOT have contradictions
- `feedback_brief_no_easy_auth` — name ONE required path
- `feedback_sync_async_distinction_is_crude` — structured concurrency disciplines transcend implementation mechanic

## Personal stake — what this arc means

Per user 2026-05-18:
> *"we've built all of my toolkit on rust -- this is my response to 'just learn rust' -- i just did - i learned rust so well i made it feel like ruby and it reads like clojure"*

This arc is the proof. Ruby's OOP discipline (services as protected mutable state) + Clojure's read-ability + Rust's performance + structured concurrency by construction = wat. The synthesis lands when this arc closes; the user never deals with this domain again.

The peer-oriented model is what Ruby's actor pattern aspires to and what Erlang has had for 35 years. wat ships it on Rust foundations with structural enforcement Ruby/Erlang can't guarantee.

## Cross-references

### Foundation references
- `scratch/DEPENDENCY-DOCTRINE.md` — authorizes the io-uring crate dep
- `wat-rs/docs/ZERO-MUTEX.md` — composes with cascade-by-construction
- `wat-rs/docs/CONVENTIONS.md` — wat naming conventions
- `wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md` — § 7 sonnet delegation protocol

### Doctrine precedents
- arc 057+ `project_holon_universal_ast` — HolonAST as universal substrate form (the wire trait)
- arc 146 — multimethod dispatch (Slice 4 polymorphic kernel verbs)
- arc 198 — `#[restricted_to(...)]` wat-level access control (sibling pattern)
- arc 203 — struct-restricted OOP (services pattern this arc rebuilds)
- arc 212 — `WatAST::children()` newtype wall (parallel structural-impossibility pattern)
- arc 213 — cascade chokepoint precursor (χ-1 + χ-2 stepping stones)

### Linux primitive doctrine
- INTERSTITIAL § 2026-05-18 (post-PURGE) "Linux 5.3+ syscall doctrine" — pidfd / clone3 / waitid(P_PIDFD); io_uring extends this discipline

### Existing substrate APIs that this arc revises
- `:wat::kernel::send` / `recv` / `try-recv` / `select` — currently channel-endpoint-oriented; revised to peer-oriented (primed during dev)
- `:wat::kernel::spawn-thread` / `spawn-process` / `spawn-program` / `fork-program` — collapsed into `:wat::kernel::spawn-program' :tier ...`
- `:wat::kernel::Thread<R>` — extended to `Thread<I,O>` peer-shape
- `:wat::kernel::Process<I,O>` — already peer-shape; semantics revised under unified spawn

### User direction (load-bearing for this arc)
- *"slow is smooth, smooth is fast"*
- *"we do it perfect now and build on top of them forever"*
- *"we get all the greatness of Ruby's OOP, FP and concurrency"*
- *"hide all the guts - don't let users make mistakes"*
- *"deadlocks are illegal"*
- *"options are why we are in a tangled mess"*
- *"a thread, a process, (a remote ...) need to communicate via the kernel"*
- *"users are not allowed to call (spawn-{thread,process} ...) .. they only get (spawn-program :tier ...)"*
- *"threads and processes should be identical in surface area"*
- *"we must be empowered to steal names from prior callers ... if the correct name exists... we just make a prime of it"*

---

**Arc OPENED 2026-05-18; DESIGN revised 2026-05-19.** Slice 1 (foundation primitives) is the first stepping stone; orchestrator drafts BRIEF + EXPECTATIONS at slice-open time per per-stone trust gate discipline.

### Slice 2 forward-correction (2026-05-19) — Mini-TCP at depth 1 (universal symmetry)

Slice 4 prep surfaced that thread tier shipped two factories (`pair()` unbounded + `bounded(n)` opt-in) when the substrate's universal discipline is mini-TCP at depth 1. Grep evidence: 22 of 22 honest substrate callers use `bounded(1)` only; `comms::thread::bounded` had zero downstream callers; `comms::thread::pair` had zero downstream callers. Pure surface clean-up. Narrative inscribed in INTERSTITIAL § "2026-05-19 (post-compaction, Slice 2 forward-correction) — Mini-TCP at depth 1: the trading-lab origin returns" — this section is the architectural reference.

**Four-questions verdict — `pub fn bounded<T>(n: usize)`:** FAILS YES YES YES YES.

- Obvious? NO — asymmetric with process tier (one factory; kernel-bounded pipes)
- Simple? NO — substrate-author can pick wrong N; pick carries semantic weight
- Honest? NO — knob the substrate's own practice proved harmful (22/22 honest callers use `bounded(1)`; n vestigial)
- Good UX? NO — `bounded(64)` silently breaks mini-TCP; no structural guard

Retired.

**Four-questions verdict — `pub fn pair<T>()` at `crossbeam_channel::bounded(1)`:** YES YES YES YES.

- Obvious? YES — symmetric with process tier; one factory per tier
- Simple? YES — N identical call sites; no choice
- Honest? YES — capacity-1 IS the mini-TCP discipline structurally enforced
- Good UX? YES — substrate-author cannot pick wrong depth; lock-step by construction

pair() flipped from unbounded to bounded(1).

**Universal symmetry (post-correction):**

| Tier | Transport | Backpressure mechanism |
|---|---|---|
| Thread | crossbeam capacity-1 | send blocks when 1 value queued |
| Process | OS pipe (kernel-bounded) | send blocks when `PIPE_BUF` fills (default 64KiB on Linux) |
| Remote (future) | TBD; same shape | TBD; backpressure via network primitive |

Units differ (frame-count vs bytes); shape is identical. Programs at any tier write `send(v)` / `recv()` and the substrate handles the transport. Composes with universe-residency principle + Convergence #13 (autoscaling of correctness): substrate manages all transport details invisibly; user picks hosting env at the outside; one supported communication pattern; entire categories of problems eliminated structurally.

**Mechanism vs discipline:** depth-1 is the MECHANISM (send blocks at 1; recv drains). Mini-TCP is the usage DISCIPLINE (per `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" line 252+) — each send pairs with a recv before the next send. Substrate doesn't enforce the pairing site-by-site; multiple producers can saturate at the same depth. But capacity-1 makes producers that try to outpace consumers block immediately rather than queuing up. The lock-step breathes with load.

**Trading-lab origin (pre-wat-rs lineage):** the depth-1 convergence predates wat-rs. The user built mailboxes (and their opposite) in `holon-lab-trading` and found N > 1 produced massive perf hits + entire categories of problems. Thread tier now matches what process tier always had (kernel-bounded pipes) + what every load-bearing pattern in this substrate ships (arc 119 ack-tx, defservice Request/Reply, Counter actor, dispatch loops).

**Three "shockingly stable" foundation pivots tallied in arc 214:** (1) Stone E tunable rejection (§ "Stone E forward-correction"); (2) bounded() process-tier rejection (§ "Universe-residency + Mini-TCP at depth 1"); (3) this — bounded(N) thread-tier rejection. Same discipline, three forms: substrate manages what substrate manages; wrong choice does not exist.

**Cross-references:**
- `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" (line 252-415) — substrate-wide articulation
- arc 119 `HologramCacheService Put ack-tx` — in-substrate naming
- INTERSTITIAL § "2026-05-19 (post-compaction, Slice 2 forward-correction) — Mini-TCP at depth 1: the trading-lab origin returns" — narrative
- INTERSTITIAL § "2026-05-16 (deeper) — Control channels: Shutdown/Final convention" — Counter actor at mini-TCP depth 1
- INTERSTITIAL § "2026-05-19 — Universe-residency principle + bounded() four-questions verdict" — principle this discipline operationalizes
- INTERSTITIAL § "2026-05-19 — Convergence #13" — sibling discipline at the resource layer
- `feedback_options_are_tangle`, `feedback_refuse_easy_solutions`, `feedback_attack_foundation_cracks`, `feedback_any_defect_catastrophic`, `feedback_inscription_immutable`
