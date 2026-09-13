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

## Declared restriction — `{:restricted-to [...]}` metadata-map / `#[restricted_to(...)]` (arc 198 → Stone 241.14)

A finer-grained access control complementing namespace privilege:
any binding can declare an allowed-caller-prefix whitelist at its
definition site via a `{:restricted-to [...]}` metadata-map clause.
The walker rejects call sites whose enclosing definition does not
match. Storage is `binding_metadata` on `CheckEnv` (sole restriction
store post-Stone 241.14; mirrored from `SymbolTable`); one walker
(`walk_for_restricted_call`) enforces, regardless of which declaration
surface populated the entry.

**Two surfaces, same mechanism:**

| Surface | Site | Form |
|---------|------|------|
| Wat | fn binding | `(:wat::core::defn :name {:restricted-to [:p1:: :p2::]} sig body)` |
| Wat | non-fn binding | `(:wat::core::def :name {:restricted-to [:p1::]} value)` |
| Rust | substrate primitive | `#[restricted_to("wat-name", "prefix1", "prefix2")]` on the fn |

**Prefix matching** (uniform across surfaces):
- Trailing `::` (e.g., `:wat::kernel::`) → namespace prefix match
- No trailing `::` (e.g., `:wat::kernel::specific-fn`) → exact FQDN match
- Empty whitelist → no callers allowed

**When substrate authors reach for `#[restricted_to(...)]`:**
- A Rust-side substrate primitive should only be callable from a
  bounded namespace set (e.g., `Thread/join-result` is only safe
  for `:wat::*` callers; user code uses `Thread/drain-and-join` or
  the bracket combinator)
- The restriction is a property of the SYMBOL, not of caller hygiene —
  declare it once at the fn site; the substrate enforces everywhere

**When wat authors reach for `{:restricted-to [...]}`:**
- A module's internal helper shouldn't be called from outside the
  module namespace
- A test-fixture helper should only be reachable from `:my::tests::*`
- The binding's access policy belongs at the binding site, not in
  README convention or post-hoc walker rules

Rust-side proc-macro: `crates/wat-macros/src/lib.rs` (arc 198 slice 2).
Walker: `walk_for_restricted_call` in `src/check.rs`.
Arc INSCRIPTION: `docs/arc/2026/05/198-defn-restricted/INSCRIPTION.md`.

**History:** Arc 198 originally used `:wat::core::def-restricted` (substrate
primitive) and `:wat::core::defn-restricted` (defmacro sugar over
`def-restricted` + `fn`). Both forms retired by Stone 241.14 — use
`def`/`defn` with `{:restricted-to [...]}` metadata-map instead.

## Namespaces

| Prefix | What lives here |
|---|---|
| `:wat::core::*` | Evaluator primitives — forms (`define`, `lambda`, `let`, `if`, `match`), primitive types (`i64`, `bool`, `String`, ...), macros, eval-family, primitive-type operations (`i64::+`, `bool::and`), core collections (`vec`, `list`, `cons`, `conj`, `HashMap`, `HashSet`, `get`, `contains?`, `assoc`). Cannot be written in wat. |
| `:wat::config::*` | Runtime-committed configuration: `capacity-mode` (`:error` / `:panic` — arc 045 renamed `:abort` → `:panic`), `dim-router` function (multi-tier dim selection per AST surface — arc 037), `presence-sigma` / `coincident-sigma` functions of `d` (arc 024), `global-seed`. Compat shim accessors `dims` / `noise-floor` return `DEFAULT_TIERS[0]` defaults. Read-only after config pass. |
| `:wat::holon::*` | Holon algebra — the `HolonAST` type, the six AST-producing primitives (`Atom`, `Bind`, `Bundle`, `Blend`, `Permute`, `Thermometer`), the four measurements (`cosine`, `dot`, `presence?`, `coincident?`), the `eval-coincident?` family (arc 026), the `CapacityExceeded` error type, and typealiases `Holons` / `BundleResult` (arcs 032, 033). One namespace for the whole holon surface. |
| `:wat::kernel::*` | CSP primitives — `spawn`, `send`, `recv`, `select`, `drop`, `join`, `make-bounded-channel`, `HandlePool`, signal handlers. |
| `:wat::io::*` | Stdio primitives — `stdin`, `stdout`, `stderr`, `println`. |
| `:wat::std::*` | Stdlib built on primitives. Each entry should be expressible (in principle) in wat itself, even if shipped as Rust for performance. `stream::*`, `service::Console`, `hermetic`, `test::*`. (LocalCache + CacheService moved to `:wat::lru::*` via arcs 013 + 036.) |
| `:wat::lru::*` | LRU cache surface (external workspace member `crates/wat-lru/`, namespace promoted to `:wat::*` via arc 036). `(:wat::lru::LocalCache :- [K V])`, `(:wat::lru::CacheService :- [K V])`. |
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

### Type-placeholders (arc 215 stone 1 + stone 2)

The `:wat::core::*` namespace includes one type-placeholder that is
NOT a value type:

| Placeholder | Meaning | Appears in |
|---|---|---|
| `:wat::type::Infer` | "Infer this type from the values" | Type-arg slots of parametric constructors (`Vector`, `HashMap`, `HashSet`) |

`:wat::type::Infer` is analogous to Rust's `_` in type position and
Haskell's `_` wildcard. It signals to check.rs: "allocate a fresh HM
unification variable here; resolve it from the actual values."

All three collection literals use `Infer` for inferred type-arg positions
(arc 215 stone 2 extended this to `[...]` and full-inference `{...}`):

| Literal | Desugared form | Layers |
|---|---|---|
| `[...]` | `(:wat::core::Vector :wat::type::Infer ...)` at check time | T inferred from first element |
| `{...}` | `(:wat::core::HashMap :wat::type::Infer :wat::type::Infer ...)` at parse time | K and V each inferred from first key/value |
| `#{...}` | `(:wat::core::HashSet :wat::type::Infer ...)` at parse time | T inferred from first element |

**Two-layer enforcement model** (arc 215 stone 2):

1. **Literal coherence** (check time): within one literal, all keys must unify to K;
   all values must unify to V; all elements must unify to T. Violation → `TypeMismatch`
   diagnostic naming the offending position.

2. **Function-signature unification** (call site): when a literal is passed to a
   function expecting `(:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])`, the inferred K/V must unify with the
   declared parameter type. This is where keyword-key conventions are enforced for
   APIs that require them — not at the language parse layer.

The checker concretizes the fresh variable from the first element; subsequent
elements must unify against it. Empty literal → fresh type variable (resolves at
first concrete use).

`:wat::type::Infer` is NOT a valid user-facing type (you cannot
declare a struct field or function parameter with this type). It is
a parse-and-check-time placeholder only.

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

### Intentionally-invalid fixtures — declared bad by EXTENSION: `.wat.bad` (arc 170, 2026-07-09)

A negative test fixture that is **supposed to be rejected** — a lexer/parser
negative (a supplementary-plane char literal, whitespace inside a keyword's
generic head), a type-error negative, a freeze-wall negative (an illegal
`:user::main`) — MUST use the **`.wat.bad`** extension, never `.wat`.

**Why it is the extension, not a `_bad` infix.** A file named `foo_bad.wat`
still ends in `.wat`, so it is indistinguishable from valid wat to any tool
that globs the corpus: the wat-scripts load gate (`Path::extension() == "wat"`),
a `wat-fix` codemod (`git ls-files '*.wat'`), a lint sweep. A single
intentionally-malformed fixture then **poisons the whole tool** — e.g.
`strip-useless-mains.wat`'s `read-string` *panics* the instant it reaches a
lexer-negative, because that file is *designed* not to parse. Declaring
bad-ness in the **extension** makes the wrong thing structurally excluded: a
`.wat.bad` file's `Path::extension()` is `"bad"`, so every `*.wat` glob and
every `== "wat"` check skips it *by construction*. The malformed fixture can no
longer masquerade as valid wat — constraint engineering, not vigilance.

**How tests load them.** By **explicit path** —
`startup_from_file("tests/foo.wat.bad")` — never `startup_beside` (which
derives `.wat`). `std::fs::read_to_string` is extension-agnostic, so loading is
unaffected; the test asserts the parse/type/freeze `Err`. (Negative fixtures
have never used `startup_beside`; they are all explicit-path already.)

**The rule.** A new fixture that must be rejected gets `.wat.bad`. A `.wat` file
is a promise that it freezes; if it does not, it is either a `.wat.bad` or a
bug — never a `.wat` that "happens to fail." (Migration 2026-07-09: 234
`*_bad.wat` → `*.wat.bad` + one non-`_bad`-named parse-invalid straggler; the
`_bad.wat` infix convention is retired.)

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
manual `wat::Guest::from_source_with_deps_and_loader` path.

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

- `snake-case` for functions: `make-bounded-channel`, `for-each`,
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
(:wat::lru::spawn capacity count reporter metrics-cadence)
   ; -> (CacheService::Spawn :- [K V])
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
`:wat::lru::*` (spawn via `:wat::lru::spawn`) and
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
3. **A `Reporter` typealias.** `[Type::Report :-> :wat::core::nil]`. The
   user's match-dispatching consumer.
4. **A `MetricsCadence :- [G]` struct.** `{gate :G, tick [G Stats :->
   (:wat::core::Tuple :- [G :wat::core::bool])]}`. Stateful rate gate. The user picks `G`; the loop
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
9. **`Type/tick-window state reporter metrics-cadence -> Step :- [G]`.**
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
    (:wat::core::fn
      ((g :wat::time::Instant) (_s :Stats) -> :(wat::time::Instant,bool))
      (:trading::log::tick-gate g 5000))))

;; Counter-based — every 100 lookups, gate = i64
(:wat::holon::lru::HologramCacheService/spawn 2 16
  :my::reporter
  (:wat::holon::lru::HologramCacheService::MetricsCadence/new
    0
    (:wat::core::fn ((n :i64) (_s :Stats) -> :(i64,bool))
      (:wat::core::if (:wat::core::i64::>= n 99) -> :(i64,bool)
        (:wat::core::Tuple 0 true)
        (:wat::core::Tuple (:wat::core::i64::+ n 1) false)))))
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

### Batch convention — substrate-shipped services (arc 119)

**Every wat-rs-shipped service exposes only batch-oriented
`get` / `put` interfaces. Console is the single exception.**

A client transmits one unit of context as a batch-of-one. The
substrate's surface is the bound discipline — users implementing
their own services pick whatever shape fits.

Shape:
- **Get** — `(get probes (:wat::core::Vector :- [K])) -> (:wat::core::Vector :- [(:wat::core::Option :- [V])])`. Data-bearing
  reply (Pattern B back-edge). Each probe maps to its slot in
  the result vec; missing keys come back as `:None`.
- **Put** — `(put entries (:wat::core::Vector :- [(Entry :- [K V])])) -> :wat::core::nil`.
  Nil-ack release (Pattern A back-edge; arc 153 renamed
  `:wat::core::unit` → `:wat::core::nil`). Caller blocks until
  the batch is durable in the service's state.

Both verbs are lock-step. Caller cannot continue until the
service confirms (Mini-TCP discipline per `ZERO-MUTEX.md`).
The cache service IS a mutex implementation: shared mutable
state lives in one program; the io::select loop serializes
batches sequentially; lock granularity = batch granularity.

Why batch-only:
- A batch-of-one is `(get [probe])` — costs one extra
  `(:wat::core::Vector :- [...])` allocation against the protocol
  surface, gains one uniform shape across every substrate service.
- Single-item interfaces lie about the lock model — they imply
  per-item acquisition when the loop already serializes.
- Users who actually have N items stay efficient by default;
  no "pipeline these calls yourself" wrapper layer needed.
- The protocol body matches what wire-level RPC services have
  always done: the request IS the unit of work.

Why Console is exempt: Console IS the sink. The driver writes
each tag+msg directly to stdout/stderr; there's no batchable
work layer to amortize. Bundling writes would force partial-
flush semantics the underlying file descriptor doesn't carry.

Substrate services obeying this convention:
- `:wat::telemetry::*` — `Request :- [E]` is `(:wat::core::Vector :- [E])` (already
  batch since arc 029)
- `:wat::telemetry::sqlite::*` — rides `(:wat::telemetry::Request :- [E])`
- `:wat::lru::*` — adopts batch via arc 119
- `:wat::holon::lru::*` — adopts batch via arc 119
- (the former Console service was retired in arc 109 § kill-std / arc 170 slice 1f-η; the ambient kernel trio `println` / `eprintln` / `readln` replaces it — no batch exemption needed)

### Composing services (the Reporter-closes-over-handles case)

When one service's Reporter closes over ANOTHER service's handles
(common case: cache reporter writes to rundb), you have two
drivers to shut down in order. The lockstep from
`SERVICE-PROGRAMS.md` Step 3 still applies, but TWICE — once per
driver. **Do not express both drivers' lockstep in one inline
`let`.** The resulting three-deep nest collapses outer/inner for
both drivers into one scope; trying to join either driver from
that scope deadlocks (the senders are still bound).

The fix is **function decomposition.** Each scope-level becomes a
small named function with the canonical two-level `let`. See
`SERVICE-PROGRAMS.md` Step 9 for the worked pattern + anti-pattern.
The real-world citation lives at
`holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/`.

### Per-service, not shared

Each service ships its own `Type::MetricsCadence :- [G]`. We keep
per-service rather than lifting to a shared
`:wat::std::service::MetricsCadence :- [G Stats]` because the cadence's
`tick` knows the service's specific Stats — sharing would force a
two-parameter generic with no clear payoff. Revisit when a third
service surfaces and the duplication is concretely painful.

## Type alias for nested-generic returns (arc 077)

If a function's return type nests **three or more** `:- [...]` parameterizations, name it. Deeply nested generics make signatures unreadable; an alias near the type definition restores grep-ability. (Historical: before arc 109 ③ this was phrased as "three or more `<` characters" — the same density judgement, counted on the current syntax.)

### Examples

```scheme
;; Before — 3 levels of :- nesting at every Service factory site
(:wat::lru::spawn :- [K V]
  [capacity <- :wat::core::i64 count <- :wat::core::i64]
  -> (:wat::kernel::HandlePool :- [(:wat::lru::ReqTx :- [K V])]))

;; After — alias near the protocol typealiases
(:wat::core::typealias :wat::lru::Spawn :- [K V]
  (:wat::kernel::HandlePool :- [(:wat::lru::ReqTx :- [K V])]))

(:wat::lru::spawn :- [K V]
  [capacity <- :wat::core::i64 count <- :wat::core::i64]
  -> (:wat::lru::Spawn :- [K V]))
```

### Aliases that ship in the substrate

| Alias | Expands to | Where |
|---|---|---|
| `:wat::kernel::Channel :- [T]` | `(:wat::core::Tuple :- [(:wat::kernel::Sender :- [T]) (:wat::kernel::Receiver :- [T])])` | `wat/kernel/channel.wat` |
| `:wat::kernel::CommResult :- [T]` | `(:wat::core::Result :- [(:wat::core::Option :- [T]) :wat::kernel::ThreadPanics])` — replaces the retired arc-110-era `:wat::kernel::Sent` | `wat/kernel/channel.wat` |
| `:wat::kernel::Chosen :- [T]` | `(:wat::core::Tuple :- [:wat::core::i64 (:wat::kernel::CommResult :- [T])])` | `wat/kernel/channel.wat` |

⚠ **Unverified — flag for a dedicated content-accuracy pass, out of this stone's scope
(syntax only):** this table's remaining rows (`:wat::stream::Stream`/`ChunkStep`/`KeyedChunkStep`,
`:wat::lru::Spawn`/`Step`/`ReqChannel`, `:wat::holon::lru::HologramCacheService::Spawn`/`Step`) name
files and shapes that no longer check out against the corpus: `wat/stream.wat` doesn't exist (`Stream`
is now a native Rust-backed type — `StreamContainer::Stream` in `src/runtime.rs` — not a tuple alias);
`ChunkStep`/`KeyedChunkStep` don't appear anywhere in the corpus or `src/`; `crates/wat-lru/` (and
`crates/wat-holon-lru/`) do not exist in this tree at all. This is CONTENT drift, not a spelling
question — deliberately NOT rewritten here per the "do not invent the rules" / STOP-3 discipline
(cannot verify a replacement shape for machinery that may have moved or been renamed elsewhere).

The same rule applies in user crates: pass the parameterization-
nesting density check at every type signature, and add aliases
adjacent to the protocol typealiases when one signature crosses
three levels of `:- [...]` nesting.

### Consumers alias the substrate's generic at their concrete instantiation

The substrate ships generic aliases — `Service::Spawn :- [E]`,
`Console::Dispatcher :- [E]`, `CacheService::Spawn :- [K V]` — so the
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
(:wat::core::typealias :wat::telemetry::Spawn :- [E]
  (:wat::telemetry::ReqTxPool :- [E]))

;; Consumer aliases the concrete instantiation at their namespace —
;; readable everywhere downstream.
(:wat::core::typealias :trading::telemetry::Spawn
  (:wat::telemetry::Spawn :- [trading::log::LogEntry]))

;; Every lab signature reads `:trading::telemetry::Spawn` instead
;; of `(:wat::telemetry::Spawn :- [trading::log::LogEntry])`.
(:wat::core::defn :trading::telemetry::Sqlite/spawn :- [G]
  [path <- :wat::core::String count <- :wat::core::i64
   cadence <- (:wat::telemetry::MetricsCadence :- [G])]
  -> :trading::telemetry::Spawn
  ...)
```

⚠ **Unverified — flag for a content-accuracy pass, out of this stone's scope (syntax only):**
`ReqTxPool` does not appear anywhere in the current corpus; this may be further content drift on
top of the spelling, in the same family as the LRU/stream table above. The `:->` and `(Head :-
[Args])` spellings above are independently verified; the specific named types are not.

The rule:

- **Substrate ships generics for reusability across consumers.**
  `E`, `K V`, `G` parameters (bound `:- [...]`) that different
  domains instantiate differently.
- **Consumers alias the concrete instantiation at their own namespace.**
  One alias per concept the app uses; the substrate's generic
  binder collapses to a single readable name at the consumer site.

References:

- `holon-lab-trading/wat/io/telemetry/Sqlite.wat` —
  `:trading::telemetry::Spawn = (Service::Spawn :- [trading::log::LogEntry])`
  is the canonical example; the lab's only telemetry consumer
  aliases its concrete shape once.

(The former second reference cited `wat-tests/service-template.wat`'s
hand-rolled `:svc::Spawn` alias — that file and the hand-rolled pattern
it taught are retired; services are now built via
`:wat::service::defservice`, per SERVICE-PROGRAMS.md § "The runnable
reference".)

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
  006's `first(stream, n) -> (:wat::core::Vector :- [T])` as a terminal would have
  needed a force-drop primitive wat deliberately doesn't ship
  (scope discipline IS shutdown discipline). Reframing as
  `take(stream, n) -> (:wat::stream::Stream :- [T])` — a stage, not a terminal —
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

## Test idioms — EDN-over-stdio vs just-eval (the no-inlined-wat rubric)

A test never inlines wat as a Rust string (`no_inlined_wat` forbids it). The wat lives in a
co-located `.wat` fixture (`startup_beside(file!())`), and the test drives it one of two ways.
Both are lint-clean; the choice follows **what the test is testing**, not taste.

> **The organizing question: does the process boundary participate in what's under test?**
> The boundary = a real process spawn + its stdio (which ships EDN) + its exit code. If the
> boundary *is* (part of) the subject → **EDN-over-stdio**. If the subject is an in-process
> value/type and the boundary would only add a spawn for nothing → **just-eval**.

### The two idioms

- **just-eval** — `call_beside_value(file!(), ":user::compute")` runs a fixture's named zero-arg
  PLAIN entry fn in-process and returns its typed `Result<Value, RuntimeError>`. The test inspects
  the typed value (or `expect_err`s a raise) directly. No spawn. This is the *white-box* vantage:
  you invoke a named entry and read what it returns.
  - **A `deftest` target goes through `call_beside` instead**, which returns a `DeftestOutcome`
    (`Passed` / `Failed { failure }` / `DidNotRun { error }`) — the test's VERDICT, not its Value.
    The two verbs refuse each other's targets. Arc 278 the vacuous-gate wall: `call_beside` used
    to return `Result<Value, _>` for everything, and gates read `.is_ok()` — which answers *"did
    it evaluate?"*, while a fired `assert-eq` inside a `deftest` is captured into the returned
    `:wat::kernel::RunResult`, not raised. Six gates were certifying nothing. `DeftestOutcome` has
    no `is_ok()`; `RunResult` is an enum (`:Passed` / `:Failed[failure]`), not a struct with an
    ignorable `Option` slot. Gate a deftest with
    `call_beside(file!(), ":user::x").expect_passed("<what this proves>")`.
- **EDN-over-stdio** — `run-hermetic` runs `:user::main` as a real spawned process and returns a
  `RunResult { stdout, stderr, failure }`. `:user::main` is `-> :wat::core::nil` by contract, so a
  program *ships* its result: `(:wat::kernel::println value)` writes the value's EDN to stdout, and
  the test `(:wat::edn::read line)` **decodes it back to the typed value** — a lossless structural
  round-trip (records-are-EDN, arc 300; proven by `tests/comms/wat_arc113_raise_round_trip`). This
  is the *black-box* vantage: you run the program and read its real stdio+exit interface, exactly
  as a live consumer would — the honest integration test.

### HARD constraints — these *force* the choice (can/can't, not preference)

Reach for **EDN-over-stdio** when the subject can only exist as a program's observable behavior:
- **A crash / exit / dying declaration** — a panic, `assertion-failed!`, terminal `eprintln`, a
  service dying. Only a real process has an exit code, and only its stderr carries the reason;
  in-process a panic just aborts the harness, so there is no "non-zero exit + reason" to assert.
- **Stdio effects** — what the program prints, to which fd, in what order.
- **Serialization / IPC fidelity** — the point *is* the encode→ship→decode round-trip surviving the wire.
- **Process isolation / cross-loci** — spawning, forking, services on a process locus, grant-before-dial.

Reach for **just-eval** when the subject can only be observed in-process:
- **A value that cannot cross the wire** — an opaque `:rust::` handle, a live `Db`/socket, a `Peer'`,
  a function value. It is impure/non-EDN by construction (293.W), so a process could not ship it —
  you must inspect it in-process.
- **A compile-time / type property** — does it freeze, does the checker reject it, what is inferred.
  There is no runtime program; inspect the `FrozenWorld` / `StartupError` directly (often just
  `startup_beside`, no call at all). A fixture that **must be rejected** (a parser / type / freeze
  negative) is a `<probe>.wat.bad`, loaded by explicit path (`startup_from_file`, never
  `startup_beside`), asserting the `Err` — see § *Intentionally-invalid fixtures* above.

### SOFT default (only for the genuinely ambiguous case)

A pure fn returning pure data, where either idiom would work: prefer **just-eval**. A pure return is
identical in-process or over the wire, so the spawn buys no fidelity. Escalate to EDN-over-stdio only
when the *claim itself* is "the program does X," not "this value is X."

### Anti-patterns (each is the rubric violated)

- Spawning a process for a pure unit check — a spawn for zero fidelity.
- In-process'ing a crash/exit test — you cannot observe the exit code or the dying reason; you are
  testing something weaker.
- Reaching `call_beside_value` *past* a program's real interface into an internal helper to peek at
  intermediate state — a caller-perspective violation (see below): test at the interface, not behind it.

**One-line decision:** claim is *"the PROGRAM does X"* (crashes / prints / ships / survives the wire)
→ EDN-over-stdio; claim is *"this VALUE or TYPE is X"* → just-eval.

### The `.edn` golden — expected output, co-located (the `no_loose_string_assert` rubric)

The mirror of the fixture rule, for the *assertion* side: when a test checks a full structured
output, the EXPECTED value lives in a co-located golden `<probe>__<label>.edn`, compared with
`wat::assert_edn_eq!(actual_edn, include_str!("<probe>__<label>.edn"))` — which parses BOTH sides as
EDN and compares **structurally** (formatting-insensitive, structure-exact). This is what
`no_loose_string_assert` demands: an exact structural match, not `assert!(s.contains("…"))` (which
passes on reordered fields, malformed maps, and appended garbage).

- **Reach for a golden** when the assertion is a deterministic structured value — a record, an enum,
  a `RunResult`, a vector of rows. Capture the whole value once; compare exactly.
- **Do NOT golden** a value that varies per run (a path / pid / hash / timestamp), or a targeted
  absence over a large output — those are the legitimately-loose cases that earn a per-site
  `// rune:lint(loose-assert) — <reason>` (cf. `dead_child_speaks`'s error-substring assert, whose
  message embeds a variable source location).
- **Regenerate, don't hand-edit** when the shape legitimately changes — the golden is captured from
  the real output, never guessed.

### The `.wat` golden — expected wat SOURCE output, co-located (the raw-text case)

Some tests assert an expected output that *is itself wat source text* — a source-to-source fixer's
emitted program (arc-251/258/269), a reader/writer's rendered form — where the claim is the **exact
text**: `;;`-comments preserved, whitespace and layout intact. That expected wat lives in a co-located
golden `<probe>__<label>.wat`, compared **byte-exact**:
`assert_eq!(actual, include_str!("<probe>__<label>.wat"))`.

It is a distinct case from the two neighbours, and the distinction is load-bearing:

- **vs the `.edn` golden** — `assert_edn_eq!` parses BOTH sides and compares *structurally*
  (formatting-insensitive). That is exactly WRONG when the fixer's job is "emit this text, comments and
  all": EDN normalization would silently discard the `;;`-comment and layout fidelity that IS the
  claim. Structured value, formatting irrelevant → `.edn` (structural). Wat source, exact text is the
  claim → `.wat` (raw, byte-exact).
- **vs the `.wat` *fixture*** — a bare `<probe>.wat` is an *evaluated* fixture (loaded via
  `startup_from_file`/`startup_beside`, frozen, run). A `<probe>__<label>.wat` golden is **never
  evaluated** — it is read as text (`include_str!`) and compared. The `__<label>` marks it a golden
  (parallel to `.edn` goldens); the bare name is the live fixture.
- **NOT `.txt`** — the content is wat (a fixer emits a wat program), so `.wat` is the honest extension;
  `.txt` denies its nature. (An expected value that is genuinely *not* wat and *not* EDN — e.g. a
  `Display` rendering carrying placeholder tokens like `<HolonAST>` — is neither golden kind; it is a
  plain string assert, and if its wat-shaped text trips the lint it earns a per-site
  `// rune:lint(no-inlined-wat) — <reason>`, the reason naming it a render, not an evaluated form.)

Because the lint scans `.rs` string literals, not `.wat` files, a `.wat` golden drives the inline-wat
count to a **true zero** — no rune, the expected text simply lives in its own wat file. Prefer a `.wat`
golden over a rune whenever the expected wat source *can* be extracted; a rune is earned only when the
inline form is *intrinsic* (a parser/reader test whose raw literal is the input; a world parameterized
at runtime that no static file can express).

Together the co-located artifacts are a test's whole world — none inlined, each named off the probe so
nothing names its own context:

- **`<probe>.wat`** — the program / fixture (a *valid* wat program; a `.wat` is a promise it freezes).
- **`<probe>.wat.bad`** — an intentionally-*invalid* fixture, for a "must be rejected" test; loaded by
  explicit path, asserting the `Err` (§ *Intentionally-invalid fixtures*). Bad-ness is in the
  extension, so every `*.wat` glob skips it by construction.
- **`<probe>__<label>.edn`** — the expected-output golden for a *structured value* (`assert_edn_eq!` +
  `include_str!`, structural).
- **`<probe>__<label>.wat`** — the expected-output golden for *wat source* whose exact text is the
  claim (`assert_eq!` + `include_str!`, byte-exact; never evaluated).
- **`<probe>.rs`** — the driver.

For an *incidental* world (a substrate test with no wat-under-test), `startup_bare()` carries no wat at
all — the honest statement of "no fixture." That is the full test-authoring surface: the co-located
artifacts (or `startup_bare`), the two drive idioms (just-eval / EDN-over-stdio), the two golden kinds
(`.edn` structural / `.wat` raw-text), and the two lints that enforce them (`no_inlined_wat`,
`no_loose_string_assert`).

## The `rune:sequi` vocabulary — a CLOSED set of four (2026-08-25)

`sequi` asks whether state follows visibly through the types. State that is reached *around* the
signature — a `thread_local!`, a process global, a link-time registry, a discarded error detail —
is a finding unless it is declared conscious with a `// rune:sequi(<category>) — <reason>` rune
immediately above it.

**The category is not free text. It is one of exactly four**, and the discriminating question is
*what does a reader lose by not seeing this in the signature?*

| category | the state is | what a reader loses | example |
|---|---|---|---|
| `ambient-context` | real DOMAIN state, reached globally or per-thread instead of threaded | the ability to reason about the answer — this state can change what is computed | `EXEC_ARENA`, `ARM_TABLE`, the rust-deps registry |
| `performance-counter` | instrumentation, off by default | nothing about the answer — arming it cannot change a result, only a measurement | the fire census TLS, `ARM_BUILDS` |
| `host-idiom` | a host mechanism carrying NO domain state, whose RESULT is threaded explicitly | nothing — the global is a counter or allocator, and the value it yields does travel through signatures | `fresh_scope()`'s monotonic `AtomicU64` |
| `reclassified-by-caller` | detail deliberately dropped, because the sole caller re-surfaces a coarser form that IS the intended UX | nothing at the boundary — the coarser message is the contract | `ArgSpecError` → `Err(())` at `:ensure :fn` |

**Why this table exists.** For most of arc 278 it did not, and the cost was exactly what you would
predict: `ARM_TABLE` (a thread-owned index holding an armed network and its lease count) was
labelled `host-idiom`, two files from `EXEC_ARENA` — the same `thread_local!` shape, the same
invisibility to the signature, holding the same kind of thing — labelled `ambient-context`. Both
runes were thoughtfully written. Neither was checkable, because there was nothing to check them
against. `sequi` caught the disagreement on a re-cast; nothing else could have.

**`no_unknown_sequi_rune` (tests/lint/) closes the SET, and that is all it can close.** It fails on
a fifth category invented at a call site. It cannot tell you that a rune picked the wrong one of
the four — only this table can, which is why the discriminating question is written down as a
question and not as a list of examples to pattern-match.

## Caller-perspective verification

> **All code is measurable from the caller's perspective. That's
> the interface to confirm.**

Every line of substrate has a caller. Service consumers, service
implementers, framework users, library users — each occupies a
vantage. The caller's vantage IS the interface. Anything else is
mechanism.

Tests verify FROM a caller's vantage. The recommended way to use a
thing is the way the test calls it — the test pattern lifted by
the next reader becomes the next consumer's code. A test at the
wrong vantage teaches the wrong shape.

### What this means for tests

- **Tests in a crate's `wat-tests/` directory verify the crate's
  recommended consumer API.** They look like a consumer's call
  site. If a helper verb / public API exists for the use case,
  the test calls it; if not, the test exposes a gap in the
  surface.
- **Wire-protocol mechanics are generated, not hand-built or
  taught from a template.** `:wat::service::defservice`
  (`wat-rs/wat/service.wat`) generates the Request/Response
  records, Op/Reply enums, and dispatch loop from a `:satisfies`
  / `:durable` / `:ephemeral` / `:init` / `:impls` form; see
  `wat-rs/wat-tests/service-cache-lru.wat` for the worked example
  (per `SERVICE-PROGRAMS.md` § "Audience boundary"). A consumer-crate
  wat-test that hand-builds Request enum constructors and calls
  raw `:wat::kernel::send`/`recv` is testing the wrong layer.
- **Rust unit tests in `src/*.rs` follow the same rule** — their
  callers are other Rust modules, and the tests should verify
  what those callers can observe.

### How to check

Reach for `/vocare` (`.claude/skills/vocare/SKILL.md`) — the spell
that scans tests for vantage. For each test it asks: from whose
vantage does this verify? If the answer is "the implementer's, not
the caller's," the test is at the wrong layer.

### Why this matters

Tests are the substrate's voice to a fresh reader. Someone reading
the cache crate's `wat-tests/` is asking "how do I use this?" If
the test answers with implementation mechanics, the answer is
dishonest at the docs boundary — the reader takes home the wrong
pattern.

Conversely, a test that mirrors the worked examples in USER-GUIDE /
README confirms the recommendation. The discipline being modeled
in the test IS the discipline a consumer should adopt.

This principle surfaced during arc 119 when a substrate protocol
change exposed a divergence between two cache crates' tests — one
tested the consumer surface (helper verbs), the other tested the
wire protocol underneath. Convergence to caller-perspective lands
both at the right vantage.

---

### `src/` module layout — a module is a DIRECTORY (2026-07-26)

**Builder-ruled while `wat-cli` was being folded into core:** *"everything in `src/` should be
namespaced… we can create as many files as we need in the namespace."* The strike had produced a
bare `src/distribution.rs`; that is the shape being ruled against.

**The rule for anything new:**

```
src/<module>/mod.rs        ← the module
src/<module>/<part>.rs     ← as many parts as the module actually wants
```

**Not** `src/<module>.rs`.

#### Why — the namespace is the affordance

A bare top-level `.rs` silently caps a module at one file. Growing it later is a rename plus a
re-path plus a diff that hides the real change inside a move. Starting as a directory costs one
extra path segment and removes that ceiling permanently: when a module grows a second concern, it
grows a second file, and the diff shows only the new concern.

It also makes `ls src/` an architecture diagram rather than a pile. That is the same standard
`intueri` applies to a file tree — *"the file tree should mirror the domain… not `utils`, `helpers`,
`common`, `misc`."*

#### The honest state of the tree — THREE shapes exist today

This convention is **forward-looking**. As of the ruling, `src/` is mixed:

| shape | count | examples |
|---|---|---|
| bare `src/foo.rs` | ~37 | `runtime.rs`, `stdlib.rs`, `edn_shim.rs`, `parser.rs`, `harness.rs` |
| `src/foo/mod.rs` | — | `kernel/`, `comms/`, `process/`, `macros/`, `rete/`, `value/` |
| `src/foo.rs` + `src/foo/` | 4 | `check`, `freeze`, `types`, and (transitionally) `distribution` |

That third shape is legal modern Rust — `src/check.rs` declares `mod error;` which resolves to
`src/check/error.rs`, with no `mod.rs`. It is **not** what this ruling asks for; the ruling is
`mod.rs`-style, so the module's own name appears exactly once in the tree, as a directory.

**Do NOT read this as a mandate to convert the existing ~37.** `runtime.rs` and `check.rs` are
enormous and load-bearing; churning them for layout alone would be a large diff with no behavioural
content, and every line moved is a line whose `git blame` gets harder to read. The rule binds **new
modules** and modules already being reshaped for other reasons. An existing single-file module that
nobody is touching stays as it is until it has a real reason to grow.

#### When a module genuinely is one file

Prefer the directory anyway. The cost is one path segment; the benefit is that the second concern —
which arrives more often than anyone predicts — lands as a new file rather than a restructure.
