# Arc 264 (STUB) — formatting: rustfmt + a companion wat-fmt

> **Status: STUB — distant/banked** (captured 2026-06-14, builder). Not the active stream; a
> tooling-hygiene arc that joins the coverage tooling already queued (arc 252 `coverage-rune` /
> task #190 — cargo-llvm-cov + warded-home coverage gate). Pick up when the formatting itch is
> worth a session, not before.

## Two halves

1. **rustfmt** — set up `rustfmt` for the Rust side (`src/`, `crates/`): a `rustfmt.toml` with the
   project's settings + a gate (CI/pre-commit check that the tree is `cargo fmt --check` clean).
   Decide the config deliberately (line width, import grouping) rather than accepting defaults — the
   codebase has strong existing conventions (one-arm-per-line match style, the `rune:` comment
   forms) that a naive rustfmt run could thrash. The first `cargo fmt` will be a large mechanical
   diff; land it as its own commit, separate from any logic change.

2. **wat-fmt** — the companion: a formatter for wat source. This is the natural next rider on the
   `fix-text` engine (`wat/fix.wat`, the comment-faithful span-edit codemod from arc 251.5) — a
   formatter is a codemod whose rule is "normalize whitespace/indentation," and it MUST be
   comment-faithful (wat source carries load-bearing doc comments). Connects to the banked
   "code-formatter pass" need (token-only strip-if deletion leaves residual whitespace — cosmetic,
   noted at C.3). wat self-hosts its own evolution (Songs #95/#96, the maturity line); a wat-fmt is
   that line extended to formatting — wat formats its own corpus, in wat, through its own CLI.

## Why both, why together

The symmetry is the point: the Rust substrate gets rustfmt; the wat language gets wat-fmt, built on
the substrate's own self-hosting codemod engine. Same hygiene, both layers, each in its own idiom.

## Out of scope until picked up

- The exact rustfmt.toml settings → decide at draw (ground against the existing conventions).
- wat-fmt's formatting rules (indentation model, alignment) → a real design (intueri the surface;
  what does idiomatic wat look like?). Likely a fix-text rule set, not a tree-reprint (never
  `write-forms` the tree — comments die; splice normalized whitespace per the fix-text discipline).
- Whether wat-fmt is a new `wat-scripts/fixes/` runner or a first-class CLI subcommand.
