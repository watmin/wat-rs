# Arc 101 — kill the `wat test <path>` CLI subcommand — INSCRIPTION

**Status:** shipped 2026-04-29.

The `wat test <path>` subcommand is gone. `cargo test` via
`wat::test!` is the canonical path for running wat tests. The
`wat` binary is now single-purpose: it runs one entry `.wat`
file. Two arcs (099 + 100) reshaped the CLI; this third arc
trimmed it down to its single honest job.

```text
wat <entry.wat>      # run a program — the only shape
```

**Predecessors:** [arc 099](../099-wat-cli-crate/INSCRIPTION.md)
+ [arc 100](../100-wat-cli-public-api/INSCRIPTION.md). Arc 099
extracted the CLI; arc 100 vended its guts as a library API;
arc 101 deleted the duplicate test path. Three arcs in one
conversation, all sealed same day.

**Surfaced by:** user direction 2026-04-29 minutes after arc 100
sealed:

> "we dropped the test functionality from the cli, right? all
> it does is run exactly one :user::main providing file?.."

> "kill it - cargo is how you test"

The arc closed in two slices same day.

---

## What shipped

### Slice 1 — Kill

`crates/wat-cli/src/lib.rs`:

- Dropped the `argv[1] == "test"` branch in `wat_cli::run`.
- Dropped `fn run_tests_command` (CLI wrapper around
  `wat::test_runner::run_tests_from_dir`).
- Dropped the `TEST_EXIT_OK` / `TEST_EXIT_FAILED` /
  `TEST_EXIT_NO_TESTS` constants.
- Dropped the `use wat::test_runner::run_tests_from_dir`
  import.
- Tightened the usage line from two-shape (`wat <entry.wat>` +
  `wat test <path>`) to single-shape (`wat <entry.wat>`).
- Updated module doc to point at the `wat::test!` macro for
  the test path and call out that arc 101 retired the duplicate
  CLI subcommand.

`crates/wat-cli/tests/wat_test_cli.rs` — deleted entirely. 15
integration tests covered the dropped subcommand; no replacement
needed because the underlying machinery
(`wat::test_runner::run_and_assert`) is exercised by every
`wat::test!`-using crate in the workspace.

`wat::test_runner` library — UNTOUCHED. `run_tests_from_dir`,
`run_tests_from_dir_with_loader`, `run_and_assert`,
`run_and_assert_with_loader`, `Summary`, `FailureSummary`, and
the discovery / shuffling / formatting all stay. The macro's
runtime arm depends on them; only the CLI wrapper went.

`cargo test --workspace` green. Binary at `target/release/wat`
prints the single-line usage on bare invocation, exits 64.

### Slice 2 — Docs

This INSCRIPTION + the DESIGN that recorded the deletion +
USER-GUIDE updates (§1's "bundled CLI" subsection, §1's "build
your own CLI" subsection, §13's testing chapter, §15's appendix
list) + `docs/README.md` arc 007 entry update + 058
FOUNDATION-CHANGELOG row in the lab repo.

Live docs reflect the current state; sealed arc folders (007,
016, 017 etc.) keep their historical references intact —
they're history, not current API.

---

## Tests

`cargo test --workspace`: every existing test still green; the
deleted `tests/wat_test_cli.rs` removed 15 integration tests
that exercised the dropped subcommand. The library code those
tests indirectly covered (test discovery, shuffling, output
formatting) is still exercised by every `wat::test!`-using
crate in the workspace, so coverage didn't drop in any
meaningful sense — same code, different harness.

Binary smoke: `target/release/wat` prints
`usage: target/release/wat <entry.wat>` on bare invocation,
exits 64. `target/release/wat test wat-tests/` (the dropped
subcommand syntax) now hits the "wrong arity" branch and
prints the same usage line — honest "this isn't a thing
anymore" feedback.

---

## What's NOT in this arc

- **Removing `wat::test_runner::run_tests_from_dir` / friends.**
  The library API stays; the `wat::test!` / `wat::test_suite!`
  macros consume it. Only the CLI wrapper went.
- **Deprecation period.** Hard removal. The substitute
  (`cargo test`) is universally available; the CLI subcommand
  has no exposed-to-end-users contract; every test in the
  workspace already uses `cargo test`. Zero callers to
  migrate.
- **Removing arc 007's other deliverables.** Arc 007 shipped a
  bundle: `:wat::test::*` stdlib, `run-sandboxed`, AST-entry
  sandbox, the `deftest` defmacro, panic-and-catch assertions,
  AND the CLI subcommand. Only the last one is going. The
  rest are alive and used.
- **Updating sealed arc folders' references to `wat test
  <path>`.** Sealed history stays sealed. The live docs reflect
  current state.

---

## Lessons

1. **"How do I test wat?" should have one answer.** Pre-arc-101
   the answer was *"cargo test if you have a Rust crate, or
   `wat test <path>` from the CLI for ad-hoc directory runs."*
   Two paths, two surfaces, two ergonomics. The user spotted
   the duplication mid-conversation and directed *"kill it —
   cargo is how you test."* The substrate is more honest with
   one answer than with two.

2. **Cargo composition is real composition.** `cargo test`
   wasn't picked because it's prettier than the CLI subcommand
   — it was picked because it composes with `--release`,
   `RUST_BACKTRACE`, `--test-threads`, filter expressions,
   IDE test runners, code-coverage tooling, and every other
   piece of the Rust toolchain. The CLI subcommand had a
   fraction of those features and would have to reimplement
   each one. Rather than build a parallel test surface, lean
   on the one that already won.

3. **Single-purpose CLIs are honest.** `wat <entry.wat>` —
   one shape, one job. Argv parsing, signal handlers, exit
   codes, usage messages all line up. Subcommand dispatch,
   second usage-line branch, "did you mean..." disambiguation
   logic, all gone. The reduced surface area is its own
   benefit.

4. **Hard removal beats deprecation for non-public surfaces.**
   The CLI subcommand had no exposed-to-end-users contract
   (no published library API, no documented shell-out from a
   stable consumer). Every consumer in the workspace already
   used `cargo test`. Zero migration burden. Hard removal
   ships in one commit; a deprecation period would have meant
   six months of "the CLI subcommand exists but please don't
   use it" comments rotting in source. *Reach for hard
   removal when the migration burden is zero and the
   substitute is universally available.*

5. **Three arcs in one conversation.** 099 → 100 → 101, same
   day, same conversation, three INSCRIPTIONs. Each arc had
   one clear architectural decision: extract the CLI, vend
   its guts, kill its dead surface. The discipline is "every
   architectural decision gets an INSCRIPTION" — not "every
   conversation gets one arc." Future readers see three
   focused arcs rather than one sprawling one. *When the next
   prompt names a related-but-distinct concern, that's a new
   arc, even if it lands minutes later.*

---

## Surfaced by (verbatim)

User direction 2026-04-29:

> "we dropped the test functionality from the cli, right? all
> it does is run exactly one :user::main providing file?.."

> "kill it - cargo is how you test"

The arc closed when slice 1's `cargo test --workspace` came
back green and slice 2's docs landed minutes later. The
substrate is what the user said it should be when he named it.

**PERSEVERARE.**
