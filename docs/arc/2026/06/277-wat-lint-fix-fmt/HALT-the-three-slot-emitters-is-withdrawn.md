# HALT — STONE-the-three-slot-emitters is WITHDRAWN. Do not strike it.

**Stop work on `[[BRIEF-STONE-the-three-slot-emitters]]`.** Its design rests on a claim I made and
then disproved: *"wat cannot write a raw string to stdout."*

## THE CLAIM WAS FALSE, and the builder found it in one sentence

> *"'wat has no raw stdout' ..... but ....... the form we want ..... is ..... precisely .... edn ......
> yes?"*

Yes. I was printing the **formatter's output String**, which `println` correctly EDN-escapes, and
concluding the tooling was missing. Printing the **value** works today:

```
(:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::rete::step-payload))

{
  :args [
    [:session :wat.rete/Session "the compiled session (network read via `session_network`)"]
    [:alpha_id :wat.core/i64 "the AlphaNode id for this condition"]
    …
  ]
  :see []
  :purity #wat.runtime.Purity/Pure []
```

Real, unescaped, pretty EDN. No decoder, no `write-file`, no shell.

## WHY THE STONE IS WRONG, NOT JUST MIS-SCOPED

The brief put the layout in `crates/wat-doc/src/print.rs` **because I believed wat could not show me
text**. With that gone, the two printers trade places:

```
                    :doc prose        structure       :examples forms
print.rs            ✓ real newlines   ✗ one line      ✗ one line
pprintln            ✗ escaped \n      ✓ laid out      ✗ generic EDN layout
```

Each already has what the other lacks — and the deciding fact is one I DID measure correctly:
**`print.rs` runs at proc-macro time and can never reach the formatter; `pprintln` runs in the
runtime and can.** So the printer that should grow is `pprintln`, not `print.rs`, and the stone
briefed the opposite.

## WHAT SURVIVES, so the re-draw does not re-derive it

- **`#wat.doc/Row {…}` reads as `FORMS=2`** — a tagged literal splits from its map and rule 3
  orphans the tag. Still a real blocker for any post-pass. Reproduced minimally.
- **`print.rs` cannot call the formatter** — `wat-doc` is a build-time dependency of the proc-macro
  crate `wat-macros`; no runtime exists at print time and adding one is a cycle. Measured.
- **`push_edn_string` deliberately violates EDN** — newlines LITERAL, continuations indented to the
  content column — which is exactly why `:doc` reads well there and badly through `pprintln`.
- **A structural golden cannot verify a layout stone.** `assert_edn_matches_file!` passed for weeks
  while `:doc` was in the wrong shape.

## THE RE-DRAW'S SUBJECT

**`pprintln` becomes the one printer**: it already lays out the structure; it needs prose strings
that keep their newlines, and `:examples` dressed by the rules it can actually reach.

Nothing in this halt is a criticism of the strike. The brief was wrong before it was read.
