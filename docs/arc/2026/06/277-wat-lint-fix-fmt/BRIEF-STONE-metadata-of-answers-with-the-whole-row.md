# BRIEF — STONE: `metadata-of` answers with the whole row

Put the six doc-contract keys `metadata-of` already carries and silently drops, make `:ret` the pair
the decoder reads, and hold both branches to one shape. Read
`[[DESIGN-STONE-metadata-of-answers-with-the-whole-row]]` first — it carries the field-by-field
enumeration, the precedent that already ruled this class a defect, and the one asymmetry to leave
alone.

## READ IN ORDER

1. **`src/runtime.rs`, `metadata-of`'s registry branch** — the `put` calls and the scope-cut comment
   naming `:args`/`:examples`/`:see` as carried-but-unrendered. **This is the site.**
2. **`src/runtime.rs`, the branch immediately below** — the wat-binding branch, decoding through
   `wat_doc::from_metadata`. Its comment states the contract this stone must keep: *"the two branches
   cannot drift apart by inspection."* **Row 4.**
3. **`src/intrinsic/mod.rs:415`, `IntrinsicEntry`** — the full field list. `args`, `examples`, `see`,
   `deprecated`, `alias_of`, `ret_type` are all there.
4. **`crates/wat-doc/src/lib.rs`, `from_metadata`** — the ONE decoder, and the keys it reads:
   `:added :alias :args :deprecated :doc :examples :ret :see :yields`. **The emitted shapes must be
   the ones it reads, not similar-looking ones.**
5. **`crates/wat-macros/src/edn_doc.rs:580`, `round_trip`** — the existing gate. Row 3 extends it.

## SKETCH

```rust
// the six, beside the nine already there. Shapes are from_metadata's, not new ones:
put(":args",       …);   // [[name type "desc"] …]   from entry.args
put(":examples",   …);   // [[expr expected] …]      from entry.examples
put(":see",        …);   // [fqdn …]                 from entry.see
put(":deprecated", …);   // Option                   from entry.deprecated
put(":alias",      …);   // Option                   from entry.alias_of
put(":ret",        …);   // ★ now the PAIR [ret_type, ret] — was the description alone

// and the SAME six on the wat-binding branch, from DocComment's identically-named fields,
// so a key from either branch is the same shape by inspection.
```

## BLAST RADIUS

```
src/runtime.rs   metadata-of — BOTH branches
tests/           the round-trip acceptance + a both-branches-agree test + row 11's control
```

**No wat. No rule files. No formatter.**

## STOP TRIGGERS

- **STOP-1 — the acceptance is the ROUND TRIP, not the key count.** Six keys carrying empty vectors
  passes row 2 and fails the stone. `from_metadata(metadata-of <fqdn>)` must succeed and produce the
  entry's own doc.
- **STOP-2 — BOTH BRANCHES OR NEITHER.** The precedent's bar is *"converging one key only MOVES the
  defect."* If the wat branch cannot supply a key the registry branch can, **STOP and report** rather
  than shipping an asymmetry.
- **STOP-3 — do NOT emit `:yields` from the registry branch.** `IntrinsicEntry` has no `yields`
  field; `DocComment` does. Emitting empty or fabricated is the papering-over this stone is against.
  **Report it as a named gap.**
- **STOP-4 — do NOT add `:syntax`.** Carried, but outside the doc contract; the decoder never reads it.
- **STOP-5 — `:ret` becomes the pair, and re-measure before you change it.** 0 of 344 call sites read
  `:ret` today; `:arity` (48), `:totality` (18), `:purity` (5), `:determinism` (5), `:name` (1) do.
  **If your census disagrees with mine, mine is the one to doubt** — say so and stop.
- **STOP-6 — if any existing test goes red, STOP.** Capture the whole block verbatim; do not re-run.

## ⚠ TRAPS

- **The builder was told FIVE keys and the disk says SEVEN carried.** `alias_of` was missed in the
  first count, and `syntax` is carried but out of contract. The DESIGN's table is the census; trust
  it over any prose count, including mine.
- **Row 5 is the anti-empty-vector check.** `:wat::rete::step-payload` declares five `@arg`s. Fewer
  than five means the emit is structural rather than real.
- **Row 10 is the purpose.** A `#wat.doc/Row` must render from the lookup ALONE — no
  `:wat::intrinsic::examples` scan. Arc 255's sweep does this 576 times; today each one is a linear
  scan over 615.
- The registry stores **FQDN** spellings; the doc-row printer converts to dotted. Not this stone's
  concern, but do not "fix" a spelling on the way through.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Run the cheap
targeted checks — the round trip, both branches, the five `@arg`s, and a row rendered from the lookup
alone — and report the numbers.
