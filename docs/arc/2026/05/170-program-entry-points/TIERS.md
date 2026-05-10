# Arc 170 — Runtime tiers

> Note: this doc captures the substrate-concept framing the user
> articulated 2026-05-09 during slice 1 review. Tiers are the
> primary structural concept. Hermeticness emerges as the ambient
> property of tier ≥ 2; it isn't a separate label or mechanism.
> See [`REALIZATIONS-SLICE-1.md`](./REALIZATIONS-SLICE-1.md) for
> the conversation thread that surfaced this framing.

## The four tiers

A wat form runs in one of four runtime tiers, ordered by what
the runtime shares with the spawning context:

| Tier | Sharing | User-visible IPC | Substrate transport | Substrate primitive | Hermetic? |
|---|---|---|---|---|---|
| **0** — runtime env | call stack | direct call `(f x y)` | call stack | (the eval loop itself) | NO |
| **1** — threads | memory (same wat world) | `Sender<T>` / `Receiver<T>` | crossbeam channels (in-memory typed Values) | `(:wat::kernel::spawn-thread fn)` | NO |
| **2** — processes | host (filesystem, OS kernel) | `Sender<T>` / `Receiver<T>` | EDN-over-pipes (substrate encodes/decodes) | `(:wat::kernel::spawn-process fn)` | YES |
| **3** — remote programs *(future)* | network | `Sender<T>` / `Receiver<T>` (Q-channel multiplex) | EDN-over-sockets | `(:wat::kernel::spawn-remote-program fn)` | YES |

Each tier "shares less" than the previous. The tier number is
the depth of isolation — tier 0 shares everything (same eval
context); tier 3 shares only the network reachability.

The user-facing **interface stays uniform** across tiers: a fn
satisfying a contract, paired with a client/server channel
metaphor. What changes between tiers is the runtime env's
sharing surface and the IPC mechanism that bridges it.

## Same abstraction at every tier — typed channels

The user writes the same `Sender<T>` / `Receiver<T>` shape at
every tier. WatAST serializes to EDN by nature; the substrate
handles encoding at the pipe/socket boundary. Users never see
strings flowing through these channels.

This is a **uniformity property**: the parent and child agree on
types `I` and `O`; both sides see typed Values; transport is the
substrate's secret. Tier 1 has no encoding step (memory is
shared); tier 2 encodes through EDN-over-pipes; tier 3 encodes
through EDN-over-sockets. From the user's POV, all three look
identical.

### The OS-boundary handling — three substrate services + ambient runtime (locked-in 2026-05-10)

Per REALIZATIONS pass 9, the OS-pipe-resources (fd 0/1/2 + argv)
are owned by substrate-managed services. User code never touches
them directly:

```
:wat::kernel::StdInService   — owns fd 0; reads + decodes EDN; serves typed Values
:wat::kernel::StdOutService  — owns fd 1; receives typed Values; serializes + writes
:wat::kernel::StdErrService  — owns fd 2; first panic event drained wins; emits + exits
:wat::runtime::current-thread — thread-local id (ambient value)
:wat::runtime::argv           — set-once at process start (ambient value)
```

`:user::main` simplifies to `[] -> :wat::core::nil` (per
REALIZATIONS pass 10 — **nil IS the exit code**). No stdio
params; no argv param. Programs reach for ambient runtime +
services as needed. Substrate maps clean nil-return to
`libc::exit(0)`; panic-cascade via StdErrService maps to
`libc::exit(N)`. User code never participates in exit-code
arithmetic. Same `[] -> :nil` shape as arc 114's
`Program<I,O>` contract — the entry signature unifies across
tier 0/1/2/3.

The previous "OS-boundary exception" framing (passing IOReader/
IOWriter/Vector\<String\> as `:user::main` params) was the
midpoint; the locked-in shape replaces it. Services own the
boundaries; ambient runtime exposes thread-id + argv; user
writes intent.

**Doctrine: structured-stderr-only.** Inside wat-land, fd 2
ONLY ever carries panic-cascade EDN. wat-cli has zero direct
stderr writes. Pretty-printing is downstream (shell user pipes
through formatter if they want).

**Doctrine: single-shot panic.** `(:wat::runtime::panic! ...)`
blocks; thread sends panic event to its registered StdErrService
pipe; service drains; emits cascade; calls `libc::exit(N)`.
Concurrent panickers in other threads queue at their pipes but
are never drained — process dies after the first panic.

### The canonical wat server form

Captured as memory `project_arc_170_canonical_server_form.md`.
This is what arc 170 delivers — the polished shape the user
direction (2026-05-10) recognized as "incredible":

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

`(:wat::kernel::println v)` writes data + newline (blocks).
`(:wat::kernel::readln)` returns `:Option<:wat::holon::Atom>`
(blocks; `:None` on fd 0 closed). Both helpers route through
per-thread Client thread-locals (set by spawn-thread's
register-with-services contract). Users typically never
instantiate Client directly. Advanced cases reach for
`(:wat::kernel::StdIn/client)` / `(:wat::kernel::StdOut/client)`
escape hatches — substrate honest about the internals; canonical
surface stays clean. Recursion is in tail position; arc 003's
trampoline handles indefinite loops without stack growth.

**Three protocol terminal states (post-pass-13):**
- `Some(:wat::core::nil)` — peer announced graceful done
- `Some(other)` — peer sent data; process; respond; loop
- `None` — fd 0 closed without graceful `:nil`; ungraceful

The substrate auto-emits `:wat::core::nil` to fd 1 after
`:user::main` returns nil cleanly (signal-cleanup path is the
user's responsibility per arc 106; substrate measures, userland
transitions). See `project_signal_cascade.md`.

The user's whole client/server program in 12 lines of wat. No
fork ceremony, no pipe plumbing, no error-routing scaffolding.
The substrate provides it all; user writes intent.

### The canonical form is what users UNDERSTAND. The helper is what users WRITE.

The full form above is visible — users see what's happening.
But typical programs reach for the substrate-provided helper:

```scheme
;; my-server.wat — three lines is the typical program
(:wat::core::load! "some-lib.wat")  ;; brings in :my::handler

(:wat::kernel::main! :my::handler)
```

`(:wat::kernel::main! handler)` is a substrate-auto-loaded
defmacro (lives in `wat/kernel/main.wat` or similar; no explicit
`load!` needed; same pattern as `:wat::core::defn`). It expands
to the canonical server-loop form above.

**`main!` accepts any handler expression** — keyword path,
inline fn-form, or factory call:

```scheme
;; Factory pattern (the polished idiom — config baked into closure)
(:wat::core::load! "some-lib.wat")
(:wat::kernel::main! (make-handler))

;; Keyword path
(:wat::core::load! "some-lib.wat")
(:wat::kernel::main! :my::handler)

;; Inline lambda
(:wat::kernel::main!
  (:wat::core::fn [req <- :MyReqType] -> :MyRespType
    (... handle req ...)))
```

For CLI utility programs (one-shot; doesn't run a service loop):

```scheme
;; my-script.wat — last form returns nil; signature satisfied
(:wat::kernel::run!
  (:wat::kernel::println "Hello, world!"))
```

`(:wat::kernel::run! form1 form2 ...)` is variadic — wraps
forms in an implicit-do; the last form's value flows through.
If the last form returns nil, signature satisfied; if it
returns non-nil, freeze diagnostic catches it. Expands to a
one-shot `:user::main`.

Per `project_wat_llm_first_design.md` ("one canonical path per
task; reject synonym features"), the macros ARE the canonical
path. The full form remains for transparency + custom
deviation. Programs reach for `main!` / `run!` by default.

Memory cross-reference: `project_arc_170_canonical_server_form.md`
captures the form for compaction-survival per user direction
2026-05-10.

### What disappears from the user's view

| Where | Before | After |
|---|---|---|
| `:user::process` contract | `[stdin <- :wat::io::IOReader stdout <- :wat::io::IOWriter stderr <- :wat::io::IOWriter] -> :wat::core::nil` | `[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil` |
| Layer 1 fn ceremony (`run-hermetic`) | user writes `(:wat::core::fn [stdin <- ... stdout <- ...] -> :wat::core::nil body)` | user writes just the body; macro generates `(fn [] -> :wat::core::nil body)` wrapper |
| Layer 2 fn ceremony (`run-hermetic-with-io`) | user writes the full fn-form with typed channels | user writes just the body; macro introduces `rx` and `tx` as bindings + generates the fn-form wrapper |
| `:wat::kernel::Process<I,O>` struct | `{ stdin :IOWriter, stdout :IOReader, stderr :IOReader, handle }` | `{ tx :Sender<I>, rx :Receiver<O>, handle }` |
| Testing lib `RunResult` | `{ stdout :Vec<String>, stderr :Vec<String>, failure }` | `{ outputs :Vec<O>, failure }` (parsed Values) |
| Testing lib stdin input | `Vec<String>` (joined to bytes) | `Vec<I>` (typed Values) |

The tier-2 substrate transport (linux fds + EDN encoding) is
substrate-internal plumbing. Users at tier 2 work in `Sender<T>` /
`Receiver<T>` like at tier 1 — same shape, different transport.

## Hermeticness is an ambient property

Hermeticness is **not a label**, not a flag, not an explicit
choice. It's what tier ≥ 2 inherently IS — because tier ≥ 2
means crossing the OS-process boundary, and the kernel gives
several isolation properties for free at that boundary:

- Memory isolation (forked process has its own address space; COW)
- Signal isolation (SIGINT / SIGTERM / SIGUSR1 / SIGUSR2 to child don't affect parent)
- Global state isolation (Rust libs with globals modified in child stay in child)
- Runtime sealing (the substrate hosting the user program is unreachable from user code)

These aren't four separate properties — they're the SAME property
manifesting in four ways, because tier ≥ 2 means a separate OS
process with its own memory + signal mask + global-state space +
substrate instance. The OS-process boundary is the seal; the
properties are what the seal provides.

You don't ASK for hermetic isolation. You choose tier ≥ 2 and
get it ambient.

## Why this matters — the proof property

The tier ≥ 2 ambient hermetic seal makes correctness of certain
properties **trivially provable by construction.** The historical
use cases that drove the substrate's hermetic capability:

- **Linux kernel signal-handling tests** — spawn a hermetic
  process; send SIG{INT, TERM, USR1, USR2}; observe behavior.
  The parent's signal mask + handler state can't be affected.
  No test-isolation harness needed.
- **Global-state isolation tests** — spawn a hermetic process
  exercising a Rust library that modifies globals. The library's
  global state stays inside the child. The parent's globals are
  untouched. No mutex coordination needed.
- **wat-cli runtime sealing** — the CLI is a hermetic boundary
  around user programs. The user program can't reach the
  substrate hosting it. The substrate is sealed off from user
  code by the same mechanism that makes signal-handling tests
  trivial.

The seal IS the proof. You don't write isolation invariants and
verify them; you choose tier ≥ 2 and the kernel enforces them.

## Closure extraction = the tier-bridging primitive

Tier 0 and tier 1 don't need a portable program description: the
fn Value lives in the shared wat world; it's used directly.

Tier 2 and tier 3 need one: the spawned context starts from a
clean slate (its own substrate, its own memory) and must
reconstruct the fn-with-its-environment from a portable
artifact.

The closure-extraction package
([`CLOSURE-EXTRACTION.md`](./CLOSURE-EXTRACTION.md)) is that
artifact:

```rust
ClosurePackage {
    prologue: Vec<WatAST>,   // captured environment (types + deps + captures)
    entry_form: WatAST,      // expression evaluating to a fn Value
}
```

Same package shape for tier 2 (passed to forked child via
`fork-program-ast` pathway) and tier 3 (serialized via wat-edn
over a socket). Tier 1 ignores it. Tier 0 doesn't have spawning.

This is the substrate primitive that **bridges tier boundaries
when tier ≥ 2.** It's the only shared piece across the hermetic
tiers — different transports, same package.

## Today's `wat/std/hermetic.wat` is a tier-2 wrapper — gets rebuilt in arc 170 slice 3

`wat/std/hermetic.wat` ships in the substrate today as a
wat-level convenience over `fork-program-ast` (forms → forked
process). It's a tier-2 spawn with the legacy input shape
(`Vec<WatAST>` with an embedded `:user::main` define) AND it
exposes substrate-level ceremony (`scope :Option<String>` —
filesystem-rooting for the child's loader; today's hermetic.wat
errors on `:Some scope`, so the parameter is leaked plumbing that
isn't even functional).

Arc 170 changes the tier-2 substrate surface to take a fn
directly. The testing tooling MUST reach its polished form on
the new substrate — not just functionally work, but **hide every
piece of ceremony that's constant for typical test usage.**

### Three-layer testing API (slice 3 target)

The substrate is honest about what it offers (full fn-with-pipes
signature; loader config; etc.). The testing lib's job is to
hide all of that for the cases tests actually use. 90%+ of tests
want one thing: "run this code; tell me if it broke."

```scheme
;; LAYER 1 — the 90% case: "run this code hermetically"
;; run-hermetic is a macro; the body runs hermetically.
;; The (fn [] -> :wat::core::nil ...) wrapper is generated by the macro;
;; the user never types it. Only the body varies between tests.
(:wat::test::run-hermetic
  (:wat::core::assert-eq 42 (my-test-helper)))
;; → returns :wat::kernel::RunResult { failure :Option<Failure> }

;; LAYER 2 — the 9% case: tests that interact with the spawned channels
;; run-hermetic-with-io is a macro that introduces rx + tx as bindings;
;; the user uses them in the body. The fn wrapper + typed-channel
;; signature are generated.
(:wat::test::run-hermetic-with-io<I,O> inputs
  (... test sends/receives via rx and tx — both in scope ...))
;; → outputs :Vec<O>; substrate encodes/decodes EDN over pipes;
;; user sees typed Values both directions

;; LAYER 3 — the 1% case: full substrate, no testing-lib wrapper
;; The substrate primitive expects an explicit fn — production code
;; opts into the full surface.
(:wat::kernel::spawn-process
  (:wat::core::fn
    [rx <- :wat::kernel::Receiver<I>
     tx <- :wat::kernel::Sender<O>]
    -> :wat::core::nil
    ...))
;; this is the production form; not for tests
```

### What disappears from the testing surface

Compared to today's `run-sandboxed-hermetic-ast (forms stdin scope)`:

- **`forms` → fn** — caller writes a fn directly; no
  `Vec<WatAST>` construction, no embedded `:user::main` define
- **`stdin` (Vec\<String\>)** — drops from Layer 1 entirely.
  Most tests don't read stdin. Layer 2 retains it for the
  unusual cases. Layer 3 has full pipe access via the substrate
- **`scope` (:Option\<String\>)** — drops from EVERY testing
  layer. It's leaked substrate plumbing that's not even
  functional in today's hermetic.wat (errors on `:Some`). If
  anyone genuinely needs file-system-rooted hermetic testing,
  Layer 3 (substrate directly) plus the appropriate loader-config
  primitive is the path. Don't drag the constant ceremony through
  every test
- **the fn parameter ceremony** — Layer 1's fn takes `[]`. No
  `stdin :IOReader stdout :IOWriter stderr :IOWriter` to type
  every time. Layer 2 has the parameters when tests actually need
  them

### Slice 3 migration

- Audit existing callers of today's `run-sandboxed-hermetic-ast`;
  classify each by which layer it needs
- Expected distribution: most → Layer 1 (massive UX collapse);
  some → Layer 2; rare/none → Layer 3 (substrate directly)
- `wat/test.wat` and any other stdlib wrapping the spawn family
  get the same three-layer treatment — the testing layer hides
  ceremony; the substrate stays honest

The same hermetic property holds across all three layers (tier 2;
ambient seal). The difference is only how much ceremony the user
sees. Substrate is full surface; tests get the convenience layer
that's right for them.

## Future tiers

The tier framework is open at the high end. Future use cases that
warrant new tiers:

- **Tier 4 — different cluster?** Same network reachability but
  different administrative domain (different K8s cluster,
  different cloud account, different mTLS trust boundary).
- **Tier 5 — different epoch?** Persisted package on disk;
  spawned later in a new substrate instance with no live network
  connection back to the originator.

Each new tier:

- Names what's shared with the spawning context
- Names the IPC mechanism that bridges the boundary
- Inherits the hermetic property if its boundary crosses an OS
  process (tier ≥ 2 territory)
- Reuses the closure-extraction package for portability

The framework slots them in cleanly without re-deriving the
hermetic property each time.

## Connections to existing memory

- **`project_wat_binary_hologram.md`** — "the binary IS the
  surface between Rust universe and wat universe... Holograms
  nest via spawn-program." Each spawn at tier ≥ 2 creates a
  nested hologram; the closure-extraction package is the artifact
  that lets the nesting work cleanly.
- **`project_pipe_protocol.md`** — "line-delimited EDN + kernel
  pipes. One protocol; four transports." Pipes are the tier-2
  IPC mechanism; the protocol generalizes.

## What this doc does NOT do

- Doesn't change arc 170 substrate scope (closure extraction stays
  Rust-internal per DESIGN line 84-85)
- Doesn't open new arcs
- Doesn't propose substrate edits beyond slice 1b's
  already-documented reshape
- Doesn't surface a wat-level `Tier` value type or hermetic-package
  value type — those would be future arc proposals if useful

This is a substrate-concept doc. It names the framework so future
arcs can reference tiers (and the ambient hermetic property at
tier ≥ 2) without re-deriving the framing.
