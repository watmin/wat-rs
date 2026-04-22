# Arc 013 — External wat crates (LRU as the proof)

**Status:** opened 2026-04-21. Planning phase.
**Motivation:** arc 012 closed. `wat-rs` is the substrate; the next
question is whether third parties can publish wat code that other
users consume as dependencies. Chapter 18's *"wat is the language,
Rust is the substrate"* becomes operational or theoretical here.
Today a wat program is either the wat-rs CLI running an ad-hoc
source file, or a bespoke Rust embedding via `wat::Harness`. There
is no mechanism by which a sibling crate ships wat source +
`#[wat_dispatch]` shims that a consumer's program composes with
its own wat source and the baked runtime stdlib.

LRU is the forcing function. `wat/std/LocalCache.wat` + its
`#[wat_dispatch] impl lru::LruCache<K,V>` shim are currently baked
into wat-rs. They are the simplest load-bearing example of "a
wat-level API wrapping a Rust library." Factoring them out into a
sibling crate — `crates/wat-lru/` — and keeping the wat-rs binary
still runnable on every non-LRU caller is the proof. If the
machinery carries LRU cleanly, it carries any future external
crate.

---

## Non-goals (named explicitly)

- **Crate registry / publishing.** This arc does not ship
  `wat-lru` to crates.io. Local path deps cover the proof. A
  future arc handles the publishing workflow when a real
  published crate surfaces demand.
- **Version resolution / lockfiles specific to wat.** Cargo
  already resolves the dep graph and lockfile. wat doesn't
  introduce a parallel resolver.
- **wat registry semantics.** No `wat-registry` service, no
  central index, no namespace ownership registry. The
  ecosystem's collision-detection lives at (a) Cargo crate
  naming uniqueness and (b) wat startup registration collision.
- **Migrating other bakeables.** `wat/std/Subtract.wat`,
  `Circular.wat`, `stream.wat`, `test.wat`, `hermetic.wat`,
  `service/Console.wat`, `service/Cache.wat` — all stay baked.
  They're substrate, not userland extensions. Only LRU externalizes
  because LRU is the one that wraps a third-party Rust crate.
- **Backward compatibility for `:wat::std::LocalCache`.** The
  retired path does not keep. Callers rename to
  `:user::wat::std::lru::LocalCache` or they break. This is the
  clean-proof choice — keeping the path baked would weaken the
  proof that externalization carries.
- **Any changes to wat-rs's binary CLI scope.** The wat-rs
  binary stays "baseline-only runner" for programs that use
  nothing outside the baked stdlib. Programs with external deps
  are their own binary crates built via `wat::main!`. Whether
  the CLI eventually grows a `--dep` flag or similar is a
  future fog, not this arc.

---

## What this arc ships

Six slices. Each shippable independently; ordered so each slice's
substrate is live before the next leans on it.

### Slice 1 — Workspace layout + `crates/wat-lru/` skeleton

`wat-rs` is already a Cargo workspace root (`wat-macros/` is a
member). Add `crates/wat-lru/` as a second member. Skeleton only
at this slice — empty `Cargo.toml`, empty `src/lib.rs`, empty
`wat/` dir. Proves the workspace structure compiles and tests
together.

### Slice 2 — `wat::Harness` accepts external stdlib sources

The `wat::Harness` API today takes a source string and assumes
the baked stdlib. Extension: accept an optional slice of
additional `StdlibFile` entries that prepend to (or interleave
with) the baked stdlib before the user's source hits macro
expansion / type registration / define registration.

Concrete shape:

```rust
impl Harness {
    pub fn from_source_with_deps(
        src: &str,
        dep_sources: &[&[stdlib::StdlibFile]],
    ) -> Result<Self, HarnessError> { ... }
}
```

`StdlibFile` needs `pub` visibility (currently module-private
or crate-private — verify at slice time). Each element of
`dep_sources` is a slice returned by one dep's `stdlib_sources()`
function. Harness concatenates them with the baked stdlib in
dep order, then freezes the world.

### Slice 3 — `wat::main!` proc-macro

Lives in the existing `wat-macros` crate. Takes named fields:

```rust
wat::main! {
    source: include_str!("program.wat"),
    deps: [wat_lru, wat_reqwest, wat_sqlite],
}
```

Expands to:

```rust
fn main() -> anyhow::Result<()> {
    let harness = wat::Harness::from_source_with_deps(
        include_str!("program.wat"),
        &[
            wat_lru::stdlib_sources(),
            wat_reqwest::stdlib_sources(),
            wat_sqlite::stdlib_sources(),
        ],
    )?;
    harness.run(&[])?;
    Ok(())
}
```

(Exact return type / error-handling shape pins in slice-time
iteration — probably `Result<(), wat::HarnessError>` or similar;
`anyhow` is not a wat-rs dep today.)

Underneath, a plain `wat::compose_and_run(src, &[deps])` function
exists and does the same work. The macro is the sugar; the
function is what the user can fall back to when they need
per-call control.

### Slice 4 — Move LocalCache + shim into wat-lru

Concrete changes:

- `wat/std/LocalCache.wat` → `crates/wat-lru/wat/lru.wat`
  (or similar); defines repath from `:wat::std::LocalCache` /
  `:wat::std::service::Cache` to `:user::wat::std::lru::LocalCache`
  / `:user::wat::std::lru::CacheService`.
- The `#[wat_dispatch] impl lru::LruCache<K,V>` Rust shim (today
  inside wat-rs's source tree; verify location at slice time) →
  `crates/wat-lru/src/lib.rs`.
- `lru = "0.12"` dep moves from wat-rs's `Cargo.toml` to
  `crates/wat-lru/Cargo.toml`. wat-rs loses a third-party dep.
- `wat-lru/src/lib.rs` exposes `pub fn stdlib_sources() ->
  &'static [wat::stdlib::StdlibFile]` returning the baked
  `wat/lru.wat`.
- wat-rs's `stdlib.rs` registration array drops LocalCache's
  entry.
- Any wat-rs tests that reference `:wat::std::LocalCache` move
  into wat-lru's own tests (or retire if they were coupled to
  the baked path by accident).

The repath is a breaking rename of a shipped surface. Deliberate
— the clean-proof choice.

### Slice 5 — Reference binary crate `examples/with-lru/`

The user-shape proof. Inside the wat-rs workspace, a third
member: `examples/with-lru/` with:

- `Cargo.toml` — depends on `wat`, `wat-lru` (path dep for this
  arc; later could be version-pinned when published).
- `src/main.rs` — a single `wat::main! { source:
  include_str!("program.wat"), deps: [wat_lru] }` invocation.
- `program.wat` — a minimal wat program that uses
  `:user::wat::std::lru::LocalCache::*` and prints a result.

`cargo run -p with-lru` produces the expected output. The entire
stack compiles. The pattern is walkable.

### Slice 6 — CONVENTIONS.md gains the namespace table

One section addition: the four-tier table, the `<org>::<name>`
shape for community crates, the claim-by-convention rule, the
collision-detection-at-startup enforcement. Sets the tone for
future ecosystem participants.

---

## Namespace discipline

Four reserved top-level prefixes. Everything lives in one of
these trees.

```
:wat::*                       wat-rs runtime (privileged; user can't declare)
:rust::*                      Rust path mirror (e.g., :rust::lru::LruCache)
:holon::*                     holon-rs types (:holon::HolonAST)
:user::*                      user composition space — everything the user composes
```

`:user::*` is the user's entire composition tree. Inside
`:user::*`, convention carves sub-trees for different kinds of
code:

```
:user::wat::std::<crate>::*      user-composed stdlib tier
                                 (symmetric with :wat::std::*;
                                  claim-by-convention, author self-selects)

:user::<org>::<name>::*          community crates (general)
                                 (npm @scope/pkg shape, Java reverse-DNS,
                                  Go module paths — author claims a unique
                                  <org> handle and scopes under it)

:user::<user-app-tree>::*        user's own program code
                                 (their project, their sub-structure)
```

**Claim-by-convention, not runtime-enforced.** An author who
believes their crate is stdlib-quality claims
`:user::wat::std::<crate>::*`. General-purpose crates claim
`:user::<org>::<name>::*`. The runtime doesn't police taste; it
polices collisions. Two crates claiming the same path fail loud
at startup via the existing duplicate-define detection.

The substrate lets `:foo::bar::baz` be a perfectly legal path
— no hard enforcement of the convention above. The table sets
the tone for good behavior. Authors who ignore it get whatever
collisions their choices earn them.

**wat-lru's self-claim:** `:user::wat::std::lru::*` (stdlib tier).

---

## Ecosystem-safety discipline

Parallel to arc 012's "fork-safety discipline" but about the
crate-composition layer rather than the process-isolation layer.

- **No runtime ownership of `:user::*`.** The runtime treats
  `:user::*` as an open namespace. All registrations go through
  the same collision-detecting path that wat-rs's own stdlib
  uses internally.
- **Cargo is the first line of crate-level collision defense.**
  Crate names are globally unique on crates.io; having two
  `wat-lru`s in one binary is Cargo-impossible. Path
  collisions inside wat can only happen if two differently-named
  crates claim the same wat namespace — detectable, fail-loud.
- **Startup collision message must name both offending crates
  when possible.** If wat-lru and hypothetical `ryan-lru-v2`
  both claim `:user::wat::std::lru::LocalCache::new`, the
  error should say so, not just "duplicate define." Helps
  ecosystem participants resolve naming quickly.
- **`stdlib_sources()` is the contract.** A crate that ships
  wat source MUST expose `pub fn stdlib_sources() -> &'static
  [wat::stdlib::StdlibFile]`. Naming the function `stdlib_sources`
  (not something generic like `wat_files()`) preserves grep-
  ability across the ecosystem.

---

## Convergence with prior art

| Ecosystem | Deps manifest | Dep namespace shape | Collision handling |
|---|---|---|---|
| Cargo | `Cargo.toml` | `crate_name::...` | Cargo enforces global unique |
| npm | `package.json` | `@scope/pkg` | scope-level uniqueness |
| Clojure | `deps.edn` | `my.org.project.*` | reverse-DNS convention |
| Go | `go.mod` | `github.com/user/repo` | module-path uniqueness |
| Haskell | `cabal.project` | `Data.Text`, etc. | package-level uniqueness |
| **wat** | **`Cargo.toml` (reuse)** | **`:user::<org>::<name>::*`** | **Cargo uniqueness + startup-collision fail-loud** |

The convergence is structural: every package ecosystem reaches
the same answer — a unique author identifier + a package name —
because that's what the problem shape requires. wat inherits
Cargo's authority (our deps ARE Cargo crates) and layers a
convention on top of its own namespace space. No parallel
registry needed.

---

## Resolved design decisions

- **2026-04-21** — **LRU externalizes cleanly.** No
  backward-compatible baking; `:wat::std::LocalCache` dies.
  Clean-proof stance: the machinery has to carry the thing or
  the proof is weak.
- **2026-04-21** — **`wat::main!` is a proc-macro, not a plain
  function.** Reads as declaration, not ceremony. The plain
  function `wat::compose_and_run` exists underneath for callers
  who need per-call control, but the macro is the user-facing
  shape.
- **2026-04-21** — **Namespace: `:user::wat::std::lru::*`.**
  Claim-by-convention stdlib tier; symmetric with `:wat::std::*`;
  the `::wat::` marker carries the "external wat library"
  signal.
- **2026-04-21** — **Community crates convention:
  `:user::<org>::<name>::*`.** Matches npm / Go / Java shape.
  Set the tone without hard-enforcing.
- **2026-04-21** — **Workspace member, not sibling repo.**
  `wat-lru` lives at `wat-rs/crates/wat-lru/` as a workspace
  member. Precedent: `wat-macros`. Cleanest for local-path dep
  resolution during the proof.
- **2026-04-21** — **Macro lives in `wat-macros`, not a new
  crate.** The existing proc-macro crate absorbs `wat::main!`
  alongside `#[wat_dispatch]`.

---

## Open questions to resolve as slices land

- **`wat::main!`'s exact signature surface.** Does it support
  `source:` as a file path (`wat::main! { source_file:
  "program.wat", ... }`) or only `source:` as a string
  (requiring `include_str!` at the call site)? Probably both,
  but pins when slice 3 lands.
- **Error return shape.** wat-rs doesn't depend on `anyhow`.
  The macro's expanded `fn main()` needs a Result-returning
  shape that's ergonomic. Options: return `Result<(), wat::
  HarnessError>`; generate a custom error type; generate
  `std::process::ExitCode` on error. Pin in slice 3.
- **Baked stdlib ordering when deps contribute additional
  sources.** The baked stdlib currently registers in a fixed
  order (stream, hermetic, test, service). Adding dep sources
  on top: should deps register BEFORE the baked stdlib, AFTER,
  or interleaved? Most natural: baked first (foundational
  macros available), deps after (may reference baked macros).
  Verify at slice 2.
- **wat-rs CLI's post-refactor role.** The baseline runner
  stance holds for now. Whether it later grows a `--dep <crate>`
  flag that shells out to Cargo to assemble a per-invocation
  binary is a fog that clears when a non-wat-rs caller demands
  it.
- **`StdlibFile` public API stability.** Making the type `pub`
  locks its shape against future changes. If we later need to
  evolve `StdlibFile` (add a priority field, a namespace hint,
  etc.), every dep crate needs to update. Worth a version-bump
  discipline note when we cross that threshold.

---

## What this arc does NOT ship

- **Publishing workflow / crates.io entries.** Path deps only.
- **Migrations of other bakeables** (stream, test, hermetic).
  They stay baked as substrate.
- **Proc-macro for authoring wat crates.** `#[wat_dispatch]` is
  sufficient today; a `#[wat_crate]` helper macro that emits
  `stdlib_sources()` boilerplate is future sugar.
- **IDE tooling / LSP / package manager UI.** Out of scope.
- **Trading lab actually using wat-lru.** That's the NEXT arc
  (or the lab's rewrite arc). This one proves the mechanism.

---

## The thread this continues

Arc 002 (rust-interop-macro) established `:rust::*` + `#[wat_dispatch]`
+ `(use! ...)` as the host-language interop layer. Arc 013
extends that layer into the ecosystem tier: *many Rust-backed
wat crates coexisting in one binary, each scoped to its own
wat namespace, composed through Cargo's dep resolution and
wat's startup-collision discipline.*

Chapter 18 said *"wat is the language, Rust is the substrate."*
Arc 013 makes that true not just for the runtime and one user's
program, but for the ecosystem of wat code itself.
