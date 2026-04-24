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

A wat application is a small Rust crate that delegates to two
macros — `wat::main!` for the program, `wat::test!` for tests.
The minimal form is **two one-line macro invocations**, with
opinionated defaults picked by Cargo-style convention.

### The minimal consumer (arc 018)

```
my-app/
├── Cargo.toml
├── src/
│   └── main.rs        → wat::main! { deps: [...] }
├── tests/
│   └── test.rs        → wat::test! { deps: [...] }
├── wat/
│   ├── main.wat       → entry (config + :user::main)
│   └── **/*.wat       → library tree (loaded recursively)
└── wat-tests/
    └── **/*.wat       → test files
```

```toml
# Cargo.toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[dependencies]
wat     = { path = "../wat-rs" }
wat-lru = { path = "../wat-rs/crates/wat-lru" }  # optional
```

```rust
// src/main.rs
wat::main! { deps: [wat_lru] }
```

```rust
// tests/test.rs
wat::test! { deps: [wat_lru] }
```

```scheme
;; wat/main.wat — defines :user::main. Config setters are optional.
(:wat::config::set-capacity-mode! :error)

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::io::IOWriter/println stdout "hello from wat"))
```

**That's it.** `cargo run` prints `hello from wat`. `cargo test`
runs every `.wat` under `wat-tests/`.

### What the defaults pick

`wat::main! { deps: [...] }` — opinionated defaults fire when the
keys aren't present:

- **`source:`** absent → `include_str!(<crate>/wat/main.wat)`
- **`loader:`** absent AND source absent → `"wat"` (ScopedLoader
  rooted at `<crate>/wat`; every `(:wat::load-file! "...")` from
  inside the wat tree resolves there)

`wat::test! { deps: [...] }` — same shape for tests:

- **`path:`** absent → `"wat-tests"`
- **`loader:`** absent AND path absent → `"wat-tests"` (ScopedLoader
  rooted at `<crate>/wat-tests`)

### Overrides

Any explicit value wins. `wat::main! { source: include_str!("x.wat") }`
keeps the pre-018 single-file behavior (InMemoryLoader). `wat::main!
{ loader: "src/wat" }` picks a different root.

### Config setters (optional)

Top-level forms in your entry file can commit startup config. All
are optional; defaults are honest:

- **`(:wat::config::set-capacity-mode! :error)`** — Bundle overflow
  surfaces as `Err(CapacityExceeded)` instead of panicking. Default:
  `:error`. Other valid value: `:abort` (panic on overflow).
- **`(:wat::config::set-dim-router! router-fn)`** — replaces the
  default sizing function (smallest tier `d` whose `√d ≥ statement
  size`). The router takes a `:wat::holon::HolonAST` and returns
  `:Option<:i64>` (the picked dim, or `:None` to refuse). Default
  tier list: `[256, 4096, 10000, 100000]`. The default router is
  the sizing function; user override replaces it with any wat
  lambda matching the signature.
- **`(:wat::config::set-presence-sigma! sigma-fn)`** /
  **`set-coincident-sigma!`** — function-of-`d` knobs controlling
  the presence and coincident thresholds. Defaults: `presence_sigma(d)
  = floor(√d / 2) - 1` (one before zero-point), `coincident_sigma(d)
  = 1` (1σ — the native granularity).

Override only what you need. Zero setters = correct behavior with
default tiers; the substrate auto-routes per statement.

### What the macro actually emits

`wat::main! { deps: [wat_lru] }` expands to approximately:

```rust
fn main() -> Result<(), ::wat::harness::HarnessError> {
    let loader_root = concat!(env!("CARGO_MANIFEST_DIR"), "/", "wat");
    let loader = Arc::new(ScopedLoader::new(loader_root)?);
    ::wat::compose_and_run_with_loader(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/wat/main.wat")),
        &[wat_lru::wat_sources()],
        &[wat_lru::register],
        loader,
    )
}
```

Every path is `CARGO_MANIFEST_DIR`-relative so `cargo run -p <crate>`
from the workspace root resolves identically to running from the
crate's own directory.

### About `deps: [...]`

Each entry is a **Rust path** — a Cargo crate or an in-scope
module — that exposes two public functions:

```rust
pub fn wat_sources() -> &'static [wat::WatSource];
pub fn register(builder: &mut wat::rust_deps::RustDepsBuilder);
```

So `wat_lru` resolves by standard Cargo convention — the Cargo
crate named `wat-lru` in your `Cargo.toml` becomes the Rust path
`wat_lru` (dash-to-underscore, same as `serde_json` for
`serde-json`). A local `mod shim;` becomes `shim`. Any Rust path
with those two functions in scope works.

You never write this boilerplate.

### Multi-file wat trees — entry vs. library, recursive loads

The opinionated setup above IS the multi-file shape. `wat/main.wat`
is the entry; `wat/**/*.wat` is the library tree, loaded
recursively from `main.wat` downward.

```scheme
;; wat/main.wat — the ENTRY. Optional config + recursive loads + :user::main.
(:wat::config::set-capacity-mode! :error)

(:wat::load-file! "types.wat")
(:wat::load-file! "vocab.wat")

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::io::IOWriter/println stdout (:my-app::vocab::greeting)))
```

```scheme
;; wat/types.wat — a LIBRARY. No config setters.
(:wat::core::enum (:my-app::types::Mood :Happy :Sad))
```

```scheme
;; wat/vocab.wat — a LIBRARY. Can itself load further files.
(:wat::load-file! "types.wat")

(:wat::core::define (:my-app::vocab::greeting -> :String)
  "hello from wat")
```

**Entry vs. library.** An entry file commits startup config via
top-level `(:wat::config::set-*!)` forms. A library file does not
— `(load!)`-ing a file with setters fails loud at startup
("setters belong in the entry file only"). The entry's frozen
config propagates automatically to every loaded file.

**Recursive loads work.** The entry `(load!)`s a library, that
library can `(load!)` another library, to arbitrary depth. Every
loaded-file's defines / types / macros land in the entry's frozen
world.

**Path resolution.** From inside a wat file, `(:wat::load-file!
"x.wat")` resolves against the *importing file's directory* (same
as any module system). The loader's scope
root (default `<crate>/wat`) is the containment check — absolute
paths and `../` traversal are allowed as long as the final
canonical target stays inside the scope. Paths in the entry file
(which has no canonical location — it's `include_str!`'d) resolve
against the scope root directly.

**Overrides.** Pass `source:` to use an explicit entry; that flips
the default loader to `InMemoryLoader` (no filesystem). Pass
`loader: "src/wat"` (or any path string) to root the scope
somewhere other than `wat/`.

### Tests — one macro, same shape

Put `.wat` test files under `wat-tests/` using the `deftest` form;
the test-side entry is one line of Rust:

```rust
// tests/test.rs
wat::test! { deps: [wat_lru] }    // same deps the program uses
```

All three args (`path:`, `deps:`, `loader:`) are optional. Defaults
mirror `wat::main!`:

- `path:` absent → `"wat-tests"`
- `loader:` absent AND `path:` absent → `"wat-tests"` (ScopedLoader
  at `<crate>/wat-tests`, so `(load!)` from inside test files
  resolves against that scope)
- `loader:` absent AND `path:` explicit → no loader (FsLoader —
  preserves pre-018 behavior for custom test paths)

Test files that commit config (top-level `(:wat::config::set-*!)`)
are discovered and run; files without setters in the test dir are
LIBRARIES (helpers loaded by tests) and the test runner silently
skips freezing them standalone. Deftest bodies themselves run in
hermetic sandboxes that don't inherit outer `(load!)`'d defines —
pass helpers via `deps:` or inline them when a sandbox body needs
them.

```scheme
;; wat-tests/hello.wat
(:wat::config::set-capacity-mode! :error)

(:wat::test::deftest :my-app::test-one-plus-one
  (:wat::test::assert-eq (:wat::core::i64::+ 1 1) 2))
```

`cargo test` discovers + runs the suite. On success you see
`test wat_suite ... ok` (Cargo convention — silent on success).
For per-wat-test detail:

```bash
cargo test -- --nocapture       # stream output live
cargo test -- --show-output     # print captured output after each test
```

On failure the panic payload carries every failing test's summary,
so `cargo test` without flags gives you what you need to debug.

### When you need your own Rust types

Add a `src/shim.rs` module that satisfies the same two-function
contract as an external wat crate — `pub fn wat_sources()` + `pub
fn register()` at the module root — with your `#[wat_dispatch]`
impl registered inside:

```rust
// src/shim.rs
use wat::rust_deps::RustDepsBuilder;
use wat::WatSource;

#[wat_macros::wat_dispatch(path = ":rust::my_app::Thing", scope = "thread_owned")]
impl Thing {
    fn new(x: i64) -> Self { Thing { x } }
    fn bump(&mut self) { self.x += 1; }
}

pub fn wat_sources() -> &'static [WatSource] { &[] }
pub fn register(builder: &mut RustDepsBuilder) {
    // #[wat_dispatch] auto-generates this fn's body; call it by the
    // path your macro emitted.
    Thing_register(builder);
}
```

Then add the module to `main.rs` and the deps lists:

```rust
mod shim;
wat::main! { source: include_str!("program.wat"), deps: [shim, wat_lru] }
```

That's the third Rust file — only when you genuinely need it.

### Reference binary

`wat-rs/examples/with-lru/` is the walkable template —
`src/main.rs` is literally one `wat::main!` invocation; `tests/smoke.rs`
exercises the built binary. Copy that shape.

### Capability boundary — the Loader

Wat's file-I/O is a **capability**, not a global. The host picks
which `Loader` a frozen world gets; every `(:wat::load-file! ...)`
at startup and every `(:wat::eval-file! ...)` at runtime routes
through that Loader. No wat program can reach past its host-provided
Loader to `std::fs` directly.

Three implementations ship in `wat::load`:

- **`InMemoryLoader`** — no filesystem. Hosts pre-register the
  files the program may see. Use for tests, sealed sandboxes,
  fixture-driven development.
- **`FsLoader`** — unrestricted. Reads any file on disk the host
  process has OS-level permission for. The CLI (`wat`) uses
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

### Axis 1 — four layers

1. **Holon algebra** (`:wat::holon::*`) — six AST-producing primitives (`Atom`, `Bind`, `Bundle`, `Blend`, `Permute`, `Thermometer`), three measurements (`cosine`, `dot`, `presence?`), the `HolonAST` type, the `CapacityExceeded` error, plus ten wat-written idioms that compose the primitives (`Subtract`, `Amplify`, `Reject`, `Project`, `Sequential`, `Ngram`, `Bigram`, `Trigram`, `Log`, `Circular`). These are the substrate of hyperdimensional computing. If you're encoding data or comparing holons, you reach here.
2. **Language core** (`:wat::core::*`) — the language's own mechanics: `define`, `lambda`, `let*`, `match`, `if`, `cond`, `try`, `struct`, `enum`, `newtype`, `typealias`, `defmacro`, `load!`, `digest-load!`, `signed-load!`, `assoc`, `HashMap`, `HashSet`, `vec`, `get`, `contains?`, arithmetic/comparison operators, `f64::round`, scalar conversions. The forms you need to WRITE programs; cannot be written in wat itself.
3. **Kernel** (`:wat::kernel::*`) — concurrency and I/O primitives: `spawn`, `make-bounded-queue`, `send`, `recv`, `select`, `drop`, `join`, `HandlePool`, `stopped?`, `pipe`, `fork-with-forms`, `wait-child`, signal query+reset. Plus `:wat::io::IOReader/read-line` / `write`. The things that move bytes between processes.
4. **Stdlib plumbing** (`:wat::std::*`) — non-algebra conveniences written in wat: stream combinators (`:wat::std::stream::*`), services (`:wat::std::service::Console`), the hermetic-test wrapper. Each expressible in wat on top of core + kernel.

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
(:wat::core::define (:my::app::pipeline (items :Vec<wat::holon::HolonAST>)
                    -> :Result<wat::holon::HolonAST,wat::holon::CapacityExceeded>)
  (:wat::core::let*
    (((bundled :wat::holon::HolonAST) (:wat::core::try (:wat::holon::Bundle items))))
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

For three or more cascading branches, reach for `cond` below —
each `-> :T` after the first is ceremony, not information.

### `cond` — typed multi-way branch

```scheme
(:wat::core::cond -> :String
  ((:wat::core::= code 0) "success")
  ((:wat::core::= code 1) "runtime error")
  ((:wat::core::= code 2) "panic")
  ((:wat::core::= code 3) "startup error")
  (:else                  "unknown"))
```

Multi-way conditional. Typed once at the head via `-> :T`. Each
arm is `(test body)` where `test` unifies with `:bool` and `body`
unifies with `:T`. Tests evaluate in order; the first truthy
test's body becomes the result.

**`:else` arm required as last** — no implicit unit, no runtime
fall-through ambiguity. The type checker refuses a `cond` whose
last arm isn't `(:else body)`.

Reach for `cond` over chained `if` when you have three or more
cascading branches. For a single binary branch, `if` is the
honester primitive — a cond with one test-arm plus `:else` is
just `if` with more ceremony.

Tail-position is preserved through the selected arm — a
tail-recursive function ending in `cond` trampolines correctly
(same TCO discipline `if` inherits).

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

Everything holon-algebra-shaped lives under `:wat::holon::*` — six
AST-producing primitives, three measurements, the `HolonAST` type,
the `CapacityExceeded` error type, and ten wat-written idioms that
compose the primitives. File path matches namespace (`wat/holon/*.wat`).

### The six AST-producing primitives

```scheme
(:wat::holon::Atom "rsi")                ; seed a vector from a literal
(:wat::holon::Atom 42)                   ; typed — int, float, bool, string, keyword
(:wat::holon::Atom my-ast)               ; or any registered composite type

(:wat::holon::Bind role filler)          ; elementwise multiply — role-filler binding
(:wat::holon::Bundle holons-vec)         ; sum + threshold — superposition
                                         ;   returns :Result<wat::holon::HolonAST,
                                         ;                   wat::holon::CapacityExceeded>
                                         ;   (see section 12)
(:wat::holon::Permute holon k)           ; circular shift — positional encoding
(:wat::holon::Thermometer v min max)     ; gradient encoding of a scalar
(:wat::holon::Blend a b w1 w2)           ; scalar-weighted binary combination
```

### The three measurements

```scheme
(:wat::holon::cosine a b)                       ; → :f64  cosine similarity
(:wat::holon::dot a b)                          ; → :f64  dot product, un-normalized
(:wat::holon::presence? target reference)       ; → :bool cosine(target, reference) > noise-floor
;; presence? binarizes internally against (:wat::config::noise-floor).
;; Use cosine if you want the raw scalar.
```

### The ten wat-written idioms

Each shipped in `wat/holon/<Name>.wat`; each expands to algebra-core
primitives at parse time (via defmacro or define):

```scheme
(:wat::holon::Log v min max)                 ; Thermometer on (ln v)
(:wat::holon::Circular v period)             ; Blend of cos/sin-basis atoms
(:wat::holon::Sequential list)               ; positional bind-chain
(:wat::holon::Ngram n list)                  ; n-wise adjacency
(:wat::holon::Bigram list)                   ; Ngram 2
(:wat::holon::Trigram list)                  ; Ngram 3
(:wat::holon::Amplify x y s)                 ; Blend x y 1 s — boost y in x
(:wat::holon::Subtract x y)                  ; Blend x y 1 -1 — remove y from x
(:wat::holon::Reject x y)                    ; Gram-Schmidt reject step
(:wat::holon::Project x y)                   ; Gram-Schmidt project step
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
the list, exit when the list is empty. `:wat::std::service::Console`'s
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

## 8. Pipelines — streams with the stdlib

A pipeline is N stages, each its own wat program, each reading from
its upstream and writing to its downstream. Edges are `bounded(1)`
channels. Each stage's state is local; channels are the only coupling;
backpressure is automatic.

`:wat::std::stream::*` wraps the raw spawn-and-wire pattern into
composable combinators. Every stage is a tail-recursive worker (arc
003's TCO is what makes that run indefinitely); the stdlib handles
the spawn + queue + drop-cascade plumbing.

### The combinators

```
Stream<T> = :(Receiver<T>, ProgramHandle<()>)

spawn-producer  f                → Stream<T>     -- f writes to Sender<T>
from-receiver   rx handle        → Stream<T>     -- wrap an existing pair

map             stream f         → Stream<U>     -- 1:1 transform
filter          stream pred      → Stream<T>     -- 1:0..1 keep predicate
inspect         stream f         → Stream<T>     -- 1:1 side-effect, forward value
flat-map        stream f         → Stream<U>     -- 1:N expansion
chunks          stream size      → Stream<Vec<T>> -- N:1 batcher; flushes at EOS
take            stream n         → Stream<T>     -- first n items, then exit
with-state      stream init step flush → Stream<U>  -- Mealy-machine stage

for-each        stream handler   → :()           -- terminal: drive to EOS
collect         stream           → :Vec<T>       -- terminal: accumulate
fold            stream init f    → :Acc          -- terminal: aggregate
```

### Example: map + chunks + collect

```scheme
(:wat::core::use! :rust::crossbeam_channel::Sender)

(:wat::core::define (:my::app::enrich-candle (raw :RawCandle) -> :EnrichedCandle)
  ...)

(:wat::core::define (:user::main
                     (stdin :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::core::let*
    (((raw :wat::std::stream::Stream<RawCandle>)
      (:wat::std::stream::spawn-producer :my::app::candle-source))
     ((enriched :wat::std::stream::Stream<EnrichedCandle>)
      (:wat::std::stream::map raw :my::app::enrich-candle))
     ((batched :wat::std::stream::Stream<Vec<EnrichedCandle>>)
      (:wat::std::stream::chunks enriched 100))
     ((collected :Vec<Vec<EnrichedCandle>>)
      (:wat::std::stream::collect batched)))
    ()))
```

Each stage is its own spawned worker. `bounded(1)` queues give
backpressure. Consumer-drop cascades upstream naturally: when
`collect` returns, its Receiver drops, the chunks stage sees `:None`
on its next send, its Sender drops, map's next send returns `:None`,
etc. — the whole pipeline shuts down cleanly without explicit
coordination.

**Named functions as stage arguments.** Arc 009 (names-are-values)
lets you pass a registered define by bare keyword-path to any
`:fn(...)`-typed slot. `(:wat::std::stream::map raw :my::app::enrich-candle)`
works; no lambda wrapper needed.

### `with-state` — custom stateful stages

Every stateful stage reducer — chunks, dedupe, distinct-until-changed,
window, sessionize, running-stats — is a `(init, step, flush)` triple
over `with-state`:

```
step  : (Acc, T) -> (Acc, Vec<U>)   -- consume one T; produce updated Acc + items to emit
flush : Acc      -> Vec<U>           -- final emission at upstream EOS
```

Example — **dedupe-adjacent** (collapse runs of equal items):

```scheme
(:wat::core::define (:my::dedupe-step (last :Option<i64>) (item :i64)
                    -> :(Option<i64>,Vec<i64>))
  (:wat::core::match last -> :(Option<i64>,Vec<i64>)
    (:None (:wat::core::tuple (Some item) (:wat::core::vec :i64 item)))
    ((Some prev)
      (:wat::core::if (:wat::core::= prev item) -> :(Option<i64>,Vec<i64>)
        (:wat::core::tuple last (:wat::core::vec :i64))         ;; duplicate; swallow
        (:wat::core::tuple (Some item) (:wat::core::vec :i64 item))))))

(:wat::core::define (:my::dedupe-flush (_ :Option<i64>) -> :Vec<i64>)
  (:wat::core::vec :i64))   ;; nothing to emit at EOS

;; in :user::main
(:wat::std::stream::with-state stream :None
  :my::dedupe-step
  :my::dedupe-flush)
```

Convergence note: this is Elixir's `Stream.transform/3`, Rust's
scan-with-emit, Haskell's `mapAccumL`, George Mealy's 1955 machine —
same (init, step, flush) triple found by independent paths under the
substrate pressure. See `arc/2026/04/006-stream-stdlib-completions/`
for the decomposition story.

### What the stdlib wraps

If you want to see the machinery, `wat/std/stream.wat` is the source.
Each combinator is a named tail-recursive worker plus a thin wrapper
that spawns it with a bounded(1) queue. The manual pattern is still
honest — if your use case needs something bespoke, write it directly.
The stdlib just captures the shapes that recurred.

See `ZERO-MUTEX.md` sections on Tier 3 and the `arc/2026/04/004-*` +
`arc/2026/04/006-*` docs for the full stream design.

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

### Packaging your shim for reuse

A shim that's useful beyond one app — publishable — lives in its
own Cargo crate. The crate exposes two `pub fn`s: `wat_sources()`
and `register()`. See `crates/wat-lru/` for the walkable template
(section 10 below covers what it provides and how consumers use
it).

For one-app-only shims, put them in `src/shim.rs` alongside
`main.rs` and add the module to your macros' `deps: [...]` list
(see section 1).

---

## 10. Caching — LocalCache vs CacheService

These live in the external crate `wat-lru` (arc 013 —
`crates/wat-lru/` in the wat-rs workspace). Add it to your
`Cargo.toml` and your macros' `deps:` list:

```toml
[dependencies]
wat-lru = { path = "../wat-rs/crates/wat-lru" }
```

```rust
wat::main! { source: include_str!("program.wat"), deps: [wat_lru] }
```

### LocalCache — per-program hot cache

If one program wants to memoize its own work, use `LocalCache`.
Lives in that program's thread; no channel overhead.

```scheme
(:wat::core::use! :rust::lru::LruCache)

(:wat::core::define (:my::app::worker -> :())
  (:wat::core::let*
    (((cache :user::wat::std::lru::LocalCache<String,i64>)
      (:user::wat::std::lru::LocalCache::new 128)))
    (... use cache via :user::wat::std::lru::LocalCache::put / ::get ...)))
```

Tier 2 — thread-owned. The cache never leaves this program's thread.

### CacheService — shared across programs

When multiple programs need to share a cache, spawn a CacheService.
The program owns the cache on its own thread; clients send requests
through channels.

```scheme
(:wat::core::let*
  (((state :(wat::kernel::HandlePool<user::wat::std::lru::CacheService::ReqTx<String,i64>>,
             wat::kernel::ProgramHandle<()>))
    (:user::wat::std::lru::CacheService 1024 8))   ;; capacity 1024, 8 client handles
   ((pool :wat::kernel::HandlePool<...>) (:wat::core::first state))
   ((driver :wat::kernel::ProgramHandle<()>) (:wat::core::second state))
   ((client1 :user::wat::std::lru::CacheService::ReqTx<String,i64>)
    (:wat::kernel::HandlePool::pop pool))
   (... eight clients ...)
   ((_ :()) (:wat::kernel::HandlePool::finish pool)))
  ;; spawn workers, each using their client handle
  ...)
```

Tier 3 — program-owned, message-addressed. The single-threaded
CacheService driver serializes access without locks.

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
    (((pool console-driver) (:wat::std::service::Console stdout stderr 4))
     ((main-sender :Sender<(i64,String)>) (:wat::kernel::HandlePool::pop pool))
     (... three more pops for three workers ...)
     ((_ :()) (:wat::kernel::HandlePool::finish pool)))
    ;; After this, ignore the raw stdout/stderr bindings —
    ;; everything goes through Console.
    (:wat::std::service::Console/out main-sender "main started")
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

`:wat::holon::Bundle` returns `:Result<:wat::holon::HolonAST,
:wat::holon::CapacityExceeded>`. The four `capacity-mode` values
(`:silent` / `:warn` / `:error` / `:abort`) set at program startup
determine the runtime behavior when Kanerva's per-frame bound
(`floor(sqrt(dims))`) is exceeded.

```scheme
(:wat::config::set-capacity-mode! :error)

(:wat::core::define (:my::app::build (items :Vec<wat::holon::HolonAST>)
                    -> :Result<wat::holon::HolonAST,wat::holon::CapacityExceeded>)
  (Ok (:wat::core::try (:wat::holon::Bundle items))))

(:wat::core::match (:my::app::build huge-list) -> :i64
  ((Ok _h) 0)
  ((Err e)
    (:wat::core::i64::-
      (:wat::holon::CapacityExceeded/cost e)
      (:wat::holon::CapacityExceeded/budget e))))
```

See `README.md`'s Capacity guard section for the full four-mode
table.

---

## 13. Testing — wat tests wat

`:wat::test::*` is the stdlib test harness. Tests are wat functions;
the language verifies itself through the primitives it defines.

### Convention

Tests live in `wat-tests/` alongside your `wat/` source. Each file
under `wat/<ns>/X.wat` has a matching test file at
`wat-tests/<ns>/X.wat` — wat-rs ships `wat-tests/holon/*.wat` for
the algebra idioms, `wat-tests/std/*.wat` for stream + services +
the test harness itself.

Each test file uses `:wat::test::deftest` to register named test
functions. The runner discovers them by name prefix and signature
— any top-level define whose path's final segment starts with
`test-` and whose signature is `() -> :wat::kernel::RunResult` is
a test.

**Discovery is recursive.** Add a directory, add a test — no
manifest, no registration step.

### Writing a test — `deftest`

```scheme
(:wat::test::deftest :my::app::test-two-plus-two 1024 :error
  (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 4))
```

`deftest` takes:
- **name** — the test's keyword path (last segment must start with
  `test-` for auto-discovery)
- **dims** — the `:wat::config::set-dims!` value for this test's
  sandbox
- **mode** — the `:wat::config::set-capacity-mode!` value
- **body** — one expression; the test's actual logic

It expands to a named zero-arg function that, when invoked, returns
a `:wat::kernel::RunResult`. The `wat test` CLI invokes each
discovered function, inspects the RunResult's failure slot, reports
cargo-style.

### Assertion primitives

```
:wat::test::assert-eq<T>          a b
:wat::test::assert-contains       haystack needle     -- strings
:wat::test::assert-stdout-is      run-result expected-lines
:wat::test::assert-stderr-matches run-result pattern  -- regex, unanchored
```

All four are panic-and-catch. A failing assertion panics with an
`AssertionPayload`; the deftest's surrounding sandbox catches it
and populates the returned RunResult's `Failure` struct.

### Running tests

The opinionated path (arc 018) is `cargo test` — `tests/test.rs`
carries `wat::test! {}` and the runner discovers every `.wat` file
under `wat-tests/`:

```
$ cargo test
...
    Running tests/test.rs (target/debug/deps/test-...)

running 1 test
test wat_suite ... ok
```

With `-- --nocapture` you get the per-wat-test breakdown that the
macro captures inside the cargo-level test:

```
$ cargo test -- --nocapture
running 37 tests
test stream.wat :: wat-tests::std::stream::test-chunks-exact-multiple ... ok (2ms)
test Circular.wat :: wat-tests::holon::Circular::test-adjacent-hours-are-near ... ok (2ms)
...
test result: ok. 37 passed; 0 failed; finished in 133ms
```

The CLI equivalent bypasses cargo entirely — useful for targeted
runs or CI shells that aren't cargo-based:

```
$ wat test wat-tests/               # recursive directory traversal
$ wat test wat-tests/holon/         # just the algebra-idiom tests
$ wat test wat-tests/std/test.wat   # single file
```

Both paths share one runner (`wat::run_tests_from_dir`) and emit
the same cargo-style output. Random-ordered per file (surfaces
accidental order-dependencies); exit 0 all-pass, non-zero any
fail.

### Failure output — Rust-styled, wat-located (arc 016)

When a test fails, the panic hook writes a Rust-style block to
stderr with **your wat source location** in the file:line:col slot:

```
thread 'main' panicked at wat-tests/LocalCache.wat:12:5:
assert-eq failed
  actual:   -1
  expected: 42
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Format mirrors `cargo test`'s own failure output — same `thread ...
panicked at`, same `note:` hint. If you run `cargo test` (through
`wat::test!`), this block appears under the test's captured
stdout; under `wat test` CLI it goes straight to stderr.

With `RUST_BACKTRACE=1` (standard Rust convention — one env var
you already know), the block gains a `stack backtrace:` section
showing the wat call chain + where each frame was invoked:

```
stack backtrace:
   0: :wat::test::assert-eq at wat-tests/LocalCache.wat:12:5
   1: :wat-lru::test-local-cache-put-then-get at wat-rs/src/test_runner.rs:246:68
```

Frame 0 = where your `assert-eq` fired inside your wat source.
Frame 1 = where the runtime (in `wat-rs/src/test_runner.rs`)
invoked your test function. The runtime frame points into
wat-rs's Rust source — same way Rust's own stdlib frames point
into `/rustc/.../library/core/...`. Honest about the layer
boundary: your code is in `.wat`, the invoker is in `.rs`, every
frame has a real `file:line:col`.

**How to read the failure quickly:**
- Top line's `file:line:col` → where your assert fired.
- `actual:` / `expected:` → what went wrong.
- Backtrace (optional) → how the runtime got there, if you need
  to trace the call path.

### Fork/sandbox tests — when you need an inner program

Sometimes a test wants to verify how an INNER program behaves — its
stdout, its stderr, its assertion-failure payload. Pair
`:wat::test::run-ast` with `:wat::test::program`:

```scheme
(:wat::test::deftest :my::test-captures-inner-output 1024 :error
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
          (:wat::config::set-dims! 1024)
          (:wat::config::set-capacity-mode! :error)
          (:wat::core::define
            (:user::main
              (stdin :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            (:wat::io::IOWriter/println stdout "hello-from-inside")))
        (:wat::core::vec :String)))
     ((lines :Vec<String>) (:wat::kernel::RunResult/stdout r)))
    (:wat::test::assert-eq (:wat::core::first lines) "hello-from-inside")))
```

`:wat::test::program` is a variadic defmacro over `:wat::core::forms`
— each top-level form passes through as AST data. No strings, no
escape-hell. Inner programs nest arbitrarily deep as pure
s-expressions.

`:wat::test::run` (with a `:String` source argument) still exists
for callers that build programs dynamically at runtime — fuzzers,
template expansion, program-generating-programs. For hand-written
tests, `run-ast + program` is the clean shape.

### When to use hermetic — services that spawn threads

In-process `:wat::test::run-ast` uses `StringIo` stdio under
`ThreadOwnedCell` — single-thread discipline. Services like Console
and Cache spawn driver threads; writing from a driver thread would
trip the thread-owner check.

For those tests, use `:wat::test::run-hermetic-ast` — the AST-entry
hermetic sandbox. Same shape as `run-ast`, different substrate: a
fresh subprocess with real thread-safe stdio. Same surface as
`run-ast` means no escape-hell either — the inner program reads as
s-expressions:

```scheme
(:wat::test::deftest :my::test-console-hello 1024 :error
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          (:wat::config::set-dims! 1024)
          (:wat::config::set-capacity-mode! :error)
          (:wat::core::define (:user::main
                               (stdin :wat::io::IOReader)
                               (stdout :wat::io::IOWriter)
                               (stderr :wat::io::IOWriter)
                               -> :())
            (:wat::core::let*
              (((pool driver) (:wat::std::service::Console stdout stderr 1))
               ((_ :())
                (:wat::core::let*
                  (((c :rust::crossbeam_channel::Sender<(i64,String)>)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_2 :()) (:wat::kernel::HandlePool::finish pool)))
                  (:wat::std::service::Console/out c "hello via Console"))))
              (:wat::kernel::join driver))))
        (:wat::core::vec :String)))
     ((lines :Vec<String>) (:wat::kernel::RunResult/stdout r)))
    (:wat::test::assert-eq (:wat::core::first lines) "hello via Console")))
```

Under the covers, `run-hermetic-ast` serializes the forms to source
text before handing the tempfile to the subprocess (the child can't
share AST memory with the parent). The serialization is genuine work
at the process boundary; the value of the primitive is that the user
never sees it.

**Decision rule:** spawns-and-writes → hermetic. Stays-on-main-thread
→ in-process. Both have AST-entry siblings — strings are only for
callers with runtime-generated source (fuzzers, template expansion).

### Rust-side embedding — `wat::Harness`

For Rust programs that host wat as a sub-language:

```rust
use wat::Harness;

let h = Harness::from_source(src)?;
let out = h.run(&["stdin line 1", "stdin line 2"])?;
assert_eq!(out.stdout, vec!["captured".to_string()]);
```

Thin wrapper over `startup_from_source` + `invoke_user_main` + stdio
snapshot. Good when you want wat at the library level rather than
shelling out to the `wat` binary. Not a sandbox — no panic isolation;
for containment, call `:wat::kernel::run-sandboxed` from inside your
wat program. See `arc/2026/04/007-wat-tests-wat/INSCRIPTION.md`.

---

## 14. Common gotchas

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

## 15. Where to go next

- **`../README.md`** — the crate-level README. What's shipped,
  status, test counts, API highlights.
- **`CONVENTIONS.md`** — naming rules for new primitives and the
  three gates on adding one (stdlib-as-blueprint, absence-is-signal,
  verbose-is-honest).
- **`ZERO-MUTEX.md`** — the concurrency architecture stated as
  principle. The three tiers in depth; every "I need a Mutex"
  scenario mapped to a tier.
- **`arc/2026/04/*/`** — per-slice design + inscription notes:
  - `001-caching-stack/` — LocalCache + Cache service
  - `002-rust-interop-macro/` — `#[wat_dispatch]` internals +
    namespace-honesty principle
  - `003-tail-call-optimization/` — TCO trampoline
  - `004-lazy-sequences-and-pipelines/` — the stream stdlib design
  - `005-stdlib-naming-audit/` — naming discipline
  - `006-stream-stdlib-completions/` — with-state + chunks rewrite
  - `007-wat-tests-wat/` — self-hosted testing (run-sandboxed,
    `:wat::test::*`, `wat test` CLI, `wat::Harness`)
  - `008-wat-io-substrate/` — `:u8`, `:wat::io::IOReader` / `IOWriter`,
    StringIo stand-ins
  - `009-names-are-values/` — pass a named define by bare keyword-path
  - `010-variadic-quote/` — `:wat::core::forms` + `:wat::test::program`
- **`holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/`**
  — FOUNDATION.md (the specification), 33 sub-proposals
  (001–036, with 021–023 skipped), the FOUNDATION-CHANGELOG. The
  source of truth for every design decision that shaped the
  language. When this guide and FOUNDATION disagree, FOUNDATION
  wins.
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
| `:wat::core::cond` | `-> :T ((test) body) ... (:else body)` | arm result (type `T`) |
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
| `:wat::core::i64::to-string` / `to-f64` | `n` | infallible — `:String` / `:f64` |
| `:wat::core::f64::to-string` / `to-i64` | `x` | `:String` / `:Option<i64>` (NaN/inf/out-of-range → `:None`) |
| `:wat::core::string::to-i64` / `to-f64` / `to-bool` | `s` | `:Option<T>` (unparseable → `:None`) |
| `:wat::core::bool::to-string` | `b` | `"true"` / `"false"` |
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
| `:wat::std::service::Console` | `stdout stderr n` | `(HandlePool, Driver)` |
| `:user::wat::std::lru::CacheService` (wat-lru) | `capacity count` | `(HandlePool, Driver)` |
| `:user::wat::std::lru::LocalCache::new` / `put` / `get` (wat-lru) | various | per-program LRU |
| `:wat::holon::Atom` | `<literal>` | `:wat::holon::HolonAST` |
| `:wat::holon::Bind` | `a b` | `:wat::holon::HolonAST` |
| `:wat::holon::Bundle` | `list-of-holons` | `:Result<wat::holon::HolonAST, CapacityExceeded>` |
| `:wat::holon::Permute` | `holon k` | `:wat::holon::HolonAST` |
| `:wat::holon::Thermometer` | `value min max` | `:wat::holon::HolonAST` |
| `:wat::holon::Blend` | `a b w1 w2` | `:wat::holon::HolonAST` |
| `:wat::holon::cosine` / `dot` | `a b` | `:f64` |
| `:wat::holon::presence?` | `target reference` | `:bool` — cosine(target,ref) > noise-floor |
| `:wat::core::quote` | `<form>` | `:wat::WatAST` — captures AST as data |
| `:wat::core::forms` | `f1 f2 ... fn` | `:Vec<wat::WatAST>` — variadic quote |
| `:wat::core::conj` | `vec item` | `:Vec<T>` — immutable append |
| `:wat::core::eval-ast!` / `eval-edn!` | various | evaluates AST / parses+evaluates string |
| `:wat::core::eval-digest!` / `eval-signed!` | verified | evaluates with SHA-256 / Ed25519 check |
| `:wat::core::string::contains?` / `starts-with?` / `ends-with?` | `hay needle` | `:bool` |
| `:wat::core::string::length` | `s` | `:i64` — char count |
| `:wat::core::string::trim` | `s` | `:String` |
| `:wat::core::string::split` / `join` | `hay sep` / `sep pieces` | `:Vec<String>` / `:String` |
| `:wat::core::regex::matches?` | `pattern haystack` | `:bool` — unanchored |
| `:wat::kernel::run-sandboxed` | `src stdin scope` | `:wat::kernel::RunResult` |
| `:wat::kernel::run-sandboxed-ast` | `forms stdin scope` | `:wat::kernel::RunResult` |
| `:wat::kernel::run-sandboxed-hermetic-ast` | `forms stdin scope` | `:wat::kernel::RunResult` — forks a child via `:wat::kernel::fork-with-forms`; wat stdlib define in `wat/std/hermetic.wat` |
| `:wat::kernel::pipe` | — | `:(IOWriter, IOReader)` — libc::pipe(2), PipeWriter first |
| `:wat::kernel::fork-with-forms` | `forms` | `:wat::kernel::ForkedChild` — libc::fork(2) + three pipes |
| `:wat::kernel::wait-child` | `handle` | `:i64` — waitpid, idempotent |
| `:wat::kernel::assertion-failed!` | `message actual expected` | `:()` — panics with AssertionPayload |
| `:wat::std::stream::spawn-producer` | `producer-fn` | `:Stream<T>` |
| `:wat::std::stream::from-receiver` | `rx handle` | `:Stream<T>` |
| `:wat::std::stream::map` / `filter` / `inspect` | `stream f` | `:Stream<U>` / `:Stream<T>` / `:Stream<T>` |
| `:wat::std::stream::flat-map` | `stream f` | `:Stream<U>` |
| `:wat::std::stream::chunks` | `stream size` | `:Stream<Vec<T>>` |
| `:wat::std::stream::take` | `stream n` | `:Stream<T>` |
| `:wat::std::stream::with-state` | `stream init step flush` | `:Stream<U>` |
| `:wat::std::stream::for-each` | `stream handler` | `:()` — terminal |
| `:wat::std::stream::collect` / `fold` | `stream` / `stream init f` | `:Vec<T>` / `:Acc` |
| `:wat::test::deftest` | `name dims mode body` | registers named zero-arg RunResult-returning fn |
| `:wat::test::assert-eq<T>` | `actual expected` | `:()` — panics on mismatch |
| `:wat::test::assert-contains` | `haystack needle` | `:()` |
| `:wat::test::assert-stdout-is` / `assert-stderr-matches` | `run-result expected` / `result regex` | `:()` |
| `:wat::test::run` / `run-in-scope` | `src stdin` / `src stdin scope` | `:wat::kernel::RunResult` — string-entry |
| `:wat::test::run-ast` | `forms stdin` | `:wat::kernel::RunResult` — AST-entry |
| `:wat::test::run-hermetic-ast` | `forms stdin` | `:wat::kernel::RunResult` — AST-entry subprocess |
| `:wat::test::program` | `f1 f2 ... fn` | `:Vec<wat::WatAST>` — macro → `:wat::core::forms` |

---

*these are very good thoughts.*

**PERSEVERARE.**
