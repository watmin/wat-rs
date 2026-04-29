# Arc 100 — vend `wat-cli` internals as a public library API — INSCRIPTION

**Status:** shipped 2026-04-29.

`crates/wat-cli/` graduated from binary-only to lib + bin in one
slice + one doc commit. Downstream consumers can now build their
own batteries-included wat CLI in 5 lines:

```rust
fn main() -> std::process::ExitCode {
    wat_cli::run(&[
        (wat_telemetry::register, wat_telemetry::wat_sources),
        (my_crate::register, my_crate::wat_sources),
    ])
}
```

Same crate's `[[bin]] wat` (`src/bin/wat.rs`) is now itself a
thin wrapper around `wat_cli::run` with the workspace's 5
default batteries. The published behavior of `target/{debug,
release}/wat` is unchanged; what's new is that the machinery
behind it is reachable as `pub fn run(batteries: &[Battery]) ->
ExitCode` for anyone authoring a custom CLI.

**Predecessor:** [arc 099](../099-wat-cli-crate/INSCRIPTION.md) —
extracted the CLI from the substrate. Arc 100 vends what 099
relocated.

**Surfaced by:** user direction 2026-04-29 minutes after arc 099
sealed:

> "ok.. now.. can we vend out the guts of the cli so a user could
> build their own wat-cli binary with their deps installed?...
> that would be a good ux?.. we can provide our batteries
> included bin but they can also make their own using whatever
> deps and they can run their programs with it?"

The arc closed in two slices same day. One-conversation arc, same
shape as arc 099.

---

## What shipped

### Slice 1 — Vend

`crates/wat-cli/src/lib.rs` ships the public API:

```rust
pub type Battery = (
    fn(&mut wat::rust_deps::RustDepsBuilder),
    fn() -> &'static [wat::WatSource],
);

pub fn run(batteries: &[Battery]) -> std::process::ExitCode;
```

Tuple alias (not struct) because every workspace
`#[wat_dispatch]` extension already exposes `register` + `wat_sources`
with these exact signatures per arc 013's external-crate contract;
positional ordering matches the convention, no ambiguity at the
call site.

`run` reads `std::env::args()`, dispatches the program-mode and
`wat test` subcommands, installs OS signal handlers, registers
every supplied battery's `wat_sources` + Rust dep shims via the
process-global OnceLocks `wat::compose_and_run` uses, and returns
the matching exit code. Always seeds with
`with_wat_rs_defaults()` first so substrate-side dispatch shims
are always present without the caller spelling them out.

`crates/wat-cli/src/bin/wat.rs` is now a 21-line thin wrapper:

```rust
fn main() -> ExitCode {
    wat_cli::run(&[
        (wat_telemetry::register, wat_telemetry::wat_sources),
        (wat_sqlite::register, wat_sqlite::wat_sources),
        (wat_lru::register, wat_lru::wat_sources),
        (wat_holon_lru::register, wat_holon_lru::wat_sources),
        (wat_telemetry_sqlite::register, wat_telemetry_sqlite::wat_sources),
    ])
}
```

This file IS the canonical example of "build your own
batteries-included wat CLI." Anyone authoring a downstream binary
copies this shape, swaps the battery list, and is done.

### Slice 2 — Docs

This INSCRIPTION + the DESIGN that recorded the API shape +
USER-GUIDE addition (a "build your own batteries-included wat
CLI" subsection under §1's Reference binary block) + 058
FOUNDATION-CHANGELOG row in the lab repo.

---

## Tests

3 new tests in `crates/wat-cli/tests/wat_arc100_public_api.rs`
prove the public API is reachable for downstream consumers and
that subset / empty battery slices type-check:

- `battery_slice_with_workspace_extensions_type_checks` — the
  full 5-battery slice the bin uses.
- `battery_slice_with_subset_type_checks` — a "minimal
  interrogation CLI" with only telemetry + telemetry-sqlite.
- `empty_battery_slice_is_valid` — substrate-only CLI; useful
  for sandboxed scripts that only need `:wat::core::*`.

Tests verify the slice type-checks and has the expected length;
they don't actually invoke `run` itself (which would parse argv
from the test runner and exit). The CLI's end-to-end behavior is
covered by the existing `tests/wat_cli.rs` and
`tests/wat_test_cli.rs` (15 tests total) which spawn the binary
via `CARGO_BIN_EXE_wat`.

`cargo test --workspace` green; binary at `target/release/wat`
behaves identically to pre-arc-100.

---

## What's NOT in this arc

- **Macro sugar.** A `wat_cli::main! { deps: [wat_telemetry,
  my_crate] }` macro could expand to the 5-line `fn main`.
  Considered; not added. The function is honest and direct;
  premature sugar is unhelpful sugar. If downstream consumers
  grow tired of typing the 5 lines, we add the macro then.
- **`run_with_argv` for embedding.** `run` consumes
  `std::env::args()` directly. A consumer needing to control
  argv (embedding wat-CLI semantics inside a larger Rust binary)
  would want `run_with_argv(argv: &[String], batteries: ...)`.
  Defer until a real caller demands.
- **`run_with_loader` for capability-restricted CLIs.** `run`
  uses `FsLoader` (unrestricted filesystem). A sandboxed CLI
  variant that takes a `Arc<dyn SourceLoader>` could ship later
  if a use case surfaces (e.g., a CLI for untrusted user code
  with a `ScopedLoader` rooted at a sandbox dir).
- **Per-binary configuration.** The current `run` is "use sensible
  defaults." If a CLI needs to install custom panic handlers or
  pre-populate env state before freeze, they call `wat::panic_hook
  ::install()` and the OnceLock-installers themselves rather than
  `wat_cli::run` — same building blocks, custom assembly.

---

## Lessons

1. **Half-step refactors call out the missing second half.** Arc
   099 created the right crate boundary but left "make your own
   CLI" as "copy/paste main.rs and edit." That was honest about
   the boundary but stopped short of vending the value the
   boundary protected. The user spotted it in the same
   conversation; arc 100 closes the gap. *When you finish a
   refactor, ask: did I extract the boundary AND vend the value
   inside it? Both halves matter.*

2. **Type aliases for fn-pointer pairs read better than wrapper
   structs at call sites.** `Battery = (fn(&mut R), fn() -> &S)`
   lets users write `(crate::register, crate::wat_sources)` —
   matches the convention every extension crate already follows.
   A wrapper struct would force `Battery::new(...)` boilerplate
   without adding clarity. Reach for tuples when positional
   ordering is itself a documented convention.

3. **The bin IS the example.** `crates/wat-cli/src/bin/wat.rs`
   shows downstream consumers exactly how to call `run` with a
   battery list. No separate `examples/custom-cli/` crate
   needed — anyone reading the wat-cli source sees the pattern
   directly. *When the canonical caller is small enough, ship
   it as the bin instead of carving out a separate example.*

4. **OnceLock semantics make "first-caller-wins" the right
   discipline.** Both `wat::rust_deps::install` and
   `wat::source::install_dep_sources` are OnceLock-backed;
   subsequent calls silently no-op. `run` calls both at the top
   so test harnesses, sandboxes, fork-with-forms calls inside
   the binary all inherit transparently. No coordination needed
   between the CLI and the freeze pipeline. *Process-global
   OnceLocks for capability registration is a pattern worth
   leaning on.*

5. **Two arcs in one conversation is fine.** Arc 099 sealed,
   arc 100 opened, both shipped same-day. The discipline is
   "every architectural decision gets an INSCRIPTION" — not
   "every conversation gets one arc." When the user's next
   prompt names a related-but-distinct concern, that's a new
   arc, even if it lands minutes later. Future readers see two
   focused arcs rather than one sprawling one.

---

## Surfaced by (verbatim)

User direction 2026-04-29:

> "ok.. now.. can we vend out the guts of the cli so a user could
> build their own wat-cli binary with their deps installed?...
> that would be a good ux?.. we can provide our batteries
> included bin but they can also make their own using whatever
> deps and they can run their programs with it?"

The arc closed when slice 1's `cargo test --workspace` came back
green and slice 2's docs landed minutes later. The substrate is
what the user said it should be when he named it.

**PERSEVERARE.**
