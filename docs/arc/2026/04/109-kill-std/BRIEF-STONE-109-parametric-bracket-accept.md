# BRIEF — 109 step ①: ACCEPT `(Head [type…])`, in both positions

Design: `DESIGN-STONE-all-parametrics-take-a-type-vector.md`.

Make the substrate **accept** the bracketed type-param group — as a type annotation AND as a
constructor. **This step is purely ADDITIVE: the existing `Head<…>` angle form must keep working,
unchanged, everywhere.** Step ② codemods the corpus; step ③ makes the angle form illegal. You are ①.

**Your role: you write the text. The orchestrator builds, floors, and clippies.** Do not run `cargo` in
any form. Run everything else in the foreground and block on it — ending your turn ends you, and
nothing will wake you. Do not commit, push, stash, or revert.

## The rule

Every parametric is a constructor receiving a vector of types:

```wat
(:wat::core::Vector [:wat::core::i64])                            ; TYPE after <- / ->, or an empty instance
(:wat::core::HashMap [:wat::core::Keyword :wat::core::String])    ; same form
(:wat::core::HashMap [:wat::core::Keyword :wat::core::String] :some-kw "some-str")  ; instance with values
(:wat::core::Tuple [:wat::core::i64 (:wat::core::HashSet [:wat::core::f64])])       ; nests uniformly
```

Position already decides type-vs-instance — that is existing, proven behaviour you are not changing.

## ROOM 1 — type position. `src/types.rs:4522`, inside `parse_type_form`

```rust
let args: Result<Vec<TypeExpr>, TypeError> = items[1..].iter()
    .map(parse_type_node)
    .collect();
```

When `items[1..]` is **exactly one `WatAST::Vector`**, parse that vector's items as the type-param
list instead. Otherwise behave exactly as today.

- `(Head [A B])` → args `[A, B]`   ·   `(Head [])` → args `[]`   ·   `(Head A B)` → unchanged
- Nesting needs no extra work: `parse_type_node` recurses and each inner `(Head [X])` meets this rule.
- Everything downstream (`Tuple`, `reject_any`, unification) consumes the parsed `Vec<TypeExpr>` and
  must not change.

⚠ `src/types.rs:4383` routes a bare `WatAST::Vector` in type position to `parse_fn_type_bracket`
(the `[A :-> B]` function type). **Do not touch that arm.** A bracket STANDING ALONE is a function
type; a bracket as a parametric head's ARGUMENT is a type-param list. Position distinguishes them.

## ROOM 2 — value position. `src/check.rs`, six constructor arms

```
:wat::core::Vector 3000 · Tuple 3106 · HashMap 3132 · PersistentMap 3142 ·
PersistentVector 3152 · HashSet 3179
```
each delegating to an `infer_*_constructor` that reads leading type keywords off `args[0..]`
(`infer_hashmap_constructor` reads `args[0]`=K, `args[1]`=V, then pairs).

**Write ONE helper, call it from each arm** — do not rewrite six constructors:

```rust
/// Accept the bracketed type-param group `(Head [K V] …)` and re-present it as the
/// positional leading type args `(Head K V …)` every constructor already understands.
/// Pass through unchanged when args[0] is not a Vector.
fn unwrap_type_param_bracket(args: &[WatAST]) -> Cow<'_, [WatAST]>
```

If `args[0]` is a `WatAST::Vector`, splice its items ahead of `args[1..]`; else borrow `args`. Each
arm calls it before delegating. The `infer_*` fns themselves stay untouched.

⚠ `PersistentMap` is the odd one: today it **rejects** leading type keywords and infers K/V from the
pairs (measured, both directions). After the helper it must accept `(PersistentMap [K V] …)`. If
routing the spliced args through `infer_persistentmap_constructor` cannot work because that fn has no
leading-type-arg path, **STOP and report** — do not invent one.

## Verify — the probe is written and its controls already fire

`wat-scripts/scratch-pad/probe-285-map-surface-over-builtins.wat` runs today. Use
`./target/release/wat --check <file>` on scratch files of your own for these (the orchestrator will
re-run every one):

| must | why |
|---|---|
| `(:wat::type::Vector [:wat::core::i64])` after `<-` CHECKS | Room 1 |
| `(:wat::core::Tuple [:wat::core::i64 (:wat::core::HashSet [:wat::core::f64])])` CHECKS | nesting |
| `(:wat::core::HashMap [:wat::core::String :wat::core::i64] "a" 1)` builds `{"a" 1}` | Room 2 |
| `(:wat::core::PersistentMap [:wat::core::String :wat::core::i64] "a" 2)` builds | the odd one |
| **`:wat::core::Vector<wat::core::i64>` STILL CHECKS** | ★ ADDITIVE — the old form must not break |
| **`[A :-> B]` standing alone STILL CHECKS** | ★ no collision with the function type |

The last two are the ones that matter most: this step adds a spelling, it removes none.

## Blast radius

`src/types.rs` (one arg-extraction) · `src/check.rs` (one helper + six call sites). No lexer change —
the angle machinery stays until ③. No `.wat`. No `tests/`. No renderer change (`format_type` /
`format_type_inner` are ③'s, per the design's rendering ruling).

## STOP triggers — each rejects; none is a fallback

1. Any existing angle-form spelling stops checking. STOP — ① is additive.
2. `[A :-> B]` as a standalone function type stops working. STOP.
3. A constructor cannot take the spliced args without changing its `infer_*` fn. STOP and report which.
4. The change needs a lexer edit. STOP — the lexer is ③'s room, and if ① needs it the design is wrong.

## Acceptance criteria

- Both rooms accept the bracket, in type and value position, nesting included.
- Every angle-form spelling still checks — additive, no removals.
- `src/types.rs` + `src/check.rs` only; no lexer, no `.wat`, no `tests/`, no renderer.
- One helper, six call sites; the `infer_*` fns unmodified.
