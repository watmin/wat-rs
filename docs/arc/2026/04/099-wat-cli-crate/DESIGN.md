# Arc 099 — `crates/wat-cli/` — extract the CLI into its own crate — DESIGN

**Status:** SETTLED — opened and closed in the same conversation
turn 2026-04-29. Decision was a small architectural refactor with
no design questions; this DESIGN exists to record the move so
future readers don't have to reverse-engineer "why is the wat
binary in `crates/wat-cli/` instead of `src/bin/`".

**Predecessor of:** [arc 093](../093-wat-telemetry-workquery/DESIGN.md)
slice 4 (the example interrogation scripts need a binary that
links wat-telemetry-sqlite — wat-cli is now that binary).

---

## The move

**Before:** `wat` binary lived at `src/bin/wat.rs` inside the
substrate crate. Per arc 013's then-prevailing stance, the binary
deliberately did NOT link any `#[wat_dispatch]` extension crates
(wat-telemetry, wat-sqlite, wat-lru, wat-holon-lru,
wat-telemetry-sqlite). Scripts using those surfaces had to be run
from a custom Rust binary that authored its own
`compose_and_run` invocation.

**After:** `wat` binary lives at `crates/wat-cli/src/main.rs` as
its own workspace member crate. The substrate library (`wat`)
is library-only — no `[[bin]]` entry, no signal handler module,
no argv parsing. The wat-cli crate is the canonical batteries-
included consumer: it depends on every workspace `#[wat_dispatch]`
extension and registers them at startup so any `.wat` file passed
via argv can use those surfaces directly.

```
wat-rs/
├── src/                      # substrate library only
├── crates/
│   ├── wat-cli/              # NEW — canonical CLI binary
│   │   ├── Cargo.toml        # depends on wat + 5 batteries crates
│   │   ├── src/main.rs       # moved from src/bin/wat.rs
│   │   └── tests/            # CLI-specific integration tests
│   ├── wat-telemetry-sqlite/
│   ├── wat-telemetry/
│   ├── wat-sqlite/
│   ├── wat-lru/
│   ├── wat-holon-lru/
│   └── wat-macros/
└── examples/
    ├── with-lru/             # mini-consumer; pre-arc-099 example
    └── ...
```

---

## Why move it

**The crate-per-extension pattern is consistent now.** Every
`#[wat_dispatch]` extension already lived in its own workspace
member; the CLI was the only piece that broke that pattern by
sitting inside the substrate crate. wat-cli completes the symmetry.

**The substrate library stays clean.** Downstream consumers
depending on `wat = { path = "..." }` no longer accidentally pull
in CLI machinery (`libc` for signal handlers, argv parsing, exit
code mapping). The substrate is what it claims to be — a library.

**Arc 093's interrogation UX gets a binary by construction.** The
user wants to run wat scripts against frozen `runs/pulse-*.db`
files. Pre-arc-099 that meant "author another binary." Post-arc-
099 the existing `wat <script.wat>` invocation works because
wat-cli already links wat-telemetry-sqlite at startup.

**Sibling binaries get a precedent.** A future `wat-interrogate`
(or any debugging tool) can be another binary in `crates/wat-cli/`
or its own crate following the same shape.

---

## What changes

### New crate

`crates/wat-cli/` — workspace member with one binary (`wat`).
Cargo.toml depends on `wat` (substrate) + the 5 batteries crates
+ `libc` (for signal handlers).

### Substrate-side deletions

- `src/bin/wat.rs` — gone. The directory `src/bin/` is removed.
- The substrate's `Cargo.toml` is unchanged (binary was auto-
  discovered from `src/bin/`; nothing to remove from `[[bin]]`).

### New batteries-installation step

`crates/wat-cli/src/main.rs::install_batteries()` runs once at
startup before any wat code:

```rust
let mut builder = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
wat_telemetry::register(&mut builder);
wat_sqlite::register(&mut builder);
wat_lru::register(&mut builder);
wat_holon_lru::register(&mut builder);
wat_telemetry_sqlite::register(&mut builder);
let _ = wat::rust_deps::install(builder.build());

let _ = wat::source::install_dep_sources(vec![
    wat_telemetry::wat_sources(),
    wat_sqlite::wat_sources(),
    wat_lru::wat_sources(),
    wat_holon_lru::wat_sources(),
    wat_telemetry_sqlite::wat_sources(),
]);
```

Both halves install via process-global OnceLocks (matches
`wat::compose_and_run`'s mechanism); first caller wins, so test
harnesses inside this binary that spin up their own world inherit
transparently.

### Test relocations

- `tests/wat_cli.rs` → `crates/wat-cli/tests/wat_cli.rs`
- `tests/wat_test_cli.rs` → `crates/wat-cli/tests/wat_test_cli.rs`

These tests use `env!("CARGO_BIN_EXE_wat")`, which Cargo only sets
for tests in the same crate as the binary. Path lookups for
`wat-tests/` walk up two levels (`../../wat-tests/`) since wat-cli
sits at `crates/wat-cli/`.

### Workspace updates

`[workspace] members` and `default-members` gain `crates/wat-cli`.
`cargo build` / `cargo test` / `cargo clippy` continue to cover
every crate.

---

## What does NOT change

- **Binary path.** Still `target/{debug,release}/wat`. The binary
  is named `wat` in wat-cli's `Cargo.toml`, so existing scripts
  / tooling that reference the path keep working.
- **Substrate-only consumers.** Anyone depending on `wat = { path
  = "..." }` for the library is unaffected; their build only
  pulls in the substrate crate.
- **Existing example consumers.** `examples/with-lru/`,
  `examples/with-loader/`, `examples/console-demo/` continue to
  use the substrate's `wat::main!` macro pattern. They're proofs
  of the external-crate mechanism for downstream consumers; they
  don't go through wat-cli.
- **Lab repo.** `holon-lab-trading` builds its own binary via
  `wat::main!` against the substrate. No change needed.
- **`wat::main!` macro.** Unchanged — it's for binaries that EMBED
  a specific `.wat` program, not run argv-supplied programs. wat-
  cli's main is bespoke (handles argv, subcommands, signal
  handlers, exit codes), same as the pre-arc-099 binary.

---

## Tradeoffs to acknowledge

### Binary size

The pre-arc-099 `wat` binary was substrate-only. Post-arc-099 it
links wat-telemetry-sqlite (which bundles rusqlite, ~3MB) plus the
other batteries. Release binary measured at 8.1MB (vs ~5MB pre).
Acceptable for a CLI tool.

### `cargo install wat`

If anyone had been running `cargo install --path .` against the
substrate to get the CLI, they'd now run `cargo install --path
crates/wat-cli`. No published-crate users yet (substrate is path-
only); the README install snippet updates to point at the new
crate.

### Reversal of arc 013's stance

Arc 013's wat CLI deliberately did NOT link external wat crates
("the proof stance" — proving the external-crate mechanism via
downstream example binaries). That stance was correct for arc
013's scope. With arc 091 (telemetry writer) and arc 093
(telemetry reader / interrogation) on the table, the user wants
to run interrogation scripts against `.db` files without
authoring a binary every time. The CLI-as-batteries-included
consumer is the natural shape; substrate vs. extension separation
is preserved by the crate boundary.

The example crates (`examples/with-lru/`, etc.) still serve as
proofs of the external-crate mechanism — they remain the
canonical reference for "how do I link wat into MY binary."

---

## Slice plan

**Slice 1** — extract — *shipped 2026-04-29*.

- Create `crates/wat-cli/` with Cargo.toml + src/main.rs.
- Move `src/bin/wat.rs` → `crates/wat-cli/src/main.rs`.
- Add `install_batteries()` step.
- Move CLI tests; adjust `wat-tests/` path lookups.
- Update workspace `members` + `default-members`.
- `cargo test --workspace` green.

**Slice 2** — docs — *shipped 2026-04-29*.

- This DESIGN.md.
- INSCRIPTION.md sealing the move.
- USER-GUIDE install path update.
- 058 FOUNDATION-CHANGELOG row in lab repo.

---

## Predecessors / dependencies

**Shipped:**
- Arc 013 — external-crate mechanism (`wat::main!` macro,
  `wat_sources()` + `register()` contract, `compose_and_run` +
  process-global OnceLocks for dep install).
- Arc 015 — `wat::test_runner` library extraction (so the CLI
  binary's `wat test` command and `wat::test!` macro share one
  codepath; the CLI is just argv → test-runner-call mapping).
- Arc 091 — telemetry writer side (creates the run.db files arc
  093's interrogation scripts will query).
- Arc 097 — Duration helpers (`hours-ago` etc.; arc 093's
  worked-example queries depend on them).
- Arc 098 — Clara-style pattern matcher (`:wat::form::matches?`;
  arc 093 slice 4's predicate language).

**Depends on:** nothing else. Pure refactor + workspace shape change.

## What this enables

- **Arc 093 implementation can begin.** wat-cli is the binary
  that runs interrogation scripts; the SQLite reader-handle +
  Stream sources land as wat-telemetry-sqlite extensions
  consumed automatically.
- **Sibling binaries.** A future `wat-interrogate` (specialized
  for `.db` interrogation with bake-in conveniences like
  `argv[1] -> reader-handle`) becomes another binary in
  `crates/wat-cli/src/bin/` or its own crate.
- **Substrate refactors.** With the binary out of the substrate,
  changes to library internals don't risk breaking the CLI's
  signal-handler / argv code; the boundary is enforced by the
  crate split.

**PERSEVERARE.**
