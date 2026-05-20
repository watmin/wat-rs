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
│   :wat::comms::thread::{Sender<T>, Receiver<T>, Select, pair, bounded}   │
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
| `:wat::comms::thread::*` | crossbeam | T: Send + 'static | `pair`, `bounded` |
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

pub fn pair<T>() -> (Sender<T>, Receiver<T>);
pub fn bounded<T>(n: usize) -> (Sender<T>, Receiver<T>);  // thread tier ONLY — see § "Universe-residency + bounded() asymmetry" below

// process tier — IDENTICAL method surface; T bound differs; io_uring underneath
pub struct Sender<T: HolonRepresentable> { /* private inner: io_uring + fd */ }
// ... (same method signatures as thread tier)
// NOTE: pair() returns std::io::Result<...> on process tier (libc::pipe(2) can fail);
// NOTE: bounded() factory NOT minted on process tier — kernel manages pipe-bound; see below
```

### Universe-residency + bounded() asymmetry (2026-05-19 architectural clarification)

User direction 2026-05-19: *"what wat wants is 'i want to run this program in a {thread,process} and it just works.. i can comm to it by sending data and getting data — i don't care where its hosted' / the user must choose a hosting env but the programs never know what env their in — they exist in a universe and that universe has provided a comm channel to use."*

**The discipline:** programs are universe-resident; the universe provides comm channels; the program never knows its transport. Hosting env chosen at the OUTSIDE; program inside writes `peer.send(v)` / `peer.recv()` and runs identically across tiers.

**Two-layer honesty:**

| Layer | Surface | Identical-across-tiers requirement |
|---|---|---|
| **Program-facing** (what the program sees) | Trait `CommSender<T>` + `CommReceiver<T>` + (future Slice 4) peer types `Thread<I,O>` / `Process<I,O>` / `Remote<I,O>` | **MANDATORY identical** — program code does not vary by tier |
| **Substrate-internal** (what the hosting env wires up) | Concrete `thread::Sender` / `process::Sender` etc. | Asymmetries permitted when STRUCTURALLY honest |

**Three substrate-internal asymmetries — each verified honest:**

1. **T bound:** `T: Send + 'static` (thread) vs `T: HolonRepresentable` (process). Transport requirements differ unavoidably; honest.

2. **`pair()` return type:** infallible (thread) vs `std::io::Result<...>` (process). `libc::pipe(2)` can fail; failure mode IS exposed; honest.

3. **`bounded()` factory:** present on thread tier; ABSENT on process tier (post Stone E-2 four-questions verdict).

**Four-questions on bounded() for process tier (verdict 2026-05-19):**

- **(A) F_SETPIPE_SZ wrapper:** FAILS Obvious + Honest (frame-count vs byte-count semantic gap)
- **(B) wat-level semaphore:** FAILS Simple (ZERO-MUTEX violation)
- **(C) DON'T MINT; kernel manages bound (PIPE_BUF; F_SETPIPE_SZ if needed substrate-internal):** YES YES YES YES

Same shape as Stone E forward-correction's "no tunable" verdict on io_uring depth. Kernel manages what kernel manages; substrate doesn't expose what's already structural. Thread tier KEEPS `bounded()` because crossbeam exposes it as a first-class transport-mode choice (different code path than unbounded); process tier never gets one because pipes don't have an equivalent mode toggle the wat layer should expose.

**Convergence:** the universe-residency principle composes with Convergence #13 (autoscaling of correctness):
- Universe-residency (program/user layer): "programs don't know transport"
- Autoscaling-of-correctness (substrate/resource layer): "substrate manages resources reflexively; users don't pick"

Both compose into: users declare hosting env; nothing else. Programs run identically across thread/process/remote; substrate handles all the resource management invisibly. The discipline propagates up via Slice 4 (peer types absorb the substrate-internal asymmetries) + Slice 7 (brackets compose peers) + Slice 8 (services as universe-resident actors).

Cross-references: memory `project_universe_residency.md`; memory `project_autoscaling_correctness.md`; INTERSTITIAL § "2026-05-19 — Universe-residency principle + bounded() four-questions verdict".

### Shared traits (in `src/comms/mod.rs`)

```rust
pub trait CommSender<T> {
    fn send(&self, t: T) -> Result<(), SendError<T>>;
    fn close(self) -> Result<(), CloseError>;
}
pub trait CommReceiver<T> {
    fn recv(&self) -> Result<T, RecvError>;
    fn try_recv(&self) -> Result<T, TryRecvError>;
    fn len(&self) -> usize;
    fn close(self) -> Result<(), CloseError>;
}

pub trait HolonRepresentable: Send + 'static {
    fn to_holon_ast(&self) -> HolonAST;
    fn from_holon_ast(ast: &HolonAST) -> Result<Self, WireError> where Self: Sized;
}

// Blanket impl — anything roundtrippable via HolonAST IS HolonRepresentable
impl<T> HolonRepresentable for T 
where T: Into<HolonAST> + TryFrom<HolonAST, Error = WireError> + Send + 'static {}

pub enum SelectOutcome<T> {
    Recv(usize, Result<T, RecvError>),
    Shutdown,
}

// Error types: SendError<T>, RecvError, TryRecvError, CloseError
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
- Factories: `pair<T>()`, `bounded<T>(n)`
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
