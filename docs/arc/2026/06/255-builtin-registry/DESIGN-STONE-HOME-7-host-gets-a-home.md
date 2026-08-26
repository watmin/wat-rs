# DESIGN — HOME #7: `src/host/` — the Rust side that hosts wat

> **Builder, 2026-08-25:** *"intueri names all"* → four casts → *"src/host/ - we build"*.
>
> The cast and its full findings: `wat-scripts/intueri/rust-entry-surface-naming.rs.intueri` (`0c1667524`).

## THE NAME WAS RULED BY THE WARD, NOT CHOSEN

Four independent casts of `intueri`, one per file, none seeing the others.

```
harness's read      `host` FIRST — "I'd have proposed it independently before seeing it listed"
test_runner's read  `host` FIRST — "my strongest independent pick"
compose's read      preferred `bootstrap`; rated host/driver "plausible... imported labels"
panic_hook's read   "the least dishonest of the set" (while arguing it is not a member)
```

`host` is the term-of-art for the program embedding an interpreter, and the one word that does not
fracture between the two consumers — a Rust binary embedding wat as a library, and the `wat` CLI
running a test file. **`embed`, the orchestrator's own pick, was killed three ways**, most sharply
by `compose`'s reader: that module's central argument is that it exists *by contrast* with
embedding, so `embed/` would hold the one file whose stated reason for existing is *"I am not
embedding."*

## ★ THE FAMILY IS THREE, NOT FOUR — AND THE WARD CONVICTED THE SELECTION METHOD

```
src/compose.rs      202   ->  src/host/compose.rs
src/harness.rs      211   ->  src/host/harness.rs
src/test_runner.rs 1215   ->  src/host/test_runner.rs
src/panic_hook.rs   426   ->  STAYS PUT
```

`panic_hook` scored **highest** of the four on the cohesion metric this campaign has used all
along — pulled by *both* `compose` and `test_runner`. Its own reader explained why that is
backwards:

> *"the cohesion is real, but it is a CONSUMER relationship — call `install()` once so failures
> print correctly while you run something — not a shared-subject one. **The file never drives
> anything.**"*

★★ And `test_runner`'s reader, independently and without seeing that:

> *"lines 620–975 are not running tests, they are shaping a failure into a structured diagnostic
> envelope … nearer to what `panic_hook.rs` is described as doing."*

**Two files, two readers, neither seeing the other, converging on a latent domain nobody had
named:** failure-reporting, holding `panic_hook.rs` entirely and ~350 lines currently inside
`test_runner.rs`. It is not this stone. It is named so it is a work item.

⚠ **The methodological finding matters more than the stone.** Every family this campaign picked was
chosen by one test — *do these files reference each other?* Reference-cohesion is not shared domain,
and the method had no test that told them apart. **A file that everything configures scores highest
and belongs least.**

## THE FORM — thin `mod.rs` plus the three, per the house

`edn/` 49 · `value/` 51 · `kernel/` 66 · `rete/` 83 · `collection/` 130.

```
src/host/mod.rs          the home (thin)
src/host/compose.rs      compose_and_run + the wat::main! macro's runtime half
src/host/harness.rs      the Harness facade — wat as a guest in a Rust process
src/host/test_runner.rs  discovery, invocation, and the reporting contract for .wat suites
```

```
crate::compose::X      ->  crate::host::compose::X
crate::harness::X      ->  crate::host::harness::X
crate::test_runner::X  ->  crate::host::test_runner::X
::wat::harness::X      ->  ::wat::host::harness::X          (and so on)
```

⛔ **NO re-exports in `src/host/mod.rs`** — third stone running, same ruling. `lib.rs`'s existing
crate-root re-exports are retargeted, never widened.

⚠ **Filenames do not change in this stone.** The ward found `harness` a Level 1 lie and `compose` a
Level 2 mumble, and both findings are recorded. Renaming `Harness` is a **public API change with
external consumers** (`examples/`, and the `wat::main!` macro's generated code) — a different stone,
on its own evidence. Files move whole; that is the precedent HOME #5 and #6 both set.

## ⛔ THE THREE TRAPS — ALL PRESENT, ALL PRE-NAMED THIS TIME

Two of these were discovered the expensive way, one stone each. They are rooms now.

**TRAP 1 — the bare crate-root re-export.** Hit by HOME #5 (`lib.rs:136`) and HOME #6 (`lib.rs:116`
and `:148`). **Three instances over two stones.** Present again, measured:

```
src/lib.rs:115   pub use compose::{compose_and_run, compose_and_run_with_loader};
src/lib.rs:144   pub use harness::{Harness, HarnessError, Outcome};
```

A crate-root re-export names its target **unprefixed by construction**, so no grep for
`crate::`/`wat::` can see it and only the compiler can. It will hit every home move; it is a room,
not a surprise.

**TRAP 2 — relative `include_str!`.** HOME #6 hit 54 of them plus one in `tests/lint/`. Measured
here: **one**, and it is inside a doc example (`//!     source: include_str!("program.wat"),`) — a
doctest snippet, not a real path. Clean, and checked rather than assumed.

**TRAP 3 — the proc-macro EMITS these paths as generated text.** This one is new and it is the
sharpest:

```
crates/wat-macros/src/lib.rs:508,517,534   fn main() -> Result<(), ::wat::harness::HarnessError>
crates/wat-macros/src/lib.rs:891           ::wat::test_runner::run_single_deftest(
```

**Nothing inside `wat-rs` type-checks these.** They are strings the macro writes, and they fail only
when the macro is *expanded* in a consumer. `examples/console-demo` and `examples/with-loader`
expand `wat::main!` / `wat_test!`, so **`--all-targets` reaches them and a plain
`cargo build --release` does not.** Same class as `\c`'s parse-time desugar and
`closure_extract`'s wire encoding: a site that constructs a name rather than calling it.

## ⚠ TWO VERIFIED LEVEL-1 DOC LIES ARE FIXED HERE, AND THAT IS DELIBERATE SCOPE

Both are in files this stone moves, both are **comments only, zero behaviour**, and both were
verified against the disk by the orchestrator — not taken from the report:

1. **`harness.rs` — `Harness::run`'s public doc** says *"Invoke `:user::main` with pre-seeded stdin
   lines and return captured stdout + stderr."* The body is `let _ = stdin;` and two hardcoded
   `Vec::new()`. A thorough WHY lives in a body comment; **the doc-reader never sees it.** The
   module doctest asserts the captured output and is marked `no_run` — it would fail if run.
2. **`test_runner.rs` — the primary public doc** states discovery rule 1 as *"the path's final
   `::`-segment starts with `test-`"*; `is_test_function`'s own doc says that filter *"has been
   dropped"* (2026-04-25). **Two contradicting docs in one file, and the stale one is on the entry
   point.**

Moving a file while knowing its doc lies is relocating the lie. Fixing a lying comment is not scope
creep; it is the cheapest possible honesty repair and it is in the blast radius already.

## ACCEPTANCE

1. **`src/` root 27 → 24 `.rs` files.** Derived: 27 at HEAD.
2. **`src/host/` holds four** (`mod.rs` + the three), all three recorded as **renames**.
3. **Zero `crate::compose::` / `crate::harness::` / `crate::test_runner::` / `wat::…` old-shape
   paths** anywhere, including `crates/wat-macros`'s emitted strings. Derived: 15 at HEAD.
4. **No `pub use` in `src/host/mod.rs`.**
5. **`cargo build --release --all-targets` green** — the lib alone cannot see TRAP 3.
6. **The two doc lies are gone**, and `Harness::run`'s doc states what the body does.
7. **`src/panic_hook.rs` is untouched** and still at root.
8. Floor green **accounted BY NAME** (baseline 5057/5057, 19 skipped); clippy 0.

## OUT OF SCOPE — affirmatively cut, each with its evidence

- **A diagnostics/report home** for `panic_hook.rs` + `test_runner.rs`'s ~350 reporting lines. Two
  independent reads converged on it; it is `partire`'s question and the largest thing this cast
  found. Its own stone.
- **Renaming `Harness` / `compose_and_run`.** Public API with external consumers and macro-generated
  call sites. The ward's verdicts are recorded; the rename is its own stone.
- **The remaining ward findings**: `HarnessError::StdioSnapshot` never constructed (`purgare`);
  `panic_hook`'s Design section omitting `assertion_failure_envelope` and its second production
  caller; `AssertionPayload::raised_error` encoded by nothing; `test_runner`'s doc-link to
  `failure_to_diagnostic`, a fn arc 296 replaced; `source_has_config_setter` ORing two conditions
  its name does not admit.
- **`src/check.rs` / `types.rs` / `freeze.rs`** — the carve begun and abandoned. **The shims** —
  `lexer.rs` (3 lines), `ast.rs` (3), `span.rs` (23), `parser.rs` (59).
