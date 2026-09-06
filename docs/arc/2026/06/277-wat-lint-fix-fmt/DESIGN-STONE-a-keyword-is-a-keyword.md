# DESIGN — STONE: a keyword is a keyword, not a tagged value

> **Builder:** *"a keyword is a keyword... it is not a tagged value..... that's a nonsense
> statement"* · and on the history: *"archaeology is useless — this is a flaw — the reasons do not
> matter — the wrong behavior is wrong."*

## THE DEFECT

Every slash-in-name accessor renders as a tagged AST map instead of a keyword:

```
support (#wat.ast/Keyword
          {:path ":wat::rete::Explained/support"}
          ex)
```

should be

```
support (:wat.rete.Explained/support ex)
```

Seven in one doc row. **89 corpus `@example` lines carry a slash-in-name accessor**, so the arc-255
sweep would write this shape into the doc comment of every one of them.

## ★★ THE BUG IS THE SPLIT, NOT THE KEYWORD

`keyword_from_wat_path` (`src/edn/render.rs:4483`) splits on the last `::` even when the name
contains a `/`:

```rust
let ns   = wat_reader::identifier::path(stripped).replace("::", ".");   // "wat.rete"
let name = wat_reader::identifier::leaf(stripped);                     // "Explained/support"
match Keyword::try_ns(&ns, name) { … Err(_) => verbatim_keyword(k) }   // ← the slash is in the NAME
```

`try_ns` refuses a `/` in the name — correctly, because `:wat.rete/Explained/support` is a
**two-slash** keyword no reader can parse. So it falls through to the verbatim carrier.

**But there is only one slash in the name because the split put it there.** The door already
provides the right pair:

```
receiver(":wat::rete::Explained/support") → ":wat::rete::Explained"
method(…)                                 → "support"

try_ns("wat.rete.Explained", "support")   → :wat.rete.Explained/support     ONE slash. Legal EDN.
```

★ **The comment at the site describes the symptom as the reason.** It says the verbatim carrier
avoids *"a two-slash keyword the reader cannot parse"* — and the two-slash form is exactly what the
wrong split creates. Split on the `/` and it never exists.

★★ **And the target shape already renders today.** The same doc row contains `:wat.rete.i64/<` —
a three-segment dotted namespace with a slash-separated name. `try_ns` accepts that shape; nothing
new is being asked of it.

## THE RULE

```
a name containing `/`   →  split on the `/`     (receiver / method)
a name without one      →  split on the last `::`  (path / leaf)      ← unchanged
```

Both accessors are on the ONE door (`crates/wat-reader/src/identifier.rs`) that
`tests/lint/one_name_grammar.rs` makes the only sanctioned name parser. **This stone uses the door
harder, it does not add to it.**

## WHY THE VERBATIM CARRIER MUST STAY

`verbatim_keyword` is not the defect and must not be deleted. Its own doc records what it replaced:
a bare `OwnedValue::String`, which was *"a SILENT TYPE CHANGE: a Keyword goes in, a String comes out,
and anything that reads the value back gets the wrong type with no diagnostic."* That reasoning is
correct and still applies to keywords EDN genuinely cannot spell.

**This stone shrinks what reaches it — it does not remove the fallback.** A keyword that still
cannot be spelled after splitting correctly should still be carried verbatim rather than mangled.

## BLAST RADIUS

```
src/edn/render.rs        keyword_from_wat_path — the split          9 call sites reach it
src/edn/bridge.rs        verbatim_keyword — UNCHANGED, just less used
```

⚠ **This changes rendered EDN across the corpus**, wherever a slash-in-name keyword is written. Every
golden and pinned EDN string carrying `#wat.ast/Keyword {:path "…X/y"}` moves to `:…X/y`. That is
the point, and it is also the risk: the acceptance must show the change is confined to that shape.

## THE CONTRACT DECISION

**The decoder must round-trip whatever the encoder now writes.** A keyword rendered as
`:wat.rete.Explained/support` has to read back as the same wat keyword `:wat::rete::Explained/support`
— otherwise this trades an ugly-but-lossless carrier for a pretty lossy one, which is strictly
worse than today.

## WHAT THIS STONE IS NOT

- **NOT deleting the verbatim carrier.** It stays for the genuinely unspellable.
- **NOT a change to `try_ns`.** It is right to refuse a slash in a name.
- **NOT the doc-row sweep.** This unblocks it; 89 examples depend on it.
