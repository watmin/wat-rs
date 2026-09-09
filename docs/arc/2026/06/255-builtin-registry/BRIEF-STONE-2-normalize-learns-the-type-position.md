# BRIEF — STONE ②: `normalize` learns the type position

## The work, in one paragraph

`src/resolve/normalize.rs`'s `normalize_form` walks a `:-` type reference as ordinary code, so the
type names inside it are asked a CALL-HEAD question. Teach it the shape `walk.rs` already knows:
in a form whose second element is the `:-` binder marker, the **head** and the **type-argument
vector** are type syntax — rewrite their namespaced symbols to keyword FQDNs *without* the
call-head validation — while **everything after the type vector stays live code** and normalizes
exactly as today.

## Read in order

1. `src/resolve/normalize.rs:98` — `normalize_form`'s `List` arm. The `Boundary` is classified from
   `items.first()` **only when it is a `Keyword`**; a Symbol head falls to `Boundary::Ordinary`,
   which rewrites every child. This is the site.
2. `src/resolve/normalize.rs:425` — `resolve_namespaced_symbol`, the fn that asks
   `is_resolvable_call_head`. Its line 461 emits the error the census sees. Do not change it.
3. `src/resolve/walk.rs:87` — the SIBLING guard, already correct:
   `items.get(1).is_some_and(crate::types::is_binder_marker)`. Copy the test, not the action.
4. `src/types.rs:4534` — **`peel_param_spec`**, arc 109's ONE DOOR for the
   `(marker, [types], rest…)` triple. Nine sites hand-rolled the `[Keyword, Vector, rest @ ..]`
   slice pattern before it existed. **Use this door; do not become the tenth site.** It takes the
   args AFTER the head, so pass `&items[1..]`, and it returns `(Some(&[type args]), rest)`.
5. `src/edn/render.rs` — `ns_to_wat_path`, the `wat.core/+` → `:wat::core::+` mapping
   `resolve_namespaced_symbol` already uses. The type-side rewrite must use the SAME mapping.
6. `tests/resolve/probe_arc255_the_type_position_has_its_own_authority.rs` — nine committed rows.
   Read the header; it records what two rejected implementations measured.

## Implementation sketch — fill it in, do not invent the shape

In `normalize_form`'s `List` arm, before the `Boundary` classification:

```rust
// A `(Head :- [T …] rest…)` form is a TYPE REFERENCE. `walk.rs:87` already knows this shape;
// normalize runs FIRST and did not.
if let (Some(_), _) = crate::types::peel_param_spec(&items[1..]) {
    // items[0]   the head            -> type syntax
    // items[1]   the `:-` marker     -> unchanged
    // items[2]   the type vector     -> type syntax (recurse: a nested `(V :- [T])` lives here)
    // items[3..] live VALUE args     -> normalize_form, exactly as today
}
```

The type-syntax rewrite is a small sibling of `resolve_namespaced_symbol`: a namespaced
`WatAST::Symbol` becomes `WatAST::Keyword(ns_to_wat_path(...), span)` **with no validation call**,
recursing through `List` and `Vector` children so nested parametrics are reached. Bare symbols (no
`/`) are type VARIABLES — leave them exactly as they are, the same rule normalize already applies.

## Blast radius

`src/resolve/normalize.rs` **only**. No new public API. No signature change to
`resolve_namespaced_symbol`, `is_resolvable_call_head`, or anything in `walk.rs`. No new type.
No change to any `.wat` file.

## Acceptance

```
cargo nextest run --release -E 'test(probe_arc255_the_type_position)'    9 rows, all pass
target/release/wat --check tests/resolve/probe_arc255_the_type_position_has_its_own_authority__control_the_four_names.wat     exit 0
target/release/wat        tests/resolve/probe_arc255_the_type_position_has_its_own_authority__control_values_after_the_type_vector.wat     exit 0, prints "["1", "b"]"
```

★ The four census names are ALREADY accepted today (the blanket accepts them), so no row can show
this stone "working" by going green. **The nine rows are the containment: they prove the stone did
not break what it touched.** The stone's real proof is a measurement the orchestrator runs
centrally — the gate census going 14 names → 10.

## STOP triggers — each is a REJECTION. Ship nothing; report.

**STOP-1.** If `peel_param_spec` cannot be used from `normalize.rs` (visibility, borrow, lifetime) —
STOP and report the exact compiler message. Do NOT hand-roll the `[Keyword, Vector, rest @ ..]`
slice pattern; becoming the tenth site is the thing arc 109 built that door to end.

**STOP-2.** If you find yourself skipping the whole `:-` subtree — STOP. That version was BUILT and
MEASURED: it `--check`s at exit 0 and dies at runtime on `UnboundSymbol`. The value arguments after
the type vector must still be normalized.

**STOP-3.** If you find yourself validating a type name against `TypeEnv`, the intrinsic registry,
or any store — STOP. Measured: `:wat::core::Tuple` and `:wat::type::Infer` are both absent from
`TypeEnv`, and both are legal. The annotation wall downstream owns that question.

**STOP-4.** If any of the nine probe rows changes verdict — STOP with the verbatim block. Six must
stay RED (`UnknownNamedType`), three must stay GREEN.

**STOP-5.** If the change wants a second file — STOP and report which file and why.

## Tier

You edit and report. Run the targeted probe above and the two binary invocations; that is your
whole measurement surface. **Do NOT run `scripts/floor.sh`, `cargo clippy`, or the full suite** —
the orchestrator runs those centrally, once, on a quiescent tree. **Do NOT commit.** Report the diff
you made, the probe result, and anything that surprised you.

Work in `/home/john/work/holon/wat-rs`. Verify with `pwd` first; any path containing
`.claude/worktrees/` is harness state and must not be operated on. Use `git -C
/home/john/work/holon/wat-rs` for any git read.
