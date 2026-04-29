# Arc 101 — kill the `wat test <path>` CLI subcommand — DESIGN

**Status:** SETTLED — opened and closed in the same conversation
turn 2026-04-29 immediately after arc 100 sealed. Decision was a
small surface deletion with no design questions; this DESIGN exists
to record the removal so future readers don't ask "why did the
CLI lose its test subcommand?"

**Predecessor:** [arc 015](../../015-wat-test-for-consumers/INSCRIPTION.md)
— `wat::test!` macro shipped, giving consumers a cargo-native
path to run wat tests.

**Predecessor of:** any future CLI surface decision. The CLI is
now single-purpose (run one program); subcommands are someone
else's problem.

---

## The deletion

**Before (post-arc-100):** `wat_cli::run` dispatched two
invocations:

```text
wat <entry.wat>      # run a program
wat test <path>      # run tests — file or directory
```

The test subcommand walked the path looking for `.wat` files,
discovered `test-`-prefixed `() -> :wat::kernel::RunResult`
defines inside each, shuffled them, ran them, printed
cargo-style summary, exited 0/1/64. Same machinery as
`wat::test!` — just reachable through argv instead of cargo.

**After:** `wat_cli::run` dispatches one invocation:

```text
wat <entry.wat>      # run a program
```

Wat tests run via `cargo test` against a Rust crate that uses
the `wat::test!` macro. The macro's runtime arm
(`wat::test_runner::run_and_assert`) is the same library code
the dropped subcommand called — the directory walk + per-test
invocation logic stays public; only the CLI wrapper goes away.

---

## Why kill it

**`wat::test!` was already the canonical path.** Every test in
the workspace — substrate, lab, examples — runs via
`cargo test` going through `wat::test!`. The CLI subcommand was
a duplicate surface that pre-dated the macro. Two paths to the
same thing means inconsistent ergonomics: which is canonical?
Where does an author look first? `cargo test` already won —
formalize it.

**`cargo test` composes with the rest of the Rust toolchain.**
`--release`, `RUST_BACKTRACE`, `--test-threads`, filter
expressions (`cargo test some-pattern`), per-test
backtraces, deterministic ordering with `--test-threads=1`,
profile-guided optimization, code coverage tooling, IDE test
runners (rust-analyzer's "Run Test" code lens) — all of these
work through cargo. The CLI subcommand had a fraction of these
features and would have to reimplement each one.

**Single-purpose CLI is honest.** The `wat` binary now has one
job: run an entry `.wat` file. Argv, signal handlers, exit
codes all line up against that single shape. No subcommand
dispatch, no usage-line ambiguity, no "but what does `wat test`
do that `cargo test` doesn't?" question to answer.

**Arc 099/100 set up the cleanup.** When the CLI lived inside
the substrate crate (pre-arc-099), pulling out the test
subcommand would have been a substrate-shape change. With the
CLI in its own crate (post-arc-099), the deletion is local to
`crates/wat-cli/`. Now is the cheap moment.

---

## What changes

### Code deletion

- `wat_cli::run` — drop the `argv[1] == "test"` branch. Drop
  the second usage-line print. Tighten to single-shape `wat
  <entry.wat>`.
- `crates/wat-cli/src/lib.rs` — drop `fn run_tests_command`,
  `TEST_EXIT_OK` / `TEST_EXIT_FAILED` / `TEST_EXIT_NO_TESTS`
  constants, `use wat::test_runner::run_tests_from_dir` import.
- `crates/wat-cli/tests/wat_test_cli.rs` — delete entire file
  (15 integration tests covered the dropped subcommand).

### Code preserved

- `wat::test_runner::run_tests_from_dir` /
  `run_tests_from_dir_with_loader` — consumed by
  `run_and_assert` (the `wat::test!` macro's runtime arm).
- `wat::test_runner::run_and_assert` /
  `run_and_assert_with_loader` — the macro entry point.
- `wat::test_runner::Summary` / `FailureSummary` /
  `print_summary` / discovery / shuffling — used by the same
  macro path.

The library API stays. Only the CLI wrapper goes.

### Doc updates

- USER-GUIDE §1's "The bundled `wat` CLI" subsection: drop the
  two-shape usage block; tighten to single shape; cross-reference
  §13 + arc 101 for the test path.
- USER-GUIDE §1's "Build your own batteries-included CLI"
  subsection: drop the `vs wat test <path> dispatch` mention from
  the "what `run` does" list.
- USER-GUIDE §13's testing chapter: drop the "CLI equivalent
  bypasses cargo entirely" block + the "under `wat test` CLI"
  conditional in the failure-output section.
- USER-GUIDE §15's appendix list: note that arc 007's CLI
  subcommand was retired in arc 101.
- `docs/README.md` arc 007 entry: same retirement note.

### Behavior change

`wat test <path>` is now a usage error (exit 64) instead of a
test-runner invocation. Anyone calling it gets the
single-line usage hint pointing at `wat <entry.wat>` only. No
backwards-compatibility shim, no deprecation period — the
substitute (`cargo test`) is universally available, has been
the canonical path for arcs.

---

## What does NOT change

- **`wat <entry.wat>` semantics.** Unchanged. Same startup
  pipeline, same signal handlers, same exit codes (0 / 1 / 2 /
  3 / 64 / 66).
- **`wat::test!` / `wat::test_suite!` macros.** Unchanged. Still
  the canonical way to run wat tests.
- **`wat::test_runner` library.** Public API unchanged. The
  directory walk + per-test invocation machinery stays; only
  the CLI wrapper goes.
- **`runtime`, `freeze`, `panic_hook`, `harness`.** Untouched.
- **Lab repo.** `holon-lab-trading`'s testing already runs via
  `cargo test` against the macro; nothing to change.
- **Substrate's own tests.** `wat-tests/` directory of `.wat`
  files is consumed by the substrate's own `wat::test!`-using
  integration tests, not by the dropped CLI subcommand. Stays.

---

## Tradeoffs to acknowledge

### Loss of "ad-hoc directory test runs without cargo"

The CLI subcommand's one unique value-add: running a directory
of `.wat` files outside any Rust crate. Anyone who wanted that
now needs:

- a Rust crate with a `tests/` dir,
- a `wat::test!` invocation pointing at the wat sources, and
- `cargo test`.

Three things instead of one CLI invocation. Tradeoff:

- **Pro:** the three things are already the canonical pattern;
  every test in the workspace already follows them.
- **Con:** an author with a folder of `.wat` files and no Rust
  crate has to scaffold the crate first.

The `examples/with-lru/` template is exactly this scaffold —
copy it, swap the wat sources, run `cargo test`. The
scaffolding cost is one-time per project, vs. the
duplicate-surface cost being permanent.

### Historical doc references

Arc 007's INSCRIPTION + DESIGN + BACKLOG, arc 016's, arc 017's
— all reference `wat test <path>` as a thing that exists. We
leave those alone (arc folders are sealed history). The
live docs (README, USER-GUIDE) get updated to reflect current
state, with cross-references to arc 101 where the retirement is
recorded.

### Removal vs deprecation

No deprecation period. Hard removal. Justification: the
substitute (`cargo test`) is universally available; the CLI
subcommand has no exposed-to-end-users contract (unlike a
published library API); the workspace is the only known
consumer; every test in the workspace already uses
`cargo test`. Zero callers to migrate.

---

## Slice plan

**Slice 1** — kill — *shipped 2026-04-29*.

- `wat_cli::run` loses the test branch + run_tests_command.
- `crates/wat-cli/tests/wat_test_cli.rs` deleted.
- Usage line tightens to `wat <entry.wat>`.
- `cargo test --workspace` green; binary smoke-tested.

**Slice 2** — docs — *shipped 2026-04-29*.

- This DESIGN.
- INSCRIPTION sealing the arc.
- USER-GUIDE updates (§1 + §13 + §15).
- README.md update.
- 058 FOUNDATION-CHANGELOG row in the lab repo.

---

## Predecessors / dependencies

**Shipped:**
- Arc 007 — introduced `wat test <path>` CLI alongside the
  `:wat::test::*` stdlib (this arc retires the CLI half; the
  stdlib survives).
- Arc 015 — `wat::test!` macro for cargo-native consumer tests.
- Arc 016 — wat-located failure messages (lessons fed both the
  CLI and the macro paths; both sides used the same panic-hook).
- Arc 017 — `loader:` parameter for `wat::test!` (extended the
  macro's surface; CLI didn't follow because cargo already
  supplies the equivalent via `loader: "wat"` etc.).
- Arc 099 — wat-cli extracted into its own crate (this arc's
  structural prerequisite; deletion is now local).
- Arc 100 — wat-cli library API vended (the single-shape `run`
  signature this arc tightens around).

**Depends on:** arc 099 + arc 100 sealed first. Both did. Pure
deletion now.

## What this enables

- **Cleaner story for "how do I test wat?"** One answer:
  `cargo test` via `wat::test!`. No CLI subcommand to maintain
  parity with.
- **Future CLI surface decisions.** Dropping the existing
  subcommand makes future "should the CLI grow `wat foo`?"
  questions easier — the answer defaults to NO; CLI is
  single-purpose by design.
- **Future binary cleanup.** The dropped 175-ish lines of
  argv/test-runner-wrapper code reduce the CLI's surface area
  enough that hand-auditing it during arc 093's interrogation
  binary work is now trivial.

**PERSEVERARE.**
