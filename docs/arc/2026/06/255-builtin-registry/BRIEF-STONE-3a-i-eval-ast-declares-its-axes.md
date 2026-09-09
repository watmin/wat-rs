# BRIEF — STONE ③a-i: `:wat::eval-ast!` declares its axes

## The work, in one paragraph

Add ONE `#[wat_special_form(":wat::eval-ast!")]` declaration — a unit struct with a doc block —
in a new `src/intrinsic/special/eval_ast.rs`, wired into `src/intrinsic/special/mod.rs`. The five
axes are ALREADY RULED in the DESIGN; do not re-derive them. This is the template ③a-ii and ③a-iii
will copy, so the doc block's quality is the deliverable.

## Read in order

1. `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-3a-i-eval-ast-declares-its-axes.md` —
   **the axes and their grounds. Ruled. Transcribe the reasoning; do not re-argue it.**
2. `src/runtime.rs:5270–5308` — `:wat::core::apply`'s doc block. **The model.** Four of five axes
   are apply's, for apply's recorded reasons; its prose shows the depth an axis ground is written at.
3. `src/intrinsic/special/macroexpand.rs:1–40` — the OTHER model: a special form whose module doc
   records what was MEASURED and which naive answer it refuted. Match that register.
4. `src/intrinsic/special/control_flow.rs` — a minimal, correct `#[wat_special_form]` (`if`).
5. `src/intrinsic/special/mod.rs` — the `pub(crate) mod …;` list. One line to add, alphabetical.
6. `src/check.rs:19493` — the existing `TypeScheme`. **Leave it alone**; it is the stronger authority.

## The declaration

```rust
/// <prose: what the verb does, and the fence — `run_constrained` refuses a hand-list of
///  declaration heads before evaluating, so a mutation-form evaluand comes back as `Err`>
///
/// <the axis grounds, in the register of `apply`'s doc block — one short paragraph per axis>
///
/// @added         1.0.0
/// @Purity        Preserving
/// @Determinism   Preserving
/// @Totality      Preserving
/// @ExpandTime    RuntimeOnly
/// @Category      ControlFlow
/// @syntax  (:wat::eval-ast! <form>)
/// @ret     :T <the evaluand's own value, in the Ok arm of a Result>
/// @example <a RUNNABLE one — see below>
#[wat_special_form(":wat::eval-ast!")]
pub(crate) struct EvalAst;
```

`@arg` is OPTIONAL for a special form — `:wat::core::let` (`special/binding.rs:28`) carries
`@syntax` and `@ret` and no `@arg`. Use `@syntax`.

★ **The `@example` must RUN** — it is verified. A measured, working one, from this session:

```
(:wat::eval-ast! (:wat::core::quote (:wat::i64::/ 1 0)))
    → (Err :wat::core::EvalError{…"division-by-zero"…})
```

and the simple positive shape works too. Prefer a runnable `@example`; `@example-norun` is the
accepted escape ONLY where an input cannot be synthesized inline (`:wat::kernel::raise!` uses it),
and `[[NOTE-a-norun-example-asserts-nothing]]` stands.

## Blast radius

Two files: the new `src/intrinsic/special/eval_ast.rs`, and one `mod` line in
`src/intrinsic/special/mod.rs`. **No change to `runtime.rs`'s dispatch arm** — a special form has
no `NativeHandler` and is dispatched by the engine. **No change to `check.rs`.**

## Acceptance

```
cargo build --release
cargo nextest run --release -E 'test(is_type)'                       unchanged
target/release/wat --check <a file calling :wat::eval-ast!>           exit 0, unchanged
(:wat::runtime::metadata-of :wat::eval-ast!)                          now ANSWERS
(:wat::core::render-doc :wat::eval-ast!)                              now ANSWERS
```

The last two are the stone working: before it, both are silent for this name.

## STOP triggers — each is a REJECTION. Ship nothing; report.

**STOP-1.** If the doc block will not satisfy `wat_doc`'s required set — STOP with the exact
`DocError`. Required universally: prose, `@added`, `@ret`, ≥1 `@example`/`@example-norun`, plus
`@Totality` and `@ExpandTime`. Do NOT drop an axis to get it compiling.

**STOP-2.** If you find yourself changing `runtime.rs`'s dispatch arm, or `check.rs` — STOP. The
scheme is the stronger authority and the arm already works.

**STOP-3.** If you find yourself adding the name to a LIST anywhere — STOP. Per-site declaration
via `inventory` is the whole point; a literal list in the registry builder is a new hand-list.

**STOP-4.** If an axis in the DESIGN looks wrong to you once you are in the code — **STOP and say
so with the evidence.** Do not silently substitute your own. One of these five was already wrong
once this session and a precedent caught it; a second correction is welcome, a silent one is not.

**STOP-5.** If the `@example` cannot be made runnable — STOP and report what you tried before
reaching for `@example-norun`.

## Tier

You edit and report. Run `cargo build --release` and the two `wat` invocations above; that is your
whole measurement surface. **Do NOT run `scripts/floor.sh`, `cargo clippy`, or the full suite** —
the orchestrator runs those centrally, once, on a quiescent tree. **Do NOT commit.**

Work in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Any path containing
`.claude/worktrees/` is harness state and must not be operated on.
