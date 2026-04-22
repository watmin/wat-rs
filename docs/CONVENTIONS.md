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

**`stdlib_sources()` + `register()` is the contract.** A wat
crate that ships wat source + a Rust shim MUST expose:

```rust
pub fn stdlib_sources() -> &'static [wat::stdlib::StdlibFile];
pub fn register(&mut wat::rust_deps::RustDepsBuilder);
```

Naming these exactly (not `wat_files()` or similar) preserves
grep-ability across the ecosystem and lets `wat::main!` find
them by convention.

**Reference:** `crates/wat-lru/` is the first external wat
crate shipped. Its shape is the walkable template:
`stdlib_sources()` returns its baked `.wat` files;
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
