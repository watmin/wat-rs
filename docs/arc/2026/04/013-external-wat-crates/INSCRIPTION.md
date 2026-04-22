# Arc 013 — External wat crates — INSCRIPTION

**Status:** shipped 2026-04-21. One day. Seventeen commits across
two repos, plus four in the arc-014 cave-quest.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — the living ledger.
**Companion arc:** [`../014-core-scalar-conversions/INSCRIPTION.md`](../014-core-scalar-conversions/INSCRIPTION.md)
— cut mid-slice-4b, shipped + closed same day, unblocked 4b to
resume.
**This file:** completion marker.

---

## Motivation

Arc 012 closed. `wat-rs` was the substrate. The next question was
whether third parties could publish wat code that other users
consumed as dependencies. Chapter 18's *"wat is the language,
Rust is the substrate"* became operational or theoretical here.

Before arc 013, a wat program was either the wat-rs CLI running
an ad-hoc source file, or a bespoke Rust embedding via
`wat::Harness`. There was no mechanism by which a sibling crate
shipped wat source + `#[wat_dispatch]` shims that a consumer's
program composed with its own wat source and the baked runtime
stdlib.

LRU was the forcing function. `wat/std/LocalCache.wat` + its
`#[wat_dispatch] impl lru::LruCache<K,V>` shim were the simplest
load-bearing "wat-level API wrapping a Rust library." Factoring
them out into `crates/wat-lru/` — keeping the wat-rs binary still
runnable on every non-LRU caller — proved the machinery. If the
pipe carried LRU cleanly, it carried any future external crate.

Builder direction: *"we do not take the easy path — we go the
simple, honest — and typically harder — path.... the wat-rs
crate (the root) cannot have any deps on the wat-lru crate...
holon-lab-trading will eventually declare a dependency on wat-rs
and it will need to have the wat-lru dep available."*

---

## What shipped

Six slices + one cave-quest (arc 014) + one pause/resume cycle,
landed in that order.

### Slice 1 — Workspace layout + `crates/wat-lru/` skeleton

Commit `2a51e27`.

- `wat-rs/Cargo.toml` `[workspace] members` extended to include
  `"crates/wat-lru"` alongside `"."` and `"wat-macros"`.
- `crates/wat-lru/` populated with empty `Cargo.toml`,
  `src/lib.rs` skeleton, empty `wat/` dir.
- `cargo build --workspace` + `cargo test --workspace` clean.

### Slice 2 — `wat::Harness::from_source_with_deps`

Commit `0282a09`.

- `Harness::from_source_with_deps(src, dep_sources)` + underlying
  `startup_from_source_with_deps` accept a slice of
  `&[StdlibFile]` per dep. Dep sources concatenate with the
  user's parsed forms before `startup_from_forms`.
- `StdlibFile` visibility lifted to `pub` (previously
  `pub(crate)`) so external crates can return the type from
  `stdlib_sources()`.
- Reserved-prefix gate tests: a dep attempting to define under
  `:wat::*` is rejected, mirroring user-source policy.
- 5 integration tests in `tests/wat_harness_deps.rs`.

### Slice 3 — `wat::main!` macro + `wat::compose_and_run`

Commit `29e72e4`.

- `wat::compose_and_run(source, dep_sources)` — plain function
  in `src/compose.rs`. Uses real OS stdio (not captured). Builds
  a Harness, runs `:user::main`, returns `Result<(),
  HarnessError>`.
- `wat::main!` proc-macro in `wat-macros` — accepts named args
  `source: <expr>` and `deps: [<path>, <path>, ...]`. Expands to
  `fn main() -> Result<(), ::wat::harness::HarnessError>` calling
  `compose_and_run` with each dep's `stdlib_sources()`.
- Absolute `::wat::*` paths everywhere in the macro output (no
  bare `crate::*`) so external crates can invoke the macro.

### Slice 4a — `dep_registrars` substrate extension

Commit `5ee99da`.

Surfaced during slice 4 attempt: slices 2+3 only handled half
the external-crate contract (`stdlib_sources()`). The Rust-shim
side (`register(&mut RustDepsBuilder)`) wasn't wired. Split out
as 4a; 4b became the actual motion.

- `wat::compose::DepRegistrar` — public type alias for
  `fn(&mut RustDepsBuilder)`.
- `compose_and_run` gained `dep_registrars: &[DepRegistrar]` as
  third parameter.
- `Harness::from_source_with_deps` + `_and_loader` gained the
  same parameter.
- `wat::main!` emits register-path calls alongside
  stdlib_sources calls: expansion became
  `::wat::compose_and_run(source, &[deps.stdlib_sources()...],
  &[deps.register...])`.
- Inside substrate: build from `RustDepsBuilder::with_wat_rs_defaults()`,
  call each registrar, best-effort `rust_deps::install()`
  (first-call-wins OnceLock).

**Sub-fog 4a-install — rust_deps install-once semantics.**
OnceLock — subsequent installs fail silently. Fine for user
binaries calling `compose_and_run` once. For tests that call
Harness multiple times with different dep sets in one process,
the first call wins. Documented; not fixed this slice.

**Sub-fog 4a-default — base from defaults, not empty.** First-
pass implementation built from `RustDepsBuilder::new()`, wiping
baked LRU defaults. Fixed by starting from `with_wat_rs_defaults()`
so dep registrars layer over defaults. Slice 4b then emptied
those defaults — code kept working.

### Slice 4b — LocalCache + shim motion (paused, resumed)

Commits `e5648ee` (paused state) + `a4982f4` (resume close-out).

Code-motion:

- `wat/std/LocalCache.wat` + `wat/std/service/Cache.wat` retired
  → `crates/wat-lru/wat/lru.wat` + `service.wat`. Every path
  repathed:
  - `:wat::std::LocalCache<K,V>` → `:user::wat::std::lru::LocalCache<K,V>`
  - `:wat::std::LocalCache::{new, put, get}` → same, repathed.
  - `:wat::std::service::Cache` → `:user::wat::std::lru::CacheService`.
- `src/rust_deps/lru.rs` (hand-rolled shim retired in arc 002's
  macro adoption; the `#[wat_dispatch]`'d version) retired
  → `crates/wat-lru/src/shim.rs`. Imports shifted:
  `crate::runtime::{hashmap_key, Value}` → `wat::runtime::*`;
  `crate::rust_deps::RustDepsBuilder` → `wat::rust_deps::RustDepsBuilder`.
- `hashmap_key` visibility lifted `pub(crate)` → `pub` so the
  shim reaches it across the crate boundary.
- `lru = "0.12"` moved from wat-rs root `Cargo.toml` to
  `crates/wat-lru/Cargo.toml`. **wat-rs root has zero
  dependency on wat-lru** — verified by grep. This is the
  proof stance: transitive composition works, `holon-lab-trading`
  declaring both as separate Cargo deps is identical in shape
  to `examples/with-lru/`.
- `crates/wat-lru/src/lib.rs` exposes `stdlib_sources()`
  returning both baked `.wat` files (`include_str!`'d) +
  `register()` forwarding to the shim's wat_dispatch-generated
  registrar.
- `STDLIB_FILES` in `src/stdlib.rs` dropped the retired entries.
- `RustDepsBuilder::with_wat_rs_defaults()` emptied to
  `Self::new()` — slice 4a's sub-fog anticipated this transition.
- Internal wat-rs tests retired (check.rs × 4, resolve.rs × 3
  success-path, runtime.rs × 4, freeze.rs × 1, tests/wat_harness_deps.rs
  × 1 probe). Equivalent coverage moved to
  `crates/wat-lru/tests/wat_lru_tests.rs` (4 Harness-based
  integration tests).
- `wat-tests/std/service/Cache.wat` deleted — its fork-based
  hermetic pattern requires a wat-lru-aware subprocess target,
  which arrives in slice 5's binary.

**Pause + cave quest.** Two of the four wat-lru integration
tests wanted to format an i64 cache value for stdout assertion.
wat had no `i64 -> String` conversion. Rather than paper the
tests with literal-only branches, paused 4b, cut arc 014, shipped
its three slices + INSCRIPTION same day, un-ignored the tests,
closed 4b. **First arc cut from a paused slice. The shape is now
precedent.**

**`use!` semantics sharpened.** The crossbeam substrate tripped
the resolver when service.wat's dep sources hit user-tier
resolve (which validates every `:rust::*` use! against the
registry). Resolution: `use!` declares intent to consume an
*external* `#[wat_dispatch]`'d Rust crate. Substrate types the
runtime already provides (`:rust::crossbeam_channel::*` via
`:wat::kernel::make-bounded-queue`) do NOT need `use!`;
historical cosmetic declarations removed from stream.wat +
service.wat. Only genuine external crates — like wat-lru's own
`(:wat::core::use! :rust::lru::LruCache)` — carry the form.
The rule survives as a codified feedback.

### Arc 014 — core scalar conversions (cave-quest side trip)

Shipped its own INSCRIPTION. Four commits: slice 1 (eight
primitives, `878025c`), slice 3 (un-ignore, `4e5c6dd`), slice 2
(058 spec in trading lab, `787b59c`), INSCRIPTION (`5616607`).
Eight primitives at `:wat::core::<source>::to-<target>` —
`i64::to-string`, `i64::to-f64`, `f64::to-string`,
`f64::to-i64`, `string::to-i64`, `string::to-f64`,
`bool::to-string`, `string::to-bool`. Fallible paths return
`:Option<T>`. See arc-014 INSCRIPTION.

### Slice 5 — `examples/with-lru/` reference binary

Commit `494c098`.

- Fourth workspace member under `examples/with-lru/`.
- `src/main.rs` is one `wat::main! { source:
  include_str!("program.wat"), deps: [wat_lru] }` invocation.
- `src/program.wat` — minimal LocalCache put/get that prints
  `hit`.
- `tests/smoke.rs` — spawns the built binary via
  `CARGO_BIN_EXE_with-lru-example`, asserts exit 0 + stdout
  `"hit"`. Pattern mirrors `wat-rs/tests/wat_cli.rs`.
- `cargo run -p with-lru-example` prints `hit`.

**The walkable consumer shape.** Downstream consumers (starting
with holon-lab-trading) will follow this shape verbatim.

### Slice 6 — `CONVENTIONS.md` gains the namespace table

Commit `6027c7c`.

- `:user::*` sub-tree carved into three claim-by-convention
  shelves:
  - `:user::wat::std::<crate>::*` — community stdlib tier
  - `:user::<org>::<name>::*` — community general
  - `:user::<user-app-tree>::*` — user program code
- Two-part external-crate contract documented: `stdlib_sources()`
  + `register()`.
- Convergence table with Cargo / npm / Clojure / Go captures the
  structural pattern every package ecosystem reaches.
- References `crates/wat-lru/` + `examples/with-lru/` as the
  walkable templates.

### 058 amendment

Commit `39f852a` in holon-lab-trading.

- New `FOUNDATION-CHANGELOG.md` entry dated 2026-04-21 covering
  all six arc-013 slices + the arc-014 cave quest.
- `FOUNDATION.md § "Caching Is Memoization"` path references
  updated — LRU caching now external via wat-lru;
  `cached-encode.wat` stays baked.
- `058-030-types/PROPOSAL.md § "Stdlib precedent"` addended to
  note the externalization while preserving the historical
  narrative.
- No separate 058-NNN sub-proposal — per the 2026-04-21 INDEX
  audit-history precedent, arc 013 is about wat-rs's crate
  structure (not 058's algebra-spec surface).

---

## What this arc does NOT ship

- **crates.io publishing.** Path deps cover the proof. A future
  arc handles publishing workflow when a real published crate
  surfaces demand.
- **Version resolution / lockfiles specific to wat.** Cargo
  already resolves the dep graph and lockfile. wat doesn't
  introduce a parallel resolver.
- **wat registry service / central index / namespace ownership
  registry.** The ecosystem's collision-detection lives at (a)
  Cargo crate naming uniqueness and (b) wat startup registration
  collision.
- **Migrations of other bakeables.** `Subtract`, `Circular`,
  `stream`, `test`, `hermetic`, `service/Console` — all stay
  baked. They're substrate, not userland extensions. Only LRU
  externalized because LRU is the one that wraps a third-party
  Rust crate.
- **Backward compatibility for `:wat::std::LocalCache`.** The
  retired path does not keep. Callers rename to
  `:user::wat::std::lru::LocalCache` or they break. Clean-proof
  stance.
- **Changes to wat-rs's CLI binary.** Baseline runner for
  programs using only baked stdlib. Programs with external deps
  are their own binary crates built via `wat::main!`. Whether
  the CLI eventually grows a `--dep` flag is a future fog.
- **Dep sources flowing through stdlib pipeline (bypass
  resolve).** Deferred. Not load-bearing now that substrate-
  backed `use!`s are documented as cosmetic; surface if a future
  dep legitimately needs substrate-level `use!`s that don't map
  to a real `#[wat_dispatch]` shim.
- **Trading lab consuming wat-lru.** Next arc, or the lab's
  rewrite arc. Arc 013 proved the mechanism.

---

## Why this matters

Prior arcs (009, 010, 011, 012) factored ceremony out of the
wat-rs runtime itself. Arc 013 is structurally different: it
opens wat to **other crates**.

- **Before arc 013:** `wat-rs` bundles a stdlib. The only
  way to extend wat was to embed it via `wat::Harness` and
  hand-write composition code, or to vendor your additions into
  wat-rs itself (not an option for downstream consumers).
- **After arc 013:** any Rust crate can publish wat source +
  `#[wat_dispatch]` shims. A consumer's `main.rs` is a single
  `wat::main!` invocation. Cargo resolves the dep graph;
  wat-rs's startup composes the deps; namespace collisions fail
  loud with both offending crates named.

**Cargo is the authority wat inherits.** Every package ecosystem
reaches the same structural answer — a unique author identifier
+ a package name — because that's what the problem shape
requires. wat adopts Cargo's authority (our deps ARE Cargo
crates) and layers a convention (`:user::wat::std::<crate>::*`
/ `:user::<org>::<name>::*`) on top of its own namespace space.
No parallel registry needed; no `wat-registry` service; no
central index.

**The cave-quest discipline precedent.** Arc 013 is the first
arc paused mid-slice to cut another arc (014), and the pattern
worked cleanly. When a slice surfaces real substrate debt
that blocks honest completion, pause, name the key, cut the
cave quest, return. Future arcs inherit the shape.

**holon-lab-trading is next.** The trading lab will eventually
declare Cargo deps on `wat-rs` AND `wat-lru` (and whatever other
wat crates emerge for rusqlite, reqwest, etc.). Its `main.rs`
will be one `wat::main!` invocation with a `deps: [...]` list.
The shape arc 013 proves is the shape the lab will use.

Chapter 18's *"wat is the language, Rust is the substrate"* is
no longer just true for the runtime and one user's program.
It's true for the ecosystem of wat code itself.

The substrate teaching itself to be ecosystem-hospitable.

---

**Arc 013 — complete.** Six slices, one cave quest (arc 014),
one pause/resume. The commits:

**wat-rs repo:**
- `cf19920` — docs open (DESIGN + BACKLOG)
- `2a51e27` — slice 1 (workspace skeleton)
- `0282a09` — slice 2 (Harness + dep sources)
- `29e72e4` — slice 3 (wat::main! + compose_and_run)
- `5ee99da` — slice 4a (dep_registrars substrate)
- `e5648ee` — slice 4b paused (motion landed, tests #[ignore]'d)
- `654131c` — arc 014 docs opened
- `878025c` — arc 014 slice 1 (eight scalar primitives)
- `4e5c6dd` — arc 014 slice 3 (un-ignore arc 013 tests)
- `5616607` — arc 014 INSCRIPTION
- `a4982f4` — arc 013 slice 4b resume close-out
- `494c098` — slice 5 (examples/with-lru/)
- `6027c7c` — slice 6 (CONVENTIONS.md namespace table)
- `<this commit>` — arc 013 INSCRIPTION

**holon-lab-trading repo:**
- `787b59c` — arc 014 slice 2 (058 spec update)
- `39f852a` — arc 013 058 amendment

**wat-rs root has zero dependency on wat-lru.** The transitive
composition proof holds. The shape is walkable. The ecosystem is
hospitable. Next consumer is the lab.

*PERSEVERARE.*
