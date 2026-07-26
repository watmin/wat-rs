# BRIEF — fold `wat-cli` into core as `wat::distribution`

> **Read `DESIGN-wat-cli-into-core.md` first** (same directory). It holds the capability statement,
> the measured scope, and the reasoning. This brief is the strike.
>
> **The capability, so it survives every summarisation:** a third party can roll their own wat
> distribution — their own crate, their own `#[wat_dispatch]` extensions, their own binary — composed
> against wat core without forking it. `run` + `Battery` are **published surface**, not internals.
> They have no in-tree consumer *by design*, because we absorbed our own batteries into core.

## The public path — RATIFIED, do not re-open

```rust
wat::distribution::run(&[...])      // was wat_cli::run
wat::distribution::Battery          // was wat_cli::Battery
```

intueri-cast and weighed. `distribution` is the builder's own word for the capability
(*"roll their own wat distribution"*), so a stranger does not translate between what they were told
and what the API says. `cli` was rejected: it names the *implementation* (argv parsing) about a thing
that is deliberately an extension point, and its one advantage — matching a `src/cli.rs` — is not a
real constraint, since the public path and the file layout need not agree.

**Name the file for the public path** (`src/distribution.rs`), rather than keeping `cli.rs` and
re-exporting. One name, not two.

## What ships

```
crates/wat-cli/src/lib.rs           → src/distribution.rs
crates/wat-cli/src/staleness.rs     → src/distribution/staleness.rs
crates/wat-cli/src/bin/wat.rs       → src/bin/wat.rs
crates/wat-cli/src/bin/cargo-wat.rs → src/bin/cargo-wat.rs
crates/wat-cli/tests/wat_cli.rs     → tests/cli/  + a [[test]] target
Cargo.toml                          + two [[bin]] entries, − the wat-cli member
crates/wat-cli/                     DELETED
```

Measured: 5 files, 1,133 source lines + 735 test lines, **16 tests**. **No new dependencies** —
`wat`, `wat-edn`, `libc`; core already has all three. **No dependency cycle** — `wat-cli` → `wat`,
never the reverse, so the edge simply disappears.

## ★ The owed guard — a SYNTHETIC battery fixture (build it, this is not optional)

Stone 5 (`83093431`) deleted the last two things that exercised the battery path, so `Battery` and
`run` currently ship with **zero traffic**. That is not an acceptable end state for a stated
capability — a wall with no traffic stops being a wall.

Land, in `tests/cli/`, a fixture that proves the extension shape **without depending on any real
extension crate**: two local functions with the signatures a `#[wat_dispatch]` crate exposes,
coerced into a `&[Battery]`. The assertion is the **compile** — if `Battery` moves, loses visibility,
or its pair signature drifts, this stops building.

**Do not reproduce the vacuous part.** The deleted `wat_arc100_public_api.rs` asserted
`assert_eq!(slice.len(), 2)` on a two-element literal — an assertion that could not fail. Ground the
real signatures from `src/rust_deps/cache.rs`'s `#[wat_dispatch]` usage (still live in core) so the
synthetic pair matches what the macro actually emits.

## Read in order

1. **`crates/wat-cli/src/lib.rs`** — `run`, `Battery`, and the doc comments that must now say
   *published surface for third-party distributions* outright. A future reader finding it unused
   in-tree must be told, at the definition site, that this is expected.
2. **`crates/wat-cli/src/staleness.rs`** — 369 lines. Note the `--check-output edn|json`
   suppression: machine-readable pipelines must stay clean. **That suppression is load-bearing;
   preserve it.**
3. **`crates/wat-cli/src/bin/{wat,cargo-wat}.rs`** — both now call `run(&[])` after Stone 5. The
   `[[bin]] name =` values are **FIXED**: `cargo wat` resolves only because the binary is literally
   named `cargo-wat`.
4. **`Cargo.toml`** — `[lib] name = "wat"` at :57, per-directory `[[test]]` targets from :122. Core
   has **no `[[bin]]` yet**; you are adding the first two.
5. **`crates/wat-cli/tests/wat_cli.rs`** — 735 lines, 16 tests. Five are real CLI-startup coverage
   (`presence_proof_hello_world`, `echo_program_reads_stdin_writes_stdout`,
   `missing_user_main_rejected`, `programs_are_atoms_hello_world`) plus
   `sigterm_to_cli_cascades_via_polling_contract`, **a known timing flake** — if that one alone is
   red, run it isolated before concluding anything.

## STOP triggers — rejection criteria

1. **If any of the 16 tests cannot move without changing what it asserts** — STOP and name it. A
   test that had to be weakened to relocate is a lost proof, not a moved one.
2. **If the `cargo wat` subcommand stops resolving** — STOP. The bin name is the mechanism.
3. **If the synthetic battery fixture cannot be written without a real extension crate** — STOP and
   report what the signature actually requires. That would be a finding about `Battery`'s shape.
4. **If the blast radius reaches `wat/` stdlib or the substrate proper** (`runtime.rs`, `check.rs`,
   `types.rs`) — STOP. This is a relocation; it should not reshape anything that stays.

## Method

- `cargo build --release --workspace` freely — and note the two new bins must actually build.
- **A narrow filtered `cargo test --release --test <target> -- <filter>` is encouraged.** Only the
  full-floor `cargo nextest run` stays the orchestrator's.
- After any `wat/` change (unlikely here), the load-order gate must print `[]`:
  ```clojure
  (:wat::core::defn :user::main [] -> :wat::core::nil
    (:wat::kernel::println (:wat::deporder::verify-stdlib)))
  ```
- **Expect the floor to HOLD at 4160**, plus whatever the synthetic fixture adds. A move should not
  change the count; a drop means a test was lost in transit. State your expected number.
- Foreground only. Do not commit.

## Your report

The diff shape; confirmation all 16 tests moved with their assertions **unchanged**; the synthetic
fixture and what its compile actually proves; that both binaries build and `cargo wat` still
resolves; your expected floor; any STOP.
