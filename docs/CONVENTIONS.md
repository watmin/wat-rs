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
| `:wat::core::*` | Evaluator primitives — forms (`define`, `lambda`, `let*`, `if`, `match`), primitive types (`i64`, `bool`, `String`, ...), macros, eval-family, primitive-type operations (`i64::+`, `bool::and`), core collections (`vec`, `list`, `cons`, `conj`, `HashMap`, `HashSet`, `get`, `contains?`, `assoc`). Cannot be written in wat. |
| `:wat::config::*` | Runtime-committed configuration: `capacity-mode` (`:error` / `:panic` — arc 045 renamed `:abort` → `:panic`), `dim-router` function (multi-tier dim selection per AST surface — arc 037), `presence-sigma` / `coincident-sigma` functions of `d` (arc 024), `global-seed`. Compat shim accessors `dims` / `noise-floor` return `DEFAULT_TIERS[0]` defaults. Read-only after config pass. |
| `:wat::holon::*` | Holon algebra — the `HolonAST` type, the six AST-producing primitives (`Atom`, `Bind`, `Bundle`, `Blend`, `Permute`, `Thermometer`), the four measurements (`cosine`, `dot`, `presence?`, `coincident?`), the `eval-coincident?` family (arc 026), the `CapacityExceeded` error type, and typealiases `Holons` / `BundleResult` (arcs 032, 033). One namespace for the whole holon surface. |
| `:wat::kernel::*` | CSP primitives — `spawn`, `send`, `recv`, `select`, `drop`, `join`, `make-bounded-queue`, `HandlePool`, signal handlers. |
| `:wat::io::*` | Stdio primitives — `stdin`, `stdout`, `stderr`, `println`. |
| `:wat::std::*` | Stdlib built on primitives. Each entry should be expressible (in principle) in wat itself, even if shipped as Rust for performance. `stream::*`, `service::Console`, `hermetic`, `test::*`. (LocalCache + CacheService moved to `:wat::lru::*` via arcs 013 + 036.) |
| `:wat::lru::*` | LRU cache surface (external workspace member `crates/wat-lru/`, namespace promoted to `:wat::*` via arc 036). `LocalCache<K,V>`, `CacheService<K,V>`. |
| `:rust::*` | Surfaced Rust types via `#[wat_dispatch]`. Paths mirror real Rust (`:rust::std::iter::Iterator`, `:rust::crossbeam_channel::Receiver`). |
| `:user::*` | User composition space — community wat crates AND user program code. See "External wat crates" below. |

### Core vs stdlib rubric (arc 021)

The distinction the two tiers encode — load-bearing enough to
name explicitly:

- **`:wat::core::*` — evaluator primitives that CANNOT be written
  in wat.** Arithmetic operators, primitive-type conversions,
  control-flow forms, macro-definition forms, and the fundamental
  collection types (Vec, HashMap, HashSet) with their constructors
  + primitive accessors. These reach Rust internals (f64 math,
  HashMap buckets, evaluator state) that wat has no way to
  express. The `:wat::core::*` namespace is the "language as
  defined by the Rust host."

- **`:wat::std::*` — stdlib EXPRESSIBLE in wat, even if shipped
  as Rust for performance.** Named compositions over core
  (algebra idioms like `Circular`, `Log`, `Subtract`), services
  implemented in wat source on top of kernel primitives (Console,
  Cache), stream / list combinators, math transcendentals (in
  principle Taylor-series'd in wat). The `:wat::std::*` namespace
  is "the library built on the language."

When adding a new primitive, ask: "could this be written as a wat
function on top of what exists?" If no — it's core. If yes, even
if shipped as Rust — it's std.

Arc 021 corrected drift where HashMap / HashSet / get / contains?
had been placed at `:wat::std::*` when they should have been at
`:wat::core::*` (they reach Rust bucket internals; can't write in
wat). `assoc` from arc 020 was already at core by this rule.

### External wat crates (arcs 013 + 036)

The `:wat::*` and `:user::*` prefixes split along a single
rule: **workspace-member crates of wat-rs claim `:wat::*`;
everyone else claims `:user::*`.**

| Sub-tree | Who claims it | Shape |
|---|---|---|
| `:wat::<crate>::*` | **First-party workspace-member crates of wat-rs** (`crates/wat-*/` sub-tree). Co-authored, co-released, co-reviewed in this repo. Promoted to the reserved-prefix tier because workspace membership IS the trust signal. | `:wat::lru::LocalCache`, future `:wat::sqlite::Connection`, `:wat::redis::Client` |
| `:user::<org>::<name>::*` | Community general-purpose crates — domain libraries, frameworks, application toolkits. Shape mirrors npm `@scope/pkg`, Java reverse-DNS, Go module paths. | `:user::acme::billing::Invoice`, `:user::holon::lab::trading::Post` |
| `:user::<user-app-tree>::*` | User's own program code — your project, your sub-structure. No collisions with community crates because your tree claims a unique root. | `:user::my-app::main`, `:user::alice::scratch::test` |

**Mechanism vs convention.** The substrate mechanism is simple:
everything registered via the stdlib-tier path
(`register_stdlib_defines` / `register_stdlib_types` / macro
`register_stdlib`) bypasses the reserved-prefix gate. Baked
stdlib and installed dep sources both flow through that path
by construction (`src/freeze.rs:362-368` + `src/stdlib.rs`'s
`stdlib_forms()`). Any installed dep *can* register under
`:wat::*`; convention is what says they *should* only do so
when they're workspace members.

**Claim-by-convention, not runtime-enforced.** Workspace members
claim `:wat::<crate>::*`. Third-party crates claim
`:user::<org>::<name>::*`. The runtime doesn't police taste;
it polices collisions. Two crates claiming the same path fail
loud at startup via duplicate-define detection.

**Why workspace membership is the bless signal.** Being in
`wat-rs/crates/<crate>/` means: same repository, same author or
co-authors, same release cadence as wat-rs itself, same review
discipline. Workspace members ship in lock-step with the
substrate they extend. A third-party crate — added to a
consumer's `Cargo.toml` from crates.io or an external git source
— doesn't share these guarantees, so it stays at `:user::*`.
Anyone can fork wat-rs and add `crates/wat-foo/`, but that's
their workspace, not this one.

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
lets `wat::main!` / `wat::test!` find them by convention.

**Reference:** `crates/wat-lru/` is the first external wat
crate shipped. Its shape is the walkable template:
`wat_sources()` returns its baked `.wat` files;
`register()` forwards to `#[wat_dispatch]`-generated code that
wires the Rust shim. `examples/with-lru/` shows the consumer
shape — `wat::main! { deps: [wat_lru] }` and a `wat/main.wat`.

### App-owned top-level roots (arc 018)

`:user::*` is the recommended root for community crates and
generic user code. A project with **durable identity** — its
own repo, its own Cargo crate, its own namespace authority —
may claim a top-level prefix outside `:user::*` if the tradeoffs
favor it. Examples:

- `:trading::*` — holon-lab-trading.
- `:ddos::*` — the kernel-level DDoS detector (future).
- `:mtg::*` — the MTG experiment (future).

**The substrate permits this.** Only `:wat::*` sub-prefixes and
`:rust::*` are in the reserved-prefix list (see
`src/resolve.rs::RESERVED_PREFIXES`). Every other top-level
prefix is user territory.

**When to claim a top-level root vs `:user::<app>::*`:**

- **Top-level** when the keyword path will appear at every call
  site inside the project and a segment saved on every path
  matters. A 10,000-LoC project with thousands of keyword paths
  saves one segment per path = honest ergonomic improvement.
- **`:user::<app>::*`** for scratch work, proofs-of-concept, or
  projects that might collide with someone else's top-level
  claim. The `:user::` prefix is the safe sandbox.

**Collision handling** is the same as under `:user::*` — Cargo
gives global crate-name uniqueness at the build boundary;
startup registration fails loud on duplicate defines. A project
that ships its Cargo crate as `holon-lab-trading` and claims
`:trading::*` cannot collide with anyone else's `:trading::*`
because the crate naming prevents it.

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
    └── test.rs         # optional — one-line wat::test!
```

Reference: `crates/wat-lru/`. Ships both sides of the contract
(`wat_sources()` returns two baked `.wat` files via
`include_str!`, `register()` forwards to `#[wat_dispatch]`-
generated code), its own `wat-tests/` with deftests, and
`tests/test.rs` invoking `wat::test! { path: "wat-tests",
deps: [wat_lru] }` — self-testing its published surface.

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
    └── test.rs         # optional — one-line: wat::test! { path: "wat-tests", deps: [...] }
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
`wat::test!` compose them identically.

### Viewing per-wat-test output under `cargo test`

`wat::test!` expands to a `#[test] fn wat_suite()` that
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

### Consumer layout (arc 018)

The opinionated default for consumer crates:

```
my-app/
├── Cargo.toml
├── src/
│   └── main.rs        → wat::main! { deps: [...] }
├── tests/
│   └── test.rs        → wat::test! { deps: [...] }
├── wat/
│   ├── main.wat       → entry (config + :user::main)
│   └── **/*.wat       → library tree, loaded recursively
└── wat-tests/
    └── **/*.wat       → test files
```

Two Rust files. Every other wat file lives under `wat/` or
`wat-tests/`. The macros pick everything else up via the defaults.

**Filenames**. `tests/test.rs` is a recommendation (symmetric with
`wat::test!`), not Cargo-enforced — any `.rs` file under `tests/`
is an integration test binary. Use whatever name; the
recommendation exists so consumer crates feel the same to readers.

**Overrides**. Pass `source:` / `path:` / `loader:` explicitly to
any macro to opt out of that default. Full escape hatch is the
manual `wat::Harness::from_source_with_deps_and_loader` path.

References: `wat-rs/examples/with-lru/` and
`wat-rs/examples/with-loader/` both follow the minimal layout
post-arc-018.

### Binary vs library — files that commit config (arc 017)

Every `.wat` file is either an **entry** or a **library**:

- **Entry**: commits startup config via top-level
  `(:wat::config::set-*!)` forms. Hosts `:user::main` (for
  binaries) or `test-*` defines (for test files).
- **Library**: no top-level config setters. Can be `(:wat::load-file!
  "...")`'d from entries (or from other libraries, recursively —
  the entry's frozen world collects every loaded-file's defines at
  arbitrary depth). Attempting to `load!` a file that contains
  setters fails loud at startup ("setters belong in the entry file
  only").

`wat::main!`'s `source:` argument is always an entry. `wat::test!`
under a test dir silently skips library files at freeze time —
they're discovered and read, but not treated as test entries. This
is how test suites share helpers: the entry test files `(load!)`
their sibling library files, and the sandbox-free freeze-time
reads populate the test file's frozen world.

USER-GUIDE § "Multi-file wat programs" has a worked example.

### Install-once discipline (arc 015 slice 3a)

Both halves of the external-crate contract install
process-globally via OnceLock — first caller wins. A test
binary is one install; a consumer `main.rs` is one install.
Tests needing different dep sets live in separate `tests/*.rs`
files (Cargo compiles each to its own test binary).

The win: once installed, every subsequent freeze (main, test,
sandbox via `run-sandboxed-ast`, fork child via
`run-hermetic-ast`) transparently sees the dep surface.
`deftest` bodies can use `:wat::lru::LocalCache::*`
because the inner sandbox's `startup_from_forms` pulls
installed deps from the global state.

### Sandbox Config inheritance (arc 031)

Entry files commit capacity-mode (and any optional
`set-dim-router!` / sigma-fn overrides — arcs 024 + 037) via
top-level `(:wat::config::set-*!)` setters. A sandbox created
inside an entry (via `:wat::kernel::run-sandboxed-ast`,
`run-sandboxed-hermetic-ast`, or `fork-program-ast`) inherits
those committed values by default. Inner setters still override
when present; absence means "take the caller's value."

Pairs with arc 027's loader inheritance — same scope-inheritance
move applied to a different environment field. A sandbox is a
proper child-of-caller scope, not a fresh reset. The single
declaration site for config is the entry file's preamble:

```scheme
;; entry preamble — needed only when overriding defaults
(:wat::test::make-deftest :deftest
  ((:wat::load-file! "my/helpers.wat")))

(:deftest :my-test body)   ;; inherits the entry's Config (:error default + active dim-router)
(:deftest :another body)   ;; same
```

Add `(:wat::config::set-capacity-mode! :panic)` (or any other
optional setter) at the top of the file when you want to override
a default; otherwise the deftests inherit the substrate's
opinionated values.

The four `:wat::test::*` macros (`deftest`, `deftest-hermetic`,
`make-deftest`, `make-deftest-hermetic`) take name + prelude +
body (or name + default-prelude for the factories) — no
mode/dims arguments. Re-declaring the config per-test would
be ceremony without information.

## Name formats

- `snake-case` for functions: `make-bounded-queue`, `for-each`,
  `spawn-producer`.
- `PascalCase` for types: `Bundle`, `HashMap`, `Console`, `Stream`.
- `PascalCase` for enum variants (arc 048): `:Buy`, `:Sell`,
  `:Up`, `:Down`, `:Valley`, `:Peak`, `:Transition`. Embodies
  Rust convention; matches built-in `Some`/`None`/`Ok`/`Err`.
- `?` suffix for predicates: `presence?`, `empty?`, `capital-available?`.
- `!` suffix for side-effect forms where the caller should notice:
  `use!`. Most side-effect primitives (`send`, `recv`, `println`)
  don't carry `!` because their purpose is visible in the name;
  `!` is for forms that would otherwise read as pure.
- `::` segments the path; `-` segments words within a segment.
- Qualifiers AFTER the base name:
  `:wat::core::i64::+` (ops on `i64`), not `:wat::core::+::i64`.

## Constructor / factory naming (arc 077)

Three kinds of "make me one of these" exist; each gets its own
suffix. **Same path, different meaning** — readers can predict
the arity and side-effects from the suffix alone.

| Suffix | What it does | Where it comes from | Arity contract |
|---|---|---|---|
| `Type/new` | **Field constructor.** Pure construction; no setup, no defaults. | Auto-derived from `(:wat::core::struct ...)` declarations. Substrate generates one per struct. | one parameter per field, in declaration order |
| `Type/make` | **Factory with internal setup.** Takes high-level args (filter, capacity), allocates internal state, reads ambient context, calls `Type/new` to assemble the struct. User-defined. | wat or Rust impl. | high-level args; never matches the field arity |
| `Type/spawn` | **Factory + spawns workers.** Everything `/make` does plus spawning thread(s); returns `(handles, ProgramHandle)` tuples or a struct holding them. | wat. | high-level args; side-effecting |

### Examples

```scheme
;; Type/new — auto-derived field constructor (3 args = 3 fields)
(:wat::holon::lru::HologramCache/new hologram lru)

;; Type/make — factory; reads ambient `dim-count`; allocates inner storage
(:wat::holon::lru::HologramCache/make filter cap)

;; Type/spawn — factory that ALSO spawns a driver thread
(:wat::lru::CacheService/spawn capacity count reporter metrics-cadence)
   ; -> CacheService::Spawn<K,V>
(:wat::console::spawn stdout stderr 4)
   ; -> Console::Spawn
(:wat::holon::lru::HologramCacheService/spawn count cap reporter metrics-cadence)
   ; -> HologramCacheService::Spawn
```

### When to pick which

- **Adding a new struct?** The `/new` is free (auto-derived). You don't write it.
- **Constructing it requires more than one of each field?** Define `Type/make` that returns `Type/new` with the assembled fields.
- **Constructing it spawns a worker?** Define `Type/spawn`.

### Rust-side primitives (`::new`)

`#[wat_dispatch]`-generated methods on Rust types use Rust's
`Type::new` convention (`:wat::lru::LocalCache::new cap`,
`:wat::kernel::HandlePool::new tag handles`). The `::` separator
in the path is what flags it as Rust-side. The `/new` vs `/make`
vs `/spawn` distinction is wat-side only.

## Service contract — Reporter + MetricsCadence (arc 078)

A *service* is a queue-addressed program with a request enum, a
driver loop, and per-request state. The substrate ships two:
`:wat::lru::CacheService<K,V>` and
`:wat::holon::lru::HologramCacheService`. Both follow the same
contract; future stdlib services do too.

The contract is a one-page recipe. Every service declares **eleven
elements** (the first six earn their slot from the moment a service
exists; the last five are the standard verbs):

1. **A typed Request enum.** What clients can ask. Variants ARE the
   RPC methods.
2. **A typed Report enum.** What the service emits outbound.
   Producer-defined; consumer dispatches via match. Slice-1 ships
   only `(Metrics stats)`; future variants (Error, Evicted,
   Lifecycle) extend additively. Same grow-by-arms pattern as the
   archive's `TreasuryRequest`.
3. **A `Reporter` typealias.** `:fn(Type::Report) -> :()`. The
   user's match-dispatching consumer.
4. **A `MetricsCadence<G>` struct.** `{gate :G, tick :fn(G,Stats) ->
   :(G,bool)}`. Stateful rate gate. The user picks `G`; the loop
   threads it through, rebuilding the struct each iteration with
   the advanced gate.
5. **A `Stats` struct.** Counter type emitted via `Report::Metrics`.
   Counter set is service-defined (e.g., `lookups`, `hits`,
   `misses`, `puts`, `cache-size` for caches).
6. **`Type/null-reporter` + `Type/null-metrics-cadence`.** The
   explicit no-reporting choice. Caller passes BOTH; opting out is
   a deliberate choice, not a default.
7. **`Type/spawn ... reporter metrics-cadence`.** The constructor.
   Order encodes the contract: factory args first, then "here's
   your reporter, then here's how often you use it for metrics."
   Both are non-negotiable.
8. **`Type/handle req state -> state'`.** Per-variant request
   dispatcher. Pure values-up.
9. **`Type/tick-window state reporter metrics-cadence -> Step<G>`.**
   Gate-fire logic; ALWAYS advances the cadence; conditionally
   emits + resets stats. Named for what it always does, not the
   conditional branch.
10. **`Type/loop`.** Driver. Threads State + Reporter +
    MetricsCadence; selects + dispatches + ticks the window.
11. **`Type/run`.** Worker entry. Wraps the loop with storage
    construction and dropping (per the thread-owned-cache
    discipline).

### The three cadence shapes the user expresses

```scheme
;; Null path — both required to be passed deliberately
(:wat::holon::lru::HologramCacheService/spawn 2 16
  :wat::holon::lru::HologramCacheService/null-reporter
  (:wat::holon::lru::HologramCacheService/null-metrics-cadence))

;; Time-based metrics gate — wall-clock tick-gate, gate = Instant
(:wat::holon::lru::HologramCacheService/spawn 2 16
  :my::reporter
  (:wat::holon::lru::HologramCacheService::MetricsCadence/new
    (:wat::time::now)
    (:wat::core::lambda
      ((g :wat::time::Instant) (_s :Stats) -> :(wat::time::Instant,bool))
      (:trading::log::tick-gate g 5000))))

;; Counter-based — every 100 lookups, gate = i64
(:wat::holon::lru::HologramCacheService/spawn 2 16
  :my::reporter
  (:wat::holon::lru::HologramCacheService::MetricsCadence/new
    0
    (:wat::core::lambda ((n :i64) (_s :Stats) -> :(i64,bool))
      (:wat::core::if (:wat::core::i64::>= n 99) -> :(i64,bool)
        (:wat::core::tuple 0 true)
        (:wat::core::tuple (:wat::core::i64::+ n 1) false)))))
```

The user's `:my::reporter` is `:fn(Report) -> :()` — a closure that
captures whatever stateful sink they want (sqlite handle, CloudWatch
tx, stdout writer).

### When a service should adopt this shape

- **Adopt** when a service owns a queue + state. The contract pays
  for itself the first time you need to wire telemetry without
  reaching for `Mutex` or threading a separate channel.
- **Skip** for trivial pure-fn services that don't earn the
  ceremony.
- **Console is the exception.** Console writes to stdout/stderr
  through tagged messages — that IS its report layer. There's no
  inner Reporter to inject; the channel writes ARE the reports.
  Any future "logging service" pattern resolves the same way:
  whatever IS the sink doesn't need a sink-injection point.

### Composing services (the Reporter-closes-over-handles case)

When one service's Reporter closes over ANOTHER service's handles
(common case: cache reporter writes to rundb), you have two
drivers to shut down in order. The lockstep from
`SERVICE-PROGRAMS.md` Step 3 still applies, but TWICE — once per
driver. **Do not express both drivers' lockstep in one inline
`let*`.** The resulting three-deep nest collapses outer/inner for
both drivers into one scope; trying to join either driver from
that scope deadlocks (the senders are still bound).

The fix is **function decomposition.** Each scope-level becomes a
small named function with the canonical two-level `let*`. See
`SERVICE-PROGRAMS.md` Step 9 for the worked pattern + anti-pattern.
The real-world citation lives at
`holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/`.

### Per-service, not shared

Each service ships its own `Type::MetricsCadence<G>`. We keep
per-service rather than lifting to a shared
`:wat::std::service::MetricsCadence<G,Stats>` because the cadence's
`tick` knows the service's specific Stats — sharing would force a
two-parameter generic with no clear payoff. Revisit when a third
service surfaces and the duplication is concretely painful.

## Type alias for nested-generic returns (arc 077)

If a function's return type contains **three or more** `<` characters, name it. Nested generics make signatures unreadable; an alias near the type definition restores grep-ability.

### Examples

```scheme
;; Before — 3 angle brackets at every Service factory site
(:wat::lru::CacheService/spawn<K,V>
  (capacity :i64) (count :i64)
  -> :(wat::kernel::HandlePool<wat::lru::CacheService::ReqTx<K,V>>,wat::kernel::ProgramHandle<()>))

;; After — alias near the protocol typealiases
(:wat::core::typealias :wat::lru::CacheService::Spawn<K,V>
  :(wat::kernel::HandlePool<wat::lru::CacheService::ReqTx<K,V>>,wat::kernel::ProgramHandle<()>))

(:wat::lru::CacheService/spawn<K,V>
  (capacity :i64) (count :i64)
  -> :wat::lru::CacheService::Spawn<K,V>)
```

### Aliases that ship in the substrate

| Alias | Expands to | Where |
|---|---|---|
| `:wat::kernel::QueuePair<T>` | `:(QueueSender<T>,QueueReceiver<T>)` | `wat/kernel/queue.wat` |
| `:wat::kernel::Sent` | `:Option<()>` | `wat/kernel/queue.wat` |
| `:wat::kernel::Chosen<T>` | `:(i64,Option<T>)` | `wat/kernel/queue.wat` |
| `:wat::std::stream::Stream<T>` | `:(Receiver<T>,ProgramHandle<()>)` | `wat/std/stream.wat` |
| `:wat::std::stream::ChunkStep<T>` | `:(Vec<T>,Vec<Vec<T>>)` | `wat/std/stream.wat` |
| `:wat::std::stream::KeyedChunkStep<K,T>` | `:((Option<K>,Vec<T>),Vec<Vec<T>>)` | `wat/std/stream.wat` |
| `:wat::console::Spawn` | factory return shape | `wat/std/service/Console.wat` |
| `:wat::lru::CacheService::Spawn<K,V>` | factory return shape | `crates/wat-lru/wat/lru/CacheService.wat` |
| `:wat::lru::CacheService::Step<K,V,G>` | one loop-step output | `crates/wat-lru/wat/lru/CacheService.wat` |
| `:wat::lru::CacheService::ReqPair<K,V>` | `:(ReqTx<K,V>,ReqRx<K,V>)` | `crates/wat-lru/wat/lru/CacheService.wat` |
| `:wat::holon::lru::HologramCacheService::Spawn` | factory return shape | `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat` |
| `:wat::holon::lru::HologramCacheService::Step<G>` | one loop-step output | `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat` |

The same rule applies in user crates: pass the angle-bracket
density check at every type signature, and add aliases adjacent
to the protocol typealiases when one signature crosses three.

### Consumers alias the substrate's generic at their concrete instantiation

The substrate ships generic aliases — `Service::Spawn<E>`,
`Console::Dispatcher<E>`, `CacheService::Spawn<K,V>` — so the
SAME factory can serve any consumer's domain type. For each
consumer, those generics resolve to ONE concrete instantiation
(the lab's `E = :trading::log::LogEntry`; an MTG engine's
`E = :mtg::log::Event`; a thinker's
`E = :thought::log::Inscription`). Aliasing that concrete
instantiation once at the consumer's namespace collapses every
downstream signature to a single readable name.

Two layers of alias compose: substrate-generic + consumer-concrete.

```scheme
;; Substrate ships the generic — reusable across consumers.
(:wat::core::typealias :wat::telemetry::Spawn<E>
  :(wat::telemetry::ReqTxPool<E>,wat::kernel::ProgramHandle<()>))

;; Consumer aliases the concrete instantiation at their namespace —
;; readable everywhere downstream.
(:wat::core::typealias :trading::telemetry::Spawn
  :wat::telemetry::Spawn<trading::log::LogEntry>)

;; Every lab signature reads `:trading::telemetry::Spawn` instead
;; of `:wat::telemetry::Spawn<trading::log::LogEntry>`.
(:wat::core::define
  (:trading::telemetry::Sqlite/spawn<G>
    (path :String) (count :i64)
    (cadence :wat::telemetry::MetricsCadence<G>)
    -> :trading::telemetry::Spawn)
  ...)
```

The rule:

- **Substrate ships generics for reusability across consumers.**
  `<E>`, `<K,V>`, `<G>` parameters that different domains
  instantiate differently.
- **Consumers alias the concrete instantiation at their own namespace.**
  One alias per concept the app uses; substrate's `<E>` collapses
  to a single readable name at the consumer site.

References:

- `holon-lab-trading/wat/io/telemetry/Sqlite.wat` —
  `:trading::telemetry::Spawn = Service::Spawn<trading::log::LogEntry>`
  is the canonical example; the lab's only telemetry consumer
  aliases its concrete shape once.
- `wat-rs/wat-tests/service-template.wat:72-73` — the canonical
  service template's `:svc::Spawn` aliases the (pool, driver) tuple
  AT the consumer's namespace; SERVICE-PROGRAMS.md § "The complete
  pattern" explicitly says to "rename the `:svc::*` namespace to
  your domain" when forking.

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

## spawn vs fork — containment naming convention (arc 104)

Two words for two transports:

- `spawn` = **thread**. Runs in the same OS process; shares
  address space, fd table, atexit handlers. Cheap (~µs).
- `fork` = **process**. Real `fork(2)`; separate address space
  (COW), separate fd table, separate `_exit`. Heavier (~ms);
  honest containment.

The matrix that follows composes left-to-right:

| Action | Source entry | AST entry |
|---|---|---|
| Thread (spawn) | `:wat::kernel::spawn-program` | `:wat::kernel::spawn-program-ast` |
| Process (fork) | `:wat::kernel::fork-program` | `:wat::kernel::fork-program-ast` |

A reader walking in cold can pick the right primitive without
reaching for docs:
- `spawn-program` → "thread-spawn a program from source"
- `fork-program-ast` → "process-fork a program from AST"

Validation: POSIX uses `pthread_create` (thread) and `fork(2)`
(process); wat-rs uses `spawn` for thread since arc 003's
`:wat::kernel::spawn`. The convention is internally consistent.

Rust's `std::thread::spawn` and `std::process::Command::spawn` both
use "spawn" — one tradition that doesn't distinguish — but wat-rs's
chosen convention is sharper.

`:wat::kernel::spawn` (the function-on-thread primitive from arc
003) is grandfathered: it predates the convention; renaming would
break embedders. The matrix above governs new primitives.

## Sources of truth

- **Canonical primitive list**:
  [`arc/2026/04/005-stdlib-naming-audit/INVENTORY.md`](./arc/2026/04/005-stdlib-naming-audit/INVENTORY.md)
- **Language specification**: `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION.md`
- **As-shipped contracts**: `arc/*/INSCRIPTION.md` — each arc's
  shipped surface. If INSCRIPTION and DESIGN disagree, INSCRIPTION
  wins.
