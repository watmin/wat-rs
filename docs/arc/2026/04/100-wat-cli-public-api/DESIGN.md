# Arc 100 — vend `wat-cli` internals as a public library API — DESIGN

**Status:** SETTLED — opened and closed in the same conversation
turn 2026-04-29 immediately after arc 099 sealed. The decision was
a small API extraction with no design questions; this DESIGN
exists to record the move so future readers don't have to
reverse-engineer "why is the wat CLI a lib + bin instead of just
a bin?"

**Predecessor:** [arc 099](../099-wat-cli-crate/INSCRIPTION.md) —
extracted the CLI into its own crate.

**Predecessor of:** any future downstream wat CLI binary. The lab
gets a clean two-line `main()` if/when it wants its own CLI.

---

## The move

**Before (post-arc-099):** `crates/wat-cli/` was a binary-only
crate. The `main()` function lived in `src/main.rs`, hardcoding
the workspace's 5 batteries. Anyone wanting a custom CLI had to
copy/paste `main.rs`, edit `install_batteries()`, and reimplement
the argv / subcommand / signal-handler boilerplate.

**After:** `crates/wat-cli/` is a lib + bin crate. The library
(`src/lib.rs`) exposes:

```rust
pub type Battery = (
    fn(&mut wat::rust_deps::RustDepsBuilder),
    fn() -> &'static [wat::WatSource],
);

pub fn run(batteries: &[Battery]) -> std::process::ExitCode;
```

The bin (`src/bin/wat.rs`) is a thin wrapper that calls `run()`
with the workspace's 5 default batteries. Downstream consumers
who want a custom CLI write a 5-line `main.rs`:

```rust
fn main() -> std::process::ExitCode {
    wat_cli::run(&[
        (wat_telemetry::register, wat_telemetry::wat_sources),
        (my_crate::register, my_crate::wat_sources),
    ])
}
```

Same shape `wat::main!` provides for embedding-a-program — `run()`
is to argv-driven CLIs what `wat::main!` is to embedded programs.
Different use cases, different surfaces, both vended from the
substrate.

---

## Why vend the guts

**Honest extension story.** Arc 099 left "make your own CLI" as
"copy/paste main.rs and edit." That's a half-step — the move
created the right crate boundary but didn't expose what made the
crate useful. Vending `run()` closes the gap.

**Argv parsing + signal handlers shouldn't be reimplemented.**
The `wat <entry.wat>` and `wat test <path>` semantics, the SIGINT
→ kernel-stop bridge, the SIGUSR1/2/HUP forwards, the exit-code
mapping — all of this is substrate-shaped CLI machinery. Keeping
it in the library means downstream binaries inherit the
maintained version automatically rather than each one drifting
from its own copy.

**Mirrors the existing `wat::main!` pattern.** Arc 013's macro
provides the embedding-a-program path; arc 100's `run` provides
the running-argv-supplied-programs path. Both vend the substrate's
work as a usable surface; both expect users to declare their
batteries and let the substrate do the rest.

**Sibling binaries become trivial.** A future
`wat-interrogate` (or `wat-with-grpc`, or any specialized
batteries set) is a 5-line `main.rs` calling `wat_cli::run` with
the relevant deps. No copy/paste; no drift; the discipline is
"declare your deps, call run."

---

## API shape

### `Battery` — the dep-pair type

```rust
pub type Battery = (
    fn(&mut wat::rust_deps::RustDepsBuilder),
    fn() -> &'static [wat::WatSource],
);
```

A tuple of two function pointers:

- **`register`**: the crate's `pub fn register(builder: &mut
  RustDepsBuilder)` — registers Rust shims (#[wat_dispatch]
  outputs) into the global registry.
- **`wat_sources`**: the crate's `pub fn wat_sources() ->
  &'static [WatSource]` — the baked-in wat source files the
  crate ships.

Every `#[wat_dispatch]` extension crate following arc 013's
external-crate contract exposes these two functions with these
exact signatures. The Battery alias is just (the type of the
register fn, the type of the wat_sources fn) zipped — no wrapper
construction needed at call sites.

### `run` — the entry point

```rust
pub fn run(batteries: &[Battery]) -> std::process::ExitCode;
```

Reads `std::env::args()`, dispatches the program-mode and
`wat test` subcommands, installs signal handlers, registers every
supplied battery's `wat_sources` + Rust dep shims, and returns
the matching exit code. Both halves of the external-crate
contract install via process-global OnceLocks (per
`wat::compose_and_run`'s docs); first caller wins.

`run` always seeds the `RustDepsBuilder` with
`with_wat_rs_defaults()` before applying the supplied batteries
— substrate-side dispatch shims (the `:wat::*` surfaces wired
through `#[wat_dispatch]` inside the substrate crate) are always
available without the caller having to spell them out.

---

## What changes

### `crates/wat-cli/` layout

- `src/lib.rs` — NEW. Library with `pub fn run` + `pub type
  Battery` + everything that used to live in `src/main.rs`
  (signal handlers, argv parsing, install_batteries, test
  command).
- `src/bin/wat.rs` — NEW. Thin wrapper:

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

- `src/main.rs` — gone (moved into `src/lib.rs` + `src/bin/wat.rs`).
- `Cargo.toml` — `[lib]` block added; `[[bin]]` block stays;
  dependencies unchanged.

### Tests

- `tests/wat_cli.rs`, `tests/wat_test_cli.rs` — unchanged;
  they exercise the binary via `CARGO_BIN_EXE_wat` and the
  binary's behavior is unchanged.
- `tests/wat_arc100_public_api.rs` — NEW. Three smoke tests
  proving the public API is reachable, that workspace extension
  crates' `(register, wat_sources)` pairs type-check as `Battery`,
  and that subset / empty slices are valid.

---

## What does NOT change

- **Binary path.** Still `target/{debug,release}/wat`. The `[[bin]]
  name = "wat"` is unchanged.
- **CLI behavior.** Same argv shapes (`wat <entry.wat>` /
  `wat test <path>`), same signal handling, same exit codes.
- **Substrate library (`wat`).** Untouched — this is purely a
  refactor inside `crates/wat-cli/`.
- **Workspace structure.** `[workspace] members` and
  `default-members` unchanged.
- **Existing batteries.** The 5 workspace `#[wat_dispatch]`
  extension crates expose the same `register` + `wat_sources`
  functions they already had per arc 013's external-crate
  contract. No changes to them.

---

## Tradeoffs to acknowledge

### Tuple alias vs struct

The `Battery` type is a tuple alias rather than a `struct
Battery { register: ..., sources: ... }`. Tradeoff:

- **Tuple wins on call-site brevity.** `(crate::register,
  crate::wat_sources)` is shorter than `Battery::new(crate::
  register, crate::wat_sources)`.
- **Struct wins on field-name clarity.** `Battery { register:
  ..., sources: ... }` documents which fn is which without
  consulting the type alias.

Tuple chosen because the call site reads naturally — every
extension crate exposes `register` + `wat_sources` with those
exact names, so positional ordering matches the convention; no
ambiguity at the call site. If a third field gets added later
(say, an optional name/version), we'd promote to a struct then.

### `run` consumes argv from `std::env::args`

Not from a user-passed slice. This means `run` is hard to compose
with test harnesses that have their own argv. Tradeoff:

- **Pro:** keeps the API minimal; matches what every CLI binary
  actually wants to do.
- **Con:** the `wat_arc100_public_api.rs` tests can prove the
  Battery slice type-checks but can't actually invoke `run`
  cleanly (it'd consume the test runner's argv). The CLI
  integration tests (`wat_cli.rs`, `wat_test_cli.rs`) cover the
  end-to-end behavior by spawning the binary.

If a downstream consumer needs to call `run` with a controlled
argv (e.g., to embed wat-CLI semantics inside a larger Rust
binary), we'd add `run_with_argv(argv: &[String], batteries:
&[Battery])` as a sibling. Defer until a caller demands.

### No macro sugar

The user's `main.rs` is 5 lines, not 1. Tradeoff:

- **Pro:** no macro infrastructure; the API is a regular Rust
  function. Trivial to read, trivial to debug.
- **Con:** verbose vs. a hypothetical
  `wat_cli::main! { deps: [wat_telemetry, my_crate] }`.

Not adding the macro yet. The function is honest and direct; if
downstream consumers grow tired of typing the 5 lines, we add
the macro then. Premature sugar is unhelpful sugar.

---

## Slice plan

**Slice 1** — vend the API — *shipped 2026-04-29*.

- Convert `crates/wat-cli/` into a lib + bin.
- Public API: `pub type Battery` + `pub fn run`.
- Bin = thin wrapper calling `run` with the 5 workspace
  defaults.
- 3 smoke tests proving the API is reachable for downstream
  consumers.
- `cargo test --workspace` green.

**Slice 2** — docs — *shipped 2026-04-29*.

- This DESIGN.
- INSCRIPTION sealing the arc.
- USER-GUIDE addition: a "build your own batteries-included CLI"
  example showing the 5-line `main.rs` pattern.
- 058 FOUNDATION-CHANGELOG row in the lab repo.

---

## Predecessors / dependencies

**Shipped:**
- Arc 013 — `wat::main!` macro + external-crate contract
  (`wat_sources()` + `register()` signatures).
- Arc 015 — `wat::test_runner` library extraction.
- Arc 099 — wat-cli crate (this arc's structural prerequisite).

**Depends on:** arc 099 sealed first. This is its natural
follow-up.

## What this enables

- **Downstream wat CLIs** with custom battery sets — 5-line
  `main.rs`, no copy/paste.
- **Future `wat-interrogate` binary** — would be its own crate
  or another binary in `crates/wat-cli/src/bin/`, calling
  `wat_cli::run(&[...])` with the interrogation-specific
  batteries.
- **Lab repo CLI option.** If `holon-lab-trading` ever wants its
  own CLI binary alongside the existing `enterprise.rs` pattern,
  it's now a 5-line addition rather than reimplementing the
  argv / signal / subcommand machinery.

**PERSEVERARE.**
