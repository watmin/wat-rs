# NOTE — `metadata-of` returns TWO SHAPES depending on which store answered

> Found 2026-08-30, immediately after the wat side entered `wat-doc`, by the builder asking the
> flat question: *"so.... registry and frozen-world are both queried to resolve metadata-of ?..."*
> Yes — and asking what each returns exposed this. **No row, nothing drawn.**

## The measurement

```clojure
(type (get (metadata-of :wat::core::sort$native)   :purity))  ;; => "wat::runtime::Purity"
(type (get (metadata-of :wat::string::capitalize)  :purity))  ;; => "wat::WatAST"
```

**Same key, same reflection verb, two different types.** And they render identically in EDN —
`:wat.runtime.Purity/Pure` either way — so nothing surfaces the difference until a consumer tries to
`match` on the value and gets an AST node where it expected an enum.

`[[feedback_a_slot_with_two_implementations_is_two_slots]]`

## Why — the two branches build differently

`eval_metadata_of` consults both stores, registry first with an early return:

```rust
if let Some(entry) = registry().lookup_entry(&name) {
    …put(":purity", entry.purity.to_enum_value())…      // from a TYPED field
    return Ok(Some(map));
}
match sym.binding_metadata.get(&name) {
    Some(meta) => for (k, v) in meta {
        map.insert(keyword(k), Value::wat__WatAST(v))    // the RAW AST, verbatim
    }
}
```

The registry branch decodes typed entry fields. The symbol branch hands back whatever the author
wrote, undecoded.

## ⚠ Whose defect this is

**The fallback branch pre-dates this session** — it existed to surface `{:restricted-to […]}`, where
a raw AST is a perfectly reasonable answer and nobody had an intrinsic record to compare it against.

**This session's stone is what put the two shapes side by side**, by making a wat verb answer with
doc axes for the first time. I reviewed and shipped that stone without checking shape parity against
the branch three lines above it. The code is old; the defect is newly *reachable*, and newly *wrong*.

## ★ The fix is already built — both branches should emit from a typed structure

The registry branch emits from `IntrinsicEntry`. The wat branch should emit from a **`DocComment`** —
which `wat_doc::from_metadata` already produces, and which already has typed fields
(`purity: Purity`, `totality: Totality`, …). It is called at registration today and its result is
discarded after validation.

```
registry branch:  IntrinsicEntry  -> map          (today)
wat branch:       raw WatAST      -> map          (today)
wat branch:       DocComment      -> map          (the fix — same typed shape, same enums)
```

★ **Then the shapes converge by construction rather than by discipline**, which is the same property
`wat-doc` itself was built for: one contract, two entry points, no drift.

⚠ **Do not fix it by decoding the AST at the reflection layer.** That would be a third decoder for a
contract that already has one, and it would drift from `from_metadata`'s the first time an axis
gains a variant.

## What this shares with `:defined-in`

Both are the reflection surface presenting something as data that is not what it appears:
`:defined-in` publishes a hard-coded constant beside derived fields; `:purity` publishes two types
under one key. **A consumer cannot tell either by looking.** They are one stone's worth of work and
probably one stone.

## What retires this NOTE

`(type (get (metadata-of X) :purity))` returning the same type for every `X`, proven with one
intrinsic and one wat verb — and the same asked of `:totality`, `:determinism`, `:expand-time`, and
`:category`, because a fix that converges one key and not the others has moved the defect rather
than removed it.
