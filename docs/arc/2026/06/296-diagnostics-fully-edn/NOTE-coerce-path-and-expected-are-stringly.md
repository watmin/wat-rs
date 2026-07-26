# NOTE — the coerce coordinate is stringly: `path` segments carry punctuation, `expected` is a keyword in a String

> **Surfaced 2026-07-25** (arc 278, `RequestMalformed` Stone 1) by the builder asking the right question of a
> value I had just called "genuinely data": *"what is path here? what does 'path' /mean/?"* It doesn't survive
> the question. Builder-ruled to stay in 296: *"if 296 is still pending keep it there… we could add a NOTE to
> go fix it."* Recorded per the arc-109 `NOTE-*.md` convention.
>
> **This is largely NOT new** — it is `AUDIT-prose-in-errors.md` **item #10** (L2), catalogued weeks ago and
> still unfixed. What arc 278 added is a **live wire consumer** of it, plus one genuinely new instance.

## The concrete value (printed off the wire, not inferred)

```clojure
#dos2.Bag.PutResponse/RequestMalformed [["items" "[0]"] ":wat::core::String" "Integer"]
```

Three fields, three different verdicts. The mixed verdict is the whole point of this note.

### 1. `path` — DEFECT (= audit item #10, now with a consumer)

**What it means:** a coordinate into the offending value — *"field `items`, element `0`"*. Genuinely
structured: an alternating walk of **field names** and **indices**.

**How it is represented** (`src/edn_shim.rs:1645-1657`):

```rust
pub struct EdnCoerceError { pub expected: String, pub got: String, pub path: String }

impl EdnCoerceError {
    fn at(mut self, segment: &str) -> Self {
        self.path = format!("{}{}", segment, self.path);   // concatenated, punctuation included
    }
}
```

One String, built by `format!`, segments carrying their own punctuation (`.items`, `[0]`), then split back
into a `Vector<String>` at the wire — **with the punctuation surviving**. So in `["items" "[0]"]`:

- `"items"` — a field name; legitimately a String. ✓
- `"[0]"` — **the integer `0` wearing bracket punctuation as text.** ✗ A consumer must detect the bracket
  form, strip it, and parse the int — a mini-parser to recover a number the substrate already had.

Audit item #10 named this and named the cure: *a `Vector` of segments / `#wat.kernel/FieldPath`.*

### 2. `expected` — DEFECT, and NEW: introduced by arc 278 Stone 1

`":wat::core::String"` — quotes included. **The declared type IS a keyword**; the substrate holds it as one;
we render it to text and the consumer must parse it back. A keyword in a String — the same class this arc
exists to kill, freshly minted.

Stone 1's four-questions ruled `expected`/`got` both Strings for **symmetry** — the argument being that an
asymmetric pair implies a comparison the substrate cannot make. That reasoning is backwards: it **degraded
the half that was structured to match the half that could not be.**

### 3. `got` — NOT a defect. Leave it alone.

`"Integer"` is the **EDN shape** of an untyped arrival (`edn_shape_name`, `src/edn_shim.rs:1659+`). The value
came off the wire with **no declaration**; it has no type to structure. Rendering it as a type form would
**fabricate information**. Stone 1's reasoning here was exactly right and must survive the fix.

## The rule this yields (for 296 generally)

**Structure what has a structured form; leave prose where none exists — and do NOT sacrifice one to match the
other.** The honest shape is *asymmetric*, because the fields have different provenance:

```clojure
(:wat::core::defenum :wat::edn::PathSeg :wat::enum::Pure
  :Field [name <- :wat::core::String]
  :Index [i    <- :wat::core::i64])

:RequestMalformed [path     <- :wat::core::Vector<wat::edn::PathSeg>   ;; [#…/Field ["items"] #…/Index [0]]
                   expected <- :wat::core::keyword                      ;; :wat::core::String  — as itself
                   got      <- :wat::core::String]                      ;; "Integer" — honestly prose
```

Every element carries its own kind; nothing needs re-parsing; a rule engine can match `Index` directly.

**This sharpens — and partly corrects — the rule recorded at Stone 1** (*"the prose-vs-structured rule binds
data the program computes on; a type rendered for a human or a log is not that"*). True of `got`. False of
`expected`, which is a live keyword the program can absolutely compute on, and false of `path`'s indices.

## Blast radius when picked up

`EdnCoerceError` (`src/edn_shim.rs:1645`) and its `RuntimeErrorKind::EdnCoerceMismatch` projection —
audit #10's original targets — **plus** every `RequestMalformed` site. Arc 278 Stone 2 is sweeping that
variant across the corpus **now**, so the fix will be a second codemod pass over those sites. Accepted
deliberately (builder: keep it in 296): a corpus sweep is a `wat/fix.wat` migration, which this project does
not fear.

## Status

**DEFERRED to 296**, builder-ruled 2026-07-25. Grounded: `src/edn_shim.rs:1645-1657` (the concatenation),
`:1659+` (`edn_shape_name`), the wire value printed above by the orchestrator's own run. Kin:
`AUDIT-prose-in-errors.md` item #10 (the root, L2, pre-existing) and
`NOTE-value-to-edn-renders-fields-positionally.md` (the sibling rendering defect found the same day).
