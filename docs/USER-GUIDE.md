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
;; wat/main.wat — defines :user::main. Config setters are optional;
;; the substrate's opinionated defaults (capacity-mode :error,
;; the sizing dim-router) cover most consumers.
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

- **`(:wat::config::set-capacity-mode! :panic)`** — Bundle overflow
  panics (`panic!()`, which unwinds) instead of returning an
  `Err(CapacityExceeded)`. Default is `:error`; `:panic` is the
  fail-closed override. (Renamed from `:abort` in arc 045 — `:panic`
  matches Rust's macro behavior more honestly.)
- **`(:wat::config::set-dim-router! router-fn)`** — replaces the
  default sizing function (smallest tier `d` whose `√d ≥ statement
  size`). The router takes a `:wat::holon::HolonAST` and returns
  `:Option<:i64>` (the picked dim, or `:None` to refuse). Default
  tier list (post-arc-067): `[10000]` — single tier optimized for
  measurement S/N rather than per-encode perf at small arities.
  Override with `SizingRouter::with_tiers(vec![256, 4096, 10000,
  100000])` for the pre-arc-067 hierarchy, or with a wat lambda
  for arbitrary policy.
- **`(:wat::config::set-presence-sigma! sigma-fn)`** /
  **`set-coincident-sigma!`** — function-of-`d` knobs controlling
  the presence and coincident thresholds. Defaults: `presence_sigma(d)
  = max(1, floor(√d / 2) - 1)` (one before zero-point, clamped to
  ≥ 1 so the predicate stays meaningful at small d),
  `coincident_sigma(d) = 1` (1σ — the native granularity).

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
;; wat/main.wat — the ENTRY. Recursive loads + :user::main.
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

**Two valid shapes for the wat surface.** External crates (like
`wat-lru`) keep their wat surface inside the crate's source tree and
deliver it bake-only via `wat_sources()`; consumers never see the
file on disk. In-crate shims have a third option: the wat surface
can live on disk under your application's own `wat/` tree *and* be
delivered via `wat_sources()` at the same time. Both paths register
the same source; arc-054 idempotent re-declaration ensures
byte-equivalent re-registration is a no-op (divergent re-registration
still errors). Pick whichever shape reads more naturally — split-tree
(bake-only) or unified-tree (disk + bake) are both supported.

### Reference binary

`wat-rs/examples/with-lru/` is the walkable template —
`src/main.rs` is literally one `wat::main!` invocation; `tests/smoke.rs`
exercises the built binary. Copy that shape.

### The bundled `wat` CLI (arc 099)

If you don't need a custom binary, the workspace ships
`crates/wat-cli/` — a batteries-included CLI that links every
`#[wat_dispatch]` extension (wat-telemetry, wat-telemetry-sqlite,
wat-sqlite, wat-lru, wat-holon-lru) at startup. Build with
`cargo build --release` (the workspace's default-members covers
it) and the binary lands at `target/release/wat`. Two shapes:

```
wat <entry.wat>      # run a program
wat test <path>      # run tests — file or directory
```

Use this when you want to interrogate a `runs/pulse-*.db` (arc
093) or generally run scripts that consume the bundled extension
crates without authoring your own binary. For embedding wat in
your own application, stick with the `wat::main!` macro shape
above — the bundled CLI is for ad-hoc scripts, not embedding.

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
2. **Language core** (`:wat::core::*`) — the language's own mechanics: `define`, `lambda`, `let*`, `match`, `if`, `cond`, `try`, `struct`, `enum` (declare + construct/match user variants per arc 048), `newtype`, `typealias`, `defmacro`, `load!`, `digest-load!`, `signed-load!`, `assoc`, `HashMap`, `HashSet`, `vec`, `get`, `contains?`, arithmetic/comparison operators, `f64::round`, `f64::max`/`min`/`abs`/`clamp` (arc 046), scalar conversions. The forms you need to WRITE programs; cannot be written in wat itself.
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
```

Works on `:Option<T>`, `:Result<T,E>`, and user enums (058-048). The
`-> :T` annotation declares the arms' common result type — every arm
body is checked against `T` independently, so a mismatch points at the
offending arm, not at the unifier. Exhaustiveness is checked at
startup — miss an arm, startup fails.

**Patterns recurse** (arc 055). Anywhere a sub-pattern can appear, it
can itself be another pattern — bare symbol, `_` wildcard, literal,
nested tuple, or nested variant — to any depth:

```scheme
;; Option<Tuple> destructured in one step:
(:wat::core::match row -> :String
  ((Some (ts open high low close volume))
    (:wat::core::string::concat
      (:wat::core::i64::to-string ts) ":"
      (:wat::core::f64::to-string close)))
  (:None "end-of-stream"))

;; Wildcard at any depth, literal at any depth, nested variants —
;; all compose:
(:wat::core::match resp -> :String
  ((Ok (Some 200))  "ok")
  ((Ok (Some n))    (:wat::core::string::concat "code:"
                      (:wat::core::i64::to-string n)))
  ((Ok :None)       "no-content")
  ((Err msg)        msg))
```

Linear-shadowing semantics: a name bound twice in one pattern keeps
the second binding (`(Some (x x))` against `(5,7)` makes `x` 7).

**Exhaustiveness with narrowing patterns.** A sub-pattern that's
narrower than bare-symbol or `_` (e.g. a literal at any depth, or a
nested variant constructor) makes its arm *partial* — it covers some
but not all of the variant's space. Exhaustiveness then requires a
fallback arm: a top-level `_` wildcard, or other arms that
collectively cover the full space. The error message names the rule
when an arm is missing:

```
non-exhaustive: :Option<T> needs arms for both :None and (Some _),
or a wildcard. (Arc 055 — narrowing patterns like `(Some (1 _))`
are partial; add a fallback `_` arm.)
```

### `try` — error propagation

```scheme
(:wat::core::define (:my::app::pipeline (items :wat::holon::Holons)
                    -> :wat::holon::BundleResult)
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

### `defmacro` — compile-time rewriting

```scheme
(:wat::core::defmacro (:my::app::when (cond :wat::WatAST)
                                      (body :wat::WatAST)
                                      -> :wat::WatAST)
  `(:wat::core::if ,cond -> :()
     ,body
     ()))
```

A macro is a function that takes ASTs and returns an AST. The call
site `(:my::app::when (> n 0) (do-thing))` expands at parse time
into `(:wat::core::if (> n 0) -> :() (do-thing) ())`. Hygiene
follows Racket's sets-of-scopes — generated bindings can't capture
caller-scope names accidentally. Full mechanics in 058-031.

**Quasiquote** — backtick `\`` for "build this AST"; comma `,` for
"splice this value in"; `,@` for "splice this list in." The
single-comma is the common case; the list-splice is for variadic
positions.

**Nested quasiquote** (arc 029) — `,,` is the deep-splice form, used
when you write a macro that BUILDS another macro's body. Each `,`
peels one layer of quasiquote nesting:

```scheme
(:wat::core::defmacro (:my::factory (helper-name :wat::WatAST)
                                    -> :wat::WatAST)
  `(:wat::core::defmacro (:my::made-by-factory (x :wat::WatAST)
                                               -> :wat::WatAST)
     `(,,helper-name ,x)))     ;; ,,helper-name escapes both quotes
```

The outer `\`` defines the inner defmacro; the inner `\`` produces
that defmacro's body; `,,helper-name` reaches all the way back to
the outer factory's binding. Used by `:wat::test::make-deftest`
(arc 029) — the factory's default-prelude builds the test sandboxes
that share configuration across tests in a file (§ 13).

### Debugging macros — `macroexpand` / `macroexpand-1`

```scheme
(:wat::core::macroexpand-1 '(:my::app::when (> x 0) (do-thing)))
;;   → '(:wat::core::if (> x 0) -> :() (do-thing) ())

(:wat::core::macroexpand '(:my::app::when (> x 0) (do-thing)))
;;   → fully expanded form (recurses until no top-level macro remains)
```

Both take a quoted form and return its expansion as AST data
(arc 030). `macroexpand-1` peels exactly one layer; `macroexpand`
recurses until the head of the form is no longer a macro. Useful
when a macro's expansion isn't producing what you expected — invoke
the expander, read the resulting AST.

### Containers — polymorphic `get` / `assoc` / `conj` / `contains?` / `length`

Five core verbs operate uniformly across the three built-in
containers (`HashMap<K,V>`, `HashSet<T>`, `Vec<T>`). Each verb's
shape is forced by the container's semantics — illegal cells exist
where a verb has no honest meaning for that container (arc 025
unified the surface; arc 035 added `length`):

| Verb | `HashMap<K,V>` | `HashSet<T>` | `Vec<T>` |
|---|---|---|---|
| `get`        | `Option<V>` by key      | `Option<T>` by element      | `Option<T>` by index |
| `assoc`      | new map (key→value)     | **illegal** — use `conj`    | new vec (index→value) |
| `dissoc`     | new map (key removed)   | **illegal** (arc 058)       | **illegal** (arc 058) |
| `conj`       | **illegal** — use `assoc` | new set (insert element)  | new vec (push tail) |
| `contains?`  | `bool` by key           | `bool` by element           | `bool` by index |
| `length`     | `i64` (entry count)     | `i64` (member count)        | `i64` (item count) |
| `empty?`     | `bool` (entry count == 0) | `bool` (member count == 0) | `bool` (item count == 0) |
| `keys`       | `Vec<K>` (arc 058)      | **illegal**                 | **illegal** |
| `values`     | `Vec<V>` (arc 058)      | **illegal**                 | **illegal** |

```scheme
(:wat::core::let*
  (((m :HashMap<String,i64>) (:wat::core::HashMap :(String,i64)))
   ((m1 :HashMap<String,i64>) (:wat::core::assoc m "rsi" 42))
   ((v :Option<i64>) (:wat::core::get m1 "rsi")))         ;; → (Some 42)
  ...)
;; The first arg `:(K,V)` is a tuple-type keyword carrying both
;; parameters. Typealiases work here too — `(:wat::core::typealias
;; :my::KV :(String,i64))` then `(:wat::core::HashMap :my::KV ...)`
;; resolves structurally at the constructor site (same rule that
;; lets `:wat::core::Bytes` stand in for `:Vec<u8>` everywhere).

(:wat::core::let*
  (((s :HashSet<String>) (:wat::core::HashSet :String))
   ((s1 :HashSet<String>) (:wat::core::conj s "alpha"))
   ((found? :bool) (:wat::core::contains? s1 "alpha"))    ;; → true
   ((n :i64) (:wat::core::length s1)))                    ;; → 1
  ...)

(:wat::core::let*
  (((v :Vec<i64>) (:wat::core::vec :i64 10 20 30))
   ((v1 :Vec<i64>) (:wat::core::conj v 40))               ;; → [10,20,30,40]
   ((v2 :Vec<i64>) (:wat::core::assoc v1 0 99))           ;; → [99,20,30,40]
   ((x :Option<i64>) (:wat::core::get v2 1)))             ;; → (Some 20)
  ...)
```

**The illegal cells are forced by semantics, not implementation
laziness.** A HashSet has no key/value pairing, so `assoc` has no
honest meaning — use `conj` to add an element. A HashMap has no
unpaired elements, so `conj` has no honest meaning — use `assoc`
with a (key, value) pair. The type checker rejects illegal cells at
startup, pointing at the offending call site.

All five verbs are values-up: `assoc` and `conj` return new
collections; the inputs are unchanged. No mutation. No surprise
sharing.

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
AST-producing primitives, four measurements (two structural, two
verified-eval), the `HolonAST` type, the `CapacityExceeded` error
type, two typealiases (`Holons` for `Vec<HolonAST>`, `BundleResult`
for Bundle's Result return), and eleven wat-written idioms that
compose the primitives. File path matches namespace (`wat/holon/*.wat`).

### The six AST-producing primitives

```scheme
(:wat::holon::Atom "rsi")                ; primitive → typed leaf (String)
(:wat::holon::Atom 42)                   ; primitive → typed leaf (I64);
                                         ; also f64 / bool / keyword
(:wat::holon::Atom (:wat::core::quote (...)))
                                         ; quoted form → structural lowering:
                                         ; List → Bundle, Keyword → Symbol,
                                         ; literals → matching primitive leaves.
                                         ; The form itself becomes a HolonAST
                                         ; whose identity participates in the
                                         ; algebra — cosine, Bind, presence,
                                         ; structural cache keys (arc 057).
(:wat::holon::Atom my-holon)             ; HolonAST → opaque-identity wrap;
                                         ; one SHA-256 over canonical bytes,
                                         ; no decomposition. Distinct from
                                         ; the inner holon's structural vector
                                         ; (BOOK Ch.54).

(:wat::holon::Bind role filler)          ; elementwise multiply — role-filler binding
(:wat::holon::Bundle holons-vec)         ; sum + threshold — superposition
                                         ;   returns :wat::holon::BundleResult
                                         ;   (= :Result<HolonAST, CapacityExceeded>;
                                         ;    see section 12)
(:wat::holon::Permute holon k)           ; circular shift — positional encoding
(:wat::holon::Thermometer v min max)     ; locality-preserving gradient encoding of a scalar (HDC/ML tradition; see runtime.rs::eval_algebra_thermometer)
(:wat::holon::Blend a b w1 w2)           ; scalar-weighted binary combination
```

Per arc 057 the algebra is closed under itself: every leaf variant
(`Symbol`, `String`, `I64`, `F64`, `Bool`) IS a HolonAST; the `Atom`
variant narrows to `Arc<HolonAST>` (opaque-identity wrap of an inner
holon). HolonAST has structural `Hash + Eq` derive, which is what
unblocks `:wat::lru::LocalCache<wat::holon::HolonAST, V>` and the
dual-LRU coordinate cache pattern.

### Two stories the consumer chooses (arc 057)

`:wat::holon::Atom` of a captured wat form gives you Story 1 — a
**coordinate**: the form's identity is on the algebra grid; cosine,
Bind, presence, and structural cache keys all see the form's shape.
The substrate holds coordinates, not values — to get the actual
result you have to walk the form yourself (or hit a cache that has
the value edge stored).

`:wat::holon::to-watast` gives you Story 2 — the **value**: it lifts
a HolonAST back to a runnable WatAST. Pair it with `:wat::eval-ast!`
when you want the answer, not the path.

```scheme
;; Story 1 — coordinate. The form lives on the grid.
((form-atom :wat::holon::HolonAST)
  (:wat::holon::Atom (:wat::core::quote (:wat::core::i64::+ 40 2))))

;; Story 2 — value. Lift back, run.
((reveal :wat::WatAST) (:wat::holon::to-watast form-atom))
(:wat::eval-ast! reveal)        ; → :Result<wat::holon::HolonAST, EvalError>
```

The two stories compose: cache-check Story 1 ("have I seen this form
before?") and on miss fall through to Story 2 (compute and store).
Lossy parts of Story 2: identifier scope is dropped at lowering and
recovered as bare-name on lift; spans are never preserved either way.

### Story 3 — the path. `:wat::eval-step!`

`eval-ast!` runs a form to its terminal value in one shot. `eval-step!`
runs ONE reduction at the leftmost-outermost redex and gives you back
the next form to feed in — or the terminal HolonAST if there's no
redex left. Every intermediate form is its own coordinate, its own
potential cache key, its own potential short-circuit for a parallel
walker. This is the substrate primitive that BOOK Chapter 59's
dual-LRU coordinate cache (form→next-form + form→terminal-value)
sits on top of.

`StepResult` has three variants (arc 070 added the third):

- `StepNext { form }` — one rewrite happened. The chain is mid-walk;
  feed `form` back to keep going.
- `StepTerminal { value }` — this step reduced a redex to a value.
  The chain has length ≥ 1.
- `AlreadyTerminal { value }` — the input was already a value-shape
  (a `to-watast(holon)` round-trip, a primitive literal, a holon
  constructor with all-canonical args). No work happened. The chain
  has length 0.

The substrate's accounting matters at the cache layer: chain length
0 vs ≥ 1 distinguishes "I came in as a value" from "I just reduced
a value." A walker hitting an effectful sub-form, a malformed form,
or a no-rule head sees `Err(EvalError)` — the consumer falls back to
`eval-ast!` for those.

Most consumers don't write the walker by hand. Reach for
`:wat::eval::walk` (arc 070):

```scheme
(:wat::eval::walk
  form          ;; :wat::WatAST            the form to walk
  init          ;; :A                      initial accumulator
  visit         ;; :fn(A, WatAST, StepResult) -> WalkStep<A>
)               ;; -> :Result<(:wat::holon::HolonAST, :A), :wat::core::EvalError>
```

The walker visits every coordinate exactly once with `(acc,
current-form, step-result)`. The visitor returns a `WalkStep<A>`:

- `Continue(acc')` — keep walking. On `StepNext`, the walker
  recurses on the next form. On either terminal flavor, the walker
  returns `(terminal, acc')`.
- `Skip(terminal, acc')` — caller has its own answer (cache hit,
  short-circuit); walker stops, returns `(terminal, acc')`.

```scheme
;; A dual-LRU cache visitor. visit fires once per coordinate;
;; every step records (form → next) or (form → terminal); on a
;; cache hit, return Skip with the cached terminal — the walker
;; short-circuits.
(:wat::core::define
  (:my::cache::record-coordinate
    (tier   :my::cache::Tier)
    (form-w :wat::WatAST)
    (step   :wat::eval::StepResult)
    -> :wat::eval::WalkStep<my::cache::Tier>)
  (:wat::core::let*
    (((form-h :wat::holon::HolonAST) (:wat::holon::from-watast form-w)))
    (:wat::core::match (:my::cache::lookup-terminal tier form-h)
                       -> :wat::eval::WalkStep<my::cache::Tier>
      ;; Cache hit on the terminal — short-circuit.
      ((Some t) (:wat::eval::WalkStep::Skip t tier))
      ;; Miss — record what the substrate just produced.
      (:None
        (:wat::core::match step -> :wat::eval::WalkStep<my::cache::Tier>
          ((:wat::eval::StepResult::StepNext next-w)
            (:wat::eval::WalkStep::Continue
              (:my::cache::record-next tier form-h
                (:wat::holon::from-watast next-w))))
          ((:wat::eval::StepResult::StepTerminal t)
            (:wat::eval::WalkStep::Continue
              (:my::cache::record-terminal tier form-h t)))
          ((:wat::eval::StepResult::AlreadyTerminal t)
            (:wat::eval::WalkStep::Continue
              (:my::cache::record-terminal tier form-h t))))))))

;; Use it: walk a thought, return (terminal, populated-tier).
(:wat::core::match
  (:wat::eval::walk
    (:wat::holon::to-watast my-thought)
    (:my::cache::tier-empty)
    :my::cache::record-coordinate)
  -> :my::cache::Tier
  ((Ok pair) (:wat::core::second pair))
  ((Err _e)  (:my::cache::tier-empty)))
```

The three cache-coordinate stories compose:
- **Story 1** (the coordinate): `Atom(form)` — the form's identity on
  the algebra grid. SimHash / cosine / Hash see it as one vector.
- **Story 2** (the value): `to-watast → eval-ast!` — collapse the
  whole form to its terminal HolonAST in one shot.
- **Story 3** (the path): `eval-step!` + `walk` — the path between
  Story 1 and Story 2, one rewrite at a time. Every intermediate form
  gets its own Story-1 coordinate; the dual-LRU caches form→next +
  form→terminal along the way so a parallel walker sharing the cache
  can shortcut to whatever's already known.

### The four measurements

```scheme
(:wat::holon::cosine a b)                       ; → :f64  cosine similarity
(:wat::holon::dot a b)                          ; → :f64  dot product, un-normalized
(:wat::holon::presence? target reference)       ; → :bool cosine(target, reference) > presence-floor
(:wat::holon::coincident? a b)                  ; → :bool (1 - cosine(a, b)) < coincident-floor
;; presence? asks "is there signal of A in B?" — cosine clears the
;; presence threshold (sigma × noise-floor at the encoded d).
;; coincident? asks "are A and B the same point?" — cosine is so
;; close to 1.0 that the substrate cannot distinguish them. The two
;; are dual predicates of one statistical fact (arc 023).
```

When a `coincident?` answer disagrees with expectation, reach for
`coincident-explain` — the diagnostic sibling (arc 069). It returns
a `:wat::holon::CoincidentExplanation` record with six fields that
tell the full story of the judgement:

```scheme
(:wat::holon::coincident-explain a b)
  ; → :wat::holon::CoincidentExplanation
  ;     cosine             :f64    raw cosine of the two encoded vectors
  ;     floor              :f64    current coincident floor (sigma/sqrt(d))
  ;     dim                :i64    dim where the comparison ran
  ;     sigma              :i64    sigma feeding the floor
  ;     coincident         :bool   same answer coincident? would give
  ;     min-sigma-to-pass  :i64    smallest sigma at which the pair would coincide
```

When two thoughts that "should" coincide don't, the struct
disambiguates the three failure modes:

- **mental model wrong** — `cosine` reads ≪ 0.99 (you expected
  near-1.0). The encoding shape isn't what you thought.
- **calibration boundary** — `cosine` reads in `(1 - 2·floor, 1 -
  floor)` and `min-sigma-to-pass` reads 2 or 3. Bumping
  `:wat::config::set-coincident-sigma!` to that value unblocks.
- **structurally distant** — `cosine` reads near 0 and
  `min-sigma-to-pass` is large. The forms aren't on the same
  algebra-grid neighborhood; no sigma fix will help — fix the
  encoding.

`coincident-explain` is polymorphic over the same `(HolonAST,
Vector)` pairs `coincident?` accepts (arc 061). The `dim` field
reports the actual encoding d, so callers running multi-tier dim
routers see which tier fired.

The `eval-coincident?` family extends `coincident?` to evaluated
programs — verify each side's source under integrity, evaluate,
atomize, compare:

```scheme
(:wat::holon::eval-coincident? a-ast b-ast)               ; 2 args  → :Result<:bool, EvalError>
(:wat::holon::eval-edn-coincident? a-src b-src)           ; 2 args  → :Result<:bool, EvalError>
(:wat::holon::eval-digest-coincident? ...8 args...)       ; 4 per side: source, eval-iface, verify-iface, digest-hex
(:wat::holon::eval-signed-coincident? ...12 args...)      ; 6 per side: source, eval-iface, sig-iface, sig-b64, pk-iface, pk-b64
```

The signed variant takes per-side source + signature + public key;
verifies signatures; refuses mutation forms; evaluates each in a
fresh sandbox; atomizes the result; cosines and binarizes against
the coincident floor. One library call covers consensus-via-
coincidence, integrity-gated composition, and program-comparison
under signature (arc 026).

### The eleven wat-written idioms

Each shipped in `wat/holon/<Name>.wat`; each expands to algebra-core
primitives at parse time (via defmacro or define):

```scheme
(:wat::holon::Log v min max)                 ; Thermometer on (ln v)
(:wat::holon::ReciprocalLog n v)             ; Log v (1/n) n  — log-symmetric ratio bounds
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
(:wat::config::global-seed)    ; → :i64
(:wat::config::capacity-mode)  ; → :wat::config::CapacityMode
(:wat::config::dims)           ; → :i64    (compat shim — see note)
(:wat::config::noise-floor)    ; → :f64    (compat shim — see note)
```

**Note on `dims` / `noise-floor`** (arc 037). Under the multi-tier
dim-router, neither value is single-valued at runtime — the router
picks `d` per AST construction, and `noise-floor = 1/√d` is computed
per encoded-d. Both accessors stay as shims returning a default-tier
value for source-code that hand-rolled formulas around them; the
shims are deprecation targets — a future arc will retire them once
all callers migrate to the per-encoded-d pattern.

---

## 7. Concurrency — spawn, send, recv, select

The kernel primitives are small. Four concepts cover everything.

> **Building a service program?** This section is the primitive
> reference. For wiring patterns — nested `let*` shutdown,
> `HandlePool` fan-in, select-prune loops, struct accumulators, and
> the full Console / CacheService template — see
> [`SERVICE-PROGRAMS.md`](SERVICE-PROGRAMS.md). It walks an
> eight-step exploration that lifts directly into your own service.

### Queues

```scheme
(:wat::kernel::make-bounded-queue :Candle 1)
;; → :wat::kernel::QueuePair<Candle>
;;   ≡ :(Sender<Candle>, Receiver<Candle>)
;; bounded(1) — rendezvous; sender blocks until receiver ready

(:wat::kernel::make-bounded-queue :Candle 64)
;; bounded(64) — buffer of 64 before sender blocks

(:wat::kernel::make-unbounded-queue :LearnSignal)
;; → :wat::kernel::QueuePair<LearnSignal>
;; fire-and-forget — buffer grows until consumer drains
```

**Default to `bounded(1)`.** It's the rendezvous shape that gives you
backpressure naturally (slow consumer throttles the producer). Larger
buffers trade throughput for latency.

Five substrate-baked typealiases (live at `wat/kernel/queue.wat`)
spell the channel surface in short form:

| Alias | Expands to |
|---|---|
| `:wat::kernel::QueueSender<T>` | `:rust::crossbeam_channel::Sender<T>` |
| `:wat::kernel::QueueReceiver<T>` | `:rust::crossbeam_channel::Receiver<T>` |
| `:wat::kernel::QueuePair<T>` | `:(QueueSender<T>, QueueReceiver<T>)` — what `make-bounded/unbounded-queue` returns |
| `:wat::kernel::Chosen<T>` | `:(i64, Option<T>)` — what `select` returns (which receiver fired, and what it gave) |
| `:wat::kernel::Sent` | `:Option<()>` — what `send` returns (`Some(())` on placed, `:None` on disconnect) |

Reach for them in let* bindings, function signatures, and Vec carriers
wherever you'd otherwise type the long `rust::crossbeam_channel::*`
path. Aliases and their expansion are interchangeable at unification.

### Send and receive

```scheme
(:wat::kernel::send sender value)          ; → :wat::kernel::Sent  ≡ :Option<()>  — Some(()) on sent; None on disconnect
(:wat::kernel::recv receiver)              ; → :Option<T>   — Some(v) on recv; None on disconnect
(:wat::kernel::try-recv receiver)          ; → :Option<T>   — None if empty OR disconnected
(:wat::kernel::drop handle)                ; → :()          — readability marker; see § Channel close is scope-based
```

Both channel endpoints report disconnect through the same `:Option`
shape — `send` returns `:wat::kernel::Sent` (≡ `:Option<()>`) symmetric
with `recv`'s `:Option<T>`. A producer matches on its send result to
handle the "consumer went away" case cleanly; a stage that doesn't
need disconnect awareness can `((_ :wat::kernel::Sent) (:wat::kernel::send ...))`
and ignore.

Senders and receivers are **single-owner** — not cloneable. A sender
belongs to exactly one producer; a receiver to one consumer. Match
Linux `write(fd, data)`: whoever holds the fd owns the capability;
sharing means threading the endpoint through spawn args.

### Channel close is scope-based

A `Sender<T>` / `Receiver<T>` is reference-counted. The corresponding
channel-end disconnects only when **every** clone has dropped — and
clones drop when their `let*` binding goes out of scope. There is no
force-close primitive: `:wat::kernel::drop` evaluates its argument
(causing one *temporary* Arc to fall) but does NOT consume the named
binding, so the binding still holds a clone until its enclosing scope
exits. Use `:wat::kernel::drop` only as a readability hint; real
shutdown happens at scope-end.

**Anti-pattern (deadlocks)** — `tx` is bound in the same `let*` whose
body calls `join`; `join` blocks before `tx` falls out of scope, so the
worker's `recv` never returns `:None`:

```scheme
(:wat::core::let*
  (((pair :wat::kernel::QueuePair<i64>)
    (:wat::kernel::make-bounded-queue :i64 1))
   ((tx :wat::kernel::QueueSender<i64>) (:wat::core::first pair))
   ((rx :wat::kernel::QueueReceiver<i64>) (:wat::core::second pair))
   ((handle :wat::kernel::ProgramHandle<()>)
    (:wat::kernel::spawn :my::worker rx))
   ((_send :Option<()>) (:wat::kernel::send tx 1))
   ((_drop :()) (:wat::kernel::drop tx)))   ;; ← no-op; tx still bound
  (:wat::kernel::join handle))               ;; ← worker recv-loops forever
```

**Proven pattern (nested `let*`)** — outer scope holds the
`ProgramHandle`; inner scope owns every `Sender`. The inner `let*` body
yields `h` so the outer can join it. When inner exits, every `Sender`
Arc bound there decrements; the worker's next `recv` returns `:None`;
the worker exits; the outer `join` unblocks:

```scheme
(:wat::core::let*
  (((handle :wat::kernel::ProgramHandle<()>)
    (:wat::core::let*
      (((pair :wat::kernel::QueuePair<i64>)
        (:wat::kernel::make-bounded-queue :i64 1))
       ((tx :wat::kernel::QueueSender<i64>) (:wat::core::first pair))
       ((rx :wat::kernel::QueueReceiver<i64>) (:wat::core::second pair))
       ((h :wat::kernel::ProgramHandle<()>) (:wat::kernel::spawn :my::worker rx))
       ((_send :wat::kernel::Sent) (:wat::kernel::send tx 1)))
      h)))                                    ;; ← pair, tx, rx all drop here
  (:wat::kernel::join handle))                ;; ← worker has exited cleanly
```

The Console and CacheService stdlib programs follow this shape: the
caller holds the driver `ProgramHandle` in an outer scope, an inner
`let*` distributes Sender handles, does the work, and exits; the drop
cascade then triggers the driver's clean shutdown.

### Fan-in via `select`

```scheme
(:wat::kernel::select receivers)
;; receivers : :Vec<wat::kernel::QueueReceiver<T>>
;; → :wat::kernel::Chosen<T>   ≡ :(i64, Option<T>)
;; — blocks until any receiver has a value or disconnects
;; — returns the index and :None if disconnected, (Some v) if produced
```

The caller owns the select loop — remove disconnected receivers from
the list, exit when the list is empty. `:wat::std::service::Console`'s
driver is the canonical example.

`:wat::kernel::Chosen<T>` is the fourth substrate alias from
`wat/kernel/queue.wat`. The variable that binds the return value is
universally named `chosen` (Console.wat does it; the docs do it; you
will too). The alias makes the type echo the variable.

### Spawning programs

```scheme
(:wat::kernel::spawn my::app::worker-fn arg1 arg2 ...)
;; → :ProgramHandle<ReturnType>
;; spawn my::app::worker-fn on a new thread with the given args

(:wat::kernel::join handle)
;; → :ReturnType  — blocks until the program exits, returns its state
;; "I trust this thread; panic the caller if it died." Use when a
;; spawn-thread death IS a bug worth halting on.

(:wat::kernel::join-result handle)
;; → :Result<:ReturnType, :wat::kernel::ThreadDiedError>  (arc 060)
;; "Death as data." Match on (Ok value) | (Err Panic|RuntimeError|
;; ChannelDisconnected) — surfaces the spawn-thread's outcome
;; in-band so supervisors / debuggers / tests can discriminate
;; cause without losing context.
```

Each spawned program is an OS thread running the named function. The
program owns its state (moved in via spawn args); when it returns,
its state is dropped or returned via join.

The choice between `join` and `join-result` parallels `assert-eq` vs
`assert-coincident` (arc 057): both verbs are honest; the call site
picks per its tolerance for panic-on-death. `join-result` is what
test harnesses, supervisors, and any code that wants to diagnose a
spawned crash should reach for; `join` stays appropriate when the
spawn-thread genuinely shouldn't fail.

```scheme
;; Story-2 / death-as-data shape:
(:wat::core::match (:wat::kernel::join-result handle) -> :()
  ((Ok _value)
    ;; thread succeeded; do whatever with value
    ())
  ((Err (:wat::kernel::ThreadDiedError::Panic msg))
    (:wat::io::IOWriter/print stderr
      (:wat::core::string::concat "thread panicked: " msg "\n")))
  ((Err (:wat::kernel::ThreadDiedError::RuntimeError msg))
    (:wat::io::IOWriter/print stderr
      (:wat::core::string::concat "thread Err'd: " msg "\n")))
  ((Err :wat::kernel::ThreadDiedError::ChannelDisconnected)
    ;; substrate bug; rare
    ()))
```

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
        // Panic with diagnostic context on failure — bare .unwrap() would
        // give "called Option::unwrap() on a None value" with no clue WHICH
        // path failed. Pattern: name the dispatch path + the input + the
        // underlying error. Same shape as crates/wat-lru/src/shim.rs.
        let inner = Connection::open(&path).unwrap_or_else(|e| {
            panic!(
                ":rust::rusqlite::Connection::open: failed for {:?} — {}",
                path, e
            )
        });
        WatConnection { inner }
    }

    /// `:rust::rusqlite::Connection::query_i64 conn sql` — asks the
    /// DB for an i64; returns `:Option<i64>`. `Some(v)` if the query
    /// yielded a row; `None` for any reason it didn't (no row, DB
    /// locked, syntax error, type mismatch). The caller asked a
    /// question; the answer is either present or absent. The shim
    /// doesn't pre-decide that *why* it's absent matters — that's
    /// the caller's logic, surfaced cleanly through `:None` rather
    /// than smuggled through a panic.
    ///
    /// Discipline: panic for **input validation** (e.g., a non-
    /// primitive map key in `wat-lru` — the wat-side input was
    /// malformed); return `Option<T>` for **lookup outcomes** (the
    /// query was well-formed but didn't find a value).
    pub fn query_i64(&mut self, sql: String) -> Option<i64> {
        self.inner
            .query_row(&sql, params![], |row| row.get(0))
            .ok()
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
    (((cache :wat::lru::LocalCache<String,i64>)
      (:wat::lru::LocalCache::new 128)))
    (... use cache via :wat::lru::LocalCache::put / ::get ...)))
```

Tier 2 — thread-owned. The cache never leaves this program's thread.

### CacheService — shared across programs

When multiple programs need to share a cache, spawn a CacheService.
The program owns the cache on its own thread; clients send requests
through channels.

```scheme
(:wat::core::let*
  (((state :(wat::kernel::HandlePool<wat::lru::CacheService::ReqTx<String,i64>>,
             wat::kernel::ProgramHandle<()>))
    (:wat::lru::CacheService 1024 8))   ;; capacity 1024, 8 client handles
   ((pool :wat::kernel::HandlePool<...>) (:wat::core::first state))
   ((driver :wat::kernel::ProgramHandle<()>) (:wat::core::second state))
   ((client1 :wat::lru::CacheService::ReqTx<String,i64>)
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

### Structured logging — `ConsoleLogger` + ledger db (arcs 086 / 087)

`Console/out` and `Console/err` take strings. For anything beyond
ad-hoc diagnostic markers, the substrate ships a structured
logger and a sqlite-backed ledger. Both are layered on top of
Console; the typical long-running program wires both.

**`:wat::telemetry::ConsoleLogger`** is a closure-over-state
struct: `(con-tx, caller, now-fn, format)`. Built once per
producer; passed by reference into hot paths. Each emission gets
its time auto-stamped and its `:caller` identity injected — the
producer never self-identifies.

```scheme
((logger :wat::telemetry::ConsoleLogger)
 (:wat::telemetry::ConsoleLogger/new
   con-tx :market.observer
   (:wat::core::lambda ((_u :()) -> :wat::time::Instant) (:wat::time::now))
   :wat::telemetry::Console::Format::Edn))

;; Per emission:
(:wat::telemetry::ConsoleLogger/info  logger (:Event::Buy 100.5 7))
(:wat::telemetry::ConsoleLogger/warn  logger (:Event::CircuitBreak "spike"))
(:wat::telemetry::ConsoleLogger/error logger (:Event::CircuitBreak "down"))
```

Level routing: `:debug` and `:info` go to stdout via `Console/out`;
`:warn` and `:error` go to stderr via `Console/err`. Custom keywords
(e.g. `:trace`) fall through to stdout. The line shape is a
`LogLine<E>` struct rendered as `[time level caller data]`.

Five render formats via `Console::Format`:

| Format | Output | Use case |
|---|---|---|
| `:Edn` | `#wat.std.telemetry/LogLine {:time ... :level ...}` | Round-trip-safe via `:wat::edn::read` |
| `:NoTagEdn` | `{:time ... :level ... :data {:_type :ns/Variant ...}}` | Lossy; human-readable EDN logs |
| `:Json` | `{"#tag":"wat.std.telemetry/LogLine","body":{...}}` | Round-trip-safe via wat-edn JSON↔EDN bridge |
| `:NoTagJson` | `{"time":"...","level":"info","caller":"...","data":{"_type":"ns/Variant",...}}` | Lossy; ELK / DataDog / CloudWatch ingestion |
| `:Pretty` | Multi-line indented EDN | Dev-time debug |

The `_type` value in tagless variants is fully-qualified
(`demo.Event/Buy`, not bare `Buy`) — bare variant names collide
across enums; the FQDN is honest identity.

**`:wat::telemetry::Sqlite/auto-spawn`** is the ledger side.
It walks a consumer-defined enum decl at startup, derives one
`CREATE TABLE` per Tagged variant (variant PascalCase →
table snake_case; field kebab → column snake; field type →
SQLite affinity), derives the per-variant INSERT, and dispatches
each batch (arc 089 slice 3 — per-batch contract; one transaction
per drained batch via internal `Db/begin` … `Db/commit`). The
consumer's enum is the schema.

The `pre-install` hook (arc 089 slice 4) runs in the worker
thread after `open` and before substrate auto-installs schemas.
Substrate ships zero default pragmas — consumers pick
`journal_mode`, `synchronous`, `foreign_keys`, etc. through this
seam:

```scheme
;; Lab-side pre-install — pick whatever pragmas your durability
;; profile wants. Substrate forwards each `:wat::sqlite::pragma`
;; call straight to `conn.pragma_update`.
(:wat::core::define
  (:my::pre-install
    (db :wat::sqlite::Db) -> :())
  (:wat::core::let*
    (((_w :()) (:wat::sqlite::pragma db "journal_mode" "WAL"))
     ((_s :()) (:wat::sqlite::pragma db "synchronous" "NORMAL")))
    ()))

((sqlite-spawn :Service::Spawn<my::log::Entry>)
 (:wat::telemetry::Sqlite/auto-spawn
   :my::log::Entry "runs/today.db" 1
   (:wat::telemetry::Service/null-metrics-cadence)
   :my::pre-install))
```

For the explicit "I'm fine with sqlite's defaults" choice,
pass `:wat::telemetry::Sqlite/null-pre-install`.

### Per-run file management — `IOWriter/open-file` (arc 088)

A long-running program that wants its own per-run files
(`runs/<id>.out`, `runs/<id>.err`, `runs/<id>.db`) opens
file-backed writers at `:user::main` startup and passes them to
Console instead of using the parent process's stdio:

```scheme
((out-writer :wat::io::IOWriter)
 (:wat::io::IOWriter/open-file "runs/today.out"))
((err-writer :wat::io::IOWriter)
 (:wat::io::IOWriter/open-file "runs/today.err"))

((con-spawn :Console::Spawn)
 (:wat::std::service::Console/spawn out-writer err-writer 1))
```

Open mode is `write+create+truncate` — fresh file each invocation.
Drop closes the fd; clean shutdown cascade releases all three
files together. The wat program owns its outputs; no shell
redirect needed.

### The double-write discipline

A producer that wants both surfaces takes both handles wired in:

```scheme
(:wat::core::define
  (:my::worker/run
    (logger :wat::telemetry::ConsoleLogger)
    (sqlite-tx :wat::telemetry::Service::ReqTx<my::log::Entry>)
    (ack-tx :wat::telemetry::Service::AckTx)
    (ack-rx :wat::telemetry::Service::AckRx)
    -> :())
  (:wat::core::let*
    (((_say :())
      (:wat::telemetry::ConsoleLogger/info logger
        (:my::Event::Heartbeat 0)))                   ;; occasional, human-friendly
     ((entries :Vec<my::log::Entry>)
      (:wat::core::vec :my::log::Entry
        (:my::log::Entry::Resolved ...)
        (:my::log::Entry::Resolved ...)))
     ((_log :())
      (:wat::telemetry::Service/batch-log         ;; high-fidelity archive
        sqlite-tx ack-tx ack-rx entries)))
    ()))
```

Console gets summary events ("this is happening"); sqlite gets the
full record ("here's exactly what happened"). The same producer
writes both. `:user::main` distributes the handles per CIRCUIT.md.

For a runnable end-to-end example, see
`holon-lab-trading/wat/programs/smoke.wat` — opens three files,
spawns Console + Sqlite, runs a producer that double-writes, joins
cascade. The post-run state is three files in `runs/`: `.out`,
`.err`, `.db`. SQL queries for analysis; EDN-per-line for live
tail.

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

`:wat::holon::Bundle` returns `:wat::holon::BundleResult` (a
typealias for `:Result<:wat::holon::HolonAST,
:wat::holon::CapacityExceeded>`, arc 032). The two `capacity-mode`
values (`:error` — default, returns `Err`; `:panic` — panics; arc
045 renamed `:abort` → `:panic`) set
at program startup determine the runtime behavior when Kanerva's
per-frame bound (`floor(sqrt(d))` for the dim picked by the active
DimRouter, arc 037) is exceeded.

```scheme
(:wat::core::define (:my::app::build (items :wat::holon::Holons)
                    -> :wat::holon::BundleResult)
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
functions. The runner discovers them by signature alone — any
top-level define returning `:wat::test::TestResult` (the role-honest
alias of the underlying `:wat::kernel::RunResult`) is a test. The
deftest macro expands to exactly that shape, so any function
written via deftest is automatically discovered. Names are
descriptive — `test-` prefix is conventional but not required for
discovery (that filter was dropped 2026-04-25 because it caused
silent skips when violated; the signature alone is unambiguous).

**Discovery is recursive.** Add a directory, add a test — no
manifest, no registration step.

### Writing a test — `deftest`

```scheme
;; wat-tests/example.wat
(:wat::test::deftest :my::app::test-two-plus-two
  (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 4))
```

`deftest` takes:
- **name** — the test's keyword path. Any descriptive name works;
  `test-` prefix is conventional but the runner discovers by the
  function's `() -> :wat::kernel::RunResult` signature alone.
- **body** — one expression; the test's actual logic

The deftest sandbox **inherits the outer file's Config** (arc 031),
so capacity-mode and any dim-router override committed at the test
file's top level apply to every test in the file. No per-test config
arguments.

It expands to a named zero-arg function that, when invoked, returns
a `:wat::kernel::RunResult`. The runner invokes each discovered
function, inspects the RunResult's failure slot, reports cargo-style.

For test files that share loads or helpers across multiple tests,
use the **`make-deftest` factory** (arc 029) — define a deftest-shaped
macro whose default-prelude carries the shared setup, then call the
factory's emitted name once per test:

```scheme
(:wat::test::make-deftest :deftest
  ((:wat::load-file! "../../wat/types/candle.wat")
   (:wat::core::define (:test::helper -> :i64) 42)))

(:deftest :my::test-uses-helper
  (:wat::test::assert-eq (:test::helper) 42))
```

Every test sandbox the factory emits gets the load + the helper
define for free; bare-name `(:deftest :name body)` is the per-test
call shape.

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
(:wat::test::deftest :my::test-captures-inner-output
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-ast
        (:wat::test::program
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

The inner program inherits the outer test file's Config — no need
for `(set-capacity-mode!)` or `(set-dim-router!)` setters inside the
`:wat::test::program` body.

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
(:wat::test::deftest :my::test-console-hello
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
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

**Capacity overflow.** A Bundle with more than `floor(sqrt(d))`
items (where `d` is the dim the active DimRouter picks for that
construction) under `:error` mode returns
`(Err (CapacityExceeded ...))`. Callers who ignore the Err by
unwrap will panic at `match` time. Fix: either handle the Err
arm, use `:wat::core::try` in a Result-returning function, or
pre-filter the list to the budget. The thrown `CapacityExceeded`
struct carries `/cost` (actual count) and `/budget` (the limit
at the active `d`) accessors so the error path can shape its
recovery against real numbers.

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

**Signed/digest loads.** `(:wat::load-file! path)` is unverified.
For production code loaded from untrusted sources, use
`(:wat::signed-load-file! path sig pk)` with an Ed25519 signature
or `(:wat::digest-load-file! path digest-hex)` with a SHA-256
digest. Startup halts if verification fails.

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
| `:wat::config::set-capacity-mode!` | `(<keyword>)` | commits mode |
| `:wat::config::set-dim-router!` | `(<router-fn>)` | commits router (arc 037) |
| `:wat::config::set-presence-sigma!` | `(<sigma-fn>)` | commits presence-sigma fn (arc 024) |
| `:wat::config::set-coincident-sigma!` | `(<sigma-fn>)` | commits coincident-sigma fn (arc 024) |
| `:wat::config::capacity-mode` | `()` | `:wat::config::CapacityMode` |
| `:wat::config::global-seed` | `()` | `:i64` |
| `:wat::config::dims` | `()` | `:i64` (compat shim — arc 037) |
| `:wat::config::noise-floor` | `()` | `:f64` (compat shim — arc 037) |
| `:wat::core::define` | `((name (p :T) ... -> :R) body)` | registers function |
| `:wat::core::lambda` | `(((p :T) ... -> :R) body)` | `:fn(T,...)->R` |
| `:wat::core::let*` | `(((b :T) rhs) ...) body` | body's type |
| `:wat::core::match` | `scrutinee -> :T arm1 arm2 ...` | arm result (type `T`) |
| `:wat::core::if` | `cond -> :T then else` | branch result (type `T`) |
| `:wat::core::cond` | `-> :T ((test) body) ... (:else body)` | arm result (type `T`) |
| `:wat::core::try` | `<result-expr>` | Ok-inner type |
| `:wat::core::struct` | `(:path (f :T) ...)` | declares struct |
| `:wat::core::enum` | `(:path v1 v2 (v3 (f :T)) ...)` | declares enum (variants PascalCase per arc 048) |
| `:Enum::Variant` (bare keyword) | — | constructs unit variant (arc 048) |
| `(:Enum::Variant arg1 arg2 ...)` | tagged-variant fields | constructs tagged variant (arc 048) — auto-synthesized constructor function |
| `(:wat::core::match v -> :T (:Enum::Variant body) ((:Enum::Variant b1 b2) body) ...)` | per-variant arms | match on user enum (arc 048); exhaustiveness checked, binders bound to fields by position |
| `:wat::load-file!` | `<path>` | registers loaded file (arc 028) |
| `:wat::load-string!` | `<source>` | registers loaded source (arc 028) |
| `:wat::digest-load-file!` / `digest-load-string!` | `<path-or-src> <hex-digest>` | SHA-256 verified load |
| `:wat::signed-load-file!` / `signed-load-string!` | `<path-or-src> <sig-b64> <pk-b64>` | Ed25519 verified load |
| `:wat::core::vec` | `:T v1 v2 ...` | `:Vec<T>` |
| `:wat::core::list` | `:T v1 v2 ...` | `:Vec<T>` (alias) |
| `:wat::core::tuple` | `v1 v2 ...` | `:(T1,T2,...)` |
| `:wat::core::first` / `second` / `third` | `<tuple-or-vec>` | tuple → `T`; Vec → `Option<T>` (arc 047 — Vec accessors return Option to honestly signal empty/short) |
| `:wat::core::last` | `<vec>` | `Option<T>` — `None` for empty, `Some(items[len-1])` otherwise (arc 047) |
| `:wat::core::find-last-index` | `xs pred-fn` | `Option<i64>` — index of rightmost element where pred holds (arc 047); `None` if no match or empty |
| `:wat::core::f64::max-of` / `min-of` | `<vec-of-f64>` | `Option<f64>` — `None` for empty (arc 047) |
| `:wat::core::length` / `empty?` / `reverse` / `take` / `drop` | list ops | various |
| `:wat::core::+/-/*//` | `a b` | polymorphic arithmetic (arc 050); both args must be numeric (`:i64` or `:f64`); result is `:f64` if either is `:f64`, else `:i64` — Lisp-traditional int → float promotion |
| `:wat::core::i64::+/-/*//` / `f64::+/-/*//` | `a b` | typed strict arithmetic — same arity, but the type checker rejects cross-type input. Reach for these when the type-guard behavior matters (e.g., index arithmetic where i64 is the only honest answer) |
| `:wat::core::f64::round` | `v digits` | round-half-away-from-zero (arc 019) |
| `:wat::core::f64::max` / `min` | `a b` | binary min/max (arc 046) — strict-f64 |
| `:wat::core::f64::abs` | `v` | absolute value (arc 046) — strict-f64 |
| `:wat::core::f64::clamp` | `v lo hi` | bound `v` into `[lo, hi]` (arc 046); errors if `lo > hi` or NaN bounds |
| `:wat::std::math::ln` / `log` / `exp` / `sin` / `cos` | `x` | unary math (`log` is alias for `ln`); accept i64 promotion |
| `:wat::std::math::pi` | — | π as `:f64` |
| `:wat::time::now` | — | `:wat::time::Instant` — current wall-clock time (arc 056; world-observing — sibling of `:wat::io::*` rather than under `:wat::std::*`) |
| `:wat::time::at` / `at-millis` / `at-nanos` | `epoch:i64` | `:wat::time::Instant` from epoch seconds / ms / ns (i64 ns saturates ~year 2262) |
| `:wat::time::from-iso8601` | `s:String` | `:Option<wat::time::Instant>` — `:None` on parse failure; accepts the RFC 3339 grammar |
| `:wat::time::to-iso8601` | `instant digits:i64` | `:String` — UTC ISO 8601 with N fractional second digits; `digits` clamps to `[0, 9]` |
| `:wat::time::epoch-seconds` / `epoch-millis` / `epoch-nanos` | `instant` | `:i64` — truncating; `epoch-nanos` panics outside i64-ns range (~1677–2262) |
| `:wat::time::Duration` | type | Non-negative time interval as i64 nanos (arc 097). Distinct from Instant so polymorphic `:wat::time::-` can dispatch on RHS variant. Always non-negative — direction lives in the operation, not the sign |
| `:wat::time::Nanosecond` / `Microsecond` / `Millisecond` / `Second` / `Minute` / `Hour` / `Day` | `n:i64` | `:wat::time::Duration` — unit constructors, ActiveSupport-style. `(:wat::time::Hour 1)` reads as "1 Hour." Panics on negative input or i64 overflow (~290k year max for Hour) (arc 097 slice 1) |
| `:wat::time::-` | `instant duration` OR `instant_a instant_b` | `:wat::time::Instant` (subtract interval) OR `:wat::time::Duration` (elapsed between, panics if negative); polymorphic on RHS variant — same shape as ActiveSupport's `time1 - time2 / time - 1.hour` (arc 097 slice 2) |
| `:wat::time::+` | `instant duration` | `:wat::time::Instant` — advance by interval. LHS-Duration arithmetic not in this arc (arc 097 slice 2) |
| `:wat::time::ago` | `duration` | `:wat::time::Instant` — `(- (now) duration)`. The Ruby `1.hour.ago` shape (arc 097 slice 3) |
| `:wat::time::from-now` | `duration` | `:wat::time::Instant` — `(+ (now) duration)`. The Ruby `2.days.from_now` shape (arc 097 slice 3) |
| `:wat::time::nanoseconds-ago` / `microseconds-ago` / `milliseconds-ago` / `seconds-ago` / `minutes-ago` / `hours-ago` / `days-ago` | `n:i64` | `:wat::time::Instant` — pre-composed sugar; `(hours-ago 1)` ≡ `(ago (Hour 1))`. Reads cleaner at every callsite (arc 097 slice 4) |
| `:wat::time::nanoseconds-from-now` / `microseconds-from-now` / `milliseconds-from-now` / `seconds-from-now` / `minutes-from-now` / `hours-from-now` / `days-from-now` | `n:i64` | `:wat::time::Instant` — pre-composed sugar; `(days-from-now 2)` ≡ `(from-now (Day 2))` (arc 097 slice 4) |
| `:wat::core::i64::to-string` / `to-f64` | `n` | infallible — `:String` / `:f64` |
| `:wat::core::f64::to-string` / `to-i64` | `x` | `:String` / `:Option<i64>` (NaN/inf/out-of-range → `:None`) |
| `:wat::core::string::to-i64` / `to-f64` / `to-bool` | `s` | `:Option<T>` (unparseable → `:None`) |
| `:wat::core::bool::to-string` | `b` | `"true"` / `"false"` |
| `:wat::holon::Vector` | type | First-class materialized algebra vector (arc 052). Usable as struct field, parameter, return type, container element. Equality is bit-exact; for graded similarity reach for `cosine` / `presence?` / `simhash` |
| `:wat::holon::encode` | `holon` | `:wat::holon::Vector` — explicit materialization of a HolonAST into a Vector at the ambient d (arc 052). Lets users hold a Vector value, store it in caches, or pass it to Vector-tier algebra |
| `:wat::core::Bytes` | type alias | Substrate-general byte buffer (arc 062). `:wat::core::Bytes ≡ :Vec<u8>` — alias resolves structurally, both forms work at call sites. The canonical name for the wire-format / storage / transmission shape used by `vector-bytes` (arc 061) and future crypto/IO/hashing/network ops |
| `:wat::core::Bytes::to-hex` | `bs` | `:String` — lowercase hex, no separators (arc 063). Deterministic; same Bytes always produce the same String |
| `:wat::core::Bytes::from-hex` | `s` | `:Option<wat::core::Bytes>` — parse hex back to Bytes (arc 063). Mixed case accepted; empty string → empty Bytes; `:None` on odd length, non-hex character, or `0x` prefix |
| `:wat::core::show` | `v` | `:String` — polymorphic value rendering (arc 064). Per-Value-variant dispatch: primitives render as their wat literal form (`true` / `42` / `"hello"` / `:foo`); Option/Result/Vec/Tuple recurse; substrate compounds (Vector/Struct/Enum/handles) render as type-named summaries. Used internally by `assert-eq` to populate failure payload's actual/expected; exposed for diagnostic prints and future assertions |
| `:wat::holon::vector-bytes` | `vec` | `:wat::core::Bytes` — serialize a Vector to a portable byte buffer (arc 061). 4-byte dim header + 2-bit-per-cell ternary packing. The wire format for the cryptographic-substrate transmission protocol |
| `:wat::holon::bytes-vector` | `bs` | `:Option<wat::holon::Vector>` — deserialize the wire format (arc 061). Takes `:wat::core::Bytes`. `:None` on short / truncated / dim-mismatched / corrupt input |
| `:wat::holon::coincident?` | `a b` | `:bool` — polymorphic over HolonAST or Vector inputs in either position (arc 061 widened from HolonAST-only); `(1 - cosine) < coincident-floor` at encoded d |
| `:wat::holon::simhash` | `holon` or `vector` | `:i64` — Charikar SimHash, polymorphic over HolonAST or Vector input (arcs 051 + 052). Cosine-similar inputs share keys; the position-allocator for content-addressed caches. Composes with `:rust::lru::LruCache<i64,V>` for bidirectional engram lookup |
| `:wat::core::>` / `=` / `<` / `>=` / `<=` | `a b` | polymorphic comparison/equality — same-type for non-numeric, cross-numeric (i64+f64) accepted with promotion (arc 050); always returns `:bool` |
| `:wat::core::i64::>` / `=` / `<` / `>=` / `<=` / `f64::*` | `a b` | typed strict comparison/equality (arc 050) — rejects cross-type at the checker; opt-in for type-guard discipline |
| `:wat::io::IOReader/read-line` | `stdin` | `:Option<String>` |
| `:wat::io::IOWriter/print` | `handle string` | `:()` |
| `:wat::kernel::spawn` | `<fn-path> args...` | `:ProgramHandle<R>` |
| `:wat::kernel::join` | `handle` | `R` — panics caller on spawn-thread death |
| `:wat::kernel::join-result` | `handle` | `:Result<R, wat::kernel::ThreadDiedError>` — death-as-data; 3 variants discriminate Panic / RuntimeError / ChannelDisconnected (arc 060) |
| `:wat::kernel::make-bounded-queue` | `:T n` | `:(Sender<T>, Receiver<T>)` |
| `:wat::kernel::make-unbounded-queue` | `:T` | `:(Sender<T>, Receiver<T>)` |
| `:wat::kernel::send` | `sender value` | `:wat::kernel::Sent` ≡ `:Option<()>` — `(Some ())` on sent, `:None` on disconnect |
| `:wat::kernel::recv` / `try-recv` | `receiver` | `:Option<T>` |
| `:wat::kernel::select` | `receivers` | `:(i64, Option<T>)` |
| `:wat::kernel::drop` | `handle` | `:()` |
| `:wat::kernel::stopped?` / `sigusr1?` / ... | `()` | `:bool` |
| `:wat::kernel::HandlePool::new` / `pop` / `finish` | various | pool ops |
| `:wat::std::service::Console` | `stdout stderr n` | `(HandlePool, Driver)` |
| `:wat::lru::CacheService` (wat-lru) | `capacity count` | `(HandlePool, Driver)` |
| `:wat::lru::LocalCache::new` / `put` / `get` (wat-lru) | various | per-program LRU |
| `:wat::holon::Atom` | `<value>` | `:wat::holon::HolonAST` — polymorphic dispatcher (arc 057). Primitive → matching typed leaf; HolonAST → opaque-identity wrap; quoted wat form → structural lowering. New code: prefer the named siblings `:wat::holon::leaf` (primitives) and `:wat::holon::from-watast` (quoted forms) — one verb per move (arc 065). Polymorphism preserved for back-compat. |
| `:wat::holon::leaf` | `<primitive>` | `:wat::holon::HolonAST` — lift a primitive (i64/f64/bool/String/keyword) to a typed HolonAST leaf (arc 065). Honest verb for the "wrap a value as a leaf" move that Atom's polymorphism left ambiguous. |
| `:wat::holon::from-watast` | `<wat-ast>` | `:wat::holon::HolonAST` — lower a quoted wat form to a HolonAST tree (arc 065). The `from-watast` / `to-watast` pair reads as one round-trip at call sites. |
| `:wat::holon::to-watast` | `holon` | `:wat::WatAST` — Story-2 recovery (arc 057): structural inverse of Atom's quote-lowering. Pair with `:wat::eval-ast!` when you want the value, not the coordinate. Lossy on identifier scope; round-trips cleanly enough for the eval-and-get-the-value workflow. |
| `:wat::holon::Bind` | `a b` | `:wat::holon::HolonAST` |
| `:wat::holon::Bundle` | `list-of-holons` | `:wat::holon::BundleResult` (arc 032) |
| `:wat::holon::Permute` | `holon k` | `:wat::holon::HolonAST` |
| `:wat::holon::Thermometer` | `value min max` | `:wat::holon::HolonAST` — locality-preserving gradient encoding of a scalar over `[min, max]`. Two near-equal values produce vectors with cosine ≈ `1 - 2·\|Δ\|/(max-min)`. Pairs with arc 057's quasi-orthogonal `F64` leaf as the consumer's choice for "discrete vs. continuous identity at this leaf." Name from the HDC/ML tradition; see `runtime.rs::eval_algebra_thermometer` for full attribution and BOOK chapters 57 + 66 for substrate role. |
| `:wat::holon::Blend` | `a b w1 w2` | `:wat::holon::HolonAST` |
| `:wat::holon::ReciprocalLog` | `n value` | `:wat::holon::HolonAST` (arc 034) |
| `:wat::holon::cosine` / `dot` | `a b` | `:f64` — polymorphic over HolonAST or Vector inputs (arc 052); mixed (one AST, one Vector) is permitted and the AST encodes at the Vector's d |
| `:wat::holon::presence?` | `target reference` | `:bool` — cosine > presence-floor |
| `:wat::holon::coincident-explain` | `a b` | `:wat::holon::CoincidentExplanation` (arc 069) — diagnostic record bundling cosine, floor, dim, sigma, the predicate result, and `min-sigma-to-pass` (smallest sigma at which the pair would coincide). Polymorphic over HolonAST/Vector. Use when a coincidence judgement disagrees with expectation |
| `:wat::holon::eval-coincident?` | `a-ast b-ast` | `:Result<:bool, EvalError>` (arc 026) |
| `:wat::holon::eval-edn-coincident?` | `a-src b-src` | `:Result<:bool, EvalError>` |
| `:wat::holon::eval-digest-coincident?` | `<8 args>` | `:Result<:bool, EvalError>` — 4 per side, SHA-256 |
| `:wat::holon::eval-signed-coincident?` | `<12 args>` | `:Result<:bool, EvalError>` — 6 per side, Ed25519 |
| `:wat::form::matches?` | `subject (:TYPE-NAME clause ...)` | `:bool` — Clara-style single-item pattern matcher (arc 098). Substrate-recognized special form. Subject can be any value; `:None` / `(Some non-struct)` / non-Struct / wrong-type-Struct return `false` (Clara semantics — no error). Clauses are bindings or constraints. Bindings `(= ?var :field)` push `?var → field-value` into scope for subsequent clauses. Constraint vocabulary inside clauses (no `:wat::core::` prefix needed): `=` `<` `>` `<=` `>=` `not=` `and` `or` `not` `where`. The `where` escape evaluates an arbitrary wat expression in the binding scope; must return `:bool`. Logic variables (`?var`) lex natively per the wat tokenizer. Pattern grammar errors surface at type-check |
| `:wat::core::quote` | `<form>` | `:wat::WatAST` — captures AST as data |
| `:wat::core::forms` | `f1 f2 ... fn` | `:Vec<wat::WatAST>` — variadic quote |
| `:wat::core::macroexpand` | `<quoted-form>` | `:wat::WatAST` — expands until non-macro head (arc 030) |
| `:wat::core::macroexpand-1` | `<quoted-form>` | `:wat::WatAST` — peels exactly one layer (arc 030) |
| `:wat::core::conj` | `vec item` | new collection — polymorphic over HashSet/Vec (arc 025) |
| `:wat::core::concat` | `v1 v2 ...` | `:Vec<T>` — variadic Vec concatenation; ≥1 arg; all args same `Vec<T>` (arc 059) |
| `:wat::core::assoc` | `coll k v` | new collection — polymorphic over HashMap/Vec (arc 025) |
| `:wat::core::dissoc` | `m k` | `:HashMap<K,V>` — new map without `k`; missing key is no-op (arc 058) |
| `:wat::core::get` | `coll k-or-i` | `:Option<T>` — polymorphic over HashMap/HashSet/Vec (arc 025) |
| `:wat::core::contains?` | `coll k-or-i` | `:bool` — polymorphic over HashMap/HashSet/Vec (arc 025) |
| `:wat::core::keys` | `m` | `:Vec<K>` — order unspecified; sort post-call for determinism (arc 058) |
| `:wat::core::values` | `m` | `:Vec<V>` — order unspecified; sort post-call for determinism (arc 058) |
| `:wat::core::empty?` | `coll` | `:bool` — polymorphic over Vec/HashMap/HashSet (extended in arc 058) |
| `:wat::eval-ast!` | `<wat-ast>` | `:Result<wat::holon::HolonAST, wat::core::EvalError>` — evaluates already-parsed AST (arc 028); arc 066 wraps the terminal value as HolonAST per scheme so `(Ok h)` is genuinely a HolonAST (use `:wat::core::atom-value` to extract the primitive). Forms whose terminal value has no HolonAST representation (Vec / Tuple / channels / etc.) return `Err` |
| `:wat::eval-step!` | `<wat-ast>` | `:Result<wat::eval::StepResult, wat::core::EvalError>` — performs ONE call-by-value reduction at the leftmost-outermost redex (arc 068). Returns `StepNext form` when a rewrite happened (`form` is the next WatAST to feed back), `StepTerminal value` when this step reduced a redex (chain length ≥ 1), `AlreadyTerminal value` when the input was already a value-shape (arc 070; chain length 0 — `to-watast(holon)` round-trips, holon-constructor calls with all-canonical args, primitive literals). Effectful ops (`:wat::kernel::*`, `:wat::io::*`, `:wat::eval-*`, `:wat::load*`, `:wat::config::*`) refuse with `EvalError(kind="effectful-in-step")`; ops without a step rule yet refuse with `kind="no-step-rule"`. The substrate primitive backing BOOK Chapter 59's dual-LRU coordinate cache: every intermediate form is its own cache key |
| `:wat::eval::StepResult` | enum | `StepNext { form: :wat::WatAST }` / `StepTerminal { value: :wat::holon::HolonAST }` / `AlreadyTerminal { value: :wat::holon::HolonAST }` — three outcomes of a single reduction step (arc 068, arc 070). Match by full keyword path: `((:wat::eval::StepResult::StepNext next) ...)` / `((:wat::eval::StepResult::StepTerminal h) ...)` / `((:wat::eval::StepResult::AlreadyTerminal h) ...)` |
| `:wat::eval::walk` | `<form> <init> <visit>` | `:Result<(:wat::holon::HolonAST, :A), :wat::core::EvalError>` — fold over the eval-step! chain (arc 070). Visitor fires once per coordinate with `(acc, form, step-result)` and returns `WalkStep<A>`: `Continue(acc')` keeps walking, `Skip(terminal, acc')` short-circuits with the caller's terminal. The substrate primitive that lifts the walker pattern proofs 015/016/017/018 each reimplemented |
| `:wat::eval::WalkStep<A>` | enum | `Continue { acc: A }` / `Skip { terminal: :wat::holon::HolonAST, acc: A }` — what `:wat::eval::walk`'s visitor returns. Generic over `A` so the consumer's accumulator can be any type (cache, trace, counter, tier) |
| `:wat::eval-edn!` / `eval-file!` | `<source>` / `<path>` | parses+evaluates string or file |
| `:wat::eval-digest-string!` / `eval-digest-file!` | `<src/path> <hex>` | SHA-256 verified eval |
| `:wat::eval-signed-string!` / `eval-signed-file!` | `<src/path> <sig> <pk>` | Ed25519 verified eval |
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
| `:wat::test::deftest` | `name body` | registers named zero-arg RunResult fn (arc 031 — inherits config) |
| `:wat::test::make-deftest` | `name (forms ...)` | registers a deftest-shaped macro with default-prelude forms (arc 029) |
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
