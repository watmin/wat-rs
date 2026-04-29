# CIRCUIT — `:user::main` is the wiring diagram

A wat program is a circuit. Programmer-built. Fixed topology. Signals
flow through wires once powered.

This is the architectural axiom of every wat program. The substrate
IS a programmable substrate in the FPGA sense — you arrange the gates
and routing at programming time; once running, the wiring is fixed and
signals flow through it. The shape of `:user::main` is the wiring
diagram.

## The rule

> **`:user::main` constructs all pipes, plugs each into its consumer,
> and starts the input stream. That's its entire job.**

No computation in main. No I/O in main. No state in main. Just wiring.

```scheme
(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::let*
    ;; 1. Construct the consumers — each spawn returns a HandlePool
    ;;    of senders + the driver's ProgramHandle.
    (((con-spawn   :Console::Spawn)
      (:wat::std::service::Console/spawn stdout stderr 4))
     ((con-pool    :HandlePool<Console::Tx>) (:wat::core::first con-spawn))
     ((con-driver  :ProgramHandle<()>)        (:wat::core::second con-spawn))

     ((tel-spawn   :Sqlite::Spawn)
      (:trading::telemetry::Sqlite/spawn "runs/today.db" 8 cadence))
     ((tel-pool    :HandlePool<Sqlite::ReqTx>) (:wat::core::first tel-spawn))
     ((tel-driver  :ProgramHandle<()>)         (:wat::core::second tel-spawn))

     ;; 2. Wire each consumer's senders to the producers that need them.
     ;;    (One let*-binding per wire; each binding plugs a pop into a
     ;;    worker that's about to be spawned.)
     ((_inner :())
      (:wat::core::let*
        (((con-tx-trader :Console::Tx) (HandlePool::pop con-pool))
         ((con-tx-broker :Console::Tx) (HandlePool::pop con-pool))
         ((tel-tx-trader :Sqlite::ReqTx) (HandlePool::pop tel-pool))
         ((_finish-con :()) (HandlePool::finish con-pool))
         ((_finish-tel :()) (HandlePool::finish tel-pool))

         ;; 3. Spawn the producers, handing each the senders it needs.
         ;;    The producers ARE the machine; main just wires them.
         ((trader-driver :ProgramHandle<()>)
          (:wat::kernel::spawn :trading::trader/run
            con-tx-trader tel-tx-trader stdin))

         ;; 4. Join the producers (their inner work powers the machine).
         ((_ :()) (:wat::kernel::join trader-driver)))
        ()))

     ;; 5. Inner exited → all client senders dropped → consumers
     ;;    see disconnect → join their drivers in any order.
     ((_ :()) (:wat::kernel::join tel-driver)))
    (:wat::kernel::join con-driver)))
```

That's the whole shape. Read top-to-bottom: spawn consumers, wire
senders to producers, run the producer (which has the input stream),
join. Every wat program of any size has this skeleton.

## What lives where

| Location | Role |
|---|---|
| `:user::main` | Wiring diagram. Constructs pipes; pops handles; spawns producers; joins everything. |
| Worker thread (consumer side) | Owns its thread-local resources (Db connection, file handle, in-memory state). Receives messages; processes them; replies if asked. |
| Worker thread (producer side) | Holds Sender clones for the consumers it talks to. Walks an input stream; pushes work into pipes; reacts to replies. |
| The input stream | The program's pulse. Single source. When it ends, the program ends. |

## Three rules that fall out

### 1. Resources are opened by the worker that uses them

The substrate has thread-owned resources (`RunDb`, `LocalCache`,
`Hologram`, anything wrapped in a `ThreadOwnedCell`). They cannot
travel across thread boundaries.

`:user::main` runs in one thread. Workers run in others. So:

- **Wrong:** open a Db in `:user::main`, pass to a worker. The
  thread-id check fires; the worker panics on first use.
- **Right:** the worker's spawn function opens its own Db, holds it
  for its lifetime, drops it when the loop exits.

The lab's `:trading::telemetry::Sqlite/spawn` does exactly this. It
takes a `db-path :String`, calls `:wat::kernel::spawn` on its own
entry fn, opens the Db inside the new thread, runs the substrate's
`Service/loop` with a local dispatcher closure that captures the
local Db.

`:user::main` only ever sees `(HandlePool<ReqTx>, ProgramHandle)` —
the pipe handles plus the driver to join. Never the Db.

### 2. Pipes cross threads; resources don't

`crossbeam_channel::Sender<T>` and `Receiver<T>` are `Send + Sync`
— they cross thread boundaries cleanly because they ARE the way
threads talk. Everything else is a candidate for thread-owned
storage.

When you find yourself wanting to share state between two workers,
the wat answer is **make one of them own it and put a pipe in
front.** The owner-worker becomes a service; the other worker
becomes a client. This is what arc 029 (`:trading::rundb::Service`)
and arc 078 (substrate cache services) settled.

The user's framing is rigid for a reason: this rule has no
exceptions. If you find yourself reaching for `Mutex<T>`, the
wiring diagram is wrong.

### 3. The input stream is the pulse

A wat program has ONE input stream, by convention. Could be:

- `stdin` for a CLI tool.
- A candle-stream from a parquet file (the trader's case).
- An HTTP request channel for a server.
- A scheduled tick for an autonomous agent.

The stream's emissions are what power the machine. A producer worker
consumes the stream and pushes work into the pipes; consumers process
and emit replies; replies feed back into the producer's loop or land
in another consumer; eventually some sink (sqlite, console, file)
absorbs the result.

When the stream ends — `:None` from a `recv`, or `Option::None` from
the parquet iterator, or stdin EOF — the producer exits. Its loop
unwinds; its sender clones drop; the consumers see disconnect; their
loops unwind; their senders drop; until every driver has joined and
`:user::main` returns `()`.

The stream is the pulse. No stream, no machine. Multiple streams
mean multiple machines fighting for the same wiring — that's a
design smell; refactor to one stream + multiple shapes of pulse.

## The paradox

The topology is rigid. Pipes constructed at startup; types fixed by
the wiring; resources owned by exactly one worker; shutdown cascades
in a determined order. **You cannot reshape the machine while it's
running.**

Inside any worker, anything goes. The worker can hold a complex state
machine, run arbitrary algorithms, reach for whatever Rust shim it
needs. **The substrate makes no rules about a worker's interior.**

This is the FPGA paradox: the gates are fixed; the configuration
inside each gate is unlimited. The rigidity at the topology layer
is what enables the freedom at the gate layer.

The same shape applies socially: the wat language makes architectural
mistakes hard (no Mutex; no implicit shared state; no pipes-after-
spawn) so reviewers and authors don't have to negotiate them. The
rigidity is the precondition for trusting what's built inside.

## Cross-references

- `SERVICE-PROGRAMS.md` — how to write ONE worker (the canonical
  service-program template). Step 9 covers the case where one
  service's reporter closes over another service's handles
  (multi-driver shutdown decomposition).
- `ZERO-MUTEX.md` — the no-Mutex discipline this wiring depends on.
  Pipes replace shared state; workers replace lock holders.
- `CONVENTIONS.md` — naming + service-contract patterns. The
  `Type/spawn` factory contract is what makes the wiring above
  read uniformly across services.

## Quick lint

If you're writing a wat program and any of these are true, the
wiring is probably wrong:

- `:user::main` opens a file, makes a network call, or computes
  something useful → move it into a worker.
- A resource is constructed in `:user::main` and passed across a
  spawn → the resource probably needs to be opened in the worker
  (look for `ThreadOwnedCell` panics).
- A spawn returns something other than `(HandlePool<...>,
  ProgramHandle<...>)` or its equivalent factory tuple → the
  spawn isn't following the substrate contract.
- A worker reaches across to another worker's state without going
  through a pipe → introduce a service.
- There's more than one input stream in `:user::main` → the
  topology is doing two things; split into two programs that
  communicate through pipes if both need to coexist.

Pass the lint, and the program shuts down cleanly without you
writing any teardown code. The cascade is in the wiring; main
returns `()` when the last driver joins.

PERSEVERARE.
