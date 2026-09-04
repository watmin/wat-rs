# BRIEF — STONE: `:wat::eval::walk` faces `:wat::WatAST`

You are a **rider**, not the orchestrator. **Ending your turn ENDS you** — nothing will wake you.
Run every command in the FOREGROUND and block on it. You may not spawn sub-agents.

Anchor: **`/home/john/work/holon/wat-rs`**. `pwd` first. Any path containing `.claude/worktrees/`
is harness state — never operate on it. Do not commit, push, stash, or revert. Do not run the full
floor; the orchestrator runs it centrally.

Read `DESIGN-STONE-eval-walk-faces-watast.md` (sibling) and
`docs/arc/2026/04/109-kill-std/NOTE-eval-walk-is-the-last-verb-that-declares-a-holon-ast.md` first.

## The work in one paragraph

`:wat::eval::walk` takes `:wat::WatAST` in, hands its callback `:wat::WatAST` at every step, and
returns its terminal form as `:wat::holon::HolonAST` — the last language verb whose declared type
mentions a holon. Change the declared element to `:wat::WatAST` and convert at the construction site
with `holon_to_watast`, which is already in scope in that file. Two callers exist; neither reads the
element.

## Rooms, in order

1. **`src/check.rs:18428-18448`** — `:wat::eval-step!`'s scheme. Read it FIRST: it is `walk`'s own
   sibling and already takes `wat_ast_ty()`. It is the convention this stone restores, not invents.
2. **`src/check.rs:18450-18482`** — `:wat::eval::walk`'s scheme, and the comment above it that says
   *"Returns (terminal-HolonAST, final-acc)"*. Element 0 of the `Tuple` is at `:18472`. The comment
   is part of the change.
3. **`src/runtime.rs:12193`** — `Value::holon__HolonAST(Arc::new(terminal))`, inside the
   `"Continue"`/terminal arm of `eval_walk`. `holon_to_watast` is already imported in this file.
4. **`src/reflect/verbs.rs`** — the four `holon_to_watast` call sites. This is the shipped pattern:
   face `WatAST`, convert at the boundary. Copy its shape, including how it handles the conversion.
5. **The two callers** — `tests/types/parametric_enum_walk_visitor.wat` and
   `wat-scripts/scratch-pad/probe-room4-cek-stepper-qualified-scrutinee.wat`. Verify what each
   reads before changing anything; the DESIGN says neither touches element 0, and you should
   confirm that rather than trust it.

## ⛔ The acceptance is a ROUND-TRIP, not a compile

`holon_to_watast` is a conversion and a conversion can lose. The current probe's terminal form is
the literal `5` — a scalar, which cannot demonstrate fidelity.

**Write a probe whose terminal form is a COMPOSITION** — a nested call, a vector, something with
structure — and prove the returned `:wat::WatAST` is the same form it would have been. Use
`:wat::core::ast->source` on the result and compare against the source you walked. Put it in
`wat-scripts/scratch-pad/` (durable, loader-gated).

⚠ Run it BEFORE your change too, and record what the holon rendered as, so the before/after is a
comparison rather than an assertion.

## STOP triggers — each rejects; none permits a smaller delivery

- **STOP-1** — if `holon_to_watast` loses any structure on a composed terminal form, STOP and report
  exactly what was lost, with both renderings. Do not ship a lossy conversion behind an honest-looking
  signature; that would be strictly worse than the asymmetry it replaces.
- **STOP-2** — do not touch the other 27 `Value::holon__HolonAST` producers in `src/runtime.rs`, the
  `:wat::holon::*` surface, or `:wat::holon::Reckoner/new-discrete`. Those are VSA or hidden residue,
  both correct. This stone is one verb.
- **STOP-3** — do not change `holon_to_watast` itself. If it is wrong, that is a finding, not a fix.
- **STOP-4** — if either caller turns out to read element 0 after all, STOP and report. The DESIGN's
  claim that neither does is the basis for calling this small.

## Verification

```
cargo nextest run --release -E 'binary_id(wat)'
cargo nextest run --release -E 'binary_id(wat::types)'
cargo nextest run --release -E 'binary_id(wat::reflection)'
cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'
cargo clippy --release --all-targets -- -D warnings
```

`.wat` stdlib files are `include_str!`ed — every `--check`/run follows a `cargo build --release`.

## What to report

The composition probe's before/after renderings, verbatim; whether anything was lost; what each of
the two callers reads; the Summary line per scoped run; and anything that surprised you.
