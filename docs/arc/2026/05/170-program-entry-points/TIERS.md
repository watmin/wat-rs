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

| Tier | Sharing | IPC mechanism | Substrate primitive | Hermetic? |
|---|---|---|---|---|
| **0** — runtime env | call stack | direct invocation `(f x y)` | (the eval loop itself) | NO |
| **1** — threads | memory (same wat world) | crossbeam channels `(tx, rx)` | `(:wat::kernel::spawn-thread fn)` | NO |
| **2** — processes | host (filesystem, OS kernel) | linux fds — stdin/stdout/stderr | `(:wat::kernel::spawn-process fn)` | YES |
| **3** — remote programs *(future)* | network | linux sockets `(tx, rx)` multiplexed | `(:wat::kernel::spawn-remote-program fn)` | YES |

Each tier "shares less" than the previous. The tier number is
the depth of isolation — tier 0 shares everything (same eval
context); tier 3 shares only the network reachability.

The user-facing **interface stays uniform** across tiers: a fn
satisfying a contract, paired with a client/server channel
metaphor. What changes between tiers is the runtime env's
sharing surface and the IPC mechanism that bridges it.

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
(`Vec<WatAST>` with an embedded `:user::main` define).

Arc 170 changes the tier-2 surface to take a fn directly. The
existing hermetic tooling MUST reach its polished form on the new
substrate — not just functionally work, but be **as good as the
new substrate allows**. That rebuild is slice 3's scope.

Polished shape (slice 3 target):

```scheme
;; before arc 170 — caller constructs Vec<WatAST> with embedded :user::main
(:wat::kernel::run-sandboxed-hermetic-ast forms stdin-data scope)

;; after arc 170 — caller passes a fn; tooling handles the ceremony
(:wat::kernel::run-sandboxed-hermetic
  (fn [stdin :wat::io::IOReader stdout :wat::io::IOWriter stderr :wat::io::IOWriter] :nil
    (... test body ...))
  stdin-data
  scope)
```

The call site collapses to "write your test as a fn; we
hermetically run it." Same hermetic property (tier 2; ambient);
same tier-bridging primitive (closure-extraction package); but
the call surface aligns with the rest of arc 170's spawn family
(fn in, Process out).

Slice 3 also migrates all callers of the legacy hermetic API to
the new shape. `wat/test.wat` and any other tooling that wraps
the spawn family get the same polish.

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
