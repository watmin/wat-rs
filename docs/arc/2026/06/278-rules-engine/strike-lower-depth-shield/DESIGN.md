# DESIGN — `lower`'s totality invariant is enforced in another subsystem, and nothing says so

**Status:** drawn 2026-09-08 from vigilia row `2W1` (circumspicere, target 2, L1). ⛔ **The row's
prescribed cure is REFUTED by driving; this design replaces it.** Read the reframe below before the
work.

## Why

`src/rete/expr_ir/mod.rs:14-19` carries the invariant the whole expression core is built on:

> *"`lower` IS TOTAL OR IT REFUSES. A `Program` that exists is one `exec` can run… `exec` therefore
> raises only on VALUES … **never on shape**. A refusal that belongs at compile time and lands at
> fire time is a defect in this file."*

`2W1` observed — correctly — that `lower_expr` / `lower_list` / `lower_hof_callee` / `lower_pat`
mutually recurse over `WatAST` with **no depth counter**: `LowerCx` has four fields and none is a
depth (`:255-262`). Re-derived this session: `depth` / `recursion` / `MAX_` / `stacker` all count
**0** across `expr_ir/mod.rs` and `expr_ir/eval.rs`. `lower_pat` additionally self-recurses
(`:809`), so there are at least three towers, not one.

The row concluded that deep nesting therefore **aborts** — citing `MAX_IMPORT_DEPTH`'s own record
that a 20,000-deep form *"killed a 2 MiB thread with `fatal runtime error: stack overflow,
aborting` — an abort, not a panic, so nothing catches it"* — and prescribed threading `depth`
through `LowerCx`.

## ⛔ THE REFRAME — driven 2026-09-08, and it changes the cure

**There is no abort. Deep nesting is refused cleanly, and the refusal comes from a different
subsystem.**

- `EXPANSION_DEPTH_LIMIT = 512` (`src/macros/expand.rs:16`) increments **per nested form**, not
  only per macro call — verified by nesting nothing but plain `:wat::rete::core::i64::+` calls,
  which are not macros, and still tripping it.
- **Binary-searched the wall: nesting depth 509 is accepted, 510 is refused** with
  `#wat.macro/ExpansionDepthExceeded`. Deterministic — a tree walk, not a stack race.
- **`import` never reaches `lower` at all.** `unpack_expr` (`src/rete/export.rs:846`) returns
  `Expr` **directly**, not `WatAST`, and carries its own `deeper` / `MAX_IMPORT_DEPTH = 300`.
- All seven production callers of `lower` / `lower_in_frame` / `lower_named_rete_fn`
  (`compiled_rhs.rs:154,193`, `compiled_cond.rs:758`, `matcher.rs:854`, `arm.rs:111,430,499`) take
  AST originating in expanded source. The two that pass a `rewritten` form are **field-reference**
  rewrites, which replace a leaf with an accessor node — bounded constant depth per leaf, not
  multiplicative.

**So the system is bounded and the file is not self-bounded.** Threading a second depth counter
through `LowerCx` would add a redundant guard behind an existing one and measure nothing new.

## What this delivers

**The defect is that a load-bearing invariant is upheld by a constant in another subsystem, and
nothing in the tree records the dependency or gates it.** Raise `EXPANSION_DEPTH_LIMIT`, or add a
route to `lower` that skips expansion, and the headline promise silently stops holding — with no
test anywhere that notices, and the first symptom would be the abort `2W1` wrongly believed was
already happening.

This is exactly `circumspicere`'s own `enforced-elsewhere` category: *"the invariant IS asserted, at
a site the rune names."* **Today no rune names it.**

Two parts:

1. **Record the shield** in `expr_ir/mod.rs`'s header — the constant, its file, the measured wall,
   and the fact that `import` bypasses this path entirely.
2. **Gate the seam** — a probe that reddens if the shield moves or a route appears that skips it.

## The one contract decision, pinned

⛔ **THE GATE ASSERTS THE DEPENDENCY, NOT THE NUMBER.** The probe imports `EXPANSION_DEPTH_LIMIT`
and asserts the wall sits at `LIMIT - K` for one documented small `K` (the constant nesting the
harness itself contributes). It must NOT hard-code `509`. Raising the limit must move the wall and
keep the test green; **removing or bypassing the shield must red it.** A test pinned to `509` would
red on any harmless re-wrapping of the fixture and teach the next hand to bump a number.

## Files touched

- `src/rete/expr_ir/mod.rs` — header comment only.
- `tests/rete/probe_arc278_lower_depth_shield.rs` — new probe.

## Out of scope = rejected

- **Threading `depth` through `LowerCx`.** Refuted above. Not "later" — rejected.
- **Changing `EXPANSION_DEPTH_LIMIT`.** It is not this strike's number to move.
- **Touching `export.rs` / `MAX_IMPORT_DEPTH`.** A separate, already-guarded path.
- **A `.wat` fixture at depth 509.** ~15 KB of generated nesting; build the source in the test.
