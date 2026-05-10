# Arc 170 — Program entry-point contracts + `:user::main` argv

**Status:** DESIGN settled 2026-05-09 across the conversation thread.
Ready for slice 1 authorship.

**Blocker for:** arc 109 v1 milestone closure.

---

## The mental model — client / server

> **Companion concept doc:** [`TIERS.md`](./TIERS.md) — runtime
> tiers 0 (eval env) → 1 (threads) → 2 (processes) → 3 (remote
> programs). Each tier shares less than the previous. Hermeticness
> is the ambient property of tier ≥ 2 — not a label or flag, but
> what the OS-process boundary inherently provides (memory + signal
> + global-state + runtime-sealing isolation, all at once because
> they're all manifestations of the same boundary).

Every "spawn a wat program in some context" primitive is, at its
heart, a **client / server** relationship:

- The **client** is the context that wants isolated work performed.
- The **server** is the spawned wat **fn** — code with a typed
  `(I, O)` shape per the variant's contract.
- Communication is a `(tx, rx)` pair on each side; client's `tx` ↔
  server's `rx`, server's `tx` ↔ client's `rx`.

Each spawn variant differs ONLY in the IPC mechanism connecting
client to server:

| Tier | Variant | User-visible IPC | Substrate transport | Sharing | Substrate primitive | Server contract |
|---|---|---|---|---|---|---|
| **1** | Thread | `Sender<T>` / `Receiver<T>` | crossbeam channels (in-memory typed Values) | memory shared | `(:wat::kernel::spawn-thread fn)` | `:user::thread` — `[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil` |
| **2** | Process | `Sender<T>` / `Receiver<T>` | EDN-over-pipes (substrate encodes/decodes) | host shared, memory boundary (hermetic ambient) | `(:wat::kernel::spawn-process fn)` | `:user::process` — `[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil` |
| **3** | Remote *(future)* | `Sender<T>` / `Receiver<T>` (Q-channel multiplex) | EDN-over-sockets | network shared, host boundary | `(:wat::kernel::spawn-remote-program fn endpoint)` | `:user::remote-program` — `[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil` |

(Tier 0 — `(f x y)` direct invocation in the current eval env — isn't a "spawn variant"; it's the base layer. See [`TIERS.md`](./TIERS.md).)

**Same user-visible abstraction at every tier.** The user writes
the same `(rx :Receiver<I> tx :Sender<O>)` shape regardless of
tier; the substrate handles transport encoding (none for tier 1;
EDN-over-pipes for tier 2; EDN-over-sockets for tier 3). WatAST
serializes to EDN by nature; users never see strings flowing
through these channels.

**The OS-boundary handling — `:user::main` + three substrate
services (locked in 2026-05-10):**

The wat-cli IS the OS-boundary. The OS shell speaks bytes
(stdin/stdout/stderr fds; argv array). But INSIDE wat-land,
strings stay at substrate-internal transport boundaries (REALIZATIONS
pass 5). The polished resolution: substrate-managed services own
the OS-pipe resources; user code never touches fd 0/1/2 directly.

**Three substrate services boot before any user code:**

```
:wat::kernel::StdInService   — owns fd 0; reads bytes; decodes EDN
                                line-by-line; serves typed Values to
                                consumers; returns :None on EOF
:wat::kernel::StdOutService  — owns fd 1; receives typed Values from
                                per-thread message-pipes; serializes
                                EDN; writes to fd 1; single-writer guard
:wat::kernel::StdErrService  — owns fd 2; first panic event drained
                                wins; emits structured cascade EDN;
                                calls libc::exit(N); process dies
```

Each service's loop selects over per-thread input pipes +
control-pipe (self-pipe trick for thread-list updates).

**`:user::main` simplifies to `[] -> :wat::core::nil`** (per
REALIZATIONS pass 10 — nil IS the exit code; substrate maps
nil-return to libc::exit(0); panic-cascade maps to libc::exit(N);
user code never participates in exit-code arithmetic). Same shape
as arc 114's `Program<I,O> -> :nil` contract.

The user's program runs against ambient resources:
- `:wat::runtime::current-thread` — thread-local id
- `:wat::runtime::argv` — process argv (set-once at start)
- StdInService/StdOutService/StdErrService — for I/O

The canonical wat server-program form (memory
`project_arc_170_canonical_server_form.md`; locked-in post-pass-13
— helpers + ambient + stopped?-poll + graceful-`:nil`):

```scheme
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::server-loop my-handler))

(:wat::core::defn :wat::kernel::server-loop
  [handler <- :wat::core::fn(wat::holon::Atom)->wat::holon::Atom]
  -> :wat::core::nil
  (:wat::core::if (:wat::kernel::stopped?)
    -> :wat::core::nil
    ;; stop signal observed; user-side returns nil; substrate emits :nil + exits
    :wat::core::nil
    (:wat::core::match (:wat::kernel::readln)
      -> :wat::core::nil
      ;; peer signaled done
      ((:wat::core::Some :wat::core::nil)
        :wat::core::nil)
      ;; data; process; loop (TCO via arc 003 trampoline)
      ((:wat::core::Some req)
        (:wat::core::let [resp (handler req)]
          (:wat::kernel::println resp)
          (:wat::kernel::server-loop handler)))
      ;; ungraceful close — peer died without :nil
      (:wat::core::None
        (:wat::runtime::panic! "stdin closed without graceful :nil")))))
```

`(:wat::kernel::println v)` writes data + newline; blocks.
`(:wat::kernel::readln)` returns `:Option<:wat::holon::Atom>`;
blocks; `:None` on fd 0 closed (per pass-13: ungraceful since
the peer didn't write `:nil` first). Both helpers route through
per-thread Client thread-locals (set by spawn-thread's
register-with-services contract per slice 1g). Users typically
never instantiate Client directly; advanced cases reach for
`(:wat::kernel::StdIn/client)` / `(:wat::kernel::StdOut/client)`
escape hatches. The recursive `(server-loop handler)` is in
tail position; arc 003's trampoline handles it without stack
growth.

**User-cleanup pattern** — when the user needs pre-exit work on
observed stop, they wrap their cleanup before returning nil.
Substrate stays out:

```scheme
(:wat::core::if (:wat::kernel::stopped?)
  -> :wat::core::nil
  (:wat::core::do
    (my::flush-caches!)
    (my::log-shutdown-state!)
    :wat::core::nil)
  ...)
```

**Signal model** (per arc 106 + memory `project_signal_cascade.md`):

| Layer | Owner | Contract |
|---|---|---|
| OS signal arrival | kernel + arc 106 handlers | flip atomic flag; return |
| Cascade across pid group | OS (`killpg`) | wat-cli broadcasts; kernel delivers |
| Stopped/sigusr polling | **user wat program** | `(stopped?)` etc. at safe checkpoints |
| Cleanup logic on observed stop | **user wat program** | drain work, close resources, return nil |
| Final `:nil` emit + libc::exit(0) | substrate | post-main epilogue (slice 1i) |

The substrate measures; userland transitions. Substrate's
automatic `:nil` runs ONLY after graceful main-return.

**Protocol terminal states (post-pass-13):**

| `(readln)` returns | Meaning | Handling |
|---|---|---|
| `Some(:wat::core::nil)` | peer announced graceful done | exit loop; substrate emits our `:nil` on main-return |
| `Some(other)` | peer sent data | process; respond; loop |
| `None` | fd 0 closed without prior `:nil` | ungraceful (SIGKILL, escaped panic); user chooses panic / log / exit |

**Doctrines:**

- **Structured-stderr-only.** Inside wat-land, fd 2 ONLY ever
  carries panic-cascade EDN. No "regular text" on stderr. wat-cli
  has zero direct stderr writes (load failures, freeze errors all
  route through StdErrService → cascade + exit). Pretty-printing
  is downstream.
- **Single-shot panic.** `(:wat::runtime::panic! ...)` blocks;
  thread sends panic event; service drains; emits cascade; calls
  `libc::exit(N)`. Concurrent panics never get processed (process
  dies after first); other threads die with the process. No
  multiplexing.
- **Console retires.** Today's `:wat::console::Console`
  (crossbeam-based; arc 109 slice K.console) was a wat-level
  service for in-thread output mediation. Substrate now provides
  this for free via StdOutService. Console-the-concept dies;
  tests using it migrate to StdOutService.
- **Server / Client unify across tiers.** `Server/run handler`
  is the canonical service-loop pattern. `Client/spawn` returns
  a Client handle with typed send/recv. Today's slice 1c PipeFd
  Sender/Receiver substrate becomes the INTERNAL implementation
  Server/Client uses for serialization across pipe boundaries.
- **`:user::process` retires entirely.** There's only `:user::main`
  for OS-boundary CLI entry; `Server/run handler` for service-loop
  pattern. Spawn-process invokes a fn `[] -> :wat::core::nil`;
  the spawned program's `:user::main` calls `Server/run` to enter
  service mode (or does one-shot work and returns nil).

**Variant table — wat-level user surface vs substrate transport:**

The previous variant table showed `Sender<T>` / `Receiver<T>` as
the user-visible IPC. Post-architecture-lock-in: the user-visible
surface is `Server` / `Client` abstractions; Sender/Receiver
become substrate-internal.

| Tier | Variant | User-visible API | Substrate transport |
|---|---|---|---|
| **1** | Thread | `Server/run` (in-thread) + `Client` handle | crossbeam channels |
| **2** | Process | `Server/run` (in fork) + `Client` handle | EDN-over-pipes via StdIn/Out services |
| **3** | Remote *(future)* | `Server/run` (remote) + `Client` handle | EDN-over-sockets (Q-channel multiplex) |

Same wat-level abstraction at every tier; substrate differs
internally.

---

## The API — `spawn-* fn`

Every spawn primitive takes a fn directly. The fn IS the program.
No discovery, no Program wrapper type, no entry-keyword — the
substrate uses the fn's own definition + its closure as the
program description.

```scheme
;; Thread — runs in parent's world; closures over let-scope work
(:wat::core::let
  [worker-fn (:my::factory my-config)
   thr (:wat::kernel::spawn-thread worker-fn)]
  ...)

;; Process — substrate extracts fn + dep closure; forks OS process
(:wat::core::let
  [worker-fn (:my::factory my-config)
   proc (:wat::kernel::spawn-process worker-fn)]
  ...)

;; Remote (future arc) — substrate serializes fn + closure;
;; ships over wire; remote freezes + invokes; returns RemoteProgram<I,O>
(:wat::core::let
  [worker-fn (:my::factory my-config)
   rp (:wat::kernel::spawn-remote-program worker-fn endpoint)]
  ...)
```

The asymmetry is **internal**:

- spawn-thread runs the fn in parent's world. No serialization needed.
- spawn-process closure-extracts the fn (free symbols, deps, captured
  values, portability check), bundles into Vec\<WatAST\>, forks via
  the existing `fork-program-ast` pathway.
- spawn-remote-program (future) does the same closure extraction,
  serializes the resulting Vec\<WatAST\> to EDN bytes, ships over a
  socket, remote freezes + invokes.

**Closure extraction is substrate-internal Rust plumbing.** It does
NOT surface as a wat-level value type or verb in arc 170. Future
remote-program arc may expose it (for serialization to disk) at
that arc's discretion.

---

## Why this arc is bigger than "add argv"

The conversation thread settled the architecture across roughly a
dozen exchanges. The chain:

1. `:user::main` initially appeared to need an `:ExitCode` return
   to communicate exit codes to OS beyond panic-vs-no-panic.
   REALIZATIONS pass 10 reverses this: panic-cascade via
   StdErrService maps to `libc::exit(N)`; clean nil-return maps to
   `libc::exit(0)`; user code never participates. **Nil IS the
   exit code.** Same shape as arc 114's `Program<I,O> -> :nil`.
2. Per arc 114, Program\<I,O\> bodies return `:nil`; values flow
   through channels/pipes. Pass-10 lock-in extends this to
   `:user::main` itself — uniform `[] -> :nil` across tier 0/1/2.
3. `:user::main` (CLI) and Program-spawned entries share the same
   return type but differ in argument shape (CLI: argv ambient;
   Program: typed channels in signature).
4. spawn-process, fork-program, fork-program-ast all currently
   invoke `:user::main`. They must switch to a Program-shaped entry
   contract.
5. The "name discovery" path (substrate looks up a canonical entry
   symbol) creates ceremony. The user's preference: **the fn IS the
   program**; pass it directly; substrate handles closure extraction
   internally.
6. spawn-thread already accepts a fn (or keyword path); it stays
   structurally the same. spawn-process gains the same surface — fn
   in, Process out.
7. fork-program* renames to spawn-process* for verb family
   consolidation.
8. spawn-program (in-thread fresh-world variant) retires entirely —
   two-mode taxonomy (parent's world OR forked) is honest.
9. wat-cli passes std::env::args() to `:user::main` (pure
   passthrough; no flag filtering).

Each decision implies the next. No subset can ship honestly without
leaving the substrate in a half-state.

---

## What ships

### 1. `:user::main` signature update

```scheme
;; before (pre arc 170)
(:wat::core::defn :user::main
  [stdin  <- :wat::io::IOReader
   stdout <- :wat::io::IOWriter
   stderr <- :wat::io::IOWriter]
  -> :wat::core::nil
  ...)

;; after (arc 170 lock-in, post REALIZATIONS pass 10)
(:wat::core::defn :user::main [] -> :wat::core::nil
  ...)
```

Stdio params drop to ambient services (StdInService /
StdOutService / StdErrService); argv drops to ambient
`:wat::runtime::argv`; return type stays `:nil` — **nil IS the
exit code**. Substrate maps nil-return to `libc::exit(0)`;
panic-cascade maps to `libc::exit(N)`. User code never
participates in exit-code arithmetic.

argv layout (pure passthrough — what the binary received is what
the program sees), accessed via `:wat::runtime::argv`:

| Position | Contents |
|---|---|
| `argv[0]` | Path to the wat binary |
| `argv[1]` | Path to the wat source file |
| `argv[2..N]` | Subsequent whitespace-delimited args |

`:user::main` is the **only strict name** in arc 170. Substrate
enforces this exact name + `[] -> :wat::core::nil` signature at
freeze.

### 2. `:user::process` and `:user::thread` — documentation contracts

These are NOT strict names. They're contract names used in
documentation; the substrate enforces only structurally (signature
match). Programs name their actual entries whatever they want
(`:my::accountant::loop`, `:my::ddb::worker`, etc.); fns satisfying
the signature satisfy the contract.

```scheme
;; :user::process contract — the SHAPE the user's fn must satisfy
;;   (post arc 170 + typed-channel substrate)
[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>]
  -> :wat::core::nil

;; :user::thread contract — same shape
[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>]
  -> :wat::core::nil

;; example: a defn satisfying :user::process
(:wat::core::defn :my::worker
  [rx <- :wat::kernel::Receiver<i64> tx <- :wat::kernel::Sender<i64>]
  -> :wat::core::nil
  (... loop receiving from rx, sending to tx ...))
```

### 3. `spawn-process` / `spawn-process-ast` — verb consolidation + fn input

Renames + reshapes:

| Pre-arc-170 | Post-arc-170 |
|---|---|
| `(:wat::kernel::fork-program src scope)` invoking `:user::main` | `(:wat::kernel::spawn-process fn)` — fn satisfies `:user::process` contract; substrate closure-extracts and forks |
| `(:wat::kernel::fork-program-ast forms scope)` invoking `:user::main` | `(:wat::kernel::spawn-process fn)` — same canonical surface; AST-of-fn IS the input |
| `(:wat::kernel::spawn-program src scope)` (in-thread Process) | DELETED — two-mode taxonomy (parent's world OR forked) |
| `(:wat::kernel::spawn-program-ast forms scope)` | DELETED |

Note the consolidation: after arc 170, there's ONE spawn-process
verb. It takes a fn. Substrate does the heavy lifting (closure
extraction; fork). No `-ast` variant — the fn's body IS the AST
already; no separate AST-input verb needed.

### 4. `spawn-thread` — contract naming, no behavioral change

```scheme
;; UNCHANGED behavior
(:wat::kernel::spawn-thread my-fn)   ;; -> :Thread<I,O>
```

spawn-thread already accepts a fn (or keyword path). The fn's body
runs in parent's world; closures over let-scope work; services
pattern preserved.

The arc 170 change to spawn-thread is naming the contract
`:user::thread` in the documentation. Substrate's existing
structural enforcement is unchanged.

### 5. wat-cli plumbs `std::env::args()` into ambient `:wat::runtime::argv`

```rust
// crates/wat-cli/src/lib.rs::run
let argv: Vec<String> = std::env::args().collect();
// ... existing flag parsing ...
runtime::set_argv(argv);          // ambient :wat::runtime::argv
runtime::start_services();        // StdInService / StdOutService / StdErrService
invoke_user_main(&world)?;        // signature is `[] -> :nil`; clean return → exit 0;
                                  // panic-cascade routes through StdErrService → libc::exit(N)
std::process::exit(0);
```

Pure passthrough. wat-cli's flags (`--check`, `--check-output`)
appear in argv if passed; they short-circuit before `:user::main`
runs (so user's main never sees them when they're set).

wat-cli has no exit-code value plumbed from main's return —
**nil IS the exit code**. Clean nil-return → libc::exit(0); panic
event drained by StdErrService → libc::exit(non-zero) from inside
the service's emit path; user code never participates in
exit-code arithmetic.

### 6. Substrate-internal closure extraction (Rust capability)

Not exposed at wat level in arc 170. Internal Rust capability that
spawn-process uses to package a fn for the child process.

Detailed in [`CLOSURE-EXTRACTION.md`](./CLOSURE-EXTRACTION.md) — the
algorithm, the portability check, the test strategy.

The same capability sets up future remote-program (which will
serialize the extracted closure to EDN bytes for over-the-wire
transport).

---

## Substrate impact (full table)

| Surface | Pre-arc-170 | Post-arc-170 |
|---|---|---|
| `:user::main` signature | `(IOReader IOWriter IOWriter) -> :nil` | `[] -> :wat::core::nil` (ambient runtime; nil IS the exit code) |
| `:user::process` contract | n/a | documentation contract — `[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil` |
| `:user::thread` contract | n/a | documentation contract — `[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil` |
| `:wat::runtime::current-thread` | n/a | thread-local id (ambient value) |
| `:wat::runtime::argv` | n/a | set-once at process start (ambient value) |
| `:wat::kernel::StdInService` | n/a | substrate runtime service; owns fd 0; serves typed Values |
| `:wat::kernel::StdOutService` | n/a | substrate runtime service; owns fd 1; receives typed Values |
| `:wat::kernel::StdErrService` | n/a | substrate runtime service; owns fd 2; first-panic-wins; libc::exit |
| `:wat::kernel::Server` / `Client` types | n/a | substrate-internal + tier 1/2/3 unification; tested + documented; unsurfaced in canonical form |
|  `:wat::kernel::println` | n/a | helper; writes data + newline to per-thread stdout client |
|  `:wat::kernel::readln` | n/a | helper; reads line + parses EDN to :wat::holon::Atom from per-thread stdin client |
| `:wat::kernel::StdIn/client` | n/a | escape hatch; returns per-thread Client (advanced cases) |
| `:wat::kernel::StdOut/client` | n/a | escape hatch; returns per-thread Client (advanced cases) |
| `:wat::kernel::main!` macro | n/a | substrate-auto-loaded; expands to canonical server-program form (uses helpers; no explicit client binding) |
| `:wat::kernel::run!` macro | n/a | substrate-auto-loaded; expands to one-shot main wrapping forms in implicit-do |
| `:wat::kernel::Program<I,O>` typealias | aliases `:Process<I,O>` (arc 109 § J slice 10a) | UNCHANGED — kept as a typealias for now; future arc may unify Thread + Process under a real Program supertype |
| `:wat::kernel::fork-program` | live; calls `:user::main` | DELETED → spawn-process |
| `:wat::kernel::fork-program-ast` | live | DELETED → consolidated into spawn-process(fn) |
| `:wat::kernel::spawn-program` | live; in-thread Process | DELETED |
| `:wat::kernel::spawn-program-ast` | live | DELETED |
| `:wat::kernel::spawn-process` | n/a | minted; takes fn; invokes via closure-extracted forms in forked process |
| `:wat::kernel::spawn-thread` | takes fn; runs in parent world | UNCHANGED behaviorally; thread now registers with services before handle-return |
| Closure extraction (Rust internal) | n/a | minted; substrate Rust capability used by spawn-process |
| `validate_user_main_signature` | enforces 3-arg + nil | enforces `[] -> :wat::core::nil` |
| wat-cli argv plumbing | empty Vec | sets `:wat::runtime::argv` ambient |
| wat-cli exit code handling | exit 0 / 1 | exit 0 on clean nil-return; libc::exit(N) from StdErrService on panic-cascade |
| Today's `:wat::console::Console` | live (crossbeam-based) | RETIRES — replaced by StdOutService |

---

## Settled design (from conversation thread 2026-05-09)

### Nil IS the exit code (no ExitCode type — superseded 2026-05-10)

Earlier in the conversation, an `ExitCode = :wat::core::u8`
typealias was settled. REALIZATIONS pass 10 reverses that:
panic-cascade via StdErrService maps to `libc::exit(N)`; clean
nil-return maps to `libc::exit(0)`. The user's framing: *"we
program for the case that assumes we'll never panic.. that's how
we demonstrate our code"* + *"nil /is/ the exit code"*.

`:user::main` signature reaches `[] -> :wat::core::nil` —
identical to arc 114's `Program<I,O> -> :nil` shape. Uniform
across tier 0/1/2. ExitCode typealias drops from arc 170 scope
(future arc may mint a helper if a CLI tool genuinely needs
0/1/2 exit-code distinction; arc 170 affirmatively does not).

### argv only at the CLI boundary

Argv is OS-level concept. Substrate-spawned programs have client/
server semantics with channel/pipe IPC. `:user::process` and
`:user::thread` don't take argv.

### Server-by-fn, not server-by-name

Per "convenience broke through incorrectly... brutal rigidity
brings the paradoxical unbounded flexibility if you play by the
rules" + "the shape of those functions /is/ the contract."

Each spawn primitive takes a fn directly. The fn IS the program.
No canonical lookup name; no entry-keyword; no Program wrapper
type. Substrate uses the fn's own definition + its closure as the
program description.

### Client / server framing

The unifying mental model. `:user::thread`, `:user::process`,
`:user::remote-program` are all server roles; each has its own
entry contract reflecting its IPC mechanism. The spawning context
is the client.

### Closure extraction is Rust-internal in arc 170

Per "we can build this first then use it as a dependency in the
rest of the re-work." The closure extraction logic is the
load-bearing substrate capability. Internal-only at arc 170; future
remote-program arc may expose it at the wat level if useful.

### The fn IS the program

Per "i think 'forms' is just equivalent of what :user::main is.
the forms /are/ the function being called on the inputs it must
receive." spawn-process takes a fn; substrate handles all
internals; no Program wrapper type at wat level.

---

## Out of arc 170 scope (affirmative)

Each item below is a future arc. Numbers reserved when the work
begins.

### `:user::remote-program` entry contract + spawn-remote-program primitive

scratch/2026/05/007-remote-program/ articulates the full
RemoteProgram architecture: typed-capability-bridge framing,
Q-channel multiplexed `Result<T, E>` wire protocol, four-tier IPC
(UDS / localhost HTTP / TLS / mTLS), seven settled questions.

Arc 170's closure extraction substrate capability sets up the
remote-program work — once a fn can be turned into portable
Vec\<WatAST\>, those bytes can go anywhere bytes go (forked process
in arc 170; over a wire in the remote-program arc).

The remote arc adds: transport (sockets, auth, mTLS), wire
protocol (Q-channel multiplex), endpoint addressing, the
`:user::remote-program` contract.

### Real `Program<I,O>` supertype unification

Today `Program<I,O>` is a typealias for `Process<I,O>`. Thread\<I,O\>
+ Process\<I,O\> remain separate concrete types. A future substrate
arc may unify them under a real Program supertype (abstracting over
IPC). Arc 170 keeps the existing types unchanged.

### wat-cli-options DSL + `user:` subcommand convention

scratch/2026/05/019-wat-cli-options/ captures both. Out of arc
170 — argv parameter lands; users hand-roll parsing or build their
own helpers until that arc opens.

### Wat-level closure extraction verb

If users want to serialize a fn to disk or build their own
transport (other than fork or remote-program), exposing the closure
extraction primitive at the wat level becomes useful. Arc 170 keeps
it Rust-internal; later arc may surface it.

---

## Slice plan

> **Status (2026-05-10):** Major plan amendment after architecture
> lock-in conversation (REALIZATIONS passes 7-13). Slices 1e/1f/1g/
> 1h/1i NEW substrate slices. Slice 3 phase A + B + 1d work in
> dirty tree on `arc-170-program-entry-points` mostly stays valid
> (closure-extraction substrate is correct); some elements need
> revision against new doctrine (`:user::main` 4-arg signature
> drops to `[] -> :wat::core::nil` per pass 10; Server/Client
> wraps slice 1c PipeFd substrate; testing-lib reshapes around
> services).

Mirrors the substrate-as-teacher pattern from arcs 167 / 168 /
169. The arc has grown into a substantial substrate-foundation
refactor — appropriate per arc 109's "make the foundation
impeccable" doctrine + the canonical server form (REALIZATIONS
pass 8).

### Slice 1 — closure extraction (Rust substrate; zero callers) — SHIPPED

Build the foundation primitive in Rust. Zero wat-level callers
initially; testable in isolation.

- Free-symbol walker
- Dep-closure builder (recursive)
- Value→AST encoder (extending existing struct→form)
- Portability type-check (channel-bearing values refused)
- Rust integration tests verifying extracted forms re-freeze
  correctly

Detailed in [`CLOSURE-EXTRACTION.md`](./CLOSURE-EXTRACTION.md).

**Shipped**: commit `787c977` + SCORE `bb155ed` (14/14 pass,
Mode A clean). **However** — review surfaced that the public
shape of `ClosurePackage { forms, entry }` carries the
entry-keyword ceremony DESIGN explicitly killed (lines 102-108
+ 484-509). See
[`REALIZATIONS-SLICE-1.md`](./REALIZATIONS-SLICE-1.md) for the
discipline lesson + slice 1b origin.

Predicted (original): 90-180 min opus. Actual: ~150 min Mode A
clean.

### Slice 1b — `ClosurePackage` reshape ("the fn IS the program") — SHIPPED

Restructures `ClosurePackage` from `{ forms, entry: String }` to
`{ prologue: Vec<WatAST>, entry_form: WatAST }`. Retires the
synthetic-name machinery (`:__closure::__pkg_<n>` counter +
wrap-in-define). The fn-form AST evaluates to a fn Value
directly; no naming required.

**Shipped**: commits `a23acf3` + `365343f` + SCORE `84b6ca6`
(17/17 pass, Mode A clean, ~40 min — under predicted 60-120 min
band). One substantive honest delta surfaced: keyword-path
`entry_form` ships as `WatAST::Keyword`, not `WatAST::Symbol`,
because eval resolves bare-Symbol via `env.lookup` (lexical only)
while top-level defns require keyword resolution via `sym.get`.
Spec intent ("name reference that evaluates to fn Value")
preserved; surface adjusted for substrate-fit.

### Slice 1c — typed-channel-over-EDN-pipes substrate + Process<I,O> reshape

Tier 2 substrate plumbing. Mints the EDN-encoded-pipe transport
behind `Sender<T>` / `Receiver<T>` so the user-visible abstraction
is uniform across tier 1 (crossbeam) and tier 2 (pipes). Reshapes
`:wat::kernel::Process<I,O>` to expose typed-channel handles
instead of byte-pipe handles, per the doctrine that strings stay
at substrate-internal transport boundaries.

- `Sender<T>` / `Receiver<T>` Value variants extended (or
  multimethod-dispatched) to support pipe-fd transport with
  EDN encoding/decoding at the boundary
- `:wat::kernel::Process<I,O>` struct reshape:
  `{ stdin :IOWriter, stdout :IOReader, stderr :IOReader, handle }`
  → `{ tx :Sender<I>, rx :Receiver<O>, handle :ProgramHandle }`
- All Process-callers in `src/fork.rs`, `src/spawn.rs`, etc.
  updated for new field shape
- Errors propagate via `Process/join-result` (existing arc 113
  cascade pattern); stderr-as-separate-channel drops at the
  user-visible level
- Rust integration tests verifying typed-channel-over-pipes
  round-trips end-to-end (parent sends typed Value; child
  receives typed Value; bytes are EDN-encoded transport detail)

Zero wat-level surface change in this slice — pure substrate
plumbing. The wat-level verbs that USE this infrastructure
(`spawn-process`, etc.) land in slice 2.

Predicted: 90-180 min opus.

### Slice 2 — substrate consumer (uses slices 1b + 1c) — SHIPPED with caveats

- `:wat::kernel::ExitCode` typealias — SHIPPED but **retires in
  slice 1e per pass 10 lock-in** (nil IS the exit code)
- `:user::main` signature update + validator — SHIPPED at 4-arg
  shape; **slice 1e revises to `[] -> :wat::core::nil` per pass 10**
- `eval_kernel_spawn_process` minted — SHIPPED
- `eval_kernel_fork_program*` / `eval_kernel_spawn_program*` arms
  unchanged during sweep window — slice 4 retires
- wat-cli `std::env::args()` passthrough — SHIPPED at 4-arg shape;
  **slice 1e revises to ambient `:wat::runtime::argv`**
- Three substrate-as-teacher walkers — SHIPPED
- 11 integration tests in `tests/wat_arc170_program_contracts.rs`
  — SHIPPED; slice 3 sweep migrates against new shape

**Shipped:** commit `09d7b04` + SCORE `9879e3b` (19/19 rows pass,
Mode A clean, ~180 min). Workspace ships RED at 1594/545 — exactly
the substrate-as-teacher input slice 3 sweeps.

### Slice 1d — closure-extraction walker substrate fixes — SHIPPED

Walker scope-tracking extended to handle match-arm pattern
bindings + wildcards. ~17 min Mode A clean. Workspace 2015/119
post-1d.

**Shipped:** commits in dirty tree (uncommitted; bundles into
slice 3 atomic commit per recovery doc § 7).

### Slice 1e — ambient runtime + drop stdio params + retire ExitCode (NEW; substrate)

Implements REALIZATIONS pass 7 lock-in (ambient runtime) +
pass 10 lock-in (nil IS the exit code; ExitCode retires):
- Mint `:wat::runtime::current-thread` (thread-local id) and
  `:wat::runtime::argv` (set-once at process start) ambient values
- Retire `:wat::runtime::stdin/stdout/stderr` ambient handles
  (pass-7 midpoint; pass-9 supersedes — services own them)
- Drop stdio params from `:user::main` signature: `[] -> :wat::core::nil`
  (pass 10 — same shape as `:user::thread`/`:user::process`)
- Drop stdio params from `:user::process` (which retires entirely
  in favor of Server pattern per pass 8)
- **Retire `:wat::kernel::ExitCode` typealias from arc 170 scope**
  (slice 2 shipped it; slice 1e drops it). Nil-return = exit 0;
  panic-cascade emits + libc::exit(N). Future arc may revisit if
  a CLI tool genuinely needs 0/1/2 exit-code distinction.
- Update `expected_user_main_signature` / `validate_user_main_signature`
  to enforce `[] -> :wat::core::nil`
- No new typealias-as-constructor work — `:wat::core::nil` already
  works at both type and value positions (per WAT-CHEATSHEET
  line 61-74); pass 10 dropping ExitCode means we don't need to
  generalize the pattern in arc 170.
- Update wat-cli to use ambient access (no parameter passing for
  stdin/stdout/stderr; no exit-code value plumbed from main's
  return)
- Update spawn-process child invocation (child returns nil)
- Walker `BareLegacyMainSignature` updates to fire on the OLD
  4-arg shape (now legacy)

Predicted: 60-120 min opus.

### Slice 1f — three substrate services (NEW; substrate)

Implements REALIZATIONS pass 9:
- Mint `:wat::kernel::StdInService` Rust runtime component
  (owns fd 0; select-loop; per-thread consumer pipes; control-pipe;
  decodes EDN; serves typed Values; returns :None on EOF)
- Mint `:wat::kernel::StdOutService` (owns fd 1; per-thread
  message-pipes; serializes EDN; writes single-writer)
- Mint `:wat::kernel::StdErrService` (owns fd 2; per-thread
  panic-pipes; first panic wins; emits cascade; libc::exit)
- Substrate runtime startup creates all three services BEFORE
  `:user::main` invokes
- Self-pipe trick / control-pipe for dynamic select-set updates
- Rust integration tests verifying each service's contract

Predicted: 180-300 min opus. This is substantial substrate work.

### Slice 1g — spawn-thread register-with-services (NEW; substrate)

Implements REALIZATIONS pass 9 thread-registration contract +
pass 11 per-thread-Client thread-locals:
- spawn-thread MUST: create per-thread pipes for In/Out/Err
  services; send `:register thread-id reader-end` to each
  service's control-pipe; wait for ack; store writers in
  thread-locals; **construct per-thread `:wat::kernel::Client`
  values for stdin/stdout and store in thread-locals (pass 11)**;
  THEN return Thread<I,O> handle to caller
- ack-before-return prevents races (new thread might panic
  before services know about it)
- `:wat::runtime::current-thread` reads from thread-local
- Per-thread stdin Client + stdout Client read from thread-locals
  (used by `println` / `readln` helpers + `StdIn/client` /
  `StdOut/client` escape hatches)
- Integration tests for register-then-spawn-then-panic flow

Predicted: 90-180 min opus. Touches spawn-thread substrate +
ProgramHandle plumbing + thread-local Client construction.

### Slice 1h — Server / Client substrate + helpers + escape hatches + entry-point macros (NEW; wat-level + substrate)

Implements REALIZATIONS pass 8 + pass 11 lock-in:
- Mint `:wat::kernel::Server` and `:wat::kernel::Client`
  substrate types (used internally + tier 1/2/3 unification)
- `(:wat::kernel::server-loop handler)` — the canonical
  service-loop fn body (post-pass-11; takes only handler;
  uses ambient per-thread Client via helpers)
- Slice 1c PipeFd Sender/Receiver substrate becomes the
  IMPLEMENTATION DETAIL Server/Client uses
- **User-facing helpers (post-pass-12; the canonical surface):**
  - `(:wat::kernel::println v)` → `:wat::core::nil`
    — writes data + newline via per-thread stdout Client; blocks
  - `(:wat::kernel::readln)` → `:Option<:wat::holon::Atom>`
    — reads line + parses EDN to HolonAST via per-thread stdin
    Client; blocks; `:None` on fd 0 closed (ungraceful)
- **Advanced escape hatches** (return per-thread Client values
  from thread-locals; users rarely need; substrate honest):
  - `(:wat::kernel::StdIn/client)` → `Client`
  - `(:wat::kernel::StdOut/client)` → `Client`
- The canonical server form (REALIZATIONS pass 13) lives + works
  with `(stopped?)` polling + three-branch `(readln)` match
  (Some(:nil), Some(req), None) + tail-recursive `(server-loop
  handler)` handled by arc 003's trampoline
- **Entry-point helper macros** (encourage the canonical pattern):
  - `(:wat::kernel::main! handler-expr)` — macro that expands
    to the canonical server-program shape. `handler-expr` can
    be ANY expression that evaluates to
    `:wat::core::fn(wat::holon::Atom)->wat::holon::Atom`:
    a keyword path (`:my::handler`), an inline fn-form,
    a factory call `(make-handler config)`, etc. The macro
    evaluates the expression at startup and binds the resulting
    fn into the server-loop. Expansion (post-pass-13; no
    explicit client binding — server-loop uses ambient helpers
    internally; the loop body includes `(stopped?)` poll +
    three-branch `(readln)` match per pass-13 protocol):
    ```scheme
    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:wat::core::let
        [handler handler-expr]
        (:wat::kernel::server-loop handler)))
    ```
  - `(:wat::kernel::run! form1 form2 ...)` — variadic macro
    that expands to a one-shot main wrapping the forms in an
    implicit-do. The last form's value flows through; if it
    returns nil, signature satisfied; otherwise freeze
    diagnostic catches it. For CLI utility programs:
    ```scheme
    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:wat::core::do form1 form2 ...))
    ```
  - Both live in substrate-auto-loaded stdlib (`wat/kernel/main.wat`
    or similar). Users don't `load!` them explicitly.
- Integration tests showing the 3-line user program works
  end-to-end across the supported handler-expr shapes:
  ```scheme
  ;; Factory pattern
  (:wat::core::load! "some-lib.wat")
  (:wat::kernel::main! (make-handler))

  ;; Keyword path
  (:wat::core::load! "some-lib.wat")
  (:wat::kernel::main! :my::handler)

  ;; Inline lambda
  (:wat::kernel::main!
    (:wat::core::fn [req <- :MyReqType] -> :MyRespType
      (... handle req ...)))

  ;; CLI script — last form returns nil; signature satisfied
  (:wat::kernel::run!
    (:wat::kernel::StdOutService/send "hello world"))
  ```

Predicted: 90-180 min mixed (opus design + sonnet wat helpers).

### Slice 1i — wat-cli exit-path discipline (NEW; substrate)

Implements REALIZATIONS pass 9 (structured-stderr-only) +
pass 13 (graceful-`:nil` epilogue) doctrines together. Both are
exit-path concerns; pair them for atomic landing.

**Structured-stderr-only (pass 9):**
- wat-cli has zero direct stderr writes
- All wat-cli "errors" (load failures, freeze errors, etc.) route
  through StdErrService → structured cascade EDN → libc::exit
- panic-cascade emit on fd 2 from Rust (replaces slice 2's flat
  marker); uses arc 113 cascade pattern via StdErrService

**Graceful-`:nil` epilogue (pass 13):**
- Substrate's exit path after `:user::main` returns nil:
  1. emit `:wat::core::nil` to fd 1 (protocol-compliance final)
  2. close fd 1
  3. libc::exit(0)
- Independent of WHY main returned (clean completion, observed
  `(stopped?)` and unwound, finished one-shot)
- Panic exit skips this path (StdErrService cascade fires
  libc::exit(N) directly; consumer sees ungraceful `None`)
- The substrate emits `:nil` on the program's behalf — adheres to
  protocol on the user's behalf; user just returns nil

**Signal model preserved (per arc 106 + memory `project_signal_cascade.md`):**
- OS handlers flip atomic flags only; substrate does NOT
  auto-trigger main-return on signal
- User polls `(stopped?)` etc.; user owns cleanup logic; user
  decides when to return nil from main
- Substrate's epilogue runs only AFTER user returns nil

**Tests:**
- Shell-level UX (cargo test invokes wat binary; verifies stderr
  is structured EDN; verifies stdout has trailing `:nil` + close)
- Hermetic test harness reads `Some(:nil)` as graceful-done marker
- Panic case: ungraceful `None` on stdout; cascade EDN on stderr;
  non-zero exit code

Predicted: 90-180 min opus (expanded from 60-120 to absorb the
epilogue work).

### Slice 3 — consumer sweep + tooling rebuild — REVISED post-architecture-lock-in

> **Status (2026-05-10):** Phase A + B + 1d work in dirty tree
> partially aligns; needs revision against locked-in
> architecture (REALIZATIONS passes 7-13). After slices 1e through
> 1i ship, slice 3 sweep finishes the migration: testing-lib
> rebuilds around services + canonical server form (REALIZATIONS
> pass 8); test fixtures use ambient runtime + Server/Client; all
> stderr-text assertions migrate to structured cascade or stdout.

Arc 170 changes the spawn surface (tier 1/2/3 framework; fn-input
contract; closure-extraction tier-bridging at tier ≥ 2). Every
existing user-facing thing that interfaces with the spawn family
MUST reach its polished form on the new substrate — not just
functionally work, but **as good as the new substrate allows**.

**Mechanical sweep (sonnet):**

- All `:user::main` definitions:
  - CLI-only programs: migrate to new signature `[] -> :wat::core::nil`;
    drop stdio params; reach for `:wat::runtime::argv` if needed;
    reach for StdOutService for output
  - Substrate-spawn-target programs: rewrite as anonymous fn
    satisfying `:user::process` contract; callers pass the fn
    directly to spawn-process
  - Both contexts: define both
- `fork-program*` callsites → `spawn-process(fn)` shape
- `spawn-program*` callsites → migrate to `spawn-process(fn)` (real
  fork) OR `spawn-thread(fn)` (parent world; services pattern)

**Tooling rebuild — testing-lib three-layer API (orchestrator + sonnet pair, more than mechanical):**

The substrate (`spawn-process fn`) is the full-power form. The
testing lib's job is to hide constant ceremony for typical test
usage. Three layers per [`TIERS.md`](./TIERS.md):

- **Layer 1 — `(:wat::test::run-hermetic body)`** — the 90% case.
  `run-hermetic` is a macro; user writes the body directly; the
  fn-wrapper (`(:wat::core::fn [] -> :wat::core::nil body)`) is
  generated. No channels in the signature, no inputs, no scope,
  no fn ceremony at all. Returns `RunResult`.
- **Layer 2 — `(:wat::test::run-hermetic-with-io<I,O> inputs body)`** —
  the 9% case. Macro introduces `rx :Receiver<I>` and
  `tx :Sender<O>` as bindings in scope of `body`; harness feeds
  Values via rx, drains Values via tx, returns parsed outputs.
  **Typed channels, not byte streams.** Substrate handles
  EDN-over-pipes encoding internally.
- **Layer 3 — `(:wat::kernel::spawn-process fn)`** — the
  substrate; full surface for production code. Caller writes an
  explicit fn-form with typed-channel signature. Tests don't
  reach here unless they really need it.

What disappears from EVERY testing layer:

- `scope :Option<String>` — leaked substrate plumbing; today's
  hermetic.wat errors on `:Some`; not functional anyway. Drops.
- `forms :Vector<WatAST>` — caller writes a fn directly; no AST
  construction.
- **`stdin :Vector<String>` and stdout/stderr as `Vec<String>`** —
  string-shaped IO drops from every testing layer. The user
  works in typed Values; the substrate handles EDN encoding at
  the pipe boundary.
- **`IOReader`/`IOWriter`** — byte-stream types drop from every
  testing layer. Layer 2's fn takes `Receiver<I>`/`Sender<O>`
  (typed channels); the substrate's pipe-fd plumbing is hidden.

What disappears from Layer 1:

- The fn's channel parameters (Layer 2 has them when needed)
- The input data parameter (Layer 2 has it when needed)
- **The fn-form wrapper itself** — `run-hermetic` is a macro;
  user writes the body directly; macro generates
  `(:wat::core::fn [] -> :wat::core::nil body)` internally. The
  empty params + nil return are constant ceremony hidden from
  every Layer 1 caller.

What disappears from Layer 2:

- **The fn-form wrapper** — `run-hermetic-with-io` is a macro;
  it introduces `rx` and `tx` as bindings in the body scope.
  User writes body using rx/tx directly; macro generates the
  fn-form with typed-channel signature.

**Migration scope:**

- **`wat/std/hermetic.wat` retires.** Replaced by
  `:wat::test::run-hermetic` (Layer 1) + `:wat::test::run-hermetic-with-io`
  (Layer 2) under the testing namespace. Path may move to
  `wat/test/` or similar; slice 3's BRIEF settles it.
- **`wat/test.wat`** — references fork-program-ast; same
  three-layer treatment per its own surface.
- **All callers of `run-sandboxed-hermetic-ast`** — classify by
  which layer they need; migrate to the appropriate layer.
  Expected distribution: most → Layer 1 (massive UX collapse);
  some → Layer 2; rare/none → Layer 3.
- **Any other stdlib that wraps the spawn family** — same
  three-layer treatment; reach the polished form.

The slice is sonnet-mechanical for the verb renames + signature
updates, but the hermetic tooling rebuild + caller migration is
orchestrator-judged. Slice 3's BRIEF (drafted post-slice-1b +
post-slice-2) splits these explicitly.

Predicted: 90-180 min sonnet for the mechanical sweep; orchestrator
time for the hermetic rebuild + caller migration TBD when slice 3
is briefed.

### Slice 4 — substrate retirement (opus + sonnet pair)

**This slice retires every bandaid arc 170 carried during its
sweep window.** The arc cannot close with bandaids in place;
INSCRIPTION reflects the final correct shape. Arc 170 INSCRIPTION
must be free of any "future arc retires X" deferral language
per FM 11.

**Bandaid inventory (2026-05-10 lock-in):**
- Process<I,O> legacy 3 byte-pipe fields (stdin/stdout/stderr) —
  slice 1c additive shape
- Empty stdout in run-hermetic's RunResult (slice 3 phase A)
- deftest-hermetic alias-of-deftest (slice 3 phase A)
- `sandbox_scope_leak_fires_with_diagnostic` `#[ignore]` (slice 3 phase A)
- Slice 1c PipeFd Sender/Receiver substrate exposed at wat level
  (becomes Server/Client internal-only in slice 1h)
- Today's `:wat::console::Console` crossbeam service (replaced
  by StdOutService in slice 1f; tests migrate in slice 3)

Walker + dispatch-arm retirements:
- `BareLegacyMainSignature` walker variant + Display + Diagnostic + body
- `BareLegacyForkProgram` walker variant + Display + Diagnostic + body
- `BareLegacySpawnProgram` walker variant + Display + Diagnostic + body
- Old `eval_kernel_fork_program*` / `eval_kernel_spawn_program*` arms
- `validate_user_main_signature` legacy 3-arg fall-through
- Vacuous walker-firing tests retired

**`:wat::kernel::Process<I,O>` legacy field retirement (slice 1c
bandaid retirement) — load-bearing for arc 170 close:**

Slice 1c shipped Process<I,O> ADDITIVE — legacy 4 fields
(stdin :IOWriter, stdout :IOReader, stderr :IOReader, handle)
PLUS new typed-channel fields (tx :Sender<I>, rx :Receiver<O>).
The legacy 3 byte-pipe fields (stdin, stdout, stderr) MUST
retire in slice 4 — that's the arc-close discipline. Slice 3's
testing-tooling rebuild migrates ALL callers to typed-channel
accessors; slice 4 destructively removes the legacy fields:

- `:wat::kernel::Process<I,O>` struct in `src/types.rs` —
  remove stdin / stdout / stderr fields; final shape is
  `{ tx :Sender<I>, rx :Receiver<O>, handle :ProgramHandle }`
- All Rust callers updated for 3-field shape (workspace breaks
  at substrate destructive-edit; sonnet sweeps any remaining
  callers slice 3 missed)
- All wat-side accessors (`Process/stdin`, `Process/stdout`,
  `Process/stderr`) retired
- Atomic-commit pattern: opus destructive (don't commit) →
  sonnet sweep (don't commit) → orchestrator commits both as
  ONE atomic commit when workspace = 0-failed

The bandaid was tolerable during the sweep window (kept slice 2
unblocked, kept stdlib tests green during slice 1c → slice 3
transition). It is NOT tolerable past arc close.

Predicted: 60-120 min opus + 30-90 min sonnet sweep =
~90-210 min total.

### Slice 5 — closure paperwork (orchestrator)

- SCORE-SLICE-1, SCORE-SLICE-2, SCORE-SLICE-3, SCORE-SLICE-4
- INSCRIPTION
- 058 changelog row (lab repo)
- USER-GUIDE update (Program client/server section + entry
  contracts + nil-IS-exit-code + argv + spawn primitives +
  closure extraction note + structured-stderr-only doctrine)
- ZERO-MUTEX cross-ref (no new mutex)
- CONVENTIONS doc update (entry-point naming convention)
- Atomic squash-merge to main

When slice 5 ships, **arc 109 v1 milestone closure unblocks**.

---

## Settled questions (conversation log compressed)

### Q1. spawn-program retires entirely. — settled (option A)

Per "A — retire." spawn-program / spawn-program-ast deleted. Two-
mode taxonomy: spawn-thread (parent's world) + spawn-process
(forked).

### Q2. *(retired — was based on wrong assumption)*

Q2 v1 asked about spawn-thread freeze cost; user correction:
threads share parent's memory (zero-mutex doctrine; services
pattern). Q retired.

### Q2-replacement. Contract names are documentation; only `:user::main` is strict. — settled

Per "the shape of those functions /is/ the contract.. i think the
only strict one is :user::main?.."

### Q3. argv pure passthrough. — settled

Per scratch/2026/05/019-wat-cli-options: "no silent argv reshaping;
what the binary received is what the program sees."

### Q4. Substrate-then-sweep parallel pattern. — settled

Per arcs 167 / 168 / 169 precedent. Slice 1 substrate ships;
slice 2 substrate consumer ships with walkers; slice 3 sweeps;
slice 4 retires. Same shape.

### Q5. spawn-process input shape — fn, not Program-typed value. — settled

Initial drafts proposed (src, scope, entry-keyword) tuple, then a
Program newtype with forms + entry. User refinement: the fn IS the
program; substrate handles closure extraction internally; no
wat-level Program type or entry-keyword; spawn-process takes a fn
directly.

### Q6. Closure extraction substrate scope — Rust-internal in arc 170. — settled

Per "we can build this first then use it as a dependency in the
rest of the re-work." Closure extraction is the load-bearing
substrate capability that enables both spawn-process AND future
spawn-remote-program. Built as a Rust capability in arc 170;
exposed at wat level later if needed.

---

## DESIGN-time conversation log (2026-05-09 → 2026-05-10)

The architecture settled across roughly 20 exchanges. Key beats:

1. User: "make :user::main accept argv"
2. Three return-type options presented (i64 / u8 / enum); user picked u8
3. User proposed `(:wat::kernel::exit N)` helper; orchestrator pushed back on libc-naming conflict; user picked typealias path (`:wat::kernel::ExitCode = :u8`)
4. Orchestrator scoped arc 170 narrowly (just argv); user pushed: spawn/fork-program don't need argv
5. User raised: forks/threads might not need :user::main at all
6. Orchestrator surfaced Program\<I,O\> + arc 114's "return nil" contract; arc 170 must split CLI from Program contracts
7. User proposed three named entries: `:user::main` (CLI), `:user::thread` (thread Program), `:user::process` (process Program); plus `:user::remote-program` future
8. User confirmed kill spawn-thread fn-ref-pattern; orchestrator surfaced verb consolidation: fork-program* → spawn-process*
9. User locked client/server framing
10. User confirmed "single very large arc with a bunch of slices"
11. User caught Q2 framing error: spawn-thread shouldn't reshape; threads share parent's memory; services pattern
12. User clarified contract-vs-name: only `:user::main` is strict; `:user::thread`/`:user::process` are documentation contracts
13. User settled Q1 (spawn-program retires); two-mode taxonomy
14. User questioned `child-src` exposing substrate plumbing (None scope param + raw String source); orchestrator pivoted toward AST-first
15. User locked AST-first: "we don't communicate strings - we communciate ast"
16. User questioned why entry-keyword is needed: "why do we even need a name if the forms /are/ the thing that matters?"
17. User refined further: the fn IS the program; spawn-process takes fn directly; no Program wrapper type; closure extraction is internal
18. User pressure-tested: does this set up remote-program? Confirmed yes — closure extraction + Vec\<WatAST\> wire form is the foundation; remote arc adds transport + protocol
19. **(2026-05-10) User raised three substrate services + structured-stderr-only + single-shot panic + canonical server form + main!/run! macros** (REALIZATIONS passes 7-9)
20. **(2026-05-10) User reversed beat #2/#3: "do we need ExitCode at all? nil /is/ the exit code"** — uniform `[] -> :wat::core::nil` across tier 0/1/2; ExitCode typealias retires from arc 170 scope (REALIZATIONS pass 10)
21. **(2026-05-10) User probed the explicit `client` binding: "what are we trying to communicate there.. we could just have... `(send-stdout! some-forms)`"** — drop client binding; mint helpers using per-thread ambient client; expose `StdIn/client` + `StdOut/client` escape hatches; Server/Client substrate types preserved (REALIZATIONS pass 11). User confirmed: *"outstanding - phenominal - we're back to ourselves again - the good UX matters - but we cannot ignore the stepping stones to deliver it.. we do the hard work to make the good work easy"*
22. **(2026-05-10) User clarified the protocol semantics: "the protocol is the newline is the end of the data... users /must/ append new line for recv to work... the ret val of recv /is data/ -> HolonAST"** — line-delimited EDN; `readln` returns `:Option<:wat::holon::Atom>` (HolonAST; user evals/casts); `println` is the canonical write verb; pass-11's `StdIn/recv`/`StdOut/send` retire from user surface. The handler signature drops parametric `<I,O>` framing; becomes `:fn(wat::holon::Atom)->wat::holon::Atom` (REALIZATIONS pass 12).
23. **(2026-05-10) User locked-in graceful-shutdown protocol: ":wat::core::nil is the final message before exit; processes communicate done before exiting"** — `Some(:nil)` from readln = graceful peer-done; `None` = ungraceful. User picked option A: substrate-automatic graceful-`:nil` epilogue on main-return. User pushed back on conflated signal-handling: *"the user must be allowed to perform their own clean up - go study how we manage signals - your response scares me - you have forgotten too much"*. Orchestrator crawled arc 106 + memory; corrected to "kernel measures, userland transitions" model. User confirmed: *"those expressions are so fucking good - don't forget those - they are absolutely arc worthy"* (REALIZATIONS pass 13).
24. **(2026-05-10) User noted the convergence meta-observation: "we reached for TCO without explicit direction and its the idealized state"** — the canonical server-loop's recursive shape is in tail position naturally; arc 003's trampoline supports it. The natural shape AND the substrate's idealized state CONVERGED without prompting — a maturity signal that the foundation has settled (REALIZATIONS pass 13 meta-observation).

Each refinement made the design simpler and more honest. The
final shape: spawn-* takes a fn matching its contract; substrate
handles all internals; no wat-level Program type; no entry-keyword;
no discovery; the fn IS the program. Nil IS the exit code AND the
graceful "done" message. The canonical 9-line server form uses
`println` + `readln` helpers; substrate carries every concern user
code drops; signal-cleanup logic stays user-side; recursion in
tail position rides arc 003's trampoline.

---

## Cross-references

- **arc 109 (kill-std)** — broader substrate refactor wave; arc
  170 is one of its v1 closure blockers
- **arc 112 (inter-process Result\<Option\<T\>,E\>)** — fork-program
  stdin/stdout/stderr wire foundation; the Result.Err on stderr
  framing arc 170 inherits
- **arc 114 (spawn-as-thread)** — established the Program\<I,O\>
  contract: "values flow only through channels; return is a
  panic-free marker"
- **arc 167 (fn-flat-signature)** — substrate-as-teacher walker
  precedent
- **arc 168 (let-flat-shape)** — multi-slice substrate-refactor +
  sweep precedent
- **arc 169 (struct-destructure-form-a)** — most recent arc;
  closest precedent for arc 170's structure
- **scratch/2026/05/019-wat-cli-options/** — full vision for argv
  + wat-cli-options + user: subcommand convention
- **scratch/2026/05/007-remote-program/** — RemoteProgram
  architecture; the future arc that arc 170 sets up via closure
  extraction
- **arc 091 slice 8 (`struct->form`)** — Value→AST infrastructure
  arc 170's closure extraction extends
- **arc 092 (wat-edn)** — EDN serialization for AST; future
  remote-program transport will use this
- **arc 098 (matches?)** — pattern-grammar precedent for
  signature-shape contracts
- **arc 102 (eval-ast!)** — child-side AST evaluation; arc 170's
  closure extraction produces AST that goes through this pathway
- **arc 113 (`:wat::test::program`)** — forms-from-quoted-forms
  capture pattern; conceptual ancestor of closure extraction

---

## Companion docs

- [`CLOSURE-EXTRACTION.md`](./CLOSURE-EXTRACTION.md) — substrate
  primitive deep-dive: algorithm, invariants, portability check,
  test strategy. The load-bearing substrate work for arc 170.
- [`EXAMPLES.md`](./EXAMPLES.md) — concrete user-facing
  client/server pair demos (square server, greeter server) with
  wire traces in pure wat. Slice 5 closure paperwork inherits.
- [`TIERS.md`](./TIERS.md) — runtime tier framework (0/1/2/3) +
  hermetic-as-ambient property + canonical form summary.
- [`REALIZATIONS-SLICE-1.md`](./REALIZATIONS-SLICE-1.md) — the
  conversation-thread record across passes 1-13.
