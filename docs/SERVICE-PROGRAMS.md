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
    (rx :wat::kernel::Receiver<i64>)
    (acc :i64)
    -> :i64)
  (:wat::core::match (:wat::kernel::recv rx) -> :i64
    ((Some _v) (:my::app::count-recv rx (:wat::core::+ acc 1)))
    (:None acc)))

(:wat::core::define
  (:my::app::run-counter
    (rx :wat::kernel::Receiver<i64>) -> :i64)
  (:my::app::count-recv rx 0))

;; Client — nested let*. Outer holds the handle; inner owns tx.
(:wat::core::let*
  (((handle :wat::kernel::ProgramHandle<i64>)
    (:wat::core::let*
      (((pair :wat::kernel::Channel<i64>)
        (:wat::kernel::make-bounded-channel :i64 1))
       ((tx :wat::kernel::Sender<i64>) (:wat::core::first pair))
       ((rx :wat::kernel::Receiver<i64>) (:wat::core::second pair))
       ((h :wat::kernel::ProgramHandle<i64>)
        (:wat::kernel::spawn :my::app::run-counter rx))
       ((_s1 :()) (:wat::core::option::expect -> :() (:wat::kernel::send tx 10) "_s1: peer disconnected"))
       ((_s2 :()) (:wat::core::option::expect -> :() (:wat::kernel::send tx 20) "_s2: peer disconnected"))
       ((_s3 :()) (:wat::core::option::expect -> :() (:wat::kernel::send tx 30) "_s3: peer disconnected")))
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
  (((pair ...) (:wat::kernel::make-bounded-channel :i64 1))
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
    (req-rx  :wat::kernel::Receiver<i64>)
    (resp-tx :wat::kernel::Sender<i64>)
    -> :())
  (:wat::core::match (:wat::kernel::recv req-rx) -> :()
    ((Some n)
      (:wat::core::let*
        (((_ack :())
          (:wat::core::option::expect -> :() (:wat::kernel::send resp-tx (:wat::core::* n 2)) "_ack: peer disconnected")))
        (:my::app::doubler-loop req-rx resp-tx)))
    (:None ())))

(:wat::core::let*
  (((handle :wat::kernel::ProgramHandle<()>)
    (:wat::core::let*
      (((req-pair  :wat::kernel::Channel<i64>)
        (:wat::kernel::make-bounded-channel :i64 1))
       ((req-tx  :wat::kernel::Sender<i64>)   (:wat::core::first req-pair))
       ((req-rx  :wat::kernel::Receiver<i64>) (:wat::core::second req-pair))
       ((resp-pair :wat::kernel::Channel<i64>)
        (:wat::kernel::make-bounded-channel :i64 1))
       ((resp-tx :wat::kernel::Sender<i64>)   (:wat::core::first resp-pair))
       ((resp-rx :wat::kernel::Receiver<i64>) (:wat::core::second resp-pair))
       ((h :wat::kernel::ProgramHandle<()>)
        (:wat::kernel::spawn :my::app::doubler-loop req-rx resp-tx))
       ((_s :()) (:wat::core::option::expect -> :() (:wat::kernel::send req-tx 21) "_s: peer disconnected"))
       ((got :i64)
        (:wat::core::option::expect -> :i64
          (:wat::kernel::recv resp-rx)
          "recv resp: doubler-loop disconnected mid-reply")))
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

`select` watches a `Vec<Receiver<T>>` and returns
`Chosen<T> ≡ (idx, Option<T>)` — *which receiver fired* and *what it
gave* (`Some v` or `:None` on disconnect).

```scheme
(:wat::core::define
  (:my::app::select-loop-step
    (rxs :Vec<wat::kernel::Receiver<i64>>)
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
((pairs :Vec<wat::kernel::Channel<i64>>)
 (:wat::core::map (:wat::core::range 0 N)
   (:wat::core::lambda ((_i :i64) -> :wat::kernel::Channel<i64>)
     (:wat::kernel::make-bounded-channel :i64 1))))

((txs :Vec<wat::kernel::Sender<i64>>)
 (:wat::core::map pairs
   (:wat::core::lambda ((p :wat::kernel::Channel<i64>)
                        -> :wat::kernel::Sender<i64>)
     (:wat::core::first p))))

((rxs :Vec<wat::kernel::Receiver<i64>>)
 (:wat::core::map pairs
   (:wat::core::lambda ((p :wat::kernel::Channel<i64>)
                        -> :wat::kernel::Receiver<i64>)
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
    (req-rx   :wat::kernel::Receiver<i64>)
    (resp-tx  :wat::kernel::Sender<i64>)
    (telem-tx :wat::kernel::Sender<i64>)
    -> :())
  (:wat::core::match (:wat::kernel::recv req-rx) -> :()
    ((Some n)
      (:wat::core::let*
        (((_r :())
          (:wat::core::option::expect -> :() (:wat::kernel::send resp-tx (:wat::core::* n 2)) "_r: peer disconnected"))
         ((_t :())
          (:wat::core::option::expect -> :() (:wat::kernel::send telem-tx n) "_t: peer disconnected")))
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
      (((pairs :Vec<wat::kernel::Channel<i64>>) ...)
       ((txs :Vec<wat::kernel::Sender<i64>>) ...)
       ((rxs :Vec<wat::kernel::Receiver<i64>>) ...)

       ((pool :wat::kernel::HandlePool<wat::kernel::Sender<i64>>)
        (:wat::kernel::HandlePool::new "my-summer" txs))

       ((h :wat::kernel::ProgramHandle<i64>)
        (:wat::kernel::spawn :my::app::run-summer rxs))

       ((tx-a :wat::kernel::Sender<i64>) (:wat::kernel::HandlePool::pop pool))
       ((tx-b :wat::kernel::Sender<i64>) (:wat::kernel::HandlePool::pop pool))
       ((tx-c :wat::kernel::Sender<i64>) (:wat::kernel::HandlePool::pop pool))
       ((_finish :()) (:wat::kernel::HandlePool::finish pool))

       ((_a :()) (:wat::core::option::expect -> :() (:wat::kernel::send tx-a 100) "_a: peer disconnected"))
       ((_b :()) (:wat::core::option::expect -> :() (:wat::kernel::send tx-b 200) "_b: peer disconnected"))
       ((_c :()) (:wat::core::option::expect -> :() (:wat::kernel::send tx-c 300) "_c: peer disconnected")))
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
    (rx    :wat::kernel::Receiver<i64>)
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
    (rx :wat::kernel::Receiver<i64>) -> :my::app::Tally)
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

## Step 9 — Composing services (multi-driver shutdown)

Steps 1–8 cover one service. Step 9 covers what happens when one
service's Reporter (per arc 078's contract) closes over ANOTHER
service's handles — the case where two drivers must shut down in
order, and the lockstep from Step 3 has to apply twice without
collapsing into a single inline `let*`.

The trap: the obvious "just nest harder" reading produces a
three-deep `let*` that puts every driver, every popped handle, and
every Sender clone in scope at the same time. The lockstep from
Step 3 said "outer holds the handle; inner owns the Senders." With
two services, "outer" and "inner" need TWO levels each — and trying
to write all four levels inline in one `let*` body is how I (the
author of the surrounding documentation) deadlocked the first
attempt at a two-service composition.

The fix is **function decomposition.** Each scope-level becomes a
small named function that owns its driver and joins it before
returning. The deftest body composes the functions. Each function's
two-level `let*` is local and obeys Step 3 verbatim.

### The shape

```scheme
;; Bottom — pure work; takes the leaf service's send/recv handles
;; as args. No driver. Returns when work is done.
(:wat::core::define
  (:my::test::drive-requests
    (cache-req-tx :CacheService::ReqTx)
    (reply-tx :GetReplyTx)
    (reply-rx :GetReplyRx)
    -> :())
  ...)

;; Middle — owns CacheService driver. Two-level let*: outer holds
;; cache-driver (joined after inner exits); inner pops cache-req-tx,
;; calls drive-requests, drops senders.
(:wat::core::define
  (:my::test::run-cache-with-rundb-tx
    (rundb-req-tx :RunDbService::ReqTx)
    (ack-tx :RunDbService::AckTx)
    (ack-rx :RunDbService::AckRx)
    -> :())
  (:wat::core::let*
    (;; Cache reporter — closes over rundb handles (function args).
     ((reporter ...) (:my::reporter/make rundb-req-tx ack-tx ack-rx))
     ((cache-spawn ...) (CacheService/spawn ... reporter))
     ((cache-pool ...) ...)
     ((cache-driver :ProgramHandle<()>) ...)
     ;; Inner — pop cache-req-tx, drive, drop.
     ((_inner :())
      (:wat::core::let*
        (((cache-req-tx ...) (HandlePool::pop cache-pool))
         ((_finish ...) (HandlePool::finish cache-pool))
         ((reply-pair ...) ...)
         ((_drive :()) (:my::test::drive-requests cache-req-tx ...)))
        ()))
     ;; cache senders dropped → cache loop exits → cache-driver is
     ;; joinable now. Reporter's captured rundb-clone drops with
     ;; the cache thread's env when the join completes.
     ((_cache-join :()) (:wat::kernel::join cache-driver)))
    ()))

;; Top — deftest body. Owns RunDbService driver.
(:deftest :my::test::full-pipeline
  (:wat::core::let*
    (((rundb-spawn ...) (RunDbService path 1 (null-cadence)))
     ((rundb-pool ...) ...)
     ((rundb-driver ...) ...)
     ;; Inner — pop rundb req-tx, build ack pair, run cache.
     ((_inner :())
      (:wat::core::let*
        (((rundb-req-tx ...) (HandlePool::pop rundb-pool))
         ((_finish ...) (HandlePool::finish rundb-pool))
         ((ack-channel ...) ...)
         ((ack-tx ...) ...)
         ((ack-rx ...) ...)
         ((_run :()) (:my::test::run-cache-with-rundb-tx
                       rundb-req-tx ack-tx ack-rx)))
        ()))
     ;; Inner exited — rundb senders dropped (popped one + reporter's
     ;; captured clone, which run-cache-with-rundb-tx already cleaned
     ;; up by joining cache before returning).
     ((_rundb-join :()) (:wat::kernel::join rundb-driver)))
    (:wat::test::assert-eq true true)))
```

Each function has the canonical Step-3 shape — outer driver, inner
senders. The composition stays clean because each function
encapsulates one driver's lifecycle in two scope levels.

### The anti-pattern (do NOT do this)

```scheme
;; Inline triple-nest — collapses both drivers' lockstep into one
;; let*. cache-req-tx and cache-driver are SAME-SCOPE bindings;
;; joining cache-driver from this scope blocks because cache-req-tx
;; is still alive.
(:wat::core::let*
  (((rundb-spawn ...) ...)
   ((rundb-driver ...) ...)
   ((rundb-req-tx ...) ...)         ; rundb sender lives same scope
   ((cache-spawn ...) ...)
   ((cache-driver ...) ...)
   ((cache-req-tx ...) ...)         ; cache sender same scope
   ((_drive :()) (drive-30 cache-req-tx ...))
   ;; Joining cache-driver here — cache-req-tx is STILL bound;
   ;; cache loop never sees disconnect; deadlock.
   ((_cache-join :()) (:wat::kernel::join cache-driver))
   ((_rundb-join :()) (:wat::kernel::join rundb-driver)))
  (:wat::test::assert-eq true true))
```

The bug is structural: `_cache-join` is bound in the same `let*`
whose body still has cache-req-tx alive. Step 3's "outer holds the
handle; inner owns every Sender" rule still applies — but the
inline mega-`let*` collapses outer and inner into one scope. The
function-decomposition above puts each driver back in its own
outer scope: `run-cache-with-rundb-tx`'s outer scope holds
cache-driver; its inner scope owns cache-req-tx; the function joins
cache-driver after inner returns; the function returns; ITS caller
then drops the rundb senders that the function-args passed in.

### When function decomposition is required

Whenever a Reporter (or any callback) closes over OUTER service
handles. The closure carries an extra ref to the outer service's
senders; that ref lives as long as the closure does. The only way
to ensure the outer service's senders are all gone before joining
the outer driver is to ensure the closure itself is gone — which
means the inner service must have FULLY shut down. A small named
function that joins-before-returning gives you that guarantee for
free.

The "two-level `let*`" rule from Step 3 still holds. Step 9 just
adds: when handles cascade across services, decompose into
functions so each cascade level has its own outer/inner pair.

### Real-world citation

`holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/`
ships this pattern: `drive-requests` / `run-cache-with-rundb-tx`
/ deftest body. Three named functions, three driver lifecycles,
one clean shutdown cascade. The first attempt at the same proof
used the inline triple-nest above and deadlocked; the function-
decomposed version passes (~290ms) and was the recognition that
earned this section.

---

## The complete pattern

The eight steps above compose into one canonical template that covers
**every in-memory request/reply service**. The whole thing is roughly:

> A driver loop holding state, fanning in N request channels via
> `select`, dispatching each request through a per-variant handler
> that returns the new state, exiting cleanly when all client scopes
> drop their Senders.

### Reply shapes

The substrate ships **two** reply shapes; user services may use a
third (see below).

| Variant shape | Reply | Substrate use |
|---|---|---|
| `Ack(... entries, ack-tx)` | `unit` | every batch-write request — caller blocks until durable |
| `Reply(... probes, reply-tx)` | `Vec<Option<V>>` | every batch-read request — caller blocks until results return |

Both shapes carry their reply channel as a field on the variant
(Pattern B routing — see `ZERO-MUTEX.md` § "Routing acks"). Both
shapes are **lock-step** — caller's recv blocks on the substrate
until the driver's send arrives, per Mini-TCP discipline.

Substrate services are bound by `CONVENTIONS.md` §
"Batch convention" — every shipped service exposes only batch-
oriented `get` / `put`. Console is the single exception (it IS
the sink; tag+msg writes don't batch). User services pick
whatever shape fits.

The third shape — `Push(value)`, fire-and-forget, no reply — is
**outside the substrate's surface**. It exists in the kernel
primitives (a `send` without a paired `recv`), and a user
service can use it freely; it does not appear in any wat-rs-
shipped service except as Console's per-message tag+msg
(which is shape-equivalent in the wire but framed by Console's
sink-is-the-report exemption).

### The runnable reference

`wat-rs/wat-tests/service-template.wat` is the canonical complete
template. **Lift it directly when starting your own service** — the
only things that should change are:

- The State struct (your domain — LRU map, treasury record, registry table)
- The Request enum's verbs (your operations)
- The `:svc::*` namespace (rename to `:your::domain::*`)

The wiring (`Service`, `Service/loop`, `Service/handle`, type aliases,
HandlePool, scope discipline) stays. The test deftest exercises both
substrate-shipped reply shapes end-to-end, including a batch-read
that reads LIVE state between two batch-writes — so you can see the
pattern survive real-world operation orders.

### Worked examples in the substrate

Three stdlib services illustrate the canonical batch convention; one
illustrates the Console exemption:

- **Batch reference (Pattern A unit-ack)** —
  `wat-rs/crates/wat-telemetry/wat/telemetry/Service.wat`. N
  producers each send `Request<E> = Vec<E>` worth of events;
  the driver drains and dispatches per batch; ack-tx releases
  the producer when the dispatcher returns.

- **Batch reference (Pattern B data-back)** —
  `wat-rs/crates/wat-lru/wat/lru/CacheService.wat`. Both verbs
  batch: `(get probes :Vec<K>) -> Vec<Option<V>>` and
  `(put entries :Vec<Entry<K,V>>) -> unit`. The Pattern B
  canonical reference per arc 109 § K + arc 119.

- **Same shape with HolonAST as both K and V** —
  `wat-rs/crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`.
  Mirrors LRU's surface exactly; HolonAST-typed.

- **Console exemption** —
  `wat-rs/wat/console.wat`. N tagged-message senders fan into one
  driver that decodes the tag and writes to stdout / stderr.
  Single tag+msg per request — Console IS the sink, so no
  batching. Read this AFTER one of the batch references to see
  how the exemption argument lands.

Read these after the eight steps. They're short and the comments are
dense.

---

## Quick reference

The substrate aliases that make all of this readable
(`wat/kernel/queue.wat`):

| Alias | Expands to |
|---|---|
| `:wat::kernel::Sender<T>` | `:rust::crossbeam_channel::Sender<T>` |
| `:wat::kernel::Receiver<T>` | `:rust::crossbeam_channel::Receiver<T>` |
| `:wat::kernel::Channel<T>` | `:(Sender<T>, Receiver<T>)` |
| `:wat::kernel::Chosen<T>` | `:(i64, Option<T>)` — `select` return |
| `:wat::kernel::Sent` | `:Option<()>` — `send` return |

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
