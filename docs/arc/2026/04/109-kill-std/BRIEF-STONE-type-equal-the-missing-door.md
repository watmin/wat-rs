# BRIEF — mint `:wat::core::type-equal?`

DESIGN: `DESIGN-STONE-type-equal-the-missing-door.md`. Read it first — it carries the measured cost
of the gap and the one contract decision.

## The work

```clojure
(:wat::core::type-equal? [a <- :wat::WatAST  b <- :wat::WatAST] -> :wat::core::bool)
```

```rust
parse_type_node(a)? == parse_type_node(b)?
```

True iff the two nodes denote the same TYPE, whatever spelling each wears. `TypeExpr` already derives
`PartialEq, Eq` — you are adding a door, not a comparison.

**This stone mints the verb only.** Do NOT rewrite any caller; `wat/service.wat` is out of scope.

## Read in order — the exemplar is one screen away

| where | why |
|---|---|
| `src/intrinsic/reflect.rs:519` `eval_type_params_used_in` | ★ **the exemplar.** Same file, same attribute, same arg-evaluation shape (`eval_inner` → `value_owned` → match `Value::wat__WatAST`), same `@example` discipline. Copy its shape. |
| `src/macros/eval.rs:666` | the F5 allow-list. `type-params-used-in` sits there; yours goes beside it. |
| `src/rete/purity.rs:345` | the purity ruling, with the comment explaining why it was RULED and not parked. |
| `src/types.rs` `parse_type_node` | the one door that reads all four spellings — keyword, `wat.type/` symbol, parametric form, `[arg… :-> ret]` bracket. Use it; write no second parser. |

## ★ The contract decision, already made — RAISE on a non-type

Given a node that is not a type, **raise**; do not return `false`. *"These are not both types"* is a
different fact from *"these are different types"*, and collapsing them makes a malformed input
indistinguishable from a legitimate mismatch — a silent pass at exactly the sites that exist to catch
mistakes. Mirror the `TypeMismatch` shape `eval_type_params_used_in` already uses for a bad argument.

## Three registrations — all mandatory

```
1. src/intrinsic/reflect.rs   #[wat_intrinsic(":wat::core::type-equal?")], with a RUNNABLE @example
2. src/macros/eval.rs:666     the F5 allow-list
3. src/rete/purity.rs:345     pure ∧ deterministic ∧ total — RULE it, do not park it
```

⚠ **F5 is the whole point of the verb.** A macro body may not call a user-defined function at all,
and the admission list is default-deny. An intrinsic missing from it is refused at DEFINITION and
takes the stdlib down — 3029 tests, on the record. If you can only do one registration correctly, do
that one.

⚠ The purity gate's remedy calls parking *"the LAST resort, only honest for a verb whose ruling is
genuinely open."* This one reads two nodes, allocates nothing observable, touches no world state, and
returns a bool.

## What "done" looks like

1. `(type-equal? :wat::kernel::Peer<A,B> (:wat::kernel::Peer :- [A B]))` → **true**. The row the
   verb exists for.
2. Nested: `Vector<HashMap<K,V>>` ≡ `(Vector :- [(HashMap :- [K V])])` → **true**.
3. ★ **Negative control:** `Peer<A,B>` vs `Peer<B,A>` → **false**. Without it, a verb that returned
   `true` unconditionally would pass rows 1 and 2.
4. Identity: a keyword against itself → **true**; two unrelated types → **false**.
5. A non-type node → **raises**, with a located error naming the offending argument.
6. ★ **Callable FROM A MACRO BODY.** Write a `defmacro` that calls it and expand that macro. Rows 1-5
   can all pass from ordinary code while F5 still refuses it at definition — and F5 refusal is a
   DEFINITION-time failure, so it will not look like a wrong answer, it will look like the stdlib
   breaking.
7. A runnable `@example` on the intrinsic, and the doctest suite green.

## Boundaries

- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally.
  ⚠ A scoped run is not the floor: on a recent stone `binary_id(wat::services)` was 128/128 green
  while the floor was red by six.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.
- Do NOT touch `wat/service.wat` or any caller. This stone mints the door; using it is a later stone.
- Write no second type parser and no base-extraction helper.

## Your own checks

`cargo build --bin wat`, then `target/debug/wat` on files under `wat-scripts/scratch-pad/` — row 6
needs a real `defmacro` + `macroexpand`, not a plain call. Also
`cargo nextest run --release -E 'binary_id(wat::reflection)'` and `-E 'test(doctest)'`.
Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 900`.
Diagnostics go to **stderr** — judge by exit code AND empty output, never grep alone.

Delete any scratch `.wat` that must fail; `tests/lint/wat_scripts_fixes_load.rs` type-checks
everything under `wat-scripts/`.

## STOP triggers — ship nothing further and report

- **STOP-1.** If row 6 fails — the verb works from ordinary code but a macro body cannot call it —
  STOP and report the verbatim definition-time error. That is the F5 registration, and it is the
  reason this verb exists.
- **STOP-2.** If `parse_type_node` cannot read one of the four spellings you need, STOP and report
  which. Do not add a second parser or a special case.
- **STOP-3.** If two spellings of what you believe is one type compare `false`, STOP and report both
  parsed `TypeExpr`s. That is either a canonicalisation gap in `parse_type_node` or a real difference,
  and guessing which would ship the wrong answer.

## Your report

The diff, per file — all three registrations shown separately. Every acceptance row with verbatim
output, rows 3 and 6 especially. What surprised you. Anything you inspected and left alone, and why.
