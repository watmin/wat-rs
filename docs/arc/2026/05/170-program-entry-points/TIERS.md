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

### The OS-boundary exception — `:user::main`

There's exactly one place where strings remain user-visible: the
wat-cli IS the OS boundary, and the OS shell speaks bytes.
`:user::main`'s contract:

- `stdin :wat::io::IOReader`, `stdout :wat::io::IOWriter`, `stderr :wat::io::IOWriter`
  — byte streams from/to the OS shell
- `argv :wat::core::Vector<wat::core::String>` — what the OS
  shell passed; strings by nature

This is the ONE place strings remain at the user-visible level,
because it's where wat meets the OS, and the OS is bytes. Wat-
internal spawning (tier 2 + tier 3) is wat-to-wat and works in
forms — the OS-shell-bytes shape stops at `:user::main`'s
threshold.

### What disappears from the user's view

| Where | Before | After |
|---|---|---|
| `:user::process` contract | `(stdin :IOReader stdout :IOWriter stderr :IOWriter)` | `(rx :Receiver<I> tx :Sender<O>)` |
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
(:wat::test::run-hermetic
  (fn [] :nil
    (:wat::core::assert-eq 42 (my-test-helper))))
;; → returns :wat::kernel::RunResult { outputs :Vec<O>, failure }
;; user wrote: their test body. Nothing else.

;; LAYER 2 — the 9% case: tests that interact with the spawned channels
(:wat::test::run-hermetic-with-io<I,O>
  (fn [rx :wat::kernel::Receiver<I> tx :wat::kernel::Sender<O>] :nil
    (... test sends/receives typed Values ...))
  inputs)
;; → outputs :Vec<O>; substrate encodes/decodes EDN over pipes;
;; user sees typed Values both directions

;; LAYER 3 — the 1% case: full substrate, no testing-lib wrapper
(:wat::kernel::spawn-process
  (fn [rx :wat::kernel::Receiver<I> tx :wat::kernel::Sender<O>] :nil
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
