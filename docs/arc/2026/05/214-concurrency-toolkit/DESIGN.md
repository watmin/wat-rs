# Arc 214 — Concurrency toolkit (foundations + brackets + services)

**Status:** OPEN 2026-05-18. Foundational arc. Ships the complete wat concurrency toolkit; structurally impossible to fuck up; deadlocks illegal forever.

## Mission

Exit this arc with **wat's complete concurrency story** — foundational tier primitives + bracket-form parallel processing + service-form protected mutable data — all structurally enforced; one canonical path per concern; no options at any layer; users cannot make mistakes because the type system + module privacy + cascade-by-construction make wrong shape impossible to express.

**This is ONE arc, not three.** Per user direction 2026-05-18:

> *"we do it perfect now and build on top of them forever"*
> *"we exit this arc with all of our concurrency tools. we have proper OOP, proper concurrent or parallel processing (each, map) -- reduce is just a consumer on map - no sugar"*
> *"we get all the greatness of Ruby's OOP, FP and concurrency"*

Three separate arcs would have three separate close conditions, three INSCRIPTIONs, three opportunities to ship in a half-correct state where consumers (brackets, services) layer on foundations that aren't yet sealed. One arc, all layers, per-stone trust gates between slices, atomic discipline.

## The four-tier architecture

| Tier | Concurrency model | Polling mechanism | Wire form | This arc ships |
|---|---|---|---|---|
| **`thread::`** | Threads-as-tasks; in-process | crossbeam `select!` (park-list) | T: Send + 'static | ✅ |
| **`process::`** | Threads-as-tasks; cross-process via FDs | **io_uring** (cascade-aware multi-arm) | T: HolonRepresentable | ✅ |
| **`remote::`** | Cross-machine | TBD (likely io_uring on sockets) | T: HolonRepresentable | empty seat (designed; not minted) |
| **`reactor::`** | Async tasks; userspace runtime | io_uring (runtime exploits batching) | T: HolonRepresentable | empty seat (designed; not minted) |

**The substrate is async-shaped already** (mini-TCP discipline + cascade-aware select + callers electing-to-block-on-round-trips IS the discipline of structured concurrency). io_uring fits this shape natively; future reactor tier adoption is gated only on userspace runtime choice, not on substrate primitive migration.

## HolonRepresentable — the universal wire form

The strange-loop closes (per `project_holon_universal_ast` — HolonAST was minted for VSA encoding arc 057, became universal AST via reflection arcs 143/201, is NOW the universal comms wire form):

```rust
pub trait HolonRepresentable: Send + 'static {
    fn to_holon_ast(&self) -> HolonAST;
    fn from_holon_ast(ast: &HolonAST) -> Result<Self, WireError> where Self: Sized;
}

// Blanket impl — anything convertible both directions IS HolonRepresentable
impl<T> HolonRepresentable for T 
where T: Into<HolonAST> + TryFrom<HolonAST, Error = WireError> + Send + 'static {}
```

One substrate "Any" form. Four uses:

| Use | Carries |
|---|---|
| VSA encoding (arc 057+) | HolonAST → high-dim vector |
| Signature reflection (arc 143) | HolonAST describes function signatures |
| Type reflection (arc 201) | HolonAST describes type structure |
| **Comms wire form (this arc)** | **HolonAST travels the transport; recipient re-hydrates** |

Existing `typed_send` / `typed_recv` already does Value → EDN → pipe → EDN → Value at the Value layer. χ generalizes this to ANY T satisfying `HolonRepresentable`. No new abstraction invented; existing universal form connected to the comms surface.

## API surface (identical across tiers)

```rust
// Same methods, same return types, same error shapes, same cascade semantics
impl<T: ...> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>>;
    pub fn close(self) -> Result<(), CloseError>;
}

impl<T: ...> Receiver<T> {
    pub fn recv(&self) -> Result<T, RecvError>;        // cascade-aware (blocks)
    pub fn try_recv(&self) -> Result<T, TryRecvError>; // non-blocking
    pub fn len(&self) -> usize;                         // non-blocking peek
    pub fn close(self) -> Result<(), CloseError>;
}

impl<T: ...> Clone for Sender<T> { /* multi-producer */ }
impl<T: ...> Clone for Receiver<T> { /* multi-consumer */ }

pub struct Select<'a, T: ...> { /* auto-registers SHUTDOWN_RX / broadcast_fd */ }

pub enum SelectOutcome<T> {
    Recv(usize, Result<T, RecvError>),
    Shutdown,  // cascade fired; caller unwinds
}
```

**Different T-bound per tier** (per Fork 1 four-questions verdict):
- `thread::Sender<T: Send + 'static>` — no Wire bound needed (crossbeam passes T directly)
- `process::Sender<T: HolonRepresentable>` — needs serialization for pipe transport

**Common trait `CommSender<T>` / `CommReceiver<T>`** implemented by both tier types — enables tier-agnostic generic functions (bracket workers; service workers; any code that should run regardless of which tier hosts it):

```rust
pub trait CommSender<T> {
    fn send(&self, t: T) -> Result<(), SendError<T>>;
    fn close(self) -> Result<(), CloseError>;
}
pub trait CommReceiver<T> { /* recv, try_recv, len, close */ }

impl<T: Send + 'static> CommSender<T> for thread::Sender<T> { ... }
impl<T: HolonRepresentable> CommSender<T> for process::Sender<T> { ... }

fn worker<S: CommSender<MyResult>, R: CommReceiver<MyTask>>(tx: S, rx: R) {
    // Tier-agnostic body
    while let Ok(task) = rx.recv() {
        tx.send(do_work(task)).ok();
    }
}
```

## Cascade-by-construction discipline

EVERY blocking method on Sender/Receiver/Select includes the cascade auto-wired:
- **Thread Receiver::recv()** → `crossbeam_channel::select! { recv(data), recv(SHUTDOWN_RX) }`
- **Process Receiver::recv()** → io_uring multi-arm submission on [data_fd, broadcast_fd]; first completion wakes
- **Thread Select / Process Select** → auto-registers cascade source as the first arm

Worker code cannot bypass the cascade. The wrapper IS the cascade. Deadlocks are illegal because no path through the public API can leave a recv parked when shutdown fires.

## Layer 1 — Brackets (wat's Parallel)

Brackets is wat's `Parallel`-equivalent. Two primitives; reduce composes from map (no sugar):

```
(parallel-each :tier N items (fn [item] ...))        ;; side-effect for-each
(parallel-map  :tier N items (fn [item] result))     ;; returns Vec<result>
(reduce + (parallel-map :thread 8 items job-fn))     ;; reduce composes
```

**Worker bodies are tier-agnostic** — use only `:wat::comm::*` polymorphic operation verbs; same function body runs in `:thread` or `:process` tier; bracket dispatches at construction site:

```clojure
(parallel-map :thread 8 items
  (fn [item]
    ;; This body uses :wat::comm::send / recv / etc. — works regardless of tier
    (heavy-compute item)))

;; Same fn body; different hosting environment
(parallel-map :process 4 items
  (fn [item]
    (heavy-compute item)))
```

**Both forms exist for both tiers** (`:thread` + `:process`) at this arc's close. Future remote/reactor tiers extend mechanically.

Retires `run-threads` (arc 170 D-stones) — its capability folds into `parallel-map-reduce` style composition over `parallel-map`.

## Layer 2 — Services (wat's OOP)

Per user 2026-05-18:
> *"i rarely used objects in ruby... maybe like.... 3 classes total per app.. all it held was mutable state no one else could get"*

Services ARE that pattern. arc 203 `ServiceWithProvisioning` rebuilt on tier wrappers:
- Service holds mutable state nobody else can touch
- Clients communicate via tier-wrapped channels
- Multi-user dispatch routes through the wrapper
- Service worker body is tier-agnostic (uses `:wat::comm::*` polymorphic verbs)
- Both thread-tier (in-process services; ~zero overhead) and process-tier (cross-process services; isolation; HolonRepresentable cost) variants

Drops typed_send/typed_recv direct usage. The Value-layer chokepoint subsumed: `SenderInner::Crossbeam(...)` becomes `wat::thread::Sender<Value>`; `SenderInner::PipeFd(...)` becomes `wat::process::Sender<Value>`. Single source of truth (tier wrappers); Value-layer is a thin shim.

## The structural wall — "users cannot fuck up because we don't let you"

Per user 2026-05-18:
> *"hide all the guts - don't let users make mistakes .. we need whatever exposure for us to test ourselves - but users cannot be given the option to fuck up - deadlocks are illegal"*

**Mechanism: maximum Rust module privacy.**
- `crossbeam_channel::*` accessible ONLY from inside `src/thread/` module (or wherever the thread tier lives) — `pub(crate)` discipline + module hierarchy
- `libc::pipe / read / write / poll / epoll_* / io_uring_*` accessible ONLY from inside `src/process/` module
- All other substrate code uses `wat::thread::*` / `wat::process::*` — bare-mechanism unreachable by construction (Rust visibility rules)
- Tests inside the substrate get full exposure via `pub(crate)` — we can test our own internals
- External code (tests, examples, downstream crates) sees ONLY the wrapper public API — can't bypass

**No build.rs scanner** (the χ-3 direction was wrong). The scanner approach was whitelist-shaped; the structural mechanism (private fields + private modules + Rust visibility) is the type-system enforcement. Scanners check from outside; the type system makes wrong shape impossible to type.

**The wall extends to libc::poll** — `poll` is BANNED. We use io_uring always. Per the "one canonical path" doctrine (`project_wat_llm_first_design`): the substrate has ONE FD-watching mechanism. Forever. epoll never enters the substrate; io_uring is the canonical choice.

## Dependencies

Per `scratch/DEPENDENCY-DOCTRINE.md` — we couple deeply when the dep is canonical, battle-tested, widely-used.

**New dep accepted:** `io-uring` crate
- Used by canonical projects: tokio-uring, glommio (Datadog), monoio
- Active maintenance (rio team; tokio-adjacent)
- Focused scope (~10k LOC; just io_uring syscall wrappers)
- Four-questions: Obvious YES (name says it) / Simple YES (focused crate) / Honest YES (well-documented) / Good UX YES (integrates with any FD-based Rust)
- wat-specific tests: zero-mutex preserving (just syscall wrappers); type-safe; canonical use

**Existing deps preserved:**
- crossbeam_channel — thread tier's underlying mechanism (per existing DEPENDENCY-DOCTRINE precedent)
- wat-edn — HolonRepresentable serialization

## Tunables — substrate config exposed via `:wat::config::set-*!`

Per user 2026-05-18: *"i think we use 512 as our internal queue depth.... we need to have it declared via a :wat::config::*-set! so users who know better can tweak it for their programs"*

**Process tier io_uring SQ/CQ depth** — the size of each `wat::process::Receiver` / `Select`'s submission + completion ring buffers.

- **Setter:** `:wat::config::set-process-tier-uring-depth!` (per `set-*!` family naming convention; matches arc 014 + arc 157 patterns)
- **Default:** 512
  - Power of 2 (io_uring requirement)
  - Midpoint between tokio-uring (256) and monoio (1024)
  - Enough for substantial batching of EDN-large messages
  - Small enough that spawned processes don't bloat memory unnecessarily
- **Validation:** must be a power of 2 in `[1, 4096]`. Out of range or non-power-of-2 → `RuntimeError` at the setter call site (per existing `:wat::config::set-*!` error discipline).
- **Cap rationale:** >4096 begins hitting kernel ring memory limits + diminishing returns; the substrate caps at a sane upper bound.

**Per-runtime semantics** (matches existing `set-*!` family per FM 7-ter):
- Atomic config value owned by substrate
- Read at `wat::process::{Receiver, Select}` CONSTRUCTION time
- Each new instance gets a ring sized at current config value
- Changing the config mid-runtime does NOT affect already-constructed rings
- Typically called once at program startup, before any process-tier channels exist

**Why this is parameter-tunability, not option-tangle** (per `feedback_options_are_tangle`):
- ONE mechanism (io_uring; canonical; not optional)
- ONE setter for the parameter (canonical)
- Tweakability is SEPARATE from "which mechanism" — power users tune the parameter; the substrate's chokepoint discipline is unchanged

**Future tunables** (NOT shipped in this arc; explicitly scoped out):
- SQPOLL mode toggle (kernel polling thread; high-perf substrate variant)
- SQ_THREAD_IDLE timeout
- Registered buffer pool size
- Linked operations enable/disable

These are progressive-disclosure tunables — added when a concrete substrate use case justifies. For arc 214 close, only `set-process-tier-uring-depth!` ships. Future arcs add more as needed.

**Naming follow-on:** if the thread tier ever surfaces tunables (channel queue depth for bounded channels?), it gets parallel naming: `:wat::config::set-thread-tier-*!`. The tier-prefix convention is the substrate-coherence shape.

## Slice decomposition

Nine slices, sequenced for dependency + per-stone trust gates. Each slice = ONE coherent concern. Stepping stones within each slice designed orchestrator-side; sonnet sees one stepping stone per work unit (per `feedback_iterative_complexity` + per-stone trust gate discipline — arc 170 has been trying to close for >1 week because sonnet got confused on bundled scope; we don't repeat that here).

### Slice 1 — Foundation primitives (atomic; ~1 stepping stone)

Mint the trait shapes + signatures + error types. NO implementations.

- `HolonRepresentable` trait + blanket impl
- `CommSender<T>` / `CommReceiver<T>` traits (tier-agnostic abstraction)
- Error types: `SendError<T>` / `RecvError` / `TryRecvError` / `CloseError`
- `SelectOutcome<T>` enum
- Cascade contract documented (blocking ops MUST wake on substrate shutdown)
- API signatures defined; no impls yet
- Smoke probe: trait compiles + a smoke `impl HolonRepresentable for String` example

### Slice 2 — Thread tier (big; ~3-4 stepping stones likely)

Implement `wat::thread::*` family using crossbeam underneath.

- `Sender<T>` newtype wrapping `crossbeam_channel::Sender<T>`; private inner
- `Receiver<T>` newtype with cascade-aware `recv()` via `select! { data, SHUTDOWN_RX }`
- `try_recv()` + `len()` (non-blocking)
- `Select<T>` cascade-aware fan-in
- Factories: `pair<T>()`, `bounded<T>(n)`
- Clone impls
- `CommSender<T>` / `CommReceiver<T>` trait impls
- Smoke probe: round-trip + sender-drop-Err + try_recv-empty + cascade-wakes-recv + Select fan-in

### Slice 3 — Process tier (big; ~5-6 stepping stones likely)

Implement `wat::process::*` family using io_uring underneath. Largest slice (new dep, new substrate mechanism, HolonRepresentable serialization, config tunable).

- Add `io-uring` crate to Cargo.toml
- Per-tier io_uring instance setup (per-receiver ring; long-lived; epoll_create-style at construction; ring size read from config at construction time)
- `Sender<T: HolonRepresentable>` with io_uring write submission + EPIPE-cascade
- `Receiver<T: HolonRepresentable>` with io_uring multi-arm read on [data_fd, broadcast_fd]
- `try_recv()` + `len()`
- `Select<T>` with io_uring multi-arm + auto-broadcast_fd registration
- HolonRepresentable serialization (HolonAST → EDN bytes via wat-edn)
- Manual `impl HolonRepresentable` for substrate-internal Rust types: StdInServiceEvent, SpawnOutcome, etc.
- **Config tunable:** `:wat::config::set-process-tier-uring-depth!` (default 512; range [1, 4096]; must be power of 2). Atomic config storage; read at receiver/select construction; per-runtime semantics matching existing `set-*!` family.
- Smoke probe: round-trip + sender-drop-Err + try_recv-empty + cascade-wakes-recv + Select fan-in + config-setter validation (rejects non-power-of-2, out-of-range)

### Slice 4 — Wat-level surface (big; ~3-4 stepping stones likely)

Expose tier wrappers to wat programs via multimethod dispatch (per arc 146 pattern).

- Tier-specific construction verbs: `:wat::thread::pair / bounded` + `:wat::process::pair / bounded`
- Polymorphic operation verbs (multimethod dispatch on Sender/Receiver variant):
  - `:wat::comm::send tx value`
  - `:wat::comm::recv rx`
  - `:wat::comm::try-recv rx`
  - `:wat::comm::select rxs`
  - `:wat::comm::close handle`
  - `:wat::comm::len rx`
- Wat-level type registrations: `:wat::thread::Sender<T>` / `:wat::thread::Receiver<T>` / `:wat::process::Sender<T>` / `:wat::process::Receiver<T>`
- Wat-side proof: worker function uses only `:wat::comm::*` operations; works against both tiers (wat-test fixture)

### Slice 5 — Migration sweep (big; ~4-6 stepping stones likely)

Migrate all bare-mechanism sites in substrate to the new tier wrappers. Substrate-as-teacher cascade per file.

Stepping stones likely:
- 5a: Migrate remaining bare-crossbeam caller sites in substrate (the ones χ-2 left as stepping stones — HandlePool len, value-typed bridges if they refactor cleanly)
- 5b: Migrate bare-libc::pipe/read/write/poll/epoll sites in substrate to `wat::process::*` (replaces existing poll-based PipeFd reader/writer with io_uring-based)
- 5c: Subsume typed_send/typed_recv — Value-layer becomes thin shim over tier wrappers
- 5d: Migrate HandlePool to use `wat::thread::Receiver<T>::len()`
- 5e: Migrate `:wat::kernel::select` to `:wat::comm::select` (or alias retiring the old)
- 5f: Ship δ-1 (the dirty tree from arc 213) atomically — by this point the cascade-completeness is achieved end-to-end; δ-1's hang vector is eliminated

### Slice 6 — Structural wall (atomic-ish; ~1-2 stepping stones likely)

Make bare mechanisms unreachable outside their wrapper modules. Maximum hiding via Rust module privacy.

- Reorganize `src/` to put thread tier code in `src/thread/` module; process tier in `src/process/`; private submodules for the bare mechanism wrappers
- `pub(crate)` discipline: tier internals accessible from inside the wat crate (tests, etc.) but not externally
- External code sees only the public `wat::thread::*` and `wat::process::*` API
- Verify: external test attempting `use crossbeam_channel::Sender;` outside thread tier → compile error (unresolved import via module privacy)

### Slice 7 — Brackets (Layer 1; big; ~4-5 stepping stones likely)

Wat's Parallel.

Stepping stones likely:
- 7a: `(parallel-each :thread N items fn)` — for-each form, no return; smoke probe
- 7b: `(parallel-each :process N items fn)` — process tier variant; smoke probe
- 7c: `(parallel-map :thread N items fn)` — map form, returns Vec; smoke probe
- 7d: `(parallel-map :process N items fn)` — process tier variant; smoke probe
- 7e: Retire `run-threads` (arc 170 D-stones) — callers migrate to `(reduce + (parallel-map ...))` style

### Slice 8 — Services (Layer 2; big; ~3-4 stepping stones likely)

ServiceWithProvisioning rebuilt on tier wrappers (per user 2026-05-18: brackets first, services second).

Stepping stones likely:
- 8a: Rebuild service Rust internals on `wat::thread::*` (drop typed_send/typed_recv direct calls)
- 8b: Process-tier service variant on `wat::process::*`
- 8c: Tier-agnostic service worker body — service Body uses `:wat::comm::*` polymorphic verbs
- 8d: Migrate existing arc 203 consumers to the rebuilt service shape; smoke probes

### Slice 9 — INSCRIPTION (atomic; ~1 stepping stone)

Closure paperwork. The complete concurrency toolkit shipped.

- INSCRIPTION.md
- 058 changelog row
- USER-GUIDE.md section (brackets + services + tier wrappers)
- Cross-references: arc 213 (cascade chokepoint precursor) + arc 198 (restriction discipline) + arc 203 (struct-restricted OOP) + arc 212 (newtype wall pattern) + arc 170 (run-threads retired into bracket-map composition)
- MEMORY entries for the doctrines this arc adds

## Slice dependency graph

```
Slice 1 (foundation traits)
   ├── Slice 2 (thread tier)  ─┐
   └── Slice 3 (process tier) ─┴── Slice 4 (wat surface)
                                       └── Slice 5 (migration sweep)
                                              └── Slice 6 (structural wall)
                                                     ├── Slice 7 (brackets) ──┐
                                                     └── Slice 8 (services) ──┴── Slice 9 (INSCRIPTION)
```

**Slice 7 BEFORE Slice 8** per user 2026-05-18 ("brackets first, services second"). Both are consumers; brackets is the simpler polymorphic-dispatch proof; services builds on the same pattern with state-management complexity. Sequential ship; per-stone trust gate between.

## Per-stone trust gate discipline

Per `feedback_iterative_complexity` + the load-bearing lesson from arc 170 closure-blocking ("sonnet getting confused or doing too much work sets us back days to hours; we've been trying to close 170 for over a week"):

**Each stepping stone within a slice is ONE coherent concern; sonnet sees only that concern; orchestrator verifies SCORE before next stepping stone spawns.** No bundled scope; no "while you're there"; no scope-creep. Slow is smooth, smooth is fast.

Stepping stones designed orchestrator-side at slice-open time; not pre-decomposed at arc-DESIGN level (that's premature; the foundation must land before downstream stepping-stones can be specified concretely).

## What this arc supersedes

- **arc 213 χ stones** (chokepoint completion via wrapper) — folds into Slice 2 as precursor; χ-1 + χ-2 are stepping stones the new arc builds on
- **arc 213 χ-3** (build.rs scanner direction) — historically inscribed at commit `40f9b95` but SUPERSEDED. The structural wall via crate-private discipline (Slice 6) replaces it. The scanner approach was whitelist-shaped; the new approach is type-system structural.
- **arc 213 δ-1** (ChildHandleInner pidfd field) — dirty tree preserved per `feedback_defect_fix_or_panic_never_revert`; ships in Slice 5f atomically with cascade-completeness proof
- **arc 213 δ-2/3 / ε / ζ / η** (process management migration + INSCRIPTION) — continue in arc 213 separately; this arc focuses on the COMMS chokepoint, not the libc::fork chokepoint
- **arc 170 D-stones** (run-threads bracket macro) — folds into Slice 7 as precursor; capability subsumed by `parallel-map-reduce` composition
- **arc 203 ServiceWithProvisioning** — folds into Slice 8; rebuilt on tier wrappers
- **typed_send / typed_recv** (Value-layer chokepoint at src/typed_channel.rs:203,295) — subsumed into tier wrappers in Slice 5c

## What this arc explicitly does NOT do

- **Remote tier** — empty seat; designed in this DESIGN; minted when we know what remote IS (future arc; transport-specific decision)
- **Reactor tier** — empty seat; designed in this DESIGN; minted when substrate adopts userspace async runtime (multi-arc architectural pivot; not bundled here)
- **Sync vs async substrate decision** — substrate stays threads-as-tasks; tier wrappers support both models; reactor tier addition is the trigger for async runtime conversation, not this arc
- **HTTP / network / TLS / async crates** — separate concerns; per DEPENDENCY-DOCTRINE; future arcs

## Discipline invariants (load-bearing for sonnet briefs)

These doctrines apply at every slice + every stepping stone:

- `feedback_options_are_tangle` — ONE canonical mechanism per concern; no "use poll here, epoll there" / no "use crossbeam here, mutex there"
- `feedback_simple_is_uniform_composition` — N identical mechanical edits IS simple; bundle uniform; split heterogeneous
- `feedback_iterative_complexity` — STOP when hitting deadlocks; build small funcs; prove each stepping stone
- `feedback_no_hang_vector_in_additive_scorecard` — additive-mint stepping stones get cargo-build-clean as verification; hang-prone regression tests belong on proof stepping stones, not mint stones
- `feedback_defect_fix_or_panic_never_revert` — active replications stay on the dirty tree; fix or panic; never revert
- `feedback_substrate_owns_not_callers_match` — when N call sites need identical setup, substrate owns it; callers are benefactors
- `feedback_never_deadlock` — every comm site lands deliberately; the wall enforces this beyond what convention can
- `feedback_brief_constraint_contradictions` — BRIEFs MUST NOT have hard constraints contradicting deliverables
- `feedback_brief_no_easy_auth` — when a BRIEF lists candidate paths, name ONE required path

## Personal stake — what this arc means

Per user 2026-05-18:
> *"we've built all of my toolkit on rust -- this is my response to 'just learn rust' -- i just did - i learned rust so well i made it feel like ruby and it reads like clojure"*

This arc is the proof. Ruby's OOP discipline (services as protected mutable state) + Clojure's read-ability + Rust's performance + structured concurrency by construction = wat. The synthesis that was being told "isn't real" — shipped, structurally enforced, never-revisit-again. The user's answer to years of dismissal lands when this arc closes.

The toolkit emerges from the foundations as unshakingly correct. Future code builds on this and inherits the discipline structurally. We never deal with this domain again.

## Cross-references

### This arc subsumes / supersedes
- `docs/arc/2026/05/213-libc-fork-mismanagement/BRIEF-213-CHI-3-COMPILE-TIME-WALL.md` (build.rs scanner; replaced by Slice 6 structural wall)
- `docs/arc/2026/04/[various]/run-threads work` (folds into Slice 7 brackets)
- `docs/arc/2026/05/203-service-with-provisioning/*` (folds into Slice 8 service rebuild)

### Foundation references
- `scratch/DEPENDENCY-DOCTRINE.md` — authorizes the io-uring crate dep
- `wat-rs/docs/ZERO-MUTEX.md` — composes with the cascade-by-construction discipline
- `wat-rs/docs/CONVENTIONS.md` — wat naming conventions for tier namespaces
- `wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md` — § 7 sonnet delegation protocol; § 11 end-of-work ritual

### Doctrine precedents
- arc 057+ `project_holon_universal_ast` — HolonAST as universal substrate form
- arc 146 — multimethod dispatch (used by Slice 4 polymorphic verbs)
- arc 198 — `#[restricted_to(...)]` wat-level access control (sibling discipline shape)
- arc 203 — struct-restricted OOP (services pattern this arc rebuilds on tier wrappers)
- arc 212 — `WatAST::children()` newtype wall (parallel structural-impossibility pattern)
- arc 213 — cascade chokepoint precursor (χ-1 + χ-2 stepping stones this arc builds on)

### Linux primitive doctrine
- INTERSTITIAL § 2026-05-18 (post-PURGE) "Linux 5.3+ syscall doctrine" — pidfd / clone3 / waitid(P_PIDFD) precedent; io_uring extends this discipline

### User direction (load-bearing for this arc)
- *"slow is smooth, smooth is fast"*
- *"we do it perfect now and build on top of them forever"*
- *"we get all the greatness of Ruby's OOP, FP and concurrency"*
- *"hide all the guts - don't let users make mistakes"*
- *"deadlocks are illegal"*
- *"options are why we are in a tangled mess"*
- *"we are Linux elitists; we use best-of-breed from Linux 5.3"*

---

**Arc OPENED 2026-05-18.** Slice 1 (foundation primitives) is the first stepping stone; orchestrator drafts BRIEF + EXPECTATIONS at slice-open time per per-stone trust gate discipline.
