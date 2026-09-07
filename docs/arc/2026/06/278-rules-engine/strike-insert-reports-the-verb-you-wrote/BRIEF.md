# BRIEF — make `insert` report `insert`

Read `DESIGN.md` first. **Live, user-visible, and already driven** — a program containing no
`insert-all` gets an error naming it. Thread one parameter; gate both arms.

## Read in order

1. `src/rete/kernel/insert.rs:142-145` — the checker's doc: *"Takes `op` rather than hardcoding one
   so the same check serves `insert` and `insert-all` and reports the verb the user actually wrote."*
   The mechanism is already built. This strike stops feeding it a constant.
2. `src/rete/kernel/insert.rs:191-200` — `insert_facts_on_session` and its
   `const OP: &str = ":wat::rete::insert-all"`. The const goes; `op` becomes a parameter.
3. `src/rete/kernel/insert.rs:55-85` — `eval_insert_public`. Its 3+-arity arm calls
   `insert_facts_on_session` at `:80`. **This is the wrong half** — the user wrote `insert`.
4. `src/rete/kernel/insert.rs:236-258` — `eval_insert_all_native`, the correct caller.
5. `tests/rete/probe_arc278_4c_retraction.rs` + its `.wat` — the house shape for a probe pair
   (a `.wat` fixture beside a `.rs` driver). Copy it.

## What ships

```rust
fn insert_facts_on_session(session: …, facts: …, list_span: &Span, sym: …, op: &'static str) { … }
```

`eval_insert_public` passes `":wat::rete::insert"`; `eval_insert_all_native` passes
`":wat::rete::insert-all"`. Delete the `const`.

## The gate — TWO arms, and read the structured field

A probe pair under `tests/rete/` driving both entries with a non-Record argument, asserting the
error's **`:op` field**, not a substring of the rendered message:

- `(:wat::rete::insert s <fact> <non-record>)` → `:op` is `:wat::rete::insert`
- `(:wat::rete::insert-all s [<fact> <non-record>])` → `:op` is `:wat::rete::insert-all`

**Both arms are required.** A single arm checking only `insert-all` passes today, over the defect —
which is exactly how this survived.

It is a **runtime** `TypeMismatch`, so the fixture loads and type-checks; it is not a `.wat.bad`.

## Mutation proof

Re-hardcode `op` at `eval_insert_public`'s call and confirm the `insert` arm REDs naming
`insert-all`; restore. Then do the same at `eval_insert_all_native` and confirm the *other* arm
reds. **Two mutations, one per caller** — the defect is that one of two callers was right, and a
single mutation cannot show that.

## Blast radius

`src/rete/kernel/insert.rs` (one signature, two call sites, one deleted const) · one new probe pair
under `tests/rete/`. **No `.wat` engine change. No arity or acceptance change.**

## STOP triggers

1. Either arm passes before the fix → STOP; the probe is reading the message text, not `:op`.
2. Threading `op` needs a change outside `insert.rs` → STOP.
3. Anything other than the reported verb changes → STOP.

## Prior result to copy for shape

`../strike-census-B-compiled-calls/SCORE.md` — two sites, two mutations, each quoted verbatim, and
the arm that fired named rather than the one predicted.
