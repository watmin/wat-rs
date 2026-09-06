# NOTE — wat-fmt cannot render a `#wat.doc/Row`, and it should not try. (2026-09-06)

> **Builder:** *"let's see if the #wat.doc/Row printer can handle our 'pretty print' in
> metadata-map form.... i think we need the rules to prove they work here before we resume 255?"*

**Proved. They do not — and two of the four reasons are not rule problems at all.**

## THE TEST

A real doc-row shape — multi-line docstring with a heredoc margin, `:args`, and an `:examples`
vector holding two wat expressions — put through `format-source`.

⚠ **The fixture lives in the session scratchpad, NOT `wat-scripts/`.** A bare data literal cannot
type-check, and `every_wat_scripts_file_loads` parses AND checks every `.wat` under `wat-scripts/`.
Putting it there would have rotted the gate.

## ⛔ FOUR DEFECTS, AND THE FIRST TWO ARE BLOCKING

### 1 — `#wat.doc/Row {…}` is TWO forms to the wat reader

```
read-string "#wat.doc/Row {:doc \"x\"}"   ->   top-level forms: 2
   kind=symbol  src=#wat.doc/Row
   kind=map     src={:doc "x"}
```

**The wat reader has no tagged-literal concept.** `#wat.doc/Row` is a symbol that happens to begin
with `#`; the map is an unrelated sibling. The formatter can never hold them together as one unit,
and duly emitted the tag alone on one line with the map indented beneath it.

### 2 — a multi-line string is re-emitted as ONE line with `\n` escapes

```
emitted:   … the length-1\n        String `s`. …
```

`write_wat_source`'s `StringLit` arm escapes `'\n' => "\\n"` — **correct for value round-tripping,
fatal to the heredoc.** The docstring's whole visual form, which is the point of the
`#wat.doc/Row` design, collapses into one very long line.

### 3 — a MAP's key/value pairs are not one-per-line

```
{:doc "…" :added "1.0.0" :purity :wat.runtime.Purity/Pure :args
```

The pair-run rule fires on LIST forms. A map literal gets no pair treatment at all.

### 4 — small kwarg calls over-explode inside `:examples`

```
(:p::T
  :a 1)
```

A one-pair constructor becomes two lines. Reported at the pair-run stone as a possible carve-out;
here it is what an example actually looks like.

## ★ THE ARCHITECTURAL ANSWER — two printers, one boundary

Defects 1 and 2 say **wat-fmt is the wrong tool for the map**. It reads *wat*; a doc row is *EDN*
with a tagged literal and a heredoc, and `crates/wat-doc/src/print.rs` **already emits both
correctly** — `src/intrinsic/char.rs` carries the proof, a real multi-line docstring with its margin
intact, produced by that printer.

> **The Rust doc printer owns the ROW — the tag, the map, the heredoc, the key order.**
> **wat-fmt owns each EXAMPLE EXPRESSION, which is ordinary wat.**

The wiring is: for each entry in `:examples`, format the expression with `wat fmt`, hand the string
back to `print.rs`, which places it in the map at the right indent. **Neither printer learns the
other's job**, and defects 1-3 stop being defects because wat-fmt never sees a map or a tag.

⚠ **Defect 4 remains real** — it is inside the expression, which IS wat-fmt's territory. A one-pair
constructor exploding to two lines is a live question for the builder, and `:examples` is where it
will be seen most.

## ⛔ WHAT THIS MEANS FOR THE ORDER

The builder asked whether the rules prove out before resuming 255. **They prove out for the
expressions and not for the row** — which is the right answer, because the row was never wat-fmt's
job. So 255 is not blocked; the boundary just has to be built rather than assumed:

```
1  wire print.rs to call wat fmt for each :examples entry     ← the actual 255 step
2  decide defect 4 (a one-pair constructor's shape)           ← the builder's, and cheap
—— neither needs any further layout rule ——
```

★ **And the earlier claim stands, narrowed honestly.** `[[PARKED-the-migration-waits-on-wat-fmt]]`
parked 255 because the sweep would bake 609 one-line examples in. **The EXAMPLES now format to 0
over 120, idempotent** — that reason is discharged. It never said the ROW would be formatted by
wat-fmt, and this NOTE is why it should not be.
