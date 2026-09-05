# DESIGN — the registry prints the replacement. The migration is not a text transform.

> **Builder, 2026-09-04:** *"ok... so.. we can use the existing registry to print these values?...
> i wonder... can we do somthing smart with the doc string? … if we pretty print a #wat.doc/Row it
> handles the doc string as a "smart heredoc"? … then... we can have the regstriry pring the
> replacements strings for the current "@names vals" parser? … its basically like how clojure does
> thier doc here strings?"*

Both halves measured. Both are yes, and together they are stronger than the migration plan they
replace.

## ⭐ THE REGISTRY IS COMPLETE — it holds every `@` field

`IntrinsicEntry` (`src/intrinsic/mod.rs:~300`) stores:

```rust
name · prose · added · syntax · args · ret_type · ret · examples · see
     · alias_of · purity · determinism · totality · expand_time · category · deprecated · yields
```

Against the corpus's fourteen directives — `Category Determinism ExpandTime Purity Totality added
alias arg example example-norun ret see syntax yields` — **nothing is missing.** `example-norun` is
carried by `ExampleSubmission`'s own run flag, not a second field.

★★★ **So the migration tool is a wat script that asks the registry and prints.** And that is
faithful in a strictly stronger sense than the plan it replaces (*"reuse the proc-macro's own
`@`-parser"*): the `@`-parser is what PRODUCED the entry, so printing the entry prints **what the
parser understood**. There is no second interpretation of the source at any point — one read, one
write, and the source text is never re-parsed by anything else.

⛔ **But the WAT-FACING surface is narrower than the entry.** `:wat::intrinsic::Row` carries 12
fields and `metadata-of` answers 13 keys; **`prose`, `args` detail, `examples`, `see`, `syntax`,
`yields` and `deprecated` are in the Rust entry and NOT askable from wat** (measured on
`:wat::core::char`). That gap is the first piece of work, and it is the RULING's *"every name
answerable"* unmet on the declaration surface itself.

## THE SMART DOCSTRING — needed, and it is the exact inverse of something already built

`write-pretty` escapes newlines today (measured,
`wat-scripts/scratch-pad/255-how-does-write-pretty-handle-a-docstring.wat`):

```
input   {:doc "line one\nline two\nline three" :added "1.0.0"}
output  "{\n  :doc \"line one\\nline two\\nline three\"\n  :added \"1.0.0\"\n}"
```

That is precisely the `\n`-escaped prose the builder rejected. What is wanted is Clojure's
docstring shape — a literal multi-line string whose continuation lines are indented to the map's
own margin:

```edn
#wat.doc/Row {
  :doc "something really long
        continuing here, indented to the margin
        and here"
  :added "1.0.0"
}
```

⭐ **The READ half already exists.** The char stone built `dedent` in
`crates/wat-macros/src/edn_doc.rs:72` — *"the fence's own least-indented line sets it, exactly like
Python's `textwrap.dedent`"*. What is missing is the WRITE half: an emitter that re-applies the
margin. **Print with margin, read with dedent** — the two must be exact inverses.

## ⭐⭐ AND THAT PAIR IS THE MIGRATION'S OWN CORRECTNESS PROOF

If the registry prints and the macro reads, then for every row:

```
read(print(entry))  ==  entry
```

is a **gate over all 558 rows**, not a text diff. It is the identical shape as
`probe_can_doc_types_reconstruct_the_checker_scheme` (432/432, already green) and as the char
stone's own byte-identity check — generalised from one row to the corpus.

★ **This changes what "verify the migration" means.** Not *"did the text convert correctly"* — a
question a diff can only answer by eye across 5,400 lines — but *"does the registry answer
identically before and after"*, which is mechanical, total, and already how this campaign measures
everything else.

## THE ORDER

```
1  WIDEN the wat-facing surface so prose/args/examples/see/syntax/yields/deprecated are askable.
     Nothing can print what it cannot ask.
2  THE WRITE HALF — a #wat.doc/Row emitter with the margin-aware docstring, exact inverse of
     `edn_doc::dedent`. Its acceptance is the round trip, not the look of the output.
3  THE ROUND-TRIP GATE — read(print(entry)) == entry, over every row. Armed BEFORE any row is
     migrated, so the sweep runs under a gate that can see it.
4  THEN the sweep: print every row, replace its `@` block, and let the gate say whether the
     registry still answers the same.
```

⛔ **Step 3 before step 4 is the same ordering the doctest gate just proved.** `no_rc_use.rs`:
*"a lint raised at zero is a wall, a lint raised at 1306 is a campaign."* A 558-row rewrite under
no gate is how a quiet corruption ships.

## WHAT THIS DOES NOT SOLVE, and must not be assumed to

- **The alias path.** `@alias` rows go through `parse_special_form` → `DocSpecialForm`, which has no
  metadata-map reader. `#wat.doc/Alias` is designed and not yet writable
  (`[[STONE(255/walls) the ```edn doc row is IMPOSED]]`).
- **The unknown-key hole.** `from_metadata` does 13 targeted lookups and never enumerates, so a
  stray key is silent — in the EDN form AND in the wat-side maps. A printer never emits a stray key,
  so the migration will not TRIP it; that is exactly why it should be closed on its own merits and
  not left to be discovered by a hand-written row later.
- **Transcoder totality.** `Tagged`/`Inst`/`Uuid`/`BigDec`/namespaced-`Symbol` have no `WatAST`
  spelling. A printer that emits only what the entry holds cannot produce one — but that is a
  property of the printer, not a proof about the reader, and the two must not be conflated.
