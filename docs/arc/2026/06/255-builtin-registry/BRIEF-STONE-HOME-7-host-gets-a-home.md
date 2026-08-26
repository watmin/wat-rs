# BRIEF — HOME #7: `src/host/`

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-HOME-7-host-gets-a-home.md` — read it whole.
THE CAST that named it: `wat-scripts/intueri/rust-entry-surface-naming.rs.intueri`.

PRIOR ART: **HOME #5** (`8ddccaaa3`, `src/edn/`) and **HOME #6** (`f0cd8bed1`, `src/load/`). Read
both commit messages — the traps below were each discovered the expensive way in one of them.

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first. **Ending your turn ENDS you** — every
command in the FOREGROUND, blocking. **You may not spawn sub-agents.** Do not commit, push, stash,
revert, or `git checkout`. There is a `git stash@{0}` that must never be touched. **Use `git mv`.**

You may run `cargo build --release` and `cargo build --release --all-targets`, and single named
tests. **Not** the full floor, **not** clippy.

---

## The work in one paragraph

Three loose files at `src/` root are the Rust side that **hosts** wat — the surface a Rust program
uses to run it. Give them `src/host/`. A move and a path rename: **15 occurrences**, the smallest of
the three home moves. Then fix two verified lying doc comments in the files you are moving.

```
src/compose.rs      ->  src/host/compose.rs
src/harness.rs      ->  src/host/harness.rs
src/test_runner.rs  ->  src/host/test_runner.rs
src/panic_hook.rs   ->  ⛔ UNTOUCHED. Stays at root. See the design for why.
```

```
crate::compose::X ->  crate::host::compose::X      (same shape for harness, test_runner)
::wat::harness::X ->  ::wat::host::harness::X
```

⛔ **NO `pub use` in `src/host/mod.rs`** — third stone running, same ruling. `lib.rs`'s existing
crate-root re-exports get **retargeted**, never widened. STOP-2.

⚠ **Filenames do not change.** The ward found `harness` a Level 1 lie and `compose` a mumble, and
both are recorded — but renaming `Harness` is a public API change with external consumers. Files
move whole. If you find yourself renaming a type or a file, that is STOP-1.

---

## ⛔ THE THREE TRAPS — all measured, all yours to hit deliberately

**TRAP 1 — the bare crate-root re-export.** `lib.rs:115` and `lib.rs:144`:

```
pub use compose::{compose_and_run, compose_and_run_with_loader};
pub use harness::{Harness, HarnessError, Outcome};
```

These name their target **unprefixed**, so no grep for `crate::`/`wat::` finds them — only the
compiler. HOME #5 hit this once, HOME #6 twice. **Three instances over two stones.** Expect it.

**TRAP 2 — relative `include_str!`.** HOME #6 hit 54. Measured here: **one**, in a doc example
(`//!     source: include_str!("program.wat"),`) — a snippet, not a real path. Confirm it needs no
change rather than assuming.

**TRAP 3 — the proc-macro EMITS these paths as text, and nothing in `wat-rs` type-checks it.**

```
crates/wat-macros/src/lib.rs:508,517,534   ::wat::harness::HarnessError
crates/wat-macros/src/lib.rs:891           ::wat::test_runner::run_single_deftest(
crates/wat-macros/src/lib.rs:365,576       the same paths in comments
```

The macro *writes these strings*. They fail only when it is **expanded** in a consumer.
`examples/console-demo` and `examples/with-loader` expand `wat::main!` / `wat_test!`, so
**`--all-targets` reaches them and a plain `cargo build --release` does not.** Report which build
produced each number.

---

## ⚠ THEN FIX TWO VERIFIED LYING DOCS — comments only, zero behaviour

Both were checked against the disk by the orchestrator. Both are in files you are moving.

**1. `harness.rs` — `Harness::run`'s public doc.** It says:

> *"Invoke `:user::main` with pre-seeded stdin lines and return captured stdout + stderr."*

The body is `let _ = stdin;` and two hardcoded `Vec::new()`. A thorough WHY exists **inside the body**
(the arc 170 slice 1e/1f comment) — and the doc-reader never sees it. **Fold that WHY into the doc
comment** so the public surface states what the function does: currently a no-op for stdio, real
capture pending slice 1f, use the wat-cli path meanwhile.

⚠ The module doctest (around lines 22–37) asserts `out.stdout == vec!["hello"]` and is marked
`no_run` — **it would fail if run.** Make it honest: either drop the assertion or mark it plainly
aspirational. `no_run` hides a lie from `cargo test`, not from a reader.

**2. `test_runner.rs` — the primary public doc on `run_tests_from_dir`.** It states discovery rule 1
as *"The path's final `::`-segment starts with `test-`."* `is_test_function`'s own doc says that
filter **"has been dropped"** (2026-04-25) — discovery is purely by signature. Two contradicting
docs in one file, the stale one on the entry point. Fix the entry-point doc to match the code, and
check whether the same stale rule is restated anywhere else in the file.

**Do NOT fix the other ward findings** — the dead `StdioSnapshot` variant, `source_has_config_setter`'s
name, the `failure_to_diagnostic` doc-link. They are named as their own stone in the design. STOP-3.

---

## STOP triggers — each means SHIP NOTHING and report

1. **STOP-1 — renaming a type, a file, or anything beyond a path.** Files move whole.
2. **STOP-2 — adding a `pub use` to `src/host/mod.rs`.**
3. **STOP-3 — fixing a ward finding this brief did not name.** Out of scope; report the pressure.
4. **STOP-4 — `src/panic_hook.rs` needs to move for the build to pass.** It is deliberately staying;
   if the compiler disagrees, that is a finding about the family and I want it.
5. **STOP-5 — a room's line number does not hold what this brief says.** Written against `0c1667524`.

---

## Acceptance you can check yourself

```bash
ls src/*.rs | wc -l                     # 27 at HEAD -> 24
ls src/host/                            # mod.rs compose.rs harness.rs test_runner.rs
test -e src/panic_hook.rs && echo "panic_hook still at root — correct"
grep -n 'pub use' src/host/mod.rs       # -> nothing
git ls-files '*.rs' | xargs grep -n 'crate::compose::\|crate::harness::\|crate::test_runner::\|wat::compose::\|wat::harness::\|wat::test_runner::' | grep -v 'host::'   # -> nothing
cargo build --release --all-targets     # ALL targets — TRAP 3 lives beyond the lib
```

⚠ Validate any pattern you invent before quoting its count. Three of my censuses today returned
confident wrong numbers — `\|` inside a `grep -E` (a literal pipe, not alternation); grepping `wat::`
across `src/` too and calling it external; and `grep -c` (lines) against a total built with `grep -o`
(occurrences). Positive-control against a file you know carries a hit.

## Report back with

- The cascade's waterfall, and **for each number, whether it came from `--all-targets` or the lib**.
- The acceptance checks above, after.
- Confirmation the three moves show as renames and `panic_hook.rs` is untouched.
- **The exact before/after text of both doc fixes.** These are honesty repairs; I want to read them.
- **Every site you edited that was not a path rename or one of the two named doc fixes**, with
  `file:line`.
- Anything the brief got wrong.
- What you did NOT do, and why.
