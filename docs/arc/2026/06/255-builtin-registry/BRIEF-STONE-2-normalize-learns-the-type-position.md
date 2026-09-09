# BRIEF — STONE ②: `normalize` learns the type position

> ⛔ **This is the SECOND draft. The first was defective and a rider implemented it faithfully into
> two regressions. The two rows that caught them are committed. Read the DESIGN's
> "THE FIRST STRIKE, AND WHAT IT TAUGHT" before you touch anything.**

## The work, in one paragraph

`src/resolve/normalize.rs` walks a `(Head :- [T …] rest…)` type reference as ordinary code, so the
type names inside it are asked a CALL-HEAD question. Teach it the shape: the **head** is asked
whether it is a resolvable call head **or** a known type (both are legal there); the
**type-argument vector** is rewritten to keyword FQDNs with no validation; **everything after the
type vector stays live code** and normalizes exactly as today.

## Read in order

1. `src/resolve/normalize.rs:98` — `normalize_form`'s `List` arm. `Boundary` is classified from
   `items.first()` only when it is a `Keyword`; a Symbol head falls to `Boundary::Ordinary`. The site.
2. `src/resolve/normalize.rs:425` — `resolve_namespaced_symbol`, which asks
   `is_resolvable_call_head` and emits the census's error at line 461.
3. `src/reflect/verbs.rs:1545` — `eval_is_type`, the `:wat::runtime::is-type?` intrinsic. **Line
   1577 is the three-store union you need**, currently INLINE in this fn.
4. `src/types.rs:5161` — the ONLY `:wat::type::` → `:wat::core::` canonicalization, inline inside
   `parse_type_expr`.
5. `src/types.rs:4534` — `peel_param_spec`, arc 109's ONE DOOR for the `(marker, [types], rest…)`
   triple, built because nine sites had hand-rolled the slice pattern. Use it.
6. `src/resolve/walk.rs:87` — the sibling `is_type_reference` test. Copy the TEST, not the action:
   walk.rs SKIPS validation here and that is the gap this stone must not copy.
7. `tests/resolve/probe_arc255_the_type_position_has_its_own_authority.rs` — **eleven** committed
   rows. Read the header.

## ★ THE ONE DOOR REQUIREMENT

The three-store union at `verbs.rs:1577` must become **one function that both `eval_is_type` and
`normalize` call.** Writing `types.contains(x) || is_builtin_primitive(y) || types.is_subtype_parent(x)`
a second time in `normalize.rs` is a second implementation of one question — the exact defect this
whole arc exists to end, and it would be reviewed as a failure even with every row green.

Put the door where the store lives (a `TypeEnv` method reads naturally: the store answering a
question about itself), and have it take the keyword and do the `:wat::type::` canonicalization
internally, so neither caller can forget it. `eval_is_type` then becomes a call to that door.

## Implementation sketch — fill it in, do not invent the shape

```rust
WatAST::List(items, span) => {
    // `(Head :- [T …] rest…)` — a type binder. `peel_param_spec` is arc 109's one door.
    // ⚠ items may be EMPTY: `&items[1..]` PANICS on a zero-length slice. Guard it.
    if items.len() >= 3 && crate::types::peel_param_spec(&items[1..]).0.is_some() {
        // items[0]   head        -> call head OR known type (the union; see below)
        // items[1]   `:-`        -> unchanged
        // items[2]   type vector -> rewritten, NOT validated (P-1's wall owns it)
        // items[3..] value args  -> normalize_form, exactly as today
    }
    …
}
```

**The head.** Keep `resolve_namespaced_symbol`'s existing behaviour and give it — in this position
only — a second acceptance: if `is_resolvable_call_head` says no, ask the type door on the
candidate FQDN before emitting the error. If neither answers yes, emit the SAME
`UnresolvedReference` it emits today; do not invent a new diagnostic.

⚠ Widen it **in this position only**, not inside `resolve_namespaced_symbol` for all callers — a
type is legal *here* because this position's grammar admits one; it is not legal in an arbitrary
call position.

**The type vector.** A small recursive rewrite: a referencing `WatAST::Symbol` becomes
`WatAST::Keyword(ns_to_wat_path(receiver, method), span)` with no validation call, recursing
through `List`/`Vector` so a nested `(V :- [T])` is reached. Bare symbols (no `/`) are type
VARIABLES — leave them untouched. **The first strike's helper for this half was correct**; its diff
is preserved at the session scratchpad as `normalize.rs.RIDER` if you want the shape, but rebuild
it rather than trusting it.

## Blast radius

Three files, at most: `src/resolve/normalize.rs`, wherever the type door lands (`src/types.rs`),
and `src/reflect/verbs.rs` (repointed at the door). No new public API. No signature change to
`is_resolvable_call_head` or anything in `walk.rs`. No `.wat` file changes.

## Acceptance

```
cargo nextest run --release -E 'test(probe_arc255_the_type_position)'          11 rows, all pass
cargo nextest run --release -E 'test(is_type)'                                 unchanged
./target/release/wat --check tests/resolve/…__control_the_four_names.wat        exit 0
./target/release/wat        tests/resolve/…__control_values_after_the_type_vector.wat
                                                                exit 0, prints ["1", "b"]
```

★ The four census names are ALREADY accepted today, so no row can show this stone "working" by
going green. **The eleven rows are containment.** The stone's proof is a measurement the
orchestrator runs centrally: the gate census going 14 names → 10.

## STOP triggers — each is a REJECTION. Ship nothing; report.

**STOP-1.** If the three-store union cannot become ONE door both callers use — STOP and report the
exact borrow/visibility/layering reason. Do NOT write it twice.

**STOP-2.** If you find yourself skipping the whole `:-` subtree — STOP. Built and measured:
`--check` exit 0, runtime `UnboundSymbol`.

**STOP-3.** If you find yourself exempting the HEAD from validation — STOP. That was the first
strike's defect: `(my.app/totally-bogus :- [i64] 1)` went from exit 1 to exit 0.

**STOP-4.** If you reach for `&items[1..]` without a length guard — STOP. `()` panicked the compiler
at exit 101 last time.

**STOP-5.** If any of the eleven probe rows changes verdict — STOP with the verbatim block.

**STOP-6.** If the change wants a FOURTH file — STOP and report which and why.

## Tier

You edit and report. Run the four acceptance commands, foreground, and nothing else. **Do NOT run
`scripts/floor.sh`, `cargo clippy`, or the full suite** — the orchestrator runs those centrally,
once, on a quiescent tree. **Do NOT commit.** Report your diff, each command's verbatim result,
which STOP triggers you considered, and anything the brief got wrong about the code.

Work in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Any path containing
`.claude/worktrees/` is harness state and must not be operated on.
