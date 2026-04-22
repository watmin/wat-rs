# Arc 013 — External wat crates — Backlog

**Opened:** 2026-04-21.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** the living ledger — tracking, decisions, open
questions as slices land.

Every item below ships in the **inscription** mode: build honestly,
write the spec that describes what landed. Same pattern as arcs
004/006/012. "Blockers as they arise" — each item's fog resolves
when the prior item lands.

---

## The gap

wat-rs has established that a single binary can link the runtime
and one user's wat program cleanly (arc 007's test harness). Arc
002 established `:rust::*` + `#[wat_dispatch]` as the interop
layer for surfacing Rust types. What's missing is the tier above:
**can many Rust-backed wat crates coexist in one binary, each
scoped to its own wat namespace, composable through Cargo?**

Today the answer is "theoretically yes, but nobody's done it."
`wat/std/LocalCache.wat` + its `#[wat_dispatch] impl lru::LruCache`
shim sit inside wat-rs's own crate. No sibling crate has ever
shipped wat source + a Rust shim that a consumer's program
composed as a dep.

LRU is the forcing function. Factoring it out of wat-rs into a
`crates/wat-lru/` sibling, with the wat-rs binary still runnable
on every non-LRU caller, proves the machinery. If LRU carries,
any future external crate carries.

---

## 1. Workspace layout + `crates/wat-lru/` skeleton

**Status:** ready. Concrete approach in hand.

**Problem:** wat-rs is already a Cargo workspace (the existing
`wat-macros/` member proves it). But the workspace's `Cargo.toml`
[workspace] section may not anticipate a `crates/` subdirectory
for additional members. Adding `crates/wat-lru/` needs a
one-line update to the workspace members list + a skeleton crate
structure.

**Approach:**
- Verify wat-rs's `Cargo.toml` [workspace] members list. If it's
  `["wat-macros"]`, extend to `["wat-macros", "crates/wat-lru"]`.
  If the structure is different, adjust accordingly.
- Create `crates/wat-lru/Cargo.toml` with minimal metadata,
  depending on `wat` (via `path = "../.."`) and `lru = "0.12"`
  (picked up from what wat-rs currently has).
- Create `crates/wat-lru/src/lib.rs` as empty `//! wat-lru —
  LRU cache for wat, wrapping `lru::LruCache<K,V>`.` + a
  placeholder `pub fn stdlib_sources() -> &'static [...] { ... }`.
- Create `crates/wat-lru/wat/` dir empty (populated in slice 4).
- `cargo build --workspace` passes; `cargo test --workspace`
  passes (with wat-lru contributing zero tests for now).

**Spec tension:** none at this slice. Pure Cargo-side scaffolding.

**Inscription target:** none yet; slice 6's CONVENTIONS.md
update captures the workspace shape.

**Unblocks:** slice 2 can develop against a real target.

---

## 2. `wat::Harness` accepts external stdlib sources

**Status:** ready. Concrete approach in hand.

**Problem:** `wat::Harness::from_source` today assumes the baked
stdlib is the complete set of registrable wat sources before the
user's source reaches macro expansion. External crates'
`stdlib_sources()` results have nowhere to hook in.

**Approach:**
- Audit `StdlibFile`'s visibility — if `pub(crate)`, lift to
  `pub` (or re-export from `wat::stdlib::StdlibFile`). External
  crates need the type in their signature.
- Add `Harness::from_source_with_deps(src: &str, dep_sources:
  &[&[StdlibFile]]) -> Result<Self, HarnessError>`. Preserves
  the old API as the zero-deps case (`from_source` delegates).
- Internally, concatenate deps' sources BETWEEN the baked
  stdlib and the user's source: `[baked..., dep1..., dep2...,
  user]`. Baked first because it provides foundational macros
  deps may reference; deps before user because user may
  reference dep stdlib.
  - Decide as code lands: is this interleaving ever wrong?
    Probably not for the proof case. Amend BACKLOG if a
    counter-example surfaces.
- Integration test: a throwaway `Vec<StdlibFile>` with one
  simple define; confirm it's callable from a user source that
  Harness runs.

**Spec tension:** adding `Harness::from_source_with_deps` is a
public API addition to wat-rs. Stable at slice time — we're not
likely to remove it. Any later evolution of `StdlibFile` itself
becomes a breaking change to all dep crates; flag in DESIGN's
open questions.

**Inscription target:** `wat::Harness` is out of scope for 058
(it's Rust-host API, not wat language surface). No 058 entry.
Noted in wat-rs's README next time that doc sweeps.

**Unblocks:** slice 3 can build `wat::main!` on top;
slice 4 can exercise the path with real wat-lru content.

---

## 3. `wat::main!` proc-macro (+ `wat::compose_and_run` function underneath)

**Status:** obvious in shape; error-shape sub-fog pins when slice
lands.

**Problem:** the user-composed binary's `main.rs` today would look
like:

```rust
fn main() -> Result<(), wat::HarnessError> {
    let harness = wat::Harness::from_source_with_deps(
        include_str!("program.wat"),
        &[wat_lru::stdlib_sources()],
    )?;
    harness.run(&[])?;
    Ok(())
}
```

Workable but ceremonial. Every external-wat-crate binary writes
the same three lines. A proc-macro that expands to this shape
reads as declaration.

**Approach:**
- Add `wat::main!` as a proc-macro inside the existing
  `wat-macros` crate. Named args: `source:` (a string expression
  — typically `include_str!("program.wat")`) and `deps:` (an
  identifier list — each identifier is a crate that exposes
  `stdlib_sources()`).
- The macro expands to a `fn main()` that calls
  `wat::compose_and_run(source, &[deps::stdlib_sources()...])`.
- `wat::compose_and_run` is the non-macro function — plain Rust
  API for callers who want per-call control (tests, embedding
  wat inside a larger Rust program's flow).
- `wat::compose_and_run` returns a type suitable for being a
  main function's `-> Result<...>`. Cleanest shape: `Result<(),
  wat::HarnessError>`.
- Error-path: compilation errors from the macro invocation (bad
  args, missing `stdlib_sources()` on a dep) surface as normal
  `compile_error!` spans. Runtime errors surface via the
  returned Result.
- Alternative shape to consider when the slice lands:
  `source_file: "program.wat"` as an ergonomic shortcut — the
  macro internally does `include_str!("program.wat")`. Cleaner
  for the common case. Probably both supported.

**Sub-fog 3a — error-return shape.** wat-rs doesn't depend on
`anyhow`. Options: return `Result<(), wat::HarnessError>`;
return `std::process::ExitCode`; return something Try-compatible
with `?`. Probably the HarnessError Result. Pin when slice lands.

**Spec tension:** adds a new public macro to wat-rs's surface.
Documented in a new README section + USER-GUIDE section when
slice lands.

**Inscription target:** no 058 entry (Rust-host API).
USER-GUIDE section on "Building a wat program in a Rust binary
crate."

**Unblocks:** slice 5 (the reference binary crate) uses
`wat::main!` directly to prove the macro ergonomics.

---

## 4. Move LocalCache.wat + shim into wat-lru; repath surfaces

**Status:** ready. Most code-motion of any slice.

**Problem:** the actual externalization. `wat/std/LocalCache.wat`
+ the `#[wat_dispatch] impl` of `lru::LruCache<K,V>` + the
baked-in registration in `stdlib.rs` + the `lru` Cargo dep all
need to move.

**Approach:**
- **wat source.** Move `wat/std/LocalCache.wat` to
  `crates/wat-lru/wat/lru.wat`. Rewrite every define's keyword
  path:
  - `:wat::std::LocalCache<K,V>` → `:user::wat::std::lru::LocalCache<K,V>`
  - `:wat::std::LocalCache::new` → `:user::wat::std::lru::LocalCache::new`
  - etc. for put / get / get-or-insert-with / size / capacity
  - Service variant: `:wat::std::service::Cache` →
    `:user::wat::std::lru::CacheService`.
- **Rust shim.** Locate the `#[wat_dispatch] impl
  lru::LruCache<K,V>` — in wat-rs's source tree today. Move the
  file to `crates/wat-lru/src/lib.rs` (or a subfile). Shim
  imports `wat` + `wat-macros` rather than being `crate::*`.
- **Dep move.** Remove `lru = "0.12"` from wat-rs's `Cargo.toml`.
  Add it to `crates/wat-lru/Cargo.toml`.
- **stdlib_sources.** `crates/wat-lru/src/lib.rs` grows:
  ```rust
  pub fn stdlib_sources() -> &'static [wat::stdlib::StdlibFile] {
      static FILES: &[wat::stdlib::StdlibFile] = &[
          wat::stdlib::StdlibFile {
              path: "wat-lru/lru.wat",
              source: include_str!("../wat/lru.wat"),
          },
      ];
      FILES
  }
  ```
- **wat-rs's stdlib.rs.** Drop the `LocalCache.wat` entry from
  `STDLIB_FILES`.
- **Tests that referenced `:wat::std::LocalCache`.** Locate (grep
  `LocalCache` in wat-rs's `tests/` and `wat-tests/`). Options
  per test:
  - If the test was INSIDE wat-rs's corpus to prove the baked
    LocalCache worked: move the test into wat-lru (as a new
    test file referencing the new path).
  - If the test was incidental / probing the substrate: update
    the path in place.
  Verify `cargo test --workspace` + `wat test wat-tests/` pass
  after the move.

**Sub-fog 4a — `wat-lru`'s wat file path key.** When
`include_str!` bakes the source, the `StdlibFile.path` field
is the logical path reported in errors. "wat-lru/lru.wat" keeps
it grep-able; could also be "crates/wat-lru/wat/lru.wat" for
filesystem clarity. Decide when slice lands.

**Sub-fog 4b — `:rust::lru::LruCache` reachability from wat-lru.**
`#[wat_dispatch]`'s generated code needs `wat::runtime::Value`
etc. in scope. Once wat-lru's shim lives in a separate crate,
that imports chain is: wat-lru depends on `wat`; the macro
expansion inside wat-lru's shim references `wat::...` paths.
Should work without changes — but verify at slice time that
`wat-macros`'s generated output uses absolute paths (prefixed
with `::wat::` or `wat::`, never bare `crate::`). If the proc-
macro generates bare `crate::` paths, that's a bug to fix in
`wat-macros` before externalization works at all.

**Spec tension:** the retired `:wat::std::LocalCache` path is a
breaking change. Named in DESIGN non-goals; no migration period.
Acknowledged in the USER-GUIDE sweep at slice 5 or after.

**Inscription target:** amendment to 058-034-stream-stdlib
recording the LocalCache externalization (LocalCache lives
inside the stream tier of the stdlib inscription). Probably not
worth a new 058-NNN proposal — externalization is about wat-rs's
crate structure, not 058's algebra-spec surface.

**Unblocks:** slice 5 has real content to exercise.

---

## 5. Reference binary crate `examples/with-lru/`

**Status:** obvious in shape once slices 1-4 land.

**Problem:** the whole pattern only proves itself when a
consumer can actually COMPOSE wat-rs + wat-lru + their own
program into a runnable binary. A reference crate inside the
workspace acts as the walkable example.

**Approach:**
- Add `examples/with-lru/` as a third workspace member (or under
  a dedicated `examples/` directory if the workspace structure
  benefits).
- `Cargo.toml`:
  ```toml
  [package]
  name = "with-lru-example"
  version = "0.1.0"
  edition = "2021"

  [dependencies]
  wat     = { path = "../.." }
  wat-lru = { path = "../../crates/wat-lru" }
  ```
- `src/main.rs`:
  ```rust
  wat::main! {
      source: include_str!("program.wat"),
      deps: [wat_lru],
  }
  ```
- `src/program.wat`:
  ```scheme
  (:wat::config::set-dims! 1024)
  (:wat::config::set-capacity-mode! :error)

  (:wat::core::use! :rust::lru::LruCache)

  (:wat::core::define (:user::main
                       (stdin :wat::io::IOReader)
                       (stdout :wat::io::IOWriter)
                       (stderr :wat::io::IOWriter)
                       -> :())
    (:wat::core::let*
      (((cache :user::wat::std::lru::LocalCache<String,i64>)
        (:user::wat::std::lru::LocalCache::new 10))
       ((_ :()) (:user::wat::std::lru::LocalCache::put cache "answer" 42))
       ((got :Option<i64>) (:user::wat::std::lru::LocalCache::get cache "answer")))
      (:wat::core::match got -> :()
        ((Some v) (:wat::io::IOWriter/println stdout "hit"))
        (:None    (:wat::io::IOWriter/println stdout "miss")))))
  ```
- `cargo run -p with-lru-example` prints `hit`. End-to-end proof.
- Integration test inside the example: `tests/smoke.rs` that
  spawns the binary and asserts on output (same pattern as
  wat-rs's `tests/wat_cli.rs`).

**Spec tension:** none. The example is informational — a
walkable demonstration, not a substrate change.

**Inscription target:** none directly. But the existence of the
example is the proof the INSCRIPTION cites when describing the
arc's close.

**Unblocks:** slice 6's CONVENTIONS.md update has a real
reference to link to.

---

## 6. CONVENTIONS.md gains the namespace table

**Status:** ready once the pattern is proven.

**Problem:** the four-tier namespace table + the
`<org>::<name>` community shape + the claim-by-convention rule
need a permanent home. Today they live only in this DESIGN.md.
CONVENTIONS.md already has a "Namespaces" section (line ~19 at
the time of this writing) — extend it.

**Approach:**
- Add a subsection "External wat crates" or "Community
  namespaces" under Namespaces.
- Include the four-tier table verbatim.
- Name the `<org>::<name>` shape with convergence notes
  (npm, Go, Java).
- State the claim-by-convention rule + collision-at-startup
  enforcement.
- Link to `crates/wat-lru/` as the reference.

**Spec tension:** CONVENTIONS is project-wide discipline
documentation. Additions have to pass the "would a new
contributor read this and follow it" test.

**Inscription target:** CONVENTIONS.md itself is the target.
No 058 entry.

**Unblocks:** INSCRIPTION writing at arc close has a
discipline-doc to reference.

---

## Open questions carried forward

- **`wat::main!`'s `source_file:` variant.** Support path-based
  shortcut alongside the string-based one? Pin at slice 3.
- **Error-return shape for the generated `main` function.** Pin
  at slice 3.
- **Baked-vs-deps source ordering at startup.** Verify baked
  first + deps after is correct; pin at slice 2.
- **`StdlibFile`'s public API shape.** Making it `pub` locks its
  form. A future evolution (adding fields) would be a breaking
  change for every dep crate. Named in DESIGN open questions;
  monitored as the ecosystem grows.
- **wat-rs CLI's role.** Baseline runner holds for now. Future
  fog — clears when a non-wat-rs caller demands something the
  CLI can't deliver.

---

## What this arc does NOT ship

- Publishing to crates.io.
- Migrating other bakeables.
- A `#[wat_crate]` author-helper proc-macro.
- Trading lab consuming wat-lru (that's the next arc or the lab
  rewrite arc).
- IDE tooling, LSP, package manager UI.

---

## Why this matters

Arc 002 established the Rust-interop layer: `:rust::*` +
`#[wat_dispatch]` let a wat program surface a single Rust type.
Arc 013 extends that layer to the ecosystem tier: **many
Rust-backed wat crates coexist in one binary, each scoped to
its own wat namespace, composed through Cargo's dep resolution
and wat's startup-collision discipline.** LRU is the forcing
function; the machinery either carries it cleanly or the
externalization isn't real.

Chapter 18's *"wat is the language, Rust is the substrate"*
becomes operational at the ecosystem tier when this arc closes.
Third parties can publish wat code. Users can compose multiple
wat crates + their own program into one honest binary. The
substrate teaching itself to be ecosystem-hospitable.
