# BRIEF — STONE 255.26: the debug build is red, and no floor could see it

**Drawn 2026-09-24 against `main` @ `8b0a5ad17`.** Release floor 6063/6063, clippy 0, census 215 non-zero,
delta **NEW 2 / RECOVERY 0**, ledger 215. **Executor tier: Opus.**

## The red — reproduced by the orchestrator on HEAD

```
$ cargo test --lib freeze::pass_order::tests::the_startup_passes_run_in_the_declared_order
thread '…' panicked at src/types.rs:1070:9:
builtin leaf :wat::core::Option already registered as a structured TypeDef
test result: FAILED. 0 passed; 1 failed
```

The arm is the first `debug_assert!` in `TypeEnv::register_builtin_leaf` (`src/types.rs` ~:1068):
`!self.types.contains_key(&name)`. `:wat::core::Option` is registered as a **structured** `TypeDef` (an
enum, through `register_builtin`) and then again as a **leaf**. 255.25a's executor measured the same
panic on `7b0cbcc20`, before its own stone. `check::tests::declared_stdlib_types_retracts_old_variant_singletons`
panics the same way. **Every debug-mode lib test that builds builtins is red.**

The floor is `cargo nextest run --release` (`scripts/floor.sh`), and release compiles `debug_assert!` out.
**No floor has been able to see this.** CLAUDE.md: *"A `debug_assert!` panic is a real failure — debug
surfaces conditions release compiles out, and 'it's only in debug' is the same dismissal wearing a
compiler flag."*

## The work

1. **Measure the whole red.** Run the debug lib and integration tests (`cargo nextest run`, no `--release`,
   captured whole like `floor.sh` does; add a flag or sibling script if one is needed, **measure first**).
   List every failing test and every distinct assertion that fired. Do not stop at the first.
2. **Find when it began.** Bisect the one reproducer over `git log`, e.g. from 255-builtin-registry's
   "storage option C" commit (the leaf door) to HEAD. Name the commit that registers a builtin both
   structurally and as a leaf. Look for the same bug in any other builtin (`Result`, the other sum types).
3. **Cure it at its cause.** A name is **either** a structured builtin **or** a leaf, never both. Decide
   which door each builtin belongs to, by what it is, and make the other door refuse it. Do not delete or
   weaken the assertion: it is the wall that found this. Its sibling (*"registered twice"*) stays too.
4. **Make the class unhideable.** Measure the cost of a debug pass over the lib tests (and the whole
   suite, if affordable), and **report the wall-clock**. Propose, but do not wire, how the floor should
   include it (a second `floor.sh` mode, a required gate, a subset). Wiring it into the floor is the
   builder's ruling.

## STOP triggers

1. More than 3 **distinct** debug-only assertion arms fire (after deduplicating by arm) → STOP, report
   them all verbatim with their first test. Each is a finding with its own cause.
2. The cure changes any **release** verdict (floor, census) → STOP, report it.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the reproducer in debug | passes after the cure; the pre-stone debug run fails (as shown above) |
| the debug red, whole | measured, listed by arm, then 0 after the cure (or STOP-1) |
| the commit it began at | named |
| the release floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` · NEW 2 / RECOVERY 0 · ≤ 215 |
| debug-pass cost | wall-clock reported; a floor proposal written, not wired |

Runtime prediction: 2–3 hours (debug builds are slow; measure one before multiplying).

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED** — a debug red included. Capture the whole log, never
  re-run to green, name the arm.
- Capture `rc=$?` on the next statement.
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add --
  <paths>`; **do not push.** Spawn no subagents. If you stop without committing, revert your own edits and
  save the patch to the scratchpad.
