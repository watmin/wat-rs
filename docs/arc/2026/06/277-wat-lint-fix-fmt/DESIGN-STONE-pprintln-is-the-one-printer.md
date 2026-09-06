# DESIGN — STONE: `pprintln` is the one printer

> **Builder:** *"'wat has no raw stdout' ..... but ....... the form we want ..... is ..... precisely
> .... edn ...... yes?"* → *"pull it back... redraw with pprintln as the one printer"*

## THE CONTAINER IS ALREADY FREE — a record prints as the ruled form

```
(:wat::core::defrecord :probe::Row [doc <- :wat::core::String  added <- :wat::core::String])
(:wat::kernel::pprintln (:probe::Row :doc "line one\nline two" :added "1.0.0"))

#probe/Row {
  :doc "line one\nline two"
  :added "1.0.0"
}
```

Tag and brace on one line, two-space entries, closing brace alone. **Exactly the form the builder
specified, today, with no work.**

★ **And this dissolves the blocker the withdrawn stone was built around.** `FORMS=2` was about
*re-reading* `#wat.doc/Row {…}` as source text through the formatter. A record printed from a value
is never re-read. **The tagged-literal problem is not on this path.**

## WHAT `pprintln` ALREADY DOES RIGHT

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

`:args` one entry per line. Enums as tagged values. Real, unescaped output — **no decoder, no
`write-file`.**

## THE TWO DEFECTS

```
:doc       "…step-payload…`\n\nArc 278 Stone P12c — the explain payload builder…"     escaped, one line
:examples  (                                                                          generic EDN layout,
             :wat.core/defrecord                                                       not the ruled shape
             :probe/StepPayloadExampleTemp
             [celsius <- :wat.core/i64]
           )
```

## ★★ AND THE PROSE MODE IS SAFE — measured, because it is the obvious hazard

Emitting literal newlines inside a string violates strict EDN. The question is whether our own
reader takes it back:

```
(:wat::edn::read "{:doc \"line one
line two\"}")      →      { :doc "line one\nline two" }
```

**It round-trips.** So `push_edn_string`'s deliberate violation (`print.rs:104` — *"newlines left
LITERAL, continuation lines indent to the content column"*) is not a violation our stack cannot read.
It is the right rendering and it is in the wrong crate.

## ★★★ WHY `pprintln` AND NOT `print.rs` — the fact that decides it

```
crates/wat-doc  →  wat-source-derive, wat-reader          (nothing else)
crates/wat-macros  →  wat-doc         ← a PROC-MACRO crate
```

`print.rs` runs at **compile time**. `wat/fmt.wat` does not exist then, and the main crate depends on
`wat-doc` transitively, so adding the runtime is a cycle. **`print.rs` can never dress `:examples`.**
`pprintln` runs in the runtime and can. That is the whole argument, and it is why the withdrawn
stone pointed the wrong way.

```
                :doc prose        structure       :examples forms      can reach the rules
print.rs        ✓ real newlines   ✗ one line      ✗ one line           ✗ NEVER
pprintln        ✗ escaped         ✓ laid out      ✗ generic layout     ✓
```

## THE THREE PIECES

1. **A `:wat::doc::Row` record** mirroring `DocComment`'s fields, so `pprintln` yields
   `#wat.doc/Row { … }`. No new printer.
2. **A prose rendering for multi-line strings** — literal newlines, continuations at the content
   column. `push_edn_string`'s behaviour, in the runtime where it can be reached.
3. **`:examples` through the fmt rules.** `pprintln` can call them; `print.rs` cannot.

## THE CONTRACT DECISION

**The prose rendering must be scoped, not global.** Every string in the corpus rendering with literal
newlines is a far wider change than a doc row, and `edn::read` tolerating it is not a licence to
impose it everywhere. **Which strings get prose treatment is the stone's one contract choice** — and
it must be derivable from the value, not from a flag the caller remembers to pass.

## WHAT THIS STONE IS NOT

- **NOT `print.rs`.** It stays as it is; it serves the proc-macro path and cannot serve this one.
  Two printers remain, and this stone makes the runtime one complete rather than merging them.
- **NOT the tagged-literal reader fix.** Still real, still blocking a *post-pass* design nobody is
  building now. Named, not scoped.
- **NOT the sweep.** Arc 255 step 4 consumes this.

## ⚠ AND A CORRECTION THIS DESIGN EXISTS BECAUSE OF

I claimed *"wat cannot write a raw string to stdout"* and drew a whole stone on it. False: I was
printing the formatter's **String** — which `println` correctly escapes — instead of a **value**.
The builder found it in one sentence. Every "the tooling is missing" instinct in this arc should be
checked against *"am I printing a value or a string?"* first.
