# Arc 099 — `crates/wat-cli/` — extract the CLI into its own crate — INSCRIPTION

**Status:** shipped 2026-04-29.

The wat command-line runner moved from `src/bin/wat.rs` (inside
the substrate crate) into `crates/wat-cli/` as its own workspace
member. The substrate library is now library-only; wat-cli is the
canonical batteries-included consumer linking every workspace
`#[wat_dispatch]` extension. Any `.wat` script passed via argv
to the `wat` binary can use `wat-telemetry`, `wat-telemetry-sqlite`,
`wat-sqlite`, `wat-lru`, `wat-holon-lru` directly without authoring
a custom Rust binary.

**Predecessor of:** [arc 093](../093-wat-telemetry-workquery/DESIGN.md)
slice 4 (telemetry interrogation example scripts).

**Surfaced by:** user direction 2026-04-29 mid-arc-093 binary
discussion:

> "or... we do we need a wat-rs/crates/wat-cli/ to provide this?...
> we move the existing wat cli parts into a dedicated crate who has
> all the things wat-rs provides?..."

The arc closed in one slice + one doc commit on the same day.

---

## What shipped

### Slice 1 — Extract

`crates/wat-cli/` created as a workspace member. The pre-arc-099
`src/bin/wat.rs` was moved verbatim to `crates/wat-cli/src/main.rs`
with three additions:

1. **`install_batteries()` runs once at startup** — calls
   `wat::rust_deps::install` + `wat::source::install_dep_sources`
   for every linked extension via the same OnceLock mechanism
   `wat::compose_and_run` uses. First-caller-wins; safe in tests.

2. **The CLI binary's `Cargo.toml` declares the batteries** —
   wat-telemetry, wat-telemetry-sqlite, wat-sqlite, wat-lru,
   wat-holon-lru as path deps. The binary's `[[bin]]` entry names
   it `wat`, so the output path is unchanged at
   `target/{debug,release}/wat`.

3. **CLI integration tests moved alongside the binary** —
   `tests/wat_cli.rs` and `tests/wat_test_cli.rs` relocated to
   `crates/wat-cli/tests/`. They use `env!("CARGO_BIN_EXE_wat")`,
   which Cargo only sets for tests in the same crate as the
   binary. Path lookups for `wat-tests/` walk up two levels
   (`../../wat-tests/`) since wat-cli sits at `crates/wat-cli/`.

The substrate's `Cargo.toml` is unchanged (binary was auto-
discovered from `src/bin/`; nothing to remove from `[[bin]]`).
The workspace `[workspace] members` and `default-members` lists
gained `crates/wat-cli` so `cargo build` / `cargo test` /
`cargo clippy` continue to cover every crate.

### Slice 2 — Docs

This INSCRIPTION + the DESIGN that recorded the decision +
USER-GUIDE install path update + 058 FOUNDATION-CHANGELOG row in
the lab repo.

---

## Tests

`cargo test --workspace`: every existing test still green; the
two relocated CLI test files (15 tests total) pass under the new
crate. Release binary builds at 8.1MB (substrate ~5MB + rusqlite
bundled in wat-telemetry-sqlite).

Smoke test: `target/release/wat` prints usage and exits 64.
`target/release/wat <some.wat>` runs scripts including those
using telemetry/sqlite surfaces (the registration happens before
freeze; the script's keyword paths resolve to the linked shims
transparently).

---

## What's NOT in this arc

- **Removing the example crates.** `examples/with-lru/`,
  `examples/with-loader/`, `examples/console-demo/` continue to
  serve as proofs of the external-crate mechanism. They remain
  the canonical reference for "how do I link wat into MY binary,"
  separate from "how does the canonical CLI ship batteries."
- **A separate `wat-interrogate` binary.** Could ship later as
  another binary in `crates/wat-cli/src/bin/` or its own crate.
  Not needed yet — the bare `wat <script.wat>` is sufficient
  for arc 093's interrogation use case.
- **Reorganizing the lab's binary.** `holon-lab-trading` builds
  its own binary via `wat::main!` against the substrate. No
  change needed; that pattern still works as designed.

---

## Lessons

1. **Reversing arc 013's "no batteries in the wat CLI" stance.**
   Arc 013's choice was correct for its scope (proving the
   external-crate mechanism via downstream example binaries
   rather than coupling them to the CLI). When arc 091 + arc
   093 surfaced the "run interrogation scripts against `.db`
   files without authoring a binary every time" use case, the
   stance flipped naturally. The example crates still prove the
   external-crate mechanism — they remain the reference.
   *Original design decisions get revisited when the use case
   shifts; the substrate is honest about its own design history.*

2. **Crate-per-extension symmetry.** Every `#[wat_dispatch]`
   extension already lived in its own workspace member; the CLI
   was the only piece that broke that pattern. Pulling it out
   restored the symmetry — the workspace shape now reads as
   "one substrate library + N opt-in extensions + 1 batteries-
   included CLI consumer." Easier to explain, easier to extend.

3. **Process-global OnceLocks make the install ordering
   trivial.** `wat::rust_deps::install` and
   `wat::source::install_dep_sources` were already designed to
   accept a single first-caller-wins install. wat-cli's
   `install_batteries()` calls both at the top of `main()` and
   every subsequent call (test harnesses, sandboxes, fork-with-
   forms) inherits transparently. No coordination needed
   between the CLI and the freeze pipeline.

4. **`CARGO_BIN_EXE_*` is per-crate.** When integration tests
   reference the binary they test via `env!("CARGO_BIN_EXE_wat")`,
   Cargo only sets that env var for tests in the same crate as
   the binary. Moving the binary forced moving its tests too.
   Caught immediately (compile error, not a runtime failure),
   so cheap to fix.

5. **One-conversation arcs are real.** This arc went from "what
   do you think?" to shipped + sealed in a single conversation
   turn. The decision was small, the refactor was mechanical,
   and the DESIGN existed to RECORD the move rather than to
   resolve open questions. Not every arc needs the multi-day
   Q&A cycle — but every arc still gets the inscription so future
   readers don't have to reverse-engineer the choice.

---

## Surfaced by (verbatim)

User direction 2026-04-29:

> "the wat binary will have wat-sqlite-telemetry included or no?"

> "or... we do we need a wat-rs/crates/wat-cli/ to provide this?...
> we move the existing wat cli parts into a dedicated crate who has
> all the things wat-rs provides?..."

> "b - do it an[d] backfill the docs"

The arc closed when slice 1's `cargo test --workspace` came back
green and the docs landed on the same day. The substrate is what
the user said it should be when he named it.

**PERSEVERARE.**
