# Naming Conventions

Rules for adding new primitives to wat. Derived from the arc 005
stdlib naming audit. When these disagree with a primitive in the
[inventory](./arc/2026/04/005-stdlib-naming-audit/INVENTORY.md),
the audit fixes the primitive — not the convention.

---

## Privileged prefixes

`:wat::*` and `:rust::*` are reserved by the runtime. User code
cannot register under them; the evaluator refuses. These two
namespaces ship only via the privileged `register_stdlib_*` paths
or through `#[wat_dispatch]` for `:rust::*`.

Every other prefix is user territory.

## Namespaces

| Prefix | What lives here |
|---|---|
| `:wat::core::*` | Evaluator primitives — forms (`define`, `lambda`, `let*`, `if`, `match`), primitive types (`i64`, `bool`, `String`, ...), macros, eval-family, primitive-type operations (`i64::+`, `bool::and`), core collection constructors (`vec`, `list`, `cons`, `conj`). Cannot be written in wat. |
| `:wat::config::*` | Runtime-committed configuration values (noise floor, dimensions). Read-only after config pass. |
| `:wat::algebra::*` | VSA primitives — `Atom`, `Bundle`, `Unbundle`, `Amplify`, `Compose`, `cosine`, `presence?`, `Resonance`, `Thermometer`, `Blend`. |
| `:wat::kernel::*` | CSP primitives — `spawn`, `send`, `recv`, `select`, `drop`, `join`, `make-bounded-queue`, `HandlePool`, signal handlers. |
| `:wat::io::*` | Stdio primitives — `stdin`, `stdout`, `stderr`, `println`. |
| `:wat::std::*` | Stdlib built on primitives. Each entry should be expressible (in principle) in wat itself, even if shipped as Rust for performance. `LocalCache`, `stream::*`, `program::Console`, `program::Cache`, `list::*`, `math::*`. |
| `:rust::*` | Surfaced Rust types via `#[wat_dispatch]`. Paths mirror real Rust (`:rust::std::iter::Iterator`, `:rust::crossbeam_channel::Receiver`). |
| `:user::*` | User composition space — community wat crates AND user program code. See "External wat crates" below. |

### External wat crates (arc 013)

The `:user::*` namespace is an open composition tree. Convention
carves three sub-trees for different kinds of code:

| Sub-tree | Who claims it | Shape |
|---|---|---|
| `:user::wat::std::<crate>::*` | Community stdlib-tier crates — wrappers around Rust libraries (LRU, SQLite, Redis) that present wat-native surfaces. The `::wat::` marker signals "I'm the external-library surface of wat." | `:user::wat::std::lru::LocalCache`, `:user::wat::std::sqlite::Connection` |
| `:user::<org>::<name>::*` | Community general-purpose crates — domain libraries, frameworks, application toolkits. Shape mirrors npm `@scope/pkg`, Java reverse-DNS, Go module paths. | `:user::acme::billing::Invoice`, `:user::holon::lab::trading::Post` |
| `:user::<user-app-tree>::*` | User's own program code — your project, your sub-structure. No collisions with community crates because your tree claims a unique root. | `:user::my-app::main`, `:user::alice::scratch::test` |

**Claim-by-convention, not runtime-enforced.** An author who
believes their crate is stdlib-quality claims
`:user::wat::std::<crate>::*`. General-purpose crates claim
`:user::<org>::<name>::*`. The runtime doesn't police taste;
it polices collisions. Two crates claiming the same path fail
loud at startup via duplicate-define detection.

**Cargo is the first line of crate-level collision defense.**
Crate names are globally unique on crates.io, so two `wat-lru`s
cannot coexist in one binary. Path collisions inside wat can
only happen if two differently-named crates claim the same wat
namespace — detectable, fail-loud at startup.

**`wat_sources()` + `register()` is the contract.** A wat
crate that ships wat source + a Rust shim MUST expose:

```rust
pub fn wat_sources() -> &'static [wat::WatSource];
pub fn register(&mut wat::rust_deps::RustDepsBuilder);
```

Naming these exactly (not `stdlib_sources()` or `wat_files()`
or similar) preserves grep-ability across the ecosystem and
lets `wat::main!` / `wat::test_suite!` find them by convention.

**Reference:** `crates/wat-lru/` is the first external wat
crate shipped. Its shape is the walkable template:
`wat_sources()` returns its baked `.wat` files;
`register()` forwards to `#[wat_dispatch]`-generated code that
wires the Rust shim. `examples/with-lru/` shows the consumer
shape — `wat::main! { source: ..., deps: [wat_lru] }` and a
user `.wat` program.

**Convergence with prior art:**

| Ecosystem | Deps manifest | Namespace shape | Collision handling |
|---|---|---|---|
| Cargo | `Cargo.toml` | `crate_name::...` | Cargo enforces global unique |
| npm | `package.json` | `@scope/pkg` | scope-level uniqueness |
| Clojure | `deps.edn` | `my.org.project.*` | reverse-DNS convention |
| Go | `go.mod` | `github.com/user/repo` | module-path uniqueness |
| **wat** | **`Cargo.toml` (reuse)** | **`:user::<org>::<name>::*`** | **Cargo uniqueness + startup-collision fail-loud** |

wat inherits Cargo's authority (our deps ARE Cargo crates) and
layers a convention on top of its own namespace space. No
parallel registry needed.

### Crate folder layouts (arc 015)

Two walkable templates — one for publishable wat crates, one
for consumer apps. Both use real `cargo` invocations; no
separate wat build tool.

#### Publishable wat crate

```
my-wat-crate/
├── Cargo.toml           # [dependencies] wat + whatever Rust crate(s) this wraps
├── src/
│   ├── lib.rs           # pub fn wat_sources() + pub fn register()
│   └── shim.rs          # optional — #[wat_dispatch] impl for wrapped Rust type(s)
├── wat/                 # optional — baked .wat files (include_str!'d from lib.rs)
│   └── *.wat
├── wat-tests/           # optional — the crate's own deftests
│   └── *.wat
└── tests/
    └── wat_suite.rs     # optional — one-line wat::test_suite!
```

Reference: `crates/wat-lru/`. Ships both sides of the contract
(`wat_sources()` returns two baked `.wat` files via
`include_str!`, `register()` forwards to `#[wat_dispatch]`-
generated code), its own `wat-tests/` with deftests, and
`tests/wat_suite.rs` invoking `wat::test_suite! { path:
"wat-tests", deps: [wat_lru] }` — self-testing its published
surface.

#### Consumer binary

```
my-app/
├── Cargo.toml           # [dependencies] wat + wat-lru + whatever wat crates
├── src/
│   ├── main.rs          # one-line: wat::main! { source: ..., deps: [...] }
│   └── program.wat      # the user's program
├── wat-tests/           # optional — the user's deftests
│   └── *.wat
└── tests/
    └── tests.rs         # optional — one-line: wat::test_suite! { path: "wat-tests", deps: [...] }
```

Reference: `examples/with-lru/`. One Rust file invokes
`wat::main!`; one wat file IS the program. For users that want
their OWN `:rust::*` symbols (app-specific Rust types), add a
`src/shim.rs` with `#[wat_dispatch]` impls + a `register()` fn,
then add the shim module to the macros' `deps: [...]` list.

### Three varieties of wat crate

A wat crate satisfies the two-part contract (`wat_sources()`
+ `register()`), but either half can be trivial. Three shapes
cover the space:

| Variety | `wat_sources()` | `register()` | Example |
|---|---|---|---|
| **Wrapper** (wat surface around Rust types) | baked `.wat` files with typealiases + thin defines | adds `#[wat_dispatch]`'d types to builder | `wat-lru` — `LocalCache`/`CacheService` over `lru::LruCache` |
| **Rust-surface** (direct `:rust::*` access) | `&[]` | adds `#[wat_dispatch]`'d types to builder | hypothetical `wat-regex` — users write `:rust::regex::Regex::matches` directly |
| **Pure-wat** (wat-only code) | baked `.wat` files | `\|_\|{}` no-op | hypothetical `wat-extra-list-combinators` using only already-registered types |

All three satisfy the same Rust-level trait — they differ only
in what their two functions actually do. `wat::main!` and
`wat::test_suite!` compose them identically.

### Viewing per-wat-test output under `cargo test`

`wat::test_suite!` expands to a `#[test] fn wat_suite()` that
Cargo's libtest captures per convention: stdout from the
outer `#[test]` is hidden on success, shown only on failure.
By default you see `test wat_suite ... ok` and nothing about
the N wat tests that ran inside.

To see the runner's per-test output live:

```bash
cargo test -- --nocapture       # stream all output as it's produced
cargo test -- --show-output     # print captured output after each test
```

Silent-on-success / loud-on-failure is standard Cargo
convention. On failure, the panic payload already includes
every failing test's summary, so `cargo test` without flags
gives you what you need to debug.

### Failure output — Rust-styled, wat-located (arc 016)

When an assertion fires, the panic hook writes Rust-styled
output to stderr with **wat-source** `file:line:col`:

```
thread 'main' panicked at wat-tests/LocalCache.wat:12:5:
assert-eq failed
  actual:   -1
  expected: 42
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Format mirrors `cargo test`'s assertion panics line-for-line.
`RUST_BACKTRACE=1` adds a `stack backtrace:` section with the
wat call chain — each frame carrying a real `file:line:col`
(user frames point into your `.wat`, runtime frames point
into `wat-rs/src/*.rs`, same convention Rust uses for stdlib
frames). USER-GUIDE § "Failure output" has a worked example.

### Install-once discipline (arc 015 slice 3a)

Both halves of the external-crate contract install
process-globally via OnceLock — first caller wins. A test
binary is one install; a consumer `main.rs` is one install.
Tests needing different dep sets live in separate `tests/*.rs`
files (Cargo compiles each to its own test binary).

The win: once installed, every subsequent freeze (main, test,
sandbox via `run-sandboxed-ast`, fork child via
`run-hermetic-ast`) transparently sees the dep surface.
`deftest` bodies can use `:user::wat::std::lru::LocalCache::*`
because the inner sandbox's `startup_from_forms` pulls
installed deps from the global state.

## Name formats

- `snake-case` for functions: `make-bounded-queue`, `for-each`,
  `spawn-producer`.
- `PascalCase` for types: `Bundle`, `HashMap`, `Console`, `Stream`.
- `?` suffix for predicates: `presence?`, `empty?`, `capital-available?`.
- `!` suffix for side-effect forms where the caller should notice:
  `use!`. Most side-effect primitives (`send`, `recv`, `println`)
  don't carry `!` because their purpose is visible in the name;
  `!` is for forms that would otherwise read as pure.
- `::` segments the path; `-` segments words within a segment.
- Qualifiers AFTER the base name:
  `:wat::core::i64::+` (ops on `i64`), not `:wat::core::+::i64`.

## When to add a primitive

The stdlib is a blueprint, not a reference library. A primitive
earns its slot when a concrete caller demands it — not
speculatively.

Before adding a new form, two checks:

**1. Absence is signal.** If the feature seems missing, ask *why*
before patching. Absence points in one of two directions and you
need to know which before reaching for code:

- **Real gap, close it.** arc 004's `reduce` was a missing
  canonical type-normalization pass — two half-passes existed that
  every shape-inspection site had to chain manually. The substrate
  work was the fix.
- **Feature that shouldn't exist, reframe the combinator.** arc
  006's `first(stream, n) -> Vec<T>` as a terminal would have
  needed a force-drop primitive wat deliberately doesn't ship
  (scope discipline IS shutdown discipline). Reframing as
  `take(stream, n) -> Stream<T>` — a stage, not a terminal —
  sidestepped the gap entirely. The missing primitive was the
  language telling us the combinator shape was wrong.

Ask which direction before patching.

**2. Verbose is honest.** Before adding an "ergonomic" form,
write out what it expands to and list what it ELIMINATES. For
each eliminated thing: ceremony or information? If information,
rejected. (See arc 004's pipeline composer — the eliminated
per-stage type annotations were information, not ceremony.)

Both lessons were captured as numbered procedures in arc 004's
INSCRIPTION. Both are memory entries
(`feedback_absence_is_signal`, `feedback_verbose_is_honest`)
because both recur across sessions.

## Sources of truth

- **Canonical primitive list**:
  [`arc/2026/04/005-stdlib-naming-audit/INVENTORY.md`](./arc/2026/04/005-stdlib-naming-audit/INVENTORY.md)
- **Language specification**: `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION.md`
- **As-shipped contracts**: `arc/*/INSCRIPTION.md` — each arc's
  shipped surface. If INSCRIPTION and DESIGN disagree, INSCRIPTION
  wins.
