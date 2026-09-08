# BRIEF — record and gate the depth shield `lower` depends on

## The work

`src/rete/expr_ir/mod.rs` promises *"`lower` IS TOTAL OR IT REFUSES … never on shape"*. It has **no
depth guard of its own** — that part is true and re-derived. But deep nesting does **not** abort:
it is refused at nesting depth **509** by `EXPANSION_DEPTH_LIMIT = 512` in `src/macros/expand.rs`,
a different subsystem, and nothing in the tree records that dependency or gates it.

Two things: a header paragraph in `expr_ir/mod.rs` recording the shield, and a new probe
`tests/rete/probe_arc278_lower_depth_shield.rs` that reddens if the shield moves or is bypassed.
**No change to lowering logic. No new depth counter.**

## Read in order

1. `docs/arc/2026/06/278-rules-engine/strike-lower-depth-shield/DESIGN.md` — **the reframe**. Row
   `2W1` prescribed threading `depth` through `LowerCx`; that is refuted and rejected. Read why
   before you write anything.
2. `src/rete/expr_ir/mod.rs:14-19` — the invariant, verbatim. Your paragraph goes with it.
3. `src/rete/expr_ir/mod.rs:255-262` — `LowerCx`, four fields, no depth. This is what makes the
   file not self-bounded.
4. `src/macros/expand.rs:16` — `EXPANSION_DEPTH_LIMIT: usize = 512`, and `:424-432` where it fires.
   **This is the shield.** Note `:471` — it recurses with `expansion_depth + 1` on nested forms
   generally, which is why plain non-macro nesting trips it.
5. `src/rete/export.rs:846` (`unpack_expr` → `Expr`, not `WatAST`) and `:370` (`MAX_IMPORT_DEPTH`)
   — the proof that `import` bypasses `lower` and is separately guarded. Also read `:360-370` for
   **the house style on a measured bound**; your paragraph should read like that one.
6. `tests/rete/probe_arc278_export.rs` — the `*_tower_past_the_depth_bound` probes, for the shape
   of a depth probe in this corpus.

## Implementation sketch

**The paragraph** — in the module header, adjacent to the invariant it qualifies. It must carry:
the constant and its file; that expansion increments per nested form, not per macro; the measured
wall (509 accepted / 510 refused, driven 2026-09-08); that `import` reaches `Expr` without `lower`
and is guarded by `MAX_IMPORT_DEPTH = 300`; and **that this file has no guard of its own and is
relying on that one**. Name the probe below as the thing that keeps it true.

**The probe** — build the source in Rust, do not ship a 15 KB fixture:

```rust
fn nested(depth: usize) -> String { /* depth × "(:wat::rete::core::i64::+ " … "n" … " 1 :undefined -1)" */ }
```

wrapped in `(:wat::rete::core::defn :probe::deep [n <- :wat::core::i64] -> :wat::core::i64 …)` plus
a `:user::main`. Drive it through `startup_from_source` the way the cost tests do.

⛔ **Assert the DEPENDENCY, not the number.** Import `EXPANSION_DEPTH_LIMIT` (it is `pub use`d at
`src/macros/mod.rs:113`) and express the wall as `LIMIT - K` with `K` a named constant and a comment
saying what those levels are. **Do not hard-code 509.** Two arms: `LIMIT - K` compiles, `LIMIT - K
+ 1` is refused **and the refusal names `ExpansionDepthExceeded`** — a bare "it failed" would pass
on any unrelated error.

## STOP triggers

1. **If the wall is not where `LIMIT - K` predicts for a single small `K`** — STOP and report the
   measured pair. The offset is the whole basis of the assertion.
2. **If you find a caller of `lower` / `lower_in_frame` / `lower_named_rete_fn` whose AST does not
   come from expanded source** — STOP. That is a real hole, it outranks this strike, and it changes
   the cure back toward a guard in `LowerCx`.
3. **If the depth-`LIMIT - K + 1` case aborts instead of refusing** — STOP and report verbatim. That
   would mean the shield is partial and `2W1` was right after all.
4. No change to `EXPANSION_DEPTH_LIMIT`, to `export.rs`, or to any lowering function's body.

## Blast radius

`src/rete/expr_ir/mod.rs` — **comments only**. One new file under `tests/rete/`. Nothing else.
