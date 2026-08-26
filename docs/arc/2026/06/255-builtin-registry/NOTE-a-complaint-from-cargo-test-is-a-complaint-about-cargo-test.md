# NOTE — a complaint from `cargo test` is a complaint about `cargo test`

Filed 2026-08-26. **This CORRECTS a claim I put in the durable record earlier the same day.**

## The claim, and it was wrong

`870d59898` (Stone B-ii) asserts, in its commit message:

> *"our floor cannot see a class of shared-state defect that `cargo test` can. Process isolation is a
> real property we depend on and have never written down as a contract."*

Both sentences are false, and the second is falsified by a file I had already read.

## What is actually true

The floor is `cargo nextest`, which runs **one process per test**. `cargo test` runs one process per
**binary** — 247 tests sharing one address space in the `collection` binary alone. Three "findings"
were collected this week from `cargo test` runs:

```
cargo test    --test collection    240 passed,  7 FAILED
cargo nextest -E binary(collection) 247 passed,  0 failed      ← same tree, same tests
```

The mechanism is a process-global `OnceLock`:

```rust
static REGISTRY: OnceLock<RustDepsRegistry> = OnceLock::new();
pub fn install(r) -> Result<(), &str> { REGISTRY.set(r).map_err(|_| "already installed") }
pub fn registry() -> &RustDepsRegistry { REGISTRY.get_or_init(|| …with_wat_rs_defaults()…) }
```

First caller wins, forever. Every `install()` call site writes `let _ = …` — **deliberately**,
because losing the race is meant to be a no-op.

**And the contract is written down, in prose, in `src/host/test_runner.rs:48-55`:**

> *"`rust_deps::install()` is a OnceLock — first-call-wins. A test binary running
> `run_tests_from_dir` once against one dep set is **the intended shape**. Callers running multiple
> invocations with *different* dep sets **in one process** will hit the first-call-wins limitation…
> **Match each dep set to its own test binary** and Cargo handles the rest."*

One process, one dep set. nextest satisfies that by construction. `cargo test` violates a documented
precondition — and the failures are the substrate correctly refusing to pretend otherwise.

## The failure this note actually records

I ran the wrong instrument, read its complaint as evidence about the substrate, and escalated it into
a claim about the project's whole testing posture — in a commit message, where it will outlive the
misunderstanding. I was one message from writing a NOTE enshrining it. The builder asked *"what
shared-state? who is sharing what when?"*, and answering that question precisely — naming the state,
the writer, the reader, and the window — dissolved the finding in three commands.

**"Who is sharing what, when" is the question that converts a shared-state hand-wave into a fact.**
None of the three instances survived it:

- `rust_deps REGISTRY` — documented contract, `cargo test` breaks the precondition. Not a defect.
- FFI shim registration (`:rust::test::MathUtils` &c.) — the same mechanism, same resolution.
- `ARM_BUILDS` — a test asserting a *delta* on a process-global build counter. Same SHAPE (a test
  assuming it owns the process); not separately re-measured, and named here so the next self does not
  inherit it as established.

## The standing rule

**The floor is `cargo nextest run --release` via `scripts/floor.sh`.** A red from `cargo test` is
information about `cargo test` until proven otherwise. Diagnosing with it is fine; *concluding* with
it is not, because it runs the substrate in a configuration the substrate says it does not support.

This does NOT loosen the red-is-a-red rule. A red from the FLOOR is still a red, always. What it says
is narrower and exact: an instrument that violates a documented precondition has not measured the
thing you think it measured. `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`
