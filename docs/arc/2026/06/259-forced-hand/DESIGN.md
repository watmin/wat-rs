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
:wat::program::Env = { wat.started-at, wat.peer-started-at }   ; both : :wat::time::Instant
```

The two fields differ in **propagation** — this is the load-bearing distinction:

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

# The corrected timing model (supersedes 259.0c's placeholder + 259.2)

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
