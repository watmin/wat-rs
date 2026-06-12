# Arc 259 — The Forced Hand

> Named by the intueri cast (2026-06-11): the reserved `wat.` prefix is the
> constraint that unlocks user freedom — the forced hand. The deliverable is the
> ambient program environment; the *identity* is the paradox. Surface names
> locked below (§ Names — locked).

## Thesis — paradoxical strict rigidity reveals unlimited expression

The builder's framing, and the spine of this arc: **every constraint here is a
constraint that *unlocks* freedom.** The reserved `wat.` prefix, the nominal
records, the forced hand of the type system — none of them fence the user in.
They are exactly what lets the user:

- **extend without collision** — `wat.*` is unnameable in user space, so a user
  env can never clobber a platform field; the platform is the sole author of its
  own data by construction, not by discipline.
- **compose without diamonds** — user data nests under per-layer slots, so it
  never enters the platform's keyword set; there is nothing to merge, no
  multiple-inheritance, no row-polymorphism.
- **test without ceremony** — the env is a plain value and install is one
  function, so a test builds any world it wants and installs it; fixed
  timestamps make timing deterministic.
- **diagnose in-band** — the substrate stamps its own timing into the env, and
  programs read it as ordinary wat data; observability is self-hosted, not a
  bolted-on profiler.

The rigidity is the freedom. What Ruby achieved with `Thread.current[:key]` by
*discipline* (a careful engineer keeping an open hash honest), wat achieves by
*structure* (the type system making the dishonest shape unrepresentable). The
forced hand is the discipline externalized into the substrate so it cannot drift.

## What this is

The **program environment**: an ambient, self-diagnosing, extensible context
threaded through every execution frame — `:user::main`, every `spawn-program'`
peer, every brackets worker. It began as task #211 ("thread `:wat::program::Env`
into the spawned peer's eval context") and grew a spine of its own.

It is **not** clojure-cutover prep — arc 251 is the *surface* (symbol heads,
types-as-forms, EDN); this is the *runtime context*. They share a bloodline
(this is only possible because A2 turned `program::Env` from a `HashMap` heresy
into a real record) and an ethos (make wat honest), but the cutover stays the
foreground we circle back to.

## The identity — wat's escape hatch for system interrogation

The deepest framing of what this record *is* (builder, 2026-06-11): the
program-env is wat's **single structured escape hatch for system interrogation** —
the one curated, typed, unforgeable window through which a pure wat program asks
the host *"what am I running as?"* (pid, thread, tier, timing, eventually host /
resource limits / cgroup) **without ever reaching for a raw syscall or FFI.**

This is the **anti-FFI**. Every other language's answer to "how do I get my pid"
is `libc::getpid()` scattered at call sites — ad-hoc, side-channel, unforgeable
only by discipline. wat's answer: the kernel stamps it into a `wat.*` field at the
seam, and the program reads it as ordinary EDN data through a record accessor.
Same capability, inverted soul — the same move as the whole substrate.

That framing is the **boundary** that keeps "grow it over time" a discipline, not
a kitchen sink:

- **Belongs in the env** = *system interrogation*: what only the kernel knows
  about *this execution* — identity, timing, tier, host, resource context. One
  channel, one reserved prefix (`wat.*`), read-only, typed, can't collide, can't
  be forged. The env can absorb every system-interrogation need wat will ever
  have, and each addition is one more `wat.*` field.
- **Does NOT belong** = *config* (the program's own choices) and *user data*
  (the `user.*` slots). Those have their own homes.

So the field set grows unboundedly *because* it is bounded: every addition is a
kernel-stamped fact a program would otherwise have reached for a syscall to learn.
(Realization-grade — the *structured escape hatch*: the typed window that replaces
scattered FFI. Candidate for the realizations chronicle.)

## The converged architecture

### 1. Storage — thread-local, RAII install (mirror `AMBIENT_STDIO`)

The env lives in a `thread_local!` slot, exactly like `AMBIENT_STDIO`
(`src/services/client.rs:326`) and `CALL_STACK` (`src/value/frame.rs:11`):

```rust
thread_local! { static PROGRAM_ENV: RefCell<Option<...>> = const { RefCell::new(None) }; }
```

Why thread-local and not a process-global (the `ARGV` `OnceLock` pattern) or a
`SymbolTable` slot:

- **The env is a property of the running peer, and a peer is a thread** — the
  mapping is exact. Each thread-tier peer is its own `std::thread`, so it gets
  its own slot for free.
- **The `worker-id` case requires it** — N brackets workers, each a thread
  sharing one address space, each needing a *distinct* `worker-id`. A global
  structurally cannot hold N distinct values; thread-local does it with zero
  ceremony. This isn't merely consistent with thread-local; it forces it.
- **It stays in the fork-safe category** the v5-deadlock realization blessed
  (atomics, thread-locals, immutable Arcs) — never a worker-thread-bearing
  global, the one shape that cost a month.
- **It decomplects from `SymbolTable`** — the env is runtime *context*, not
  symbol *definitions*; a `sym` slot would braid the two.

Install is **RAII-scoped** (save/restore), mirroring `install_ambient_stdio` /
`take_ambient_stdio`, so nested in-process invocation and test isolation are
clean by construction.

### 2. The install seam — post-bootstrap, pre-`:user::main`

`invoke_user_main_orchestrated` (`src/freeze.rs`) already has the exact window:

```rust
let runtime = bootstrap_wat_vm_process(...)?;   // VM LIVE: services + ThreadIO up
//  ← install the env here: full VM, main not yet called, runtime.symbols() live
apply_function(main_func, args, runtime.symbols(), ...);   // main runs here
```

The gap between `bootstrap_wat_vm_process` and `apply_function(main)` is "VM
live, main not started, we can do wat work." The env is built and installed into
*this thread's* slot there; main runs synchronously on the same thread and reads
it. For user-extension (later), the user's init is a **wat function run by the
live VM at this same seam** — self-hosted: the VM constructs the env on itself
before handing control to main. `invoke_user_main` gains an env parameter (the
CLI passes the launch env; tests pass a test env — "hermetic tests take an env
arg").

### 3. The base fields — kernel-stamped, the floor

```
:wat::program::Env = { wat.started-at, wat.peer-started-at   ; : :wat::time::Instant            (timing)
                       wat.process-id, wat.os-thread-id      ; : :wat::core::i64                 (identity)
                       wat.peer-kind }                       ; : :wat::program::PeerKind         (identity)
;; PeerKind = (:thread | :process)  — :thread shares its address space, :process owns it
;; (root :user::main + forked peers = :process). All five SHIPPED.
```

**Timing fields** differ in **propagation** — this is the load-bearing distinction:

- **`wat.started-at` — inherited.** The app's epoch. Stamped once at CLI boot,
  propagated *unchanged* down the entire spawn tree. One monotonic anchor for
  the whole program.
- **`wat.peer-started-at` — re-stamped.** This frame's epoch. Set to `now` at
  each thread's *actual* start (in the peer closure, not the call site — threads
  start async), via `assoc` (which preserves the concrete subtype and updates
  the one field — the reason A2's "assoc returns the specific type" matters here).

Three diagnostics fall out, all pure wat, all self-hosted:

```
(now - wat.started-at)                      ; total app uptime  (any peer, any depth)
(now - wat.peer-started-at)               ; THIS peer's uptime
(wat.peer-started-at - wat.started-at)    ; this peer's spawn latency
```

Plus the **startup-latency metric**: `wat.started-at` is captured at the
*earliest* point (wat-cli's Rust `main`, before the fork — `CLOCK_MONOTONIC`
survives fork on Linux, staying comparable), so a program reading
`(now - wat.started-at)` at its first statement measures the real boot cost.

### 4. Access — an ambient verb, no signature change

Arc 170 slice 1e retired *all* explicit `:user::main` args (stdin/stdout/stderr/
argv → ambient); canonical main is `[] -> :nil`. The env follows that idiom: an
ambient verb returns the whole record, read via accessors. No signature change.
(Side find: `crates/wat-cli/src/lib.rs`'s module-doc still shows the *retired*
3-arg contract — stale, fold a fix in.)

### 5. The reserved prefixes — the forced-hand authority

- **`wat.*` = platform-owned** (kernel *and* blessed stdlib like brackets). You
  read it; you never write it. Enforced at record-declaration: a user record
  cannot *name* a `wat.*` field. So the user literally cannot forge `wat.worker-id`
  or lie about `wat.started-at` — there is nothing to overwrite, because the keys
  are unnameable in user space. The platform is the sole writer by construction.
- **`user.*` = user slots** (see § 6). The user's *actual* fields live *inside*
  those slot records, so nothing collides at the top level.

### 6. User-extension — per-layer nested slots (the "two-slot one-up")

Each context layer is a `(platform field(s) + user slot + user-init)` triple:

```
:wat::program::Env              wat.started-at · wat.peer-started-at · user.program          (process layer)
  └─ :wat::bracket::Env  <:     + wat.worker-id                        · user.bracket  (bracket layer)
```

- **No overlap** — distinct keys, namespaced by layer. *"run my binary `<with
  this>`"* fills `user.program` at process start; *"run my fanout `<with additional>`"*
  fills `user.bracket` at bracket start.
- **Nesting dissolves the diamond** — user data never enters the platform's
  keyword set, so there is nothing to merge; the platform chain stays a clean
  linear subtype chain.
- **One slot per layer = one owner per layer** — the process is one owner; each
  fanout is one owner. (Multi-tenant *within* a layer is the only thing that
  would ever reopen the protocol question — genuinely far off.)
- **Any user record fits** — the slot is typed `:wat::Record` (the root), and
  every record is a subtype of `:wat::Record`, so any user record is assignable.
- **Construction** — the user's init produces *only* their (non-`wat.*`) record;
  the platform stamps the `wat.*` fields. Both brackets and `spawn-program'`
  accept a user-init that builds their layer's slot. (`init-fn → record →
  platform stamps wat.* → install` — the transient is gated by the record's `of`,
  not a runtime bag.)

### 7. Records are maps (the grounding that made § 6 click)

A wat record *is* a keyword→data map with a **closed**, **nominal**, **typed**
key set: the attr set is literally `field_names: Vec<String>`; the value is the
data, accessed name→index. Strip closed+nominal+typed and it's an EDN map. That
is the entire difference — and it's why "embed the user's record under a `user.`
slot" is trivial: it's just a value in a field.

### 8. `:remote` — the perpetual forcing function

`spawn-program' :remote` stays unbuilt *on purpose*. It will need a rich env —
`{:remote-url, :signing-key, …}` passed in — and keeping it unbuilt forces every
part of this design to stay general enough to admit a rich platform env, never
baking in "envs are only the kernel-nominal shape." It is the env system's
location-transparency: the unbuilt tier keeps the built path honest.

### 9. Testing — the payoff

The env is a value; install is one function. A test builds any env (fixed
`wat.started-at` → deterministic timing; fake `worker-id`; whatever), installs
it via the RAII guard, runs the code, asserts — no CLI, no fork, same seam as
production. This is the `install_ambient_stdio` / `take_ambient_stdio` test
pattern, proven in the suite.

## Why nominal, not structural (the fork, resolved)

The composition problem (cross-cutting `worker-id` vs user fields under single
inheritance) has three classical answers: nominal linear chain (rigid, diamond),
row polymorphism (free composition, off the nominal-ADT compass), or a nominal
protocol (`defprotocol` — the right shape *if* we ever needed structural
interface compliance, and the far-off answer for multi-tenant-within-a-layer).
**Nesting (§ 6) deletes the problem** rather than solving it: user data sandboxed
per layer never needs to compose with platform fields. So we stay nominal, use
only existing machinery (subtyping + assignable-to-`:wat::Record`), and add
nothing structural. The ADT compass holds.

## Stone decomposition

- **259.0 — the floor.** `program::Env { wat.started-at, wat.peer-started-at }`
  + `PROGRAM_ENV` thread-local + RAII install + the ambient access verb + install
  at the seam + `invoke_user_main` env arg. (The probe
  `tests/probe_arc211_program_env_ambient.rs` is this stone's disconfirmer — note
  `c01` false-greens via the known reserved-prefix-blanket-accept leniency, so the
  real verb test is the value-flow C03.) Ripple: the 1-arg `program::Env`
  constructor → 2-arg breaks ~11 call sites (arc258 + peer-verb probes) — migrate.
- **259.1 — spawn re-stamp** (the literal #211): `spawn-program'` stops discarding
  `_program_env`; the peer closure `assoc`s a fresh `wat.peer-started-at` (inherit
  `started-at`) and installs into the peer's thread-local.
- **259.2 — the startup-latency metric**: capture the boot `Instant` in wat-cli's
  Rust `main`, thread it through the fork, stamp `wat.started-at`.
- **259.3 — brackets integration** (rides #196): `bracket::Env <: program::Env` +
  `wat.worker-id` + `user.bracket`; per-worker install.
- **259.4 — user-extension**: `user.program` / `user.bracket` slots, user-inits,
  the reserved-`wat.*` declaration check (the authority enforcement).
- **259.N — inscription.**

**Built now:** 259.0–259.2 (the floor + the metric — the forced hand for the
platform's own fields). **Enabled-not-built:** 259.3 (brackets), 259.4
(user-extension), `:remote`. The floor types everything to `:wat::program::Env`
(the base), so every deferred layer rides through with zero retyping.

## Names — locked (intueri cast, 2026-06-11)

- arc: **"The Forced Hand"** / `259-forced-hand`
- access verb: **`:wat::program::env`** (the type lives in `program::`; `runtime::` is VM self-inspection)
- base fields: **`wat.started-at`** (the `-at` lock; `-time` mumbles point-vs-duration) + **`wat.peer-started-at`** (tier-agnostic — `thread-` would lie to a `:process` peer)
- user slots: **`user.program`** (process layer) + **`user.bracket`** (bracket layer) — no trailing `.env` (re-invokes the overloaded ambient noun)
- Rust API: **`PROGRAM_ENV`** thread-local · **`install_program_env`** (RAII write) · **`current_program_env`** (read-many, NOT `take_`)

### (original open questions, resolved above)

- the **arc title** (working: "the program environment")
- the **access verb** — `:wat::program::env`? (the type lives in `program::`;
  `:wat::runtime::argv` is the ambient-query precedent — runtime:: vs program::)
- the **base fields** — `wat.started-at` / `wat.peer-started-at` (the locked
  `-at` EDN-point idiom) **vs** the builder's recent `wat.start-time` /
  `wat.thread-start-time` (reintroduces the `-time` "type-mumble" the lock
  rejected). Resolve the contradiction.
- the **user slots** — `user.program` / `user.bracket`
- the **Rust install API** — `install_program_env` / `current_program_env` /
  the `PROGRAM_ENV` thread-local
- the **per-frame field's word** — RESOLVED: `peer-started-at` (tier-agnostic;
  `thread-` lies to `:process` peers)

## Lineage

Born of #211 (214-era typed peers) atop A2's `program::Env` record rewrite (which
exposed + fixed the collapsed record type system, `5f6178aa`). Pairs with #196
(brackets), the `:remote` forcing function, and the testing doctrine
(`feedback_embed_doctrine_rigs_are_universes`). Returns to arc 251 (clojure
cutover) on close.

---

# The spawn primitive — `(spawn-program <host> <prog>)` (LOCKED 2026-06-11)

The deepest decision of the arc, because a core primitive's *signature* is the one
thing you cannot cheaply change later. Locked through a long design duet; the
governing constraint: **set future selves up for extension without sig churn or
mass rewrites.** (This obsoletes the original stone 259.1's "thread the env-value
into the peer" framing — the env-value-as-arg is the broken thing this replaces.)

## Two discrete concerns, forever

```clojure
(spawn-program <host> <prog>)
;;              host = WHERE to run it — as complex as the tier demands
;;              prog = WHAT to run     — always a 0-ary fn (a self-contained program entry)
```

The asymmetry is the whole insight: **the host carries all the complexity; the
prog never changes shape.** "Where do I host this fn" is the only question, and its
answer's complexity lives entirely in the host. The signature is two args, forever
— every future option grows *inside the host*, never in the arg list. That is the
"no mass rewrite" guarantee made structural: the arity never moves.

## The host — typed opts, clause-dispatched, growable

`spawn-program` is a **clause-set matching on the host type.** Each host is a typed
opts record built by an ergonomic constructor:

```clojure
(spawn-program (thread)                          prog)   ; trivial host
(spawn-program (thread :init my-env-init)        prog)   ; + a user env-init
(spawn-program (process)                         prog)
;; (spawn-program (remote …) prog) — ILLUSTRATIVE of the pattern ONLY. The remote
;; door is PERPETUALLY AWAITING ITS KEY (the forcing function): RemoteOpts's struct
;; shape is NOT agreed and is deliberately unbuilt (see spawn.wat). Whatever it
;; becomes, its constructor's arity will be the lock.
```

- The host's *type* is the tier (`ThreadOpts` / `ProcessOpts` built; a future
  `RemoteOpts` etc.) — not a stringly keyword. The constructor enforces each tier's
  requirements via its **arity**: when a remote host eventually materializes, its
  constructor will be uncallable without whatever it needs to reach its host, so
  "a remote that can't reach its host" is **unrepresentable at the call site** —
  the forced hand, one level up from the env. (`RemoteOpts`'s actual fields are NOT
  yet agreed — the principle is the arity-lock, not any specific shape.)
- **New hosting kinds = new clauses against new host types.** A future
  `(remote-over-quic …)`, `(gpu …)`, `(lambda …)` is one new opts type + one new
  clause; zero existing clauses change, zero callers rewrite. The clause mechanism
  IS the extension door — open-closed by construction. (And there will be *many*
  remotes; the door is propped open on purpose.)
- Each clause does its own localized spawn + `wat.*` stamping at the peer's start.
  The measurement is the env's *birth* inside the clause, not a separate guard.

## The prog — per-tier contract, NOT one uniform shape (corrected 2026-06-11)

**The real invariant is "each host pairs with its prog contract," NOT "the prog is
always 0-ary."** The earlier 0-ary claim was wrong — falsified by the capability
discipline (a comm channel is a handle, not data: it can't ride in the EDN env, and
ambient-abuse hides the grant) and then dissolved entirely by the clause dispatch.
`spawn-program` is a `defclause` that dispatches on the **host type**, and each
clause carries its tier's **own prog contract**; the forced hand makes the pairing
unforgeable (a `ThreadOpts` demands a `ThreadProg`; handing it a `ProcessProg` is a
type error). **The asymmetry is load-bearing, not a smell** — it falls straight out
of address spaces:

- **`ProcessProg` — a stdio `:user::main`.** A process is a *fresh* address space →
  *fresh* fd 0/1/2 → the child can just *be* a stdio program. The process clause
  binds the child's 0/1/2 to the comms pipe; the parent reads/writes the bound
  handles. The child **doesn't know it's a peer** — it does ambient `readln`/`println`
  like any wat program. **Keep the existing `:process` model.** (`:remote` will be
  the same shape — a socket-backed stdio program — when its door is specified.)
- **`ThreadProg = [self: ThreadPeer] -> nil`.** A thread shares the process's
  address space → shares its *ambient stdio* → it **cannot** use stdio for data
  (collision). So the thread prog **must be handed its `(rx, tx)` self-peer
  explicitly** and `recv'`/`send'` on it. Transport-*aware* by necessity. This
  REPLACES the platform apply-loop. (`ThreadPeer` carries raw Values — crossbeam, no
  EDN; `ProcessPeer` carries EDN over the pipe; the comms layer already abstracts
  this.)
- **`RemoteProg` — perpetually awaiting** (its own args: a socket, signing, …; the
  door's key uncut until specified).

So "every peer is a `:user::main`" is *true for process + remote* (fresh-fds stdio
programs) and the **thread is the principled exception** — shared stdio forces an
explicit channel. The exception has a cause, so it is not a wart. The clause dispatch
carries the asymmetry; neither end pretends.

## The two typed structures (never conflate them)

- **host** (opts) — configures the *spawn*; consumed by `spawn-program`, never seen
  by the prog. Typed (constructor-enforced requirements).
- **env** (`:wat::program::Env`) — what the *prog reads*; the ambient record
  `{wat.started-at, wat.peer-started-at, user.<slot>}`. Built by the host's clause
  at peer start (stamp `wat.*` + run the host's `init-fn` → the user slot). Typed
  record (the forced-hand contract).

No dynamic bag anywhere — opts is typed, env is typed, prog is a thunk.

# The corrected timing model (supersedes 259.0c's placeholder + 259.2) — ✅ SHIPPED

> SHIPPED 2026-06-12: a pid-keyed, fork-safe boot clock (`crate::time::process_boot_instant`
> / `set_process_boot_instant`) primed at wat-cli's earliest point; the seam stamps
> `started-at` from it (via `at-nanos`), `peer-started-at` stays the seam's `now`. The
> two stamps are now distinct; the gap reads out as the real boot→entry latency. A
> program measures it with the Duration readout family: `(seconds (- peer-started-at
> started-at))`. Test-injectable via `set_process_boot_instant`.

`259.0c` shipped `started-at = peer-started-at = now` as a *placeholder* — which
collapses the measurement and is wrong. Corrected:

- **`wat.started-at` = process-level, captured at the *earliest* point** — wat-cli's
  Rust `main`, measuring `now` before anything else. A **fork-safe, pid-aware
  process-global**: re-captured when the pid changes across a fork, so a `:process`
  peer (its own universe) measures *its own* boot, never the parent's stale value.
  **This is the v5-deadlock lesson applied** — a set-once `OnceLock` would inherit
  the parent's value across fork and lie, exactly `SHUTDOWN_RX`'s shape; pid-identity
  makes the wrong state unrepresentable. Shared *within* a process (`:thread` peers
  inherit it); re-captured *at* a process boundary.
- **`wat.peer-started-at` = per-env, captured just before that env's entry runs** —
  before `:user::main` for the main thread; before the peer's 0-ary prog for a peer.
  The two stamps are **never equal**; their gap is the boot→entry latency.
- **nanosecond fidelity** — `Instant` is nanosecond-monotonic; a nanos duration op
  exposes it (the sub-millisecond startup metric needs it; `epoch-millis` is too
  coarse).
- **multi-phase** — boot (`started-at`) → [init-call] → entry (`peer-started-at`);
  each boundary a free in-band probe. With the host's `init-fn`, boot→init and
  init→entry become separately measurable: *how slow is the user's own init*,
  distinct from platform boot.
- **test-injectable** — a test passes `(fn [] <known-record>)` as the host's
  `init-fn`, or directly installs a known env; all logic over the env's stamped
  fields is deterministic against numbers you declared.

# Prior art — who else stood here

The design re-derived a timeless answer from the forced-hand discipline; naming
where the masters already stood (the "we found a great" path-signal — reach for the
tool, land on a named great, you're on the ridgeline):

- **Java's `Executor` (Doug Lea, `java.util.concurrent`)** — `executor.execute(runnable)`
  / `submit(callable)`: separate the *where* (the executor — a thread pool, a remote,
  a ForkJoinPool — *many kinds*) from the *what* (a **0-ary** `Runnable.run()` /
  `Callable.call()`). The closest match for "many kinds of remotes": the Executor is
  the polymorphic host; the task is the uniform 0-ary thunk. Our clause-per-host is
  the typed, multimethod-dispatched Executor.
- **Erlang's `spawn_opt(Node, Fun, Opts)` (Joe Armstrong; Hewitt's actors realized)**
  — a **0-ary fun** + a node (location) + an options list. `spawn(Node, Fun)` puts
  the *where* as a param; the actor is a thunk; location transparency. "Many remotes"
  = Erlang distribution (nodes).
- **Cloud Haskell / `distributed-process` (Epstein, Black, Peyton-Jones)** —
  `spawn :: NodeId -> Closure (Process ()) -> Process ProcessId`: a NodeId (where) +
  a **serializable 0-ary closure** (what). This *is* program-over-the-wire — the
  closure travels to the node — our `:remote` = `:user::main`-over-a-pipe, named in
  Haskell years ago.
- **Rust's `thread::Builder::spawn(closure)`** — the Builder is the host config (name,
  stack size — *as complex as needed*); the closure is a **0-ary `FnOnce`**.
  `thread::spawn(closure)` is the trivial-host shorthand. Host-builder + 0-ary thunk,
  exactly our shape.
- **Kubernetes (the Pod spec)** — the infra-scale version: declarative *placement*
  (node affinity, tolerations, resources — the host, as complex as it must be) + a
  *container* (the what). The scheduler matches placement, runs the container.
- **Multimethods (CLOS generic functions; Clojure `defmulti`; wat's own `defclause`)**
  — the clause-on-host-type dispatch itself; behavior keyed on the host's type, open
  for new methods.

The convergence is the one the RAII-IPC realization (2026-06-07) named: **discipline
applied honestly re-grows the timeless architecture from the inside.** We were not
imitating Doug Lea or Joe Armstrong — we reached for "how do I host a thunk somewhere
arbitrary, safely, extensibly," let the forced hand cut the shape, and landed on the
*Executor / actor-spawn / program-over-the-wire* pattern they each found. The masters
did not hand us the toolkit; we re-derived it, and recognize it as theirs now that we
hold it.

## Revised stone decomposition (the spawn-redesign floor)

The floor grew — correctly, because the sig is the foundational decision:
- **259.0 ✅ SHIPPED** (`a50d19b6` + `42189663`) — the env record + thread-local +
  verb + the in-process/fork-child install seam. (259.0c's `started-at` is the
  placeholder the timing-correction below fixes.)
- **259.1 — the spawn redesign (THE sig lock):** `spawn-program` → clause-set on the
  host type; `(thread)` / `(process)` host constructors + typed opts; the 0-ary prog
  model (collapse the `:thread` apply-loop + `:process` forms); `wat.*` stamping
  per-clause at the peer's start; migrate the peer-verb callers. `:remote` stays an
  **unbuilt clause** (the forcing function).
- **259.2 — the corrected timing:** pid-aware process-global `started-at` (wat-cli
  Rust-main capture; freeze-time for in-process) ≠ `peer-started-at`; the nanos
  duration op; tighten c04 to assert the two stamps differ.
- **259.3 / 259.4** — brackets (`bracket` host clause + `wat.worker-id`) and
  user-extension (`init-fn` → user slot) ride the locked sig with zero churn.

---

# THE CONVERGED MODEL — one entry point (LOCKED 2026-06-11)

> Supersedes the `ThreadSelf'` framing (DESIGN-STONE-259.S2a.md) and the
> user-facing-`close'` assumption wherever they conflict. Reached through a long
> design duet + recovery of the arc-170 / arc-214 prior work — we were reinventing
> already-built machinery (`ThreadPeer`, `run-threads`, platform-owned join, the
> "deadlocks illegal" doctrine, arc 214 DESIGN:300). The four-questions hold on the
> unification: Obvious / Simple / Honest / Good-UX all YES — it is the smallest
> concurrency surface the substrate can have.

## The shape

ONE mechanism, TWO interfaces, ZERO user-held lifecycle:

```
USER SURFACE (idealized — no primes):
  spawn-program   — THE primitive. One long-lived worker.   (spawn-program <host> <prog>)
  brackets        — built FROM spawn-program. Fan-out interface. N workers.
  → the ONLY two entry points to concurrency, period.

PLATFORM-INTERNAL (never user-callable):
  spawn-thread / spawn-process — per-tier kernel primitives; ONLY spawn-program invokes them.
  close / drain / join         — internal lifecycle; the platform owns it.

USER HOLDS: a pipes-only Peer<S,R> — send / recv, nothing else.
GUARANTEE: zero deadlocks, STRUCTURAL — the user never holds the rope.
```

## The unified Peer (kills `ThreadSelf'`)

The worker's self-peer and the parent's handle are the SAME pipes-only type:
`Peer<S,R>` = `{ tx: Sender<S>, rx: Receiver<R> }`; `send'`→S, `recv'`→R, **uniform**
(no mirror projection). This kills the bespoke `ThreadSelf'` — a duplicate of the
already-built `ThreadPeer` concept (the arc-170 C1 test already mints the param-swap
mirror `ThreadPeer<i64,String>` ↔ `ThreadPeer<String,i64>`). The two ends are
param-swaps: parent `Peer<I,O>`, worker `Peer<O,I>`. The join/lifecycle handle is
**not in the Peer** — it is platform-internal.

## `close` is internal — RAII Drop + orchestration-explicit

`close` leaves the user's hands entirely:
- The Peer's **RAII `Drop`** IS the internal close — drain (drop the platform's
  senders) → join, in the cascade-correct order the drain-then-join walker already
  enforces. **Hang-free by construction:** `recv'`/`send'` auto-wire `SHUTDOWN_RX`
  and **raise** on disconnect/shutdown (`comms/thread.rs:10-14`; `eval_peer_recv_prime`
  returns an `EvalBreak`, not a value) — so Drop dropping the platform's sender → the
  worker's next `recv'` raises → the worker unwinds → `join()` completes. A worker
  cannot hang on a cascade-aware `recv'`.
- The **orchestration layer** (brackets, spawn-program teardown) may call the SAME
  internal `close` explicitly for deterministic mid-scope teardown. Idempotent (the
  `Option::take` single-close pattern, already on disk).
- The **user** calls neither. User-facing `close'` retires.

**Honest boundary:** this guarantees termination for a worker blocked on
`recv'`/`send'`. A non-terminating pure-compute loop that never touches its channel is
not a deadlock — no structured-concurrency model can structurally forbid it. Don't
oversell "zero deadlocks" into "zero infinite loops."

## `brackets` is built FROM `spawn-program`

`brackets` is a wat layer over `spawn-program`: it calls `spawn-program` N times within
a scope that owns all N reaps (drop-all → cascade → join-all). The existing arc-170
`run-threads` (built on the legacy `ThreadPeer`-struct path) is the prototype; it is
**rebuilt over `spawn-program`**, and the struct path retires. `spawn-program` =
long-lived; `brackets` = fan-out; one engine, two interfaces.

## The asymmetry lives inside `spawn-program`'s dispatch (unchanged)

`:thread` worker = explicit self-`Peer` (shares ambient stdio → must take a channel).
`:process` worker = stdio `:user::main` (fresh fds → ambient `readln`/`println`; KEEP).
The host-type clause carries the asymmetry; neither end pretends.

## Stone decomposition — re-derived (2026-06-11)

From the current mid-migration disk (prime path: `spawn-program'` intrinsic +
`Thread'`/`Process'` opaque + user `close'`; arc-170 path: `ThreadPeer` struct +
`run-threads`) → the idealized end state. Stepping-stone ordered; each an independently
provable strike.

| Stone | What | Provable by | Depends on |
|---|---|---|---|
| **S2a** | **Unified pipes-only `Peer` + thread self-peer handoff.** Kill `ThreadSelf'`; worker self-peer = `Peer<O,I>` (uniform `send'`/`recv'`). Rewrite the `:thread` arm: apply-loop → construct the child `Peer` **in-thread** (owner-thread invariant) → call prog ONCE. Keeps the 3-arg call shape + user `close'` for now. | wat probe: thread prog drives its self-peer (revise the committed probe off `ThreadSelf'`) | — (the hard new machinery) |
| **S2b** | **Internalize `close`: RAII Drop + orchestration-explicit.** Custom `Drop` on the Peer = drain→join (cascade-safe). The parent handle becomes a pipes-only `Peer` whose Drop reaps. `close` becomes an internal op (additive; user `close'` still works until S2d). | lib-test: dropped peer reaps without hanging; explicit internal close idempotent | S2a |
| **S2c** | **`spawn-program'` → host-type defclause (choice A) + the real env stamping.** 2-arg `(host prog)`; wat defclause on `ThreadOpts`/`ProcessOpts`; calls internal `spawn-thread'`/`spawn-process'` (non-user-callable); retire `infer_spawn_program_prime` keyword inference. **The host opts carry the program `init-fn`** (`(thread :init f)` — still 2-arg). The clause does the real peer-start env build: **pid-aware `wat.started-at` + `wat.peer-started-at` + nanos duration op + run `init-fn` → `user.program` slot** — kills the `259.0c` placeholder lie. | wat probe: dispatch on host type; wrong host = type error; env timing fields distinct; `user.program` carries the init result | S2a |
| **S2d** | **Migrate callers + the cut.** All callers → `(spawn-program (thread\|process) prog)`; drop user `close'` calls (rely on RAII); REMOVE `close'` / `spawn-thread'` / `spawn-process'` from the user surface. | full non-ignored gate green | S2b, S2c |
| **S3** | **`brackets` rebuilt over `spawn-program` + the bracket env (the dual init-fn).** wat fan-out layer; retire arc-170 `run-threads`(struct) + `ThreadPeer`/`ProcessPeer` struct + peer `readln`/`println`. **`bracket::Env <: program::Env` + `wat.worker-id` + `user.bracket` + the bracket `init-fn`** — the SECOND init-fn consumer that forces the extension/nesting design to be general. Folds in #196 / 214 Slice 7 / 259.3/259.4. | wat probe: `brackets` fan-out + structured join-all; per-worker `wat.worker-id` distinct; `user.bracket` carries the init result | S2d |
| **S4** | **The prime-drop sweep.** Drop the `'` from `spawn-program'` / `send'` / `recv'` / `Peer'` / `select'` → the idealized no-prime names. The migration-end cosmetic cut. | corpus + gate green | S3 |

**The only thing NOT built: `:remote`** (perpetually-awaiting — the forcing function on the
*location* axis; its opts shape is deliberately unagreed). Everything else — the timing
correction, the program init-fn (S2c), and the bracket init-fn (S3) — is **built now**: the
**dual init-fn (program + bracket) is itself the forcing function on the *extension* axis** —
two consumers prove the nesting design is general, where banking one would bake in single-layer
assumptions. Building both is the forced hand.

## The user env: defaulted, never optional — and the host-constructor space (SETTLED 2026-06-11)

The user env is **always a record, never nil/absent** — that is what dodges
`optional-is-a-smell`. The slot type never flips to `Record | nil`:

```
user.program : :wat::Record   ← ALWAYS
  custom:   the init-fn's record (recoverable by its runtime type)
  default:  :wat::program::EmptyEnv  (a 0-field NOMINAL record — not anonymous {}, not nil)
```

Empty-record-**not-nil** is what makes "didn't provide one" honest: there is no nil
branch, no two-grammar type rule.

**The surface dodges the optional token via two complete constructors per tier** (not an
optional `:init` keyword):

```clojure
(spawn-program (thread)         prog)   ; default — user.program = EmptyEnv   (common case)
(spawn-program (thread/init f)  prog)   ; custom  — f : [] -> SomeRecord       (f REQUIRED here)
```

`(thread)` is not `(thread/init)` with an omitted arg — it is a complete, distinct
constructor whose user-env *is* the empty record. `thread/init` requires `f`. No optional
token anywhere; the user picks intent by *verb* (the `Vec::new()` vs `Vec::with_capacity(n)`
shape). (Names intueri-finalizable.)

### The thesis incarnate — rigidity unlocks unbounded evolution

This is "paradoxical strict rigidity reveals unlimited expression" made literal. The
**rigidity** — each host constructor *complete* (no optional token, returns the fully-typed
opts), the sig *frozen* at `(host prog)`, the host carrying *all* complexity — is exactly
what makes the **host-constructor space open-closed and unboundedly growable**:
`(thread)` · `(thread/init f)` · `(thread/pinned cpu)` · `(gpu dev)` · `(remote url key)` …
each a new complete verb that **never touches an existing one, never adds a flag, never
churns the sig.** Zero interaction surface → infinite headroom.

The inversion is the whole point: the path that *felt* flexible — optional keyword flags on
`(thread …)` — is the one that **caps** evolution (flags accrete into a kitchen sink, interact
combinatorially, fuzz the type, become unmaintainable). Optionality is the cap dressed as
flexibility. The rigid form — complete named constructors — felt strict and is the one with
unlimited room. The Keymaker cuts endless keys precisely *because* each cut is complete, not
because the lock has adjustable teeth. (Realization-grade; candidate for the realizations doc.)

**Supersession note:** `DESIGN-STONE-259.S2a.md` + the STRIKE-READY commit
(`2529cce5`, probe annotating `ThreadSelf'`) predate this convergence; S2a is
re-authored onto the unified `Peer` when struck.
