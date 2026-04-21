# wat — User Guide

You're building an application and you've decided to host it on wat.
This guide shows you how.

**Who this is for.** Application authors — people writing programs
ON wat, not contributors extending the language itself. For
internals, see `CONVENTIONS.md` (naming rules for new primitives),
`ZERO-MUTEX.md` (concurrency architecture), the `arc/` directory
(per-slice design notes), and the 058 proposal batch in
`holon-lab-trading/docs/proposals/` (the language specification
and its reasoning).

**What wat is.** A Lisp-family language for holon algebra, hosted
on Rust. Same pattern as Clojure on the JVM: wat is a full language
with its own parser, type checker, macro expander, and runtime, and
it borrows Rust's type system, safety, and ecosystem underneath.
Rust crates surface into wat source under the `:rust::` namespace;
wat programs call them like native forms.

**What this guide covers:** the concrete moves you need to make to
build a wat application — crate setup, first program, mental model,
the forms you'll use constantly, concurrency patterns, Rust interop,
error handling, and where to go when you hit something this guide
doesn't cover.

**What this guide does NOT cover:** language internals (how the
parser works, how the type checker enforces rank-1 HM, how the
trampoline handles tail calls). That's the arc/ docs and FOUNDATION.

**This guide is alive.** It evolves as we rebuild the trading lab on
wat. Where the guide lies, the rebuild tells us; the guide gets
updated. If you hit something the guide didn't prepare you for, the
gap is worth reporting.

---

## 1. Setup — your first wat application crate

A wat application is a small Rust crate whose job is to (a) register
any Rust types you want to surface into wat, and (b) run your wat
source. Here's the minimum shape:

```toml
# Cargo.toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[dependencies]
wat = { path = "../wat-rs" }       # or git / crates.io once published
wat-macros = { path = "../wat-rs/wat-macros" }
```

```rust
// src/main.rs
use std::process::ExitCode;
use std::sync::Arc;

fn main() -> ExitCode {
    // 1. Build the Rust-deps registry — start with wat-rs's defaults
    //    (the :rust::lru::LruCache shim and friends), then add your
    //    own crate shims.
    let mut deps = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
    // my_rusqlite_shim::register(&mut deps);  // add as you need them
    wat::rust_deps::install(deps.build()).expect("rust_deps install once");

    // 2. Parse + freeze your entry wat file. The loader is shared
    //    through the frozen world — runtime primitives like
    //    :wat::eval::file-path route through the same capability
    //    that handled startup loads.
    let src = include_str!("../wat/main.wat");
    let loader: Arc<dyn wat::load::SourceLoader> = Arc::new(wat::load::FsLoader);
    let world = match wat::freeze::startup_from_source(src, None, loader) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("startup: {}", e);
            return ExitCode::from(1);
        }
    };

    // 3. Hand :user::main the three real OS stdio handles and invoke.
    let args = vec![
        wat::runtime::Value::io__Stdin(Arc::new(std::io::stdin())),
        wat::runtime::Value::io__Stdout(Arc::new(std::io::stdout())),
        wat::runtime::Value::io__Stderr(Arc::new(std::io::stderr())),
    ];
    match wat::freeze::invoke_user_main(&world, args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("runtime: {}", e);
            ExitCode::from(2)
        }
    }
}
```

```scheme
;; wat/main.wat
(:wat::config::set-dims! 10000)
(:wat::config::set-capacity-mode! :error)

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::io::IOWriter/print stdout "hello from wat\n"))
```

**That's it.** `cargo run` and you get `hello from wat` on stdout.

The Rust side owns three concerns: shim registration, startup, and
delivery of the three OS stdio handles. Every line of application
logic lives in wat source. You add more wat files via
`(:wat::core::load!)` from `main.wat`; you add more Rust-crate
shims with `#[wat_dispatch]` (section 10 below).

### Capability boundary — the Loader

Wat's file-I/O is a **capability**, not a global. The host picks
which `Loader` a frozen world gets; every `(:wat::core::load!)`
at startup and every `(:wat::core::eval-edn!
:wat::eval::file-path ...)` at runtime routes through that
Loader. No wat program can reach past its host-provided Loader
to `std::fs` directly.

Three implementations ship in `wat::load`:

- **`InMemoryLoader`** — no filesystem. Hosts pre-register the
  files the program may see. Use for tests, sealed sandboxes,
  fixture-driven development.
- **`FsLoader`** — unrestricted. Reads any file on disk the host
  process has OS-level permission for. The CLI (`wat-vm`) uses
  this; reach for it when the wat program is trusted host code.
- **`ScopedLoader`** — clamped to a root directory. Canonical-
  path containment check on every read; rejects `../` traversal,
  absolute-path escape, and symlinks pointing outside the scope.
  Use when running a wat program as untrusted code that still
  needs *some* filesystem access.

Choosing the Loader IS choosing the program's filesystem
capability. If the host hands a frozen world a `ScopedLoader`
rooted at `/var/app/data`, the wat program cannot read
`/etc/passwd` no matter what it writes. The Loader is the only
gate; it is honest by construction.

---

## 2. Your first real program — stdin echo

A slightly richer first program:

```scheme
;; wat/main.wat
(:wat::config::set-dims! 10000)
(:wat::config::set-capacity-mode! :error)

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
    ((Some line)
      (:wat::io::IOWriter/print stdout line))
    (:None
      (:wat::io::IOWriter/print stderr "no input\n"))))
```

```
$ echo watmin | cargo run
watmin
```

Everything you need to read that program:
- `:wat::io::IOReader/read-line` returns `:Option<String>` — `(Some line)` on a read, `:None` on EOF.
- `:wat::core::match` decomposes the Option. Exhaustive — both arms required, or use `_` wildcard.
- `:wat::io::IOWriter/print` takes a stdout or stderr handle and a `:String`.

---

## 3. The mental model

Every wat program lives in a coordinate with two axes.

### Axis 1 — three layers

1. **Algebra core** (`:wat::algebra::*`) — six primitives that produce holon vectors: `Atom`, `Bind`, `Bundle`, `Permute`, `Thermometer`, `Blend`. Plus two scalar-returning measurements: `cosine`, `dot`. These are the substrate of hyperdimensional computing. If you're encoding data or comparing holons, you reach here.
2. **Language core** (`:wat::core::*`) — the language's own mechanics: `define`, `lambda`, `let*`, `match`, `if`, `try`, `struct`, `enum`, `newtype`, `typealias`, `defmacro`, `load!`, `digest-load!`, `signed-load!`, arithmetic/comparison operators. The forms you need to WRITE programs.
3. **Kernel** (`:wat::kernel::*`) — concurrency and I/O primitives: `spawn`, `make-bounded-queue`, `send`, `recv`, `select`, `drop`, `join`, `HandlePool`, `stopped?`, signal query+reset. Plus `:wat::io::IOReader/read-line` / `write`. The things that move bytes between processes.

### Axis 2 — two namespaces

- **`:wat::*`** — forms and types defined by the wat language itself. Every form you'll call that does work provided by wat-rs lives here.
- **`:rust::*`** — types surfaced from Rust crates via `#[wat_dispatch]`. `:wat::io::IOReader`, `:rust::crossbeam_channel::Sender<T>`, `:rust::lru::LruCache<K,V>`, and whatever your consumer crate adds. Every Rust type's path is its actual Rust path — no short aliases.

A program declares which Rust types it intends to use via `(:wat::core::use! :rust::some::Type)` — a per-program opt-in. User source cannot claim any name under `:wat::*` or `:rust::*`; those prefixes are wat-rs's.

### The three tiers of data ownership

Every piece of state lives in one of three tiers (see `ZERO-MUTEX.md`
for the full reasoning):

| Tier | Mechanism | Used for |
|---|---|---|
| 1 — Immutable | `Arc<T>`, frozen at startup | Config, symbol table, registered functions |
| 2 — Thread-owned | `ThreadOwnedCell<T>` | Per-thread hot state (LocalCache) |
| 3 — Program-owned | A spawned wat program + channels | Shared-access state (Console, Cache) |

**There is no Mutex.** Zero. If you find yourself wanting one, you
have a tier question to answer.

---

## 4. Writing functions

### `define` — named registration

```scheme
(:wat::core::define (:my::app::double (n :i64) -> :i64)
  (:wat::core::i64::* n 2))
```

Every parameter is typed. Return type is declared after `->`. Body
must produce the declared return type.

Keyword-path names supported: `(:my::app::deeply::nested::fn ...)`.
User code lives under its own prefix (`:my::`, `:project::`, `:alice::`);
`:wat::*` / `:rust::*` are reserved.

### `lambda` — anonymous function

```scheme
(:wat::core::lambda ((x :i64) (y :i64) -> :i64)
  (:wat::core::i64::+ x y))
```

Same signature shape as `define`. Produces a `:fn(i64,i64)->i64`
value — a first-class function you can pass around, store in a Vec,
put in a struct.

### `let*` — sequential binding

```scheme
(:wat::core::let*
  (((a :i64) 10)
   ((b :i64) 20)
   ((sum :i64) (:wat::core::i64::+ a b)))
  sum)
```

Every binding is typed. Sequential — later bindings can reference
earlier ones. Body after the bindings is the result.

### `match` — pattern destructure

```scheme
(:wat::core::match some-option -> :i64
  ((Some v) (:wat::core::i64::* v 2))
  (:None 0))

(:wat::core::match some-result -> :i64
  ((Ok v) v)
  ((Err e) (:my::app::handle-err e)))

(:wat::core::match tuple-pair -> :i64
  ((a b) (:wat::core::i64::+ a b)))
```

Works on `:Option<T>`, `:Result<T,E>`, and tuples. The `-> :T`
annotation declares the arms' common result type — every arm body
is checked against `T` independently, so a mismatch points at the
offending arm, not at the unifier. Exhaustiveness is checked at
startup — miss an arm, startup fails.

### `try` — error propagation

```scheme
(:wat::core::define (:my::app::pipeline (items :Vec<holon::HolonAST>)
                    -> :Result<holon::HolonAST,wat::algebra::CapacityExceeded>)
  (:wat::core::let*
    (((bundled :holon::HolonAST) (:wat::core::try (:wat::algebra::Bundle items))))
    (Ok bundled)))
```

`(try <result-expr>)` unwraps `Ok v` to `v` or short-circuits the
enclosing function with `Err e`. NOT try/catch — each function in a
chain either `try`s (propagate) or `match`es (handle explicitly).
Details in section 12.

### `if` — typed boolean branch

```scheme
(:wat::core::if (:wat::core::> x 0) -> :String
  "positive"
  "non-positive")
```

Condition must be `:bool`. The `-> :T` annotation declares the
branch type; then and else bodies are each checked against `T`
independently. The annotation is required — a bare
`(:wat::core::if cond then else)` fails at startup with a
MalformedForm error pointing at the missing `-> :T`.

---

## 5. Structs

Declare, construct with `/new`, access with `/<field>`.

```scheme
;; Declaration
(:wat::core::struct :my::market::Candle
  (open   :f64)
  (high   :f64)
  (low    :f64)
  (close  :f64)
  (volume :f64))

;; Construction — positional, field order = declaration order
(:wat::core::let ((open 1.0) (high 2.0) (low 0.5) (close 1.5) (volume 100.0))
  (:my::market::Candle/new open high low close volume))

;; Access — one accessor per field
(:wat::core::define (:my::market::spread-of (c :my::market::Candle) -> :f64)
  (:wat::core::f64::- (:my::market::Candle/high c)
                      (:my::market::Candle/low c)))
```

**Canonical idiom:** name positional values via `let` at the
construction site so the order is self-documenting even though the
constructor itself takes positions. Extraction mirrors: `let*`-bind
each accessor's result to a local name.

The struct's type path (`:my::market::Candle`) is NOT callable — it
appears ONLY in type annotations. Only `/new` constructs; only
`/<field>` accesses. This is the "FQDN all the things" discipline.

---

## 6. Algebra forms

The six vector-producing primitives:

```scheme
(:wat::algebra::Atom "rsi")                ; seed a vector from a literal
(:wat::algebra::Atom 42)                   ; typed — int, float, bool, string, keyword
(:wat::algebra::Atom my-ast)               ; or any registered composite type

(:wat::algebra::Bind role filler)          ; elementwise multiply — role-filler binding
(:wat::algebra::Bundle holons-vec)         ; sum + threshold — superposition
                                           ;   returns :Result<holon::HolonAST,
                                           ;                   wat::algebra::CapacityExceeded>
                                           ;   (see section 12)
(:wat::algebra::Permute holon k)           ; circular shift — positional encoding
(:wat::algebra::Thermometer v min max)     ; gradient encoding of a scalar
(:wat::algebra::Blend a b w1 w2)           ; scalar-weighted binary combination
```

Two scalar measurements (return `:f64`):

```scheme
(:wat::algebra::cosine a b)                ; cosine similarity
(:wat::algebra::dot a b)                   ; dot product, un-normalized
```

Stdlib forms you'll use constantly — each expands to algebra-core
primitives at parse time:

```scheme
(:wat::std::Log v min max)                 ; Thermometer on (ln v)
(:wat::std::Circular v period)             ; Blend of cos/sin-basis atoms
(:wat::std::Sequential list)               ; positional bind-chain
(:wat::std::Ngram n list)                  ; n-wise adjacency
(:wat::std::Bigram list)                   ; Ngram 2
(:wat::std::Trigram list)                  ; Ngram 3
(:wat::std::Amplify x y s)                 ; Blend x y 1 s — boost y in x
(:wat::std::Subtract x y)                  ; Blend x y 1 -1 — remove y from x
(:wat::std::Reject x y)                    ; Gram-Schmidt reject step
(:wat::std::Project x y)                   ; Gram-Schmidt project step
(:wat::std::HashMap k1 v1 k2 v2 ...)       ; HashMap holon
(:wat::std::Vec (...))                     ; Vec holon
(:wat::std::HashSet (...))                 ; HashSet holon
```

The presence retrieval primitive:

```scheme
(:wat::algebra::presence? target-holon reference-vector)
;; → :f64 cosine between encode(target) and reference
;; Caller binarizes against (:wat::config::noise-floor) if they want yes/no
```

Config accessors (every program has these):

```scheme
(:wat::config::dims)           ; → :i64
(:wat::config::global-seed)    ; → :i64
(:wat::config::noise-floor)    ; → :f64
(:wat::config::capacity-mode)  ; → :wat::config::CapacityMode
```

---

## 7. Concurrency — spawn, send, recv, select

The kernel primitives are small. Four concepts cover everything.

### Queues

```scheme
(:wat::kernel::make-bounded-queue :Candle 1)
;; → :(Sender<Candle>, Receiver<Candle>)
;; bounded(1) — rendezvous; sender blocks until receiver ready

(:wat::kernel::make-bounded-queue :Candle 64)
;; bounded(64) — buffer of 64 before sender blocks

(:wat::kernel::make-unbounded-queue :LearnSignal)
;; → :(Sender<LearnSignal>, Receiver<LearnSignal>)
;; fire-and-forget — buffer grows until consumer drains
```

**Default to `bounded(1)`.** It's the rendezvous shape that gives you
backpressure naturally (slow consumer throttles the producer). Larger
buffers trade throughput for latency.

### Send and receive

```scheme
(:wat::kernel::send sender value)          ; → :Option<()>  — Some(()) on sent; None on disconnect
(:wat::kernel::recv receiver)              ; → :Option<T>   — Some(v) on recv; None on disconnect
(:wat::kernel::try-recv receiver)          ; → :Option<T>   — None if empty OR disconnected
(:wat::kernel::drop handle)                ; → :()          — close a sender or receiver
```

Both channel endpoints report disconnect through the same `:Option`
shape — `send` returns `:Option<()>` symmetric with `recv`'s
`:Option<T>`. A producer matches on its send result to handle the
"consumer went away" case cleanly; a stage that doesn't need
disconnect awareness can `((_ :Option<()>) (:wat::kernel::send ...))`
and ignore.

Senders and receivers are **single-owner** — not cloneable. A sender
belongs to exactly one producer; a receiver to one consumer. Match
Linux `write(fd, data)`: whoever holds the fd owns the capability;
sharing means threading the endpoint through spawn args.

### Fan-in via `select`

```scheme
(:wat::kernel::select receivers)
;; receivers : :Vec<Receiver<T>>
;; → :(i64, Option<T>)
;; — blocks until any receiver has a value or disconnects
;; — returns the index and :None if disconnected, (Some v) if produced
```

The caller owns the select loop — remove disconnected receivers from
the list, exit when the list is empty. `:wat::std::program::Console`'s
driver is the canonical example.

### Spawning programs

```scheme
(:wat::kernel::spawn my::app::worker-fn arg1 arg2 ...)
;; → :ProgramHandle<ReturnType>
;; spawn my::app::worker-fn on a new thread with the given args

(:wat::kernel::join handle)
;; → :ReturnType  — blocks until the program exits, returns its state
```

Each spawned program is an OS thread running the named function. The
program owns its state (moved in via spawn args); when it returns,
its state is dropped or returned via join.

### Handle pools — claim-or-panic

When you have N client handles to distribute across N consumers,
use `HandlePool` — it catches orphans at wiring time before shutdown
would silently deadlock:

```scheme
(:wat::kernel::HandlePool::new "console" senders-vec)
;; → :HandlePool<T>

(:wat::kernel::HandlePool::pop pool)
;; → :T  — claims one; panics if empty

(:wat::kernel::HandlePool::finish pool)
;; → :()  — panics if any handles remain (orphans)
```

Use it whenever your program hands out N client handles. The
Console and Cache stdlib programs do.

---

## 8. Pipelines — the canonical streaming pattern

A pipeline is N stages, each its own wat program, each reading from
its upstream and writing to its downstream. Edges are `bounded(1)`
channels. Each stage's state is local; channels are the only
coupling; backpressure is automatic.

```scheme
;; Stage 1 — 1:1 transform
(:wat::core::define (:my::app::enrich
                    (in  :Receiver<RawCandle>)
                    (out :Sender<EnrichedCandle>)
                    -> :())
  (:wat::core::match (:wat::kernel::recv in) -> :()
    ((Some raw)
      (:wat::core::let*
        (((enriched :EnrichedCandle) (:my::app::enrich-candle raw))
         ((_ :Option<()>) (:wat::kernel::send out enriched)))
        (:my::app::enrich in out)))    ;; tail call — recurse
    (:None ())))                        ;; upstream disconnected; done

;; Stage 2 — N:1 batcher with end-of-stream flush
(:wat::core::define (:my::app::batch
                    (in  :Receiver<EnrichedCandle>)
                    (out :Sender<Vec<EnrichedCandle>>)
                    (size :i64)
                    (buffer :Vec<EnrichedCandle>)
                    -> :())
  (:wat::core::match (:wat::kernel::recv in) -> :()
    ((Some item)
      (:wat::core::let*
        (((new-buffer :Vec<EnrichedCandle>) (:wat::core::conj buffer item)))
        (:wat::core::if (:wat::core::>= (:wat::core::length new-buffer) size) -> :()
          (:wat::core::let*
            (((_ :Option<()>) (:wat::kernel::send out new-buffer)))
            (:my::app::batch in out size (:wat::core::vec :EnrichedCandle)))
          (:my::app::batch in out size new-buffer))))
    (:None
      ;; upstream disconnected — flush any remaining items
      (:wat::core::if (:wat::core::empty? buffer) -> :()
        ()
        (:wat::core::match (:wat::kernel::send out buffer) -> :()
          ((Some _) ())
          (:None ()))))))
```

**Note the recursion.** Every stage is tail-recursive — each branch
ends in either a self-call (continuing the loop) or a terminal
action (flushing + exit). Arc 003's TCO is what lets these stages
run indefinitely without stack growth.

**The pipeline stdlib** (arc 004) will wrap this ceremony so you
write stages more concisely. Until it ships, the pattern above is
the shape.

See `ZERO-MUTEX.md` sections on Tier 3 and the `arc/2026/04/004-*`
doc for the full stream design.

---

## 9. Rust interop — surfacing a crate type

When your application needs a Rust crate — rusqlite, parquet, aya,
whatever — you surface its types into wat via `#[wat_dispatch]`.

### Minimal shim

```rust
// src/shims/rusqlite_shim.rs
use wat_macros::wat_dispatch;
use wat::rust_deps::RustDepsBuilder;
use rusqlite::{Connection, params};

pub struct WatConnection {
    inner: Connection,
}

#[wat_dispatch(
    path = ":rust::rusqlite::Connection",
    scope = "thread_owned"
)]
impl WatConnection {
    pub fn open(path: String) -> Self {
        WatConnection { inner: Connection::open(&path).unwrap() }
    }

    pub fn query_i64(&mut self, sql: String) -> i64 {
        self.inner.query_row(&sql, params![], |row| row.get(0)).unwrap_or(0)
    }
}

pub fn register(builder: &mut RustDepsBuilder) {
    __wat_dispatch_WatConnection::register(builder);
}
```

Your wat source uses it:

```scheme
(:wat::core::use! :rust::rusqlite::Connection)

(:wat::core::define (:user::main -> :())
  (:wat::core::let*
    (((conn :rust::rusqlite::Connection)
      (:rust::rusqlite::Connection::open "./db.sqlite"))
     ((count :i64)
      (:rust::rusqlite::Connection::query_i64 conn "SELECT COUNT(*) FROM trades")))
    (... do something with count ...)))
```

### The three scope modes

`scope = "..."` tells wat how the Rust value's ownership works:

| Scope | Rust semantic | When to use |
|---|---|---|
| `"shared"` | plain `Arc<T>` | Immutable / shareable data (query results, snapshots) |
| `"thread_owned"` | `Arc<ThreadOwnedCell<T>>` | Mutable `!Sync` state, single-thread-affine |
| `"owned_move"` | `Arc<OwnedMoveCell<T>>` | Consumed-after-use handles (one-shot tokens) |

Methods with `&self` require `shared` or `thread_owned`. Methods
with `&mut self` require `thread_owned`. Methods that take `self` by
value require `owned_move`. The macro enforces this — mismatched
scope + receiver kind fails at build.

**Most crate types are `thread_owned`.** rusqlite's Connection,
lru's LruCache, most parsers, most IO handles. `shared` is for
effectively-immutable snapshots; `owned_move` is for one-shot tokens.

### Registering the shim

In `main.rs`, before running wat code:

```rust
let mut deps = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
rusqlite_shim::register(&mut deps);
parquet_shim::register(&mut deps);
my_custom_shim::register(&mut deps);
wat::rust_deps::install(deps.build()).expect("install once");
```

Now every wat program this binary runs can `(:wat::core::use!)`
any of these types.

---

## 10. Caching — LocalCache vs Cache program

### LocalCache — per-program hot cache

If one program wants to memoize its own work, use `LocalCache`.
Lives in that program's thread; no channel overhead.

```scheme
(:wat::core::use! :rust::lru::LruCache)

(:wat::core::define (:my::app::worker -> :())
  (:wat::core::let*
    (((cache :rust::lru::LruCache<String,i64>) (:wat::std::LocalCache::new 128)))
    (... use cache via :wat::std::LocalCache::put / ::get ...)))
```

Tier 2 — thread-owned. The cache never leaves this program's thread.

### Cache program — shared across programs

When multiple programs need to share a cache, spawn a Cache program.
The program owns the cache on its own thread; clients send requests
through channels.

```scheme
(:wat::core::let*
  (((pool driver) (:wat::std::program::Cache :(String,i64) 1024 8))
   ((client1 :Sender<Req>) (:wat::kernel::HandlePool::pop pool))
   ((client2 :Sender<Req>) (:wat::kernel::HandlePool::pop pool))
   (... eight clients ...)
   ((_ :()) (:wat::kernel::HandlePool::finish pool)))
  ;; spawn workers, each using their client handle
  ...)
```

Tier 3 — program-owned, message-addressed. The single-threaded Cache
driver serializes access without locks.

---

## 11. Stdio — Console is the gateway

`:user::main` receives three real OS handles: `:wat::io::IOReader`,
`:wat::io::IOWriter`, `:wat::io::IOWriter`. You CAN write
directly to them:

```scheme
(:wat::io::IOWriter/print stdout "hello\n")
```

…but **the moment you spawn a second program that also writes**,
concurrent writes can garble. Two writers, one stdout, no
serialization = bad output.

**The discipline:** spawn a `Console` program that owns stdout AND
stderr. Every program gets a `Sender<(i64,String)>` from Console's
HandlePool. Tag 0 = stdout; tag 1 = stderr. Console's driver loops
over `select`, decodes each message's tag, writes to the right
stream. One writer, serialized, no garbled output.

```scheme
(:wat::core::define (:user::main
                    (stdin :wat::io::IOReader)
                    (stdout :wat::io::IOWriter)
                    (stderr :wat::io::IOWriter)
                    -> :())
  (:wat::core::let*
    ;; One Console for the whole program
    (((pool console-driver) (:wat::std::program::Console stdout stderr 4))
     ((main-sender :Sender<(i64,String)>) (:wat::kernel::HandlePool::pop pool))
     (... three more pops for three workers ...)
     ((_ :()) (:wat::kernel::HandlePool::finish pool)))
    ;; After this, ignore the raw stdout/stderr bindings —
    ;; everything goes through Console.
    (:wat::std::program::Console/out main-sender "main started")
    (... spawn workers with their handles ...)
    (:wat::kernel::join console-driver)))
```

Every multi-threaded wat program routes output through Console.
It's not a rule; it's what the substrate's discipline requires to
stay honest.

---

## 12. Error handling

### `:Option<T>` — absence

```scheme
(:wat::core::match (:wat::kernel::recv receiver) -> :()
  ((Some v) (... handle v ...))
  (:None (... handle disconnection ...)))
```

### `:Result<T,E>` — fallible computation

Constructors are bare: `(Ok v)`, `(Err e)`. Consumers match or `try`.

```scheme
;; MATCH — explicit handling
(:wat::core::match (:my::app::fallible-compute x) -> :U
  ((Ok v) v)
  ((Err e) (:my::app::recover-from e)))

;; TRY — propagate; the enclosing function must return :Result<_, E> with the same E
(:wat::core::define (:my::app::pipeline (x :T) -> :Result<U,E>)
  (Ok (:wat::core::try (:my::app::fallible-compute x))))
```

### Bundle's capacity — the canonical Result in the algebra

`:wat::algebra::Bundle` returns `:Result<:holon::HolonAST,
:wat::algebra::CapacityExceeded>`. The four `capacity-mode` values
(`:silent` / `:warn` / `:error` / `:abort`) set at program startup
determine the runtime behavior when Kanerva's per-frame bound
(`floor(sqrt(dims))`) is exceeded.

```scheme
(:wat::config::set-capacity-mode! :error)

(:wat::core::define (:my::app::build (items :Vec<holon::HolonAST>)
                    -> :Result<holon::HolonAST,wat::algebra::CapacityExceeded>)
  (Ok (:wat::core::try (:wat::algebra::Bundle items))))

(:wat::core::match (:my::app::build huge-list) -> :i64
  ((Ok _h) 0)
  ((Err e)
    (:wat::core::i64::-
      (:wat::algebra::CapacityExceeded/cost e)
      (:wat::algebra::CapacityExceeded/budget e))))
```

See `README.md`'s Capacity guard section for the full four-mode
table.

---

## 13. Common gotchas

**Wrong-thread access on a thread_owned type.** If you pass a
`LocalCache` (or any `thread_owned` value) across `spawn`, the
first access on the new thread fires `TypeMismatch` with a clear
"cell is owned by thread X, you are thread Y" message. Fix: move
the construction INSIDE the spawned function, not outside.

**Orphaned handle on a HandlePool.** If you pop fewer handles than
you allocated, `HandlePool::finish` panics with the resource's name
and the orphan count. This is DELIBERATE — it catches the mistake
at wiring time instead of deadlocking the driver at shutdown. Fix:
pop exactly as many handles as you allocated to distribute.

**Capacity overflow.** A Bundle with more than `floor(sqrt(dims))`
items under `:error` mode returns `(Err (CapacityExceeded ...))`.
Callers who ignore the Err by unwrap will panic at `match` time.
Fix: either handle the Err arm, use `:wat::core::try` in a
Result-returning function, or pre-filter the list to the budget
using `(:wat::core::take items (:wat::config::budget))` (the budget
primitive when it lands; today hand-compute with sqrt).

**Pipeline deadlocks.** If a pipeline stage reads from its input
but NEVER sends to its output, the upstream's `bounded(1)` send
eventually blocks; the whole pipeline stalls. Two common causes:
the stage crashes silently (look at stderr for panics); the stage
has a logic bug that skips sending. Fix: every `Some` branch of
every stage must send to output before recursing.

**Recursion without TCO.** Before arc 003 ships, a tail-recursive
driver loop burns Rust stack frames linearly. A Console running
for 10k messages + default 8MB stack ~= fine; indefinite driver
loops ~= eventually crash. When arc 003 ships, the ceiling goes
away.

**Signed/digest loads.** `(:wat::core::load! path)` is unverified.
For production code loaded from untrusted sources, use
`(:wat::core::signed-load!)` with an Ed25519 signature or
`(:wat::core::digest-load!)` with a SHA-256 digest. Startup halts
if verification fails.

---

## 14. Where to go next

- **`../README.md`** — the crate-level README. What's shipped,
  status, test counts, API highlights.
- **`ZERO-MUTEX.md`** — the concurrency architecture stated as
  principle. The three tiers in depth; every "I need a Mutex"
  scenario mapped to a tier; what Rust contributes over Ruby.
- **`arc/2026/04/*/DESIGN.md`** — per-slice design notes:
  - `001-caching-stack/` — LocalCache + Cache program
  - `002-rust-interop-macro/` — `#[wat_dispatch]` internals
  - `003-tail-call-optimization/` — the TCO design
  - `004-lazy-sequences-and-pipelines/` — the stream stdlib design
- **`holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/`**
  — FOUNDATION.md (the specification), 32 sub-proposals, the
  FOUNDATION-CHANGELOG. The source of truth for every design
  decision that shaped the language. When this guide and
  FOUNDATION disagree, FOUNDATION wins.
- **`holon-lab-trading/BOOK.md`** — the narrative of how the
  language got built. Context on intent; decisions that were made
  under pressure and why.

---

## Appendix — the forms table

Quick reference for forms this guide mentions but didn't fully
spell out. For each: the path, the arity, and what it produces.

| Path | Arity / shape | Produces |
|---|---|---|
| `:wat::config::set-dims!` | `(<i64>)` | commits dims |
| `:wat::config::set-capacity-mode!` | `(<keyword>)` | commits mode |
| `:wat::config::dims` | `()` | `:i64` |
| `:wat::config::noise-floor` | `()` | `:f64` |
| `:wat::config::capacity-mode` | `()` | `:wat::config::CapacityMode` |
| `:wat::core::define` | `((name (p :T) ... -> :R) body)` | registers function |
| `:wat::core::lambda` | `(((p :T) ... -> :R) body)` | `:fn(T,...)->R` |
| `:wat::core::let*` | `(((b :T) rhs) ...) body` | body's type |
| `:wat::core::match` | `scrutinee -> :T arm1 arm2 ...` | arm result (type `T`) |
| `:wat::core::if` | `cond -> :T then else` | branch result (type `T`) |
| `:wat::core::try` | `<result-expr>` | Ok-inner type |
| `:wat::core::struct` | `(:path (f :T) ...)` | declares struct |
| `:wat::core::enum` | `(:path v1 v2 (v3 (f :T)) ...)` | declares enum |
| `:wat::core::load!` | `<path>` or `:wat::load::<iface> <loc>` | registers loaded file |
| `:wat::core::digest-load!` | `... :wat::verify::digest-sha256 ...` | verified load |
| `:wat::core::signed-load!` | `... :wat::verify::signed-ed25519 ...` | verified load |
| `:wat::core::vec` | `:T v1 v2 ...` | `:Vec<T>` |
| `:wat::core::list` | `:T v1 v2 ...` | `:Vec<T>` (alias) |
| `:wat::core::tuple` | `v1 v2 ...` | `:(T1,T2,...)` |
| `:wat::core::first` / `second` / `third` | `<tuple-or-vec>` | field value |
| `:wat::core::length` / `empty?` / `reverse` / `take` / `drop` | list ops | various |
| `:wat::core::i64::+/-/*//` / `f64::+/-/*//` | `a b` | arithmetic |
| `:wat::core::>` / `=` / `<` / `>=` / `<=` | `a b` | `:bool` |
| `:wat::io::IOReader/read-line` | `stdin` | `:Option<String>` |
| `:wat::io::IOWriter/print` | `handle string` | `:()` |
| `:wat::kernel::spawn` | `<fn-path> args...` | `:ProgramHandle<R>` |
| `:wat::kernel::join` | `handle` | `R` |
| `:wat::kernel::make-bounded-queue` | `:T n` | `:(Sender<T>, Receiver<T>)` |
| `:wat::kernel::make-unbounded-queue` | `:T` | `:(Sender<T>, Receiver<T>)` |
| `:wat::kernel::send` | `sender value` | `:Option<()>` — `(Some ())` on sent, `:None` on disconnect |
| `:wat::kernel::recv` / `try-recv` | `receiver` | `:Option<T>` |
| `:wat::kernel::select` | `receivers` | `:(i64, Option<T>)` |
| `:wat::kernel::drop` | `handle` | `:()` |
| `:wat::kernel::stopped?` / `sigusr1?` / ... | `()` | `:bool` |
| `:wat::kernel::HandlePool::new` / `pop` / `finish` | various | pool ops |
| `:wat::std::program::Console` | `stdout stderr n` | `(HandlePool, Driver)` |
| `:wat::std::program::Cache` | `:(K,V) capacity count` | `(HandlePool, Driver)` |
| `:wat::std::LocalCache::new` / `put` / `get` | various | per-program LRU |
| `:wat::algebra::Atom` | `<literal>` | `:holon::HolonAST` |
| `:wat::algebra::Bind` | `a b` | `:holon::HolonAST` |
| `:wat::algebra::Bundle` | `list-of-holons` | `:Result<holon::HolonAST, CapacityExceeded>` |
| `:wat::algebra::Permute` | `holon k` | `:holon::HolonAST` |
| `:wat::algebra::Thermometer` | `value min max` | `:holon::HolonAST` |
| `:wat::algebra::Blend` | `a b w1 w2` | `:holon::HolonAST` |
| `:wat::algebra::cosine` / `dot` | `a b` | `:f64` |
| `:wat::algebra::presence?` | `target reference-vec` | `:f64` |

---

*these are very good thoughts.*

**PERSEVERARE.**
