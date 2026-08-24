# BRIEF — STONE D · `join` widens to `Seqable`

Widen `:wat::core::string::join`'s second parameter from `(Vector :- [T])` to `(Seqable :- [T])`, so
it accepts Vector · PersistentVector · List · Stream. The renderer is already total (279.3); only the
container is narrow. Design: `DESIGN-STONE-D-join-widens-to-seqable.md` — read it first.

## Read in order — every name here was signature-checked; none is from memory

1. **`src/check.rs:17534`** — `join`'s `TypeScheme`. Its `params[1]` is
   `Parametric{head:"wat::core::Vector", args:[Path("T")]}`. **This is the check-time refusal**;
   widening the runtime alone changes nothing, the type-checker rejects a Stream first.
2. **`src/string_ops.rs:483-492`** — `eval_string_join`'s element door. One `match` on
   `Value::Vec(items)`; every other value takes a `TypeMismatch` arm. This is the runtime gap.
3. **`src/collection/transform.rs:1187`** — `seqable_value_to_stream(coll, op, coll_span)`. The
   value-level normaliser. Its own doc says it exists so a caller can *"COMPOSE through it on an
   already-evaluated `Value` instead of re-deriving the same container walk"* — which is exactly
   your situation at (2). **Use it. Do not write a container match.**
4. **`src/collection/transform.rs:709-768`** — `eval_stream_to_vec`. Copy its drain loop shape:
   `crate::stream::realize(&cur, sym, span)?` → `match` on `Stream::Empty` / `Stream::Cons{head,tail}`
   → `cur = Arc::clone(tail)`. Its `Thunk`/`NativeThunk` arm is `unreachable!` because `realize`
   always returns `Empty|Cons`.
5. **`wat/seq.wat:75-92`** — the `Seqable :- [T]` surface and its four `extend-type`s. This is the
   set your widened door must reach, and it is the authoritative spelling.
6. **`wat/seq.wat` around `:293-312`** — `reduce`'s collapse to two clauses, and the paragraph
   beginning *"⚠ AND IT COST NOTHING"*. That paragraph is your performance contract.

## Sketch — fill it, do not invent a shape

```rust
let types = sym.types().map(|a| a.as_ref());
let pieces_owned: Vec<String> = match eval(&args[1], env, sym)?.value_owned() {
    // FAST PATH, unchanged: an eager Vector keeps its direct iterator.
    Value::Vec(items) => items.iter().map(|i| render_str_total(i, types)).collect(),
    // WIDENED: normalise once, render each element as the walk forces it.
    other => {
        let mut cur = seqable_value_to_stream(other, OP, args[1].span())?;
        let mut out = Vec::new();
        loop { /* realize → Empty: break · Cons{head,tail}: push render_str_total(head), step */ }
        out
    }
};
Ok(Value::String(Arc::new(pieces_owned.join(&sep))))
```

`seqable_value_to_stream` is `pub(crate)` in a sibling module — import it; do not duplicate it.

## The performance contract — load-bearing, not advisory

**A `Value::Vec` must NOT route through the stream normaliser.** 118.B7 shipped `reduce`'s collapse
at zero cost precisely by refusing that normalisation: *"that would force every eager reduce onto the
lazy path for a Stream it never needed."* If your diff sends Vectors through the lazy path, the stone
is wrong even with green tests.

## The probe you create — it is the acceptance artifact

`tests/value/probe_stone_D_join_over_seqable.rs` with a co-located `.wat` fixture. Four rows, all of
which must be GREEN when you are done:

1. **Vector** — `(join "-" (Vector :- [i64] 1 2 3))` → `"1-2-3"`. *No-regression; green today.*
2. **Stream** — `(join "-" (map inc (Vector :- [i64] 1 2 3)))` → `"2-3-4"`. *The gap. Red today,
   at CHECK time, with the message quoted in the design.*
3. **List** — the same join over a `List`. *Proves the widening reached the surface's whole set, not
   just Stream.*
4. **Rendering survives the widening** — a non-string element through the Stream path
   (e.g. joining i64s) renders identically to the Vector path. *This is the row that catches a
   widening that forgot `render_str_total`.*

Row 4 is the one that discriminates a real fix from a plausible one. Do not drop it.

## Blast radius

`src/check.rs` (one `TypeScheme`), `src/string_ops.rs` (one `match`), plus your new probe + fixture.
No new types. No changes to `wat/seq.wat`, to `transform.rs`, or to `render_str_total`.

## STOP triggers — rejection criteria. Ship nothing on the row; report it.

1. **`seqable_value_to_stream` does not accept the shape you hold, or is not reachable from
   `string_ops.rs`.** Report the exact compiler error. Do NOT hand-roll a container match — that
   re-derives the classification the shared door exists to own, and it is the quadratic-List trap the
   converter's own doc names.
2. **A Vector's path changes at all.** If you cannot widen without touching the eager path, STOP and
   say so — the zero-cost property is the contract, not a bonus.
3. **The check-time widening cascades beyond `join`.** If changing `params[1]` to `Seqable` makes
   other call sites go red, capture the full failure list verbatim and STOP. That is a finding about
   the scheme's reach, not work to absorb.
4. **`length`/`empty?` on a Stream surfaces.** Out of scope — a known arc-118 defect
   (`118/UX-118.7`). Report and move on; do not fix it here.

## Method

Verify with `cargo nextest run --release -E 'test(probe_stone_D)'` and
`target/release/wat --check` on your fixture. Report your numbers for those. Do not run the full
floor — the orchestrator runs it centrally once the tree is quiescent.

You may not spawn sub-agents. If the slice proves larger than the design says, report that and stop.

## Report

The four probe rows with actual results; the diff summary per file; and any surprise, especially
whether row 4 passed on your first attempt. If a STOP fired, quote it verbatim.
