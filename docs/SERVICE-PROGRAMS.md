# Service Programs in wat

A **service program** is a long-running wat program that holds state
across many requests, communicates with callers through channels, and
shuts down cleanly when its callers leave. Console, CacheService,
RunDb — the substrate's worked examples — all wear this shape.

This doc teaches the wiring. `USER-GUIDE.md § 7 Concurrency` is the
primitive reference (signatures, type aliases, what each `:wat::kernel::*`
form does). Read this when you want to *build* one of these programs;
read the user guide when you want to look up a specific call.

The teaching follows an eight-step exploration that lives at
`holon-lab-trading/wat-tests-integ/experiment/008-treasury-program/explore-handles.wat`
— each step is a real, passing deftest. Lift the patterns from there
when you're writing your own.

---

## The lockstep

The single most important rule: **a channel-end disconnects only when
every Sender (or every Receiver) clone has dropped.** Clones drop when
their `let*` binding exits scope. There is no force-close primitive —
`:wat::kernel::drop` is a no-op marker, not a `mem::drop`.

This means the *shape of your `let*` nests is the shape of your
shutdown sequence.* Get the nesting right and the program shuts down
cleanly without any explicit teardown code. Get it wrong and you
deadlock — your `join` call blocks forever because some sender Arc is
still alive in the same `let*`.

The proven shape:

> **Outer scope holds the `ProgramHandle`. Inner scope owns every Sender.**
> The inner `let*` body yields the handle so the outer can join it.
> When inner exits, every Sender in this thread drops; the worker's
> next `recv` returns `:None`; the worker exits; the outer `join`
> unblocks.

Every step below is a variation on this nesting.

---

## Step 1 — spawn + join (Ok path)

The smallest service program: spawn a function on a new thread, wait
for its return value.

```scheme
(:wat::core::define
  (:my::app::return-42 -> :i64) 42)

(:wat::core::let*
  (((handle :wat::kernel::ProgramHandle<i64>)
    (:wat::kernel::spawn :my::app::return-42)))
  (:wat::core::match (:wat::kernel::join-result handle) -> :()
    ((Ok 42) ())
    ((Ok n)  (:wat::test::assert-eq "wrong-value" ""))
    ((Err _) (:wat::test::assert-eq "spawn-died" ""))))
```

**What it proves**: the spawn + `join-result` round-trip carries a
value end-to-end. No channels yet — just the program-handle plumbing.

**Scope ledger**: only `handle` exists. The spawned program holds its
own state (none, here); the handle is what the caller waits on.

---

## Step 2 — death as data

Spawn a function that panics. `join-result` returns
`Err(ThreadDiedError::Panic msg)` instead of unwinding the caller.

```scheme
(:wat::core::define
  (:my::app::boom -> :())
  (:wat::kernel::assertion-failed! "intentional panic" :None :None))

(:wat::core::let*
  (((handle :wat::kernel::ProgramHandle<()>)
    (:wat::kernel::spawn :my::app::boom)))
  (:wat::core::match (:wat::kernel::join-result handle) -> :()
    ((Ok _) (:wat::test::assert-eq "expected-panic" ""))
    ((Err (:wat::kernel::ThreadDiedError::Panic msg))
      (:wat::test::assert-eq msg "intentional panic"))
    ((Err _) (:wat::test::assert-eq "wrong-error-variant" ""))))
```

**What it proves**: the substrate captures the panic, routes it through
the Result channel rather than killing the test thread, and preserves
the message.

**When to use**: tests, supervisors, any code that needs to *diagnose*
a spawn-thread's outcome. Use plain `:wat::kernel::join` when a thread
death IS a bug worth halting on (the panic propagates to the caller).

---

## Step 3 — counted recv (scope-based close)

The first channel program. Worker holds `rx` and recv-loops; client
sends N messages; client's scope exits; worker sees `:None` and
returns its count.

```scheme
;; Worker — tail-recursive recv loop.
(:wat::core::define
  (:my::app::count-recv
    (rx :wat::kernel::QueueReceiver<i64>)
    (acc :i64)
    -> :i64)
  (:wat::core::match (:wat::kernel::recv rx) -> :i64
    ((Some _v) (:my::app::count-recv rx (:wat::core::+ acc 1)))
    (:None acc)))

(:wat::core::define
  (:my::app::run-counter
    (rx :wat::kernel::QueueReceiver<i64>) -> :i64)
  (:my::app::count-recv rx 0))

;; Client — nested let*. Outer holds the handle; inner owns tx.
(:wat::core::let*
  (((handle :wat::kernel::ProgramHandle<i64>)
    (:wat::core::let*
      (((pair :wat::kernel::QueuePair<i64>)
        (:wat::kernel::make-bounded-queue :i64 1))
       ((tx :wat::kernel::QueueSender<i64>) (:wat::core::first pair))
       ((rx :wat::kernel::QueueReceiver<i64>) (:wat::core::second pair))
       ((h :wat::kernel::ProgramHandle<i64>)
        (:wat::kernel::spawn :my::app::run-counter rx))
       ((_s1 :Option<()>) (:wat::kernel::send tx 10))
       ((_s2 :Option<()>) (:wat::kernel::send tx 20))
       ((_s3 :Option<()>) (:wat::kernel::send tx 30)))
      h)))                                     ;; ← pair, tx, rx all drop here
  (:wat::core::match (:wat::kernel::join-result handle) -> :()
    ((Ok 3) ())
    ((Ok _) (:wat::test::assert-eq "wrong-count" ""))
    ((Err _) (:wat::test::assert-eq "worker-died" ""))))
```

**What it proves**: the nested-`let*` shutdown shape. The inner let\*
body returns `h`; when it returns, *everything else bound in the inner
scope drops*. With every local Sender clone gone, the worker's next
`recv` returns `:None` and the loop exits.

**Scope ledger**:

| Scope | Bound | Lives until |
|---|---|---|
| outer | `handle` | end of test |
| inner | `pair`, `tx`, `rx`, `h` | inner body returns |

**Anti-pattern that deadlocks**:

```scheme
(:wat::core::let*                    ;; ← ONE flat let*
  (((pair ...) (:wat::kernel::make-bounded-queue :i64 1))
   ((tx ...) (:wat::core::first pair))
   ((rx ...) (:wat::core::second pair))
   ((handle ...) (:wat::kernel::spawn :my::app::run-counter rx))
   ((_s1 ...) (:wat::kernel::send tx 10))
   ((_drop :()) (:wat::kernel::drop tx)))   ;; ← no-op; tx still bound
  (:wat::kernel::join-result handle))         ;; ← worker recv-loops forever
```

`tx` is bound in the same `let*` whose body calls `join-result`.
`join-result` blocks before `tx` falls out of scope, so the worker's
`recv` never returns `:None`.

**The bounded(1) backpressure** keeps every send synchronous with a
matching recv — `(:wat::kernel::send tx 10)` blocks until the worker
has consumed the previous message. This is the rendezvous shape and
the right default.

---

## Step 4 — request / response (two channels)

Worker recv-loops on `req-rx` and emits each response on `resp-tx`.
Client sends a request, recvs the response, exits.

```scheme
(:wat::core::define
  (:my::app::doubler-loop
    (req-rx  :wat::kernel::QueueReceiver<i64>)
    (resp-tx :wat::kernel::QueueSender<i64>)
    -> :())
  (:wat::core::match (:wat::kernel::recv req-rx) -> :()
    ((Some n)
      (:wat::core::let*
        (((_ack :Option<()>)
          (:wat::kernel::send resp-tx (:wat::core::* n 2))))
        (:my::app::doubler-loop req-rx resp-tx)))
    (:None ())))

(:wat::core::let*
  (((handle :wat::kernel::ProgramHandle<()>)
    (:wat::core::let*
      (((req-pair  :wat::kernel::QueuePair<i64>)
        (:wat::kernel::make-bounded-queue :i64 1))
       ((req-tx  :wat::kernel::QueueSender<i64>)   (:wat::core::first req-pair))
       ((req-rx  :wat::kernel::QueueReceiver<i64>) (:wat::core::second req-pair))
       ((resp-pair :wat::kernel::QueuePair<i64>)
        (:wat::kernel::make-bounded-queue :i64 1))
       ((resp-tx :wat::kernel::QueueSender<i64>)   (:wat::core::first resp-pair))
       ((resp-rx :wat::kernel::QueueReceiver<i64>) (:wat::core::second resp-pair))
       ((h :wat::kernel::ProgramHandle<()>)
        (:wat::kernel::spawn :my::app::doubler-loop req-rx resp-tx))
       ((_s :Option<()>) (:wat::kernel::send req-tx 21))
       ((got :Option<i64>) (:wat::kernel::recv resp-rx)))
      h)))
  (:wat::core::match (:wat::kernel::join-result handle) -> :() ...))
```

**What it proves**: two channels per worker is just two more bindings
in the inner scope. Both pairs and both senders drop together when the
inner scope exits; both `recv`s in the worker fall through to `:None`.

**Reading the response inside inner scope** is mandatory — bounded(1)
means the worker's next iteration blocks on `send resp-tx ...` until
the client consumes the previous response.

---

## Step 5 — multi-channel select

`select` watches a `Vec<QueueReceiver<T>>` and returns
`Chosen<T> ≡ (idx, Option<T>)` — *which receiver fired* and *what it
gave* (`Some v` or `:None` on disconnect).

```scheme
(:wat::core::define
  (:my::app::select-loop-step
    (rxs :Vec<wat::kernel::QueueReceiver<i64>>)
    (acc :i64)
    -> :i64)
  (:wat::core::if (:wat::core::empty? rxs) -> :i64
    acc
    (:wat::core::let*
      (((chosen :wat::kernel::Chosen<i64>) (:wat::kernel::select rxs))
       ((idx :i64)              (:wat::core::first chosen))
       ((maybe :Option<i64>)    (:wat::core::second chosen)))
      (:wat::core::match maybe -> :i64
        ((Some _v)
          (:my::app::select-loop-step rxs (:wat::core::+ acc 1)))
        (:None
          (:my::app::select-loop-step
            (:wat::std::list::remove-at rxs idx) acc))))))
```

**What it proves**: the prune-on-disconnect pattern. Each channel that
runs out of senders removes itself from the Vec; when the Vec is empty
the loop exits.

**Build the receivers Vec from pairs** with `map first` / `map second`:

```scheme
((pairs :Vec<wat::kernel::QueuePair<i64>>)
 (:wat::core::map (:wat::core::range 0 N)
   (:wat::core::lambda ((_i :i64) -> :wat::kernel::QueuePair<i64>)
     (:wat::kernel::make-bounded-queue :i64 1))))

((txs :Vec<wat::kernel::QueueSender<i64>>)
 (:wat::core::map pairs
   (:wat::core::lambda ((p :wat::kernel::QueuePair<i64>)
                        -> :wat::kernel::QueueSender<i64>)
     (:wat::core::first p))))

((rxs :Vec<wat::kernel::QueueReceiver<i64>>)
 (:wat::core::map pairs
   (:wat::core::lambda ((p :wat::kernel::QueuePair<i64>)
                        -> :wat::kernel::QueueReceiver<i64>)
     (:wat::core::second p))))
```

`Console.wat` and `CacheService.wat` use this exact pattern for their
fan-in drivers.

---

## Step 6 — secondary write surface

A worker can hold N senders. The Treasury shape: respond on `resp-tx`,
*also* emit telemetry on `telem-tx`. Both happen inside the same recv
handler.

```scheme
(:wat::core::define
  (:my::app::telemetry-loop
    (req-rx   :wat::kernel::QueueReceiver<i64>)
    (resp-tx  :wat::kernel::QueueSender<i64>)
    (telem-tx :wat::kernel::QueueSender<i64>)
    -> :())
  (:wat::core::match (:wat::kernel::recv req-rx) -> :()
    ((Some n)
      (:wat::core::let*
        (((_r :Option<()>)
          (:wat::kernel::send resp-tx (:wat::core::* n 2)))
         ((_t :Option<()>)
          (:wat::kernel::send telem-tx n)))
        (:my::app::telemetry-loop req-rx resp-tx telem-tx)))
    (:None ())))
```

**What it proves**: extra Senders compose without changing the
shutdown story. The client's inner scope owns three Sender Arcs (req,
resp, telem) — all drop together; all three of the worker's channel
ends disconnect together.

**Bounded(1) with multiple write surfaces** means the client must read
both surfaces per request — an unread `telem-rx` blocks the worker on
its next `telem-tx` send. Use unbounded queues or larger buffers for
fire-and-forget telemetry where the producer must not block on the
consumer.

---

## Step 7 — many clients (HandlePool fan-in)

When N callers each need their own Sender into one selecting worker,
use `HandlePool` — it's the orphan-detector. Pop N handles, call
`finish()` to assert pool empty, distribute. A handle left in the pool
at `finish()` time means a wiring mistake; you'd rather panic at
construction than deadlock at shutdown.

```scheme
(:wat::core::let*
  (((handle :wat::kernel::ProgramHandle<i64>)
    (:wat::core::let*
      (((pairs :Vec<wat::kernel::QueuePair<i64>>) ...)
       ((txs :Vec<wat::kernel::QueueSender<i64>>) ...)
       ((rxs :Vec<wat::kernel::QueueReceiver<i64>>) ...)

       ((pool :wat::kernel::HandlePool<wat::kernel::QueueSender<i64>>)
        (:wat::kernel::HandlePool::new "my-summer" txs))

       ((h :wat::kernel::ProgramHandle<i64>)
        (:wat::kernel::spawn :my::app::run-summer rxs))

       ((tx-a :wat::kernel::QueueSender<i64>) (:wat::kernel::HandlePool::pop pool))
       ((tx-b :wat::kernel::QueueSender<i64>) (:wat::kernel::HandlePool::pop pool))
       ((tx-c :wat::kernel::QueueSender<i64>) (:wat::kernel::HandlePool::pop pool))
       ((_finish :()) (:wat::kernel::HandlePool::finish pool))

       ((_a :Option<()>) (:wat::kernel::send tx-a 100))
       ((_b :Option<()>) (:wat::kernel::send tx-b 200))
       ((_c :Option<()>) (:wat::kernel::send tx-c 300)))
      h)))
  (:wat::core::match (:wat::kernel::join-result handle) -> :() ...))
```

**What it proves**: the HandlePool API is small (`new`, `pop`,
`finish`) and composes with the same nested-scope shutdown. Multi-thread
clients work the same way — each client thread is spawned with one
popped handle as an argument, holds it for its lifetime, drops it when
the client exits.

**Why a pool and not a bare `Vec<Sender>`?** The pool gives you
claim-or-panic at *construction*. Without it, an unused handle
silently keeps a channel alive forever — the kind of bug that only
shows up as "my program hangs at shutdown."

---

## Step 8 — stateful loop with struct accumulator

A real service holds state — a paper-trade table, an LRU cache, a
treasury record. Wat's discipline is **values up, not in-place
mutation**: each loop iteration constructs a NEW state with the
modifications, recurses with the new value, returns the final value at
disconnect.

```scheme
(:wat::core::struct :my::app::Tally
  (count :i64)
  (sum   :i64))

(:wat::core::define
  (:my::app::tally-loop
    (rx    :wat::kernel::QueueReceiver<i64>)
    (tally :my::app::Tally)
    -> :my::app::Tally)
  (:wat::core::match (:wat::kernel::recv rx) -> :my::app::Tally
    ((Some v)
      (:wat::core::let*
        (((next :my::app::Tally)
          (:my::app::Tally/new
            (:wat::core::+ (:my::app::Tally/count tally) 1)
            (:wat::core::+ (:my::app::Tally/sum   tally) v))))
        (:my::app::tally-loop rx next)))
    (:None tally)))

(:wat::core::define
  (:my::app::run-tally
    (rx :wat::kernel::QueueReceiver<i64>) -> :my::app::Tally)
  (:my::app::tally-loop rx (:my::app::Tally/new 0 0)))
```

**What it proves**: the loop's "state" can be any wat type — a struct,
a tuple, a Vec, a HashMap. The recurse-with-new-value pattern flows
naturally: read fields with `/<field>`, build a new instance with
`/new`, recurse with the new value as the next argument. The whole
final value rides through `join-result` so the caller can pattern-match
on `(Ok tally)` and read its fields.

**This is the canonical Treasury shape.** Treasury's recv-loop will
hold a state struct (papers map, position records, treasury balance);
each request builds a new state with the updates; the loop returns the
final state at shutdown. Same pattern, richer struct.

The reference is `holon-lab-trading/wat/encoding/atr-window.wat` —
`AtrWindow::push` reads with accessors, builds with `/new`, returns a
new window. Treasury and every other service-state shape will mirror
this.

---

## The full service template

The eight steps above compose into the canonical service-program
template. Both `Console` and `CacheService` are this template applied:

```scheme
(:wat::core::define
  (:my::service<K,V>
    (capacity :i64)
    (count :i64)
    -> :(wat::kernel::HandlePool<wat::kernel::QueueSender<Request<K,V>>>,
         wat::kernel::ProgramHandle<()>))
  (:wat::core::let*
    (;; Build N request channels.
     ((pairs :Vec<wat::kernel::QueuePair<Request<K,V>>>) ...)
     ((req-txs :Vec<wat::kernel::QueueSender<Request<K,V>>>) ...)
     ((req-rxs :Vec<wat::kernel::QueueReceiver<Request<K,V>>>) ...)

     ;; Pool the senders so callers claim-or-panic.
     ((pool :wat::kernel::HandlePool<wat::kernel::QueueSender<Request<K,V>>>)
      (:wat::kernel::HandlePool::new "my-service" req-txs))

     ;; Spawn the driver — owns the state, fans in the receivers.
     ((driver :wat::kernel::ProgramHandle<()>)
      (:wat::kernel::spawn :my::service/loop capacity req-rxs)))
    (:wat::core::tuple pool driver)))
```

The driver loop combines steps 5 + 8: select over `req-rxs`, on each
`Some(req)` pattern-match the request, build new state, recurse with
new state and the rxs Vec; on `:None` for any channel, prune that rx
and recurse; exit when the rxs Vec is empty.

The caller pattern is the nested-scope shape from step 3, scaled:

```scheme
(:wat::core::let*
  (;; Outer holds driver handles.
   ((service-state ...) (:my::service ...))
   ((driver ...) (:wat::core::second service-state))

   ;; Inner scope owns the popped handles + does the work.
   ((_ :())
    (:wat::core::let*
      (((pool ...) (:wat::core::first service-state))
       ((req-tx ...) (:wat::kernel::HandlePool::pop pool))
       ((_finish :()) (:wat::kernel::HandlePool::finish pool))
       ;; ... per-client reply channel, request/response calls, etc.
       )
      ()))                                ;; ← inner scope exits, all senders drop

   ;; Driver sees disconnect, exits cleanly.
   ((_ :()) (:wat::kernel::join driver)))
  ())
```

**The two worked examples** in the substrate:

- `wat-rs/wat/std/service/Console.wat` — N tagged-message senders fan
  into one driver that decodes the tag and writes to stdout / stderr.
  Tested by `wat-rs/wat-tests/std/service/Console.wat`.

- `wat-rs/crates/wat-lru/wat/lru/CacheService.wat` — N request senders
  carry their own reply-to addresses; the driver routes responses
  without a sender-index map (per-caller channels).

Both are short and the comments are dense — read them straight through
once you're past the eight steps.

---

## Quick reference

The substrate aliases that make all of this readable
(`wat/kernel/queue.wat`):

| Alias | Expands to |
|---|---|
| `:wat::kernel::QueueSender<T>` | `:rust::crossbeam_channel::Sender<T>` |
| `:wat::kernel::QueueReceiver<T>` | `:rust::crossbeam_channel::Receiver<T>` |
| `:wat::kernel::QueuePair<T>` | `:(QueueSender<T>, QueueReceiver<T>)` |
| `:wat::kernel::Chosen<T>` | `:(i64, Option<T>)` — `select` return |

The shutdown rules in one paragraph:

> Channel-ends disconnect when every clone has dropped. Clones drop
> when their `let*` binding exits scope. `:wat::kernel::drop` is a
> no-op marker, not a force-close. Therefore: hold the `ProgramHandle`
> in an outer scope and the Senders in an inner scope; when the inner
> scope exits, the Senders drop, the worker's `recv` returns `:None`,
> the worker exits, the outer `join` unblocks. Get the nesting right
> and you don't write any teardown code at all.

The eight working programs at
`holon-lab-trading/wat-tests-integ/experiment/008-treasury-program/explore-handles.wat`
are the canonical reference. When in doubt, lift the nearest step.
