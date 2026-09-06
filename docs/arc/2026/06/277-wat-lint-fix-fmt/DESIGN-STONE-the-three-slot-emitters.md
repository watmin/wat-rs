# DESIGN — STONE: the three slot emitters, and the tag that will not stay attached

> **Builder, after seeing the real output:** *"we need a formatter for printing metadata-maps …
> that handles the `:doc` string correctly … the form must be `#wat.doc/Row { :doc "…" … }`"*
> then: *"draw the stone for the three slot emitters"*

## WHAT ALREADY WORKS — printed and read, not inferred

`wat_doc::print`'s real output, captured by a `println!` inside the round-trip test (the only way
to see it — see the tooling note below):

```edn
#wat.doc/Row {
  :doc "`(:wat::rete::step-payload …) -> :wat::rete::DerivationStep`

        Arc 278 Stone P12c — the explain payload builder. Given one (sfact, alpha-id) match edge
        from a Token's matches chain, builds the full `DerivationStep` payload:
        …"
  :added "1.0.0"
  :purity :wat.runtime.Purity/Pure
  …
}
```

**The container is exactly the ruled form**, and `:doc` is already correct — `push_edn_string`
(`print.rs:104`) leaves newlines LITERAL and indents continuations to the content column, which is
what its own doc says it is for. **Neither needs building.**

## THE DEFECT — three slots emit as one line

```
:args     [[session :wat.rete/Session "…"] [alpha_id …] [bindings …] [sfact …] [supporting …]]   ~430 cols
:ret      [:wat.rete/DerivationStep "the per-edge explain payload (…)"]                          137 cols
:examples [[(:wat.core/do (:wat.core/defrecord …) … )]]                                        ~1500 cols
```

`print_args`, `print_ret` and `print_examples` exist (`print.rs:165/188/197`) and each returns a
single line. That is the whole gap.

## ★★ AND ONE DESIGN IS STRUCTURALLY IMPOSSIBLE — measured, not judged

**`print.rs` cannot call the formatter.**

```
crates/wat-doc  depends on:  wat-source-derive, wat-reader          (and nothing else)
crates/wat-macros  depends on:  wat-doc          ← and wat-macros is a PROC-MACRO crate
```

`print` therefore runs at **compile time**, inside the macro expansion that registers the row. The
runtime — and `wat/fmt.wat` with it — does not exist at that point, and the main crate depends on
`wat-doc` transitively, so adding one would be a cycle. **"Wire `print.rs` to call `wat fmt`", which
the 294 seam has named as the next step all along, cannot be built as written.**

## THE SPLIT THAT FALLS OUT OF THAT

```
:args   a vector of [name type "desc"] triples   ← EDN DATA. Rust lays it out. No duplication.
:ret    a pair [type "desc"]                     ← EDN DATA. Rust lays it out. No duplication.
:examples  wat FORMS                             ← the rules' job, and Rust must NOT do it
```

★ **`:examples` is the one that must not be laid out in Rust.** Doing so mints a second formatter
beside `wat/fmt.wat` — the class `one_name_grammar` exists for, which went red on this arc's `:then`
stone and which the `holon_is_vsa_only` ruling calls *"a convention stated in prose"* failing twice.

**So `:examples` is dressed by a POST-PASS**, and that is already proven to work: feeding the whole
row through `format-source` after the three-spellings seam produces the ruled shape —

```
(:wat.core/defrecord :probe/StepPayloadExampleTemp
  [celsius <- :wat.core/i64])
(:wat.core/let
  [rules (:wat.rete/collect-rules :probe)
   session (:wat.rete/compile rules)
```

⚠ It did **not** work before that seam, because the row's heads are dotted. I measured the failure,
recorded it as *"the row cannot go through wat-fmt"*, and carried that stale result for several
stones. It is re-measured here.

## ⛔ AND THE POST-PASS HAS A BLOCKER — the tag will not stay attached

```
input:   #wat.doc/Row {:a 1 :b 2}
output:  FORMS=2
         #wat.doc/Row

         {:a 1 :b 2}
```

**The reader sees a tagged literal as TWO forms** — a symbol and a map — so rule 3 (one blank line
after each top-level form) separates them and orphans the tag. Minimal case, reproduced above.

**Until `#tag {…}` is one form to the formatter, the post-pass destroys the very shape the builder
specified.** That is this stone's first deliverable, not a footnote.

## THE CONTRACT DECISION

**Rust lays out DATA; the rules lay out FORMS; and the tag is one form.** Each of the three slots is
assigned by what it holds, not by which language is convenient at that moment.

## ⚠ A TOOLING GAP THIS STONE DEPENDS ON AND DOES NOT FIX

**wat cannot write a raw string to stdout.** All six `kernel` stdio verbs EDN-encode their argument —
`println` and `pprintln` alike — so a String comes back quoted with `\n` escapes. The only raw path
is `io::write-file` then reading the file. Every render in this arc was therefore read through a
shell decoder, and a decoder mis-handling `\n` inside a string is what made an hour of this session's
output inconsistent. **Named because the acceptance below must not be read through one.**

## WHAT THIS STONE IS NOT

- **NOT `print.rs` calling the formatter.** Structurally impossible; see above.
- **NOT a layout engine in `wat-doc`.** `:args`/`:ret` are data and get simple emitters; `:examples`
  is the rules' and stays theirs.
- **NOT the sweep.** Arc 255 step 4 consumes this; it is not this.

## FILES

```
wat/fmt.wat or a rule       #tag {…} is ONE form              ← the blocker, first
crates/wat-doc/src/print.rs print_args, print_ret             ← data layout, Rust
tests/                      goldens compared as BYTES, not structurally (see below)
```

⚠ **The existing golden is compared STRUCTURALLY** (`assert_edn_matches_file!`), so it passed while
its `:doc` was a one-line escaped string and `print`'s was real newlines. **A layout stone cannot be
verified by a structural compare.** Its acceptance must diff bytes.
