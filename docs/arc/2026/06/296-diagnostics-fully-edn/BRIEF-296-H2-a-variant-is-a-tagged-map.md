# BRIEF — 296 H-2: a variant is a tagged map

> Read `DESIGN-STONE-H-variants-are-maps.md` first, including BOTH amendments (2026-08-20 `:-` on the
> payload; 2026-09-06 the match arm). The probe is committed and RED:
> `tests/types/probe_arc296_h2_variant_tag_and_body.rs`.

## THE WORK, IN ONE PARAGRAPH

A variant renders `#ns.Enum/Variant [f0 f1 …]` — a positional vector under a tag that a RECORD can
produce byte-identically. Make it render `#ns/Enum.Variant {:f0 … :f1 …}` — the dot moves into the
tag's NAME half (where H-1's wall guarantees no record can forge it) and the body becomes the same
keyed map every other named datum already gets. The field names are ALREADY on the value
(`EnumValue.names`, arc 296 G′) — this is the writer's shape, not a lookup.

## READ IN ORDER — the rooms

```
src/edn/render.rs:4252   Value::Enum WRITE arm — builds `{type_path}::{variant}` then
                         tag_from_type_path, and emits OwnedValue::Vector. THE SUBJECT.
                         Its comment claims "body-shape is a perfect discriminator
                         (map=record, vector=variant, nil=unit)" — that comment is what
                         this stone deletes, and it is why the tag was never asked to
                         discriminate.
src/edn/render.rs:3163   enum_variant_ns — the READ side's expected namespace. One caller
                         (:3044). Mirror of the writer; must move in the same motion.
src/edn/render.rs:3030   the enum coerce — splits the tag, matches expected_ns, then reads
   ..3130                the body. Unit arm at :3065 demands an EMPTY VECTOR `[]`.
src/edn/render.rs:4290   Value::ForeignVariant WRITE arm — same vector body. ⛔ see STOP-2.
src/edn/render.rs:4525   tag_from_type_path — 13 callers. RECORDS MUST NOT MOVE. Do not
                         change this fn's behaviour for a non-variant path; give the
                         variant its own tag builder or a variant-aware argument.
src/value/value.rs:1180  ForeignVariantValue { enum_class, variant, fields } — NO names.
                         ⛔ see STOP-2.
src/edn/render.rs:3206   value_to_json_natural's Enum arm — ALREADY a map with `_type` +
                         named fields (G′). Read it: the JSON path has already made this
                         exact move, and it is the shape to mirror.
```

## SKETCH

```rust
// render.rs:4252 — the write arm
Value::Enum(ev) => {
    let tag = variant_tag(&ev.type_path, &ev.variant_name);   // ns/Enum.Variant
    let entries: Vec<(OwnedValue, OwnedValue)> = ev.names.iter().zip(ev.fields.iter())
        .map(|(n, fv)| (OwnedValue::Keyword(Keyword::new(n.clone())), value_to_edn_with(fv, types)))
        .collect();
    OwnedValue::Tagged(tag, Box::new(OwnedValue::Map(entries)))   // unit → Map(vec![]) = `{}`
}
```

A unit variant is `{}`, not `[]` — an empty map, mirroring `#wat.core/Option.None {}`. The read side
inverts it: split the tag's NAME half on the dot, `Enum` must equal the declared type's leaf, the
part after the dot is the variant, and the body's keys are matched against the declared field names.

## BLAST RADIUS

```
Rust      the 7 rooms above. No new types except ForeignVariantValue's names field.
.edn      395 occurrences / 307 files — REGENERATED, not edited: UPDATE_EDN=1 over the suite,
          then PROVE data-equal per file (H-2a's method — see EXPECTATIONS row 7).
.wat      33 occurrences / 18 files — wat-fix codemod (R21). NEVER hand edits.
.rs        71 occurrences / 20 files — inline literals; targeted.
```

## STOP TRIGGERS — rejection criteria, not permission slots

- **STOP-1 — a record's tag moves.** `tag_from_type_path` has 13 callers and records are NOT this
  stone's subject. If making the variant tag dotted changes ANY record's rendering, stop and report:
  the variant needs its own builder, not a mutation of the shared one.
- **STOP-2 — `ForeignVariantValue` has no field names.** Its sibling `ForeignRecordValue`
  self-carries its keys precisely because a `read-foreign` consumer lacks the type. A foreign
  variant read from `#ns/Enum.Variant {:x 1}` HAS keys on the wire and nowhere to put them, so the
  round-trip would drop them — reintroducing the `field-N` loss this stone exists to delete, on the
  foreign path. Carry the names the way `ForeignRecordValue` does. **If that turns out to change the
  foreign READER's contract, stop and report** — do not invent a key naming scheme.
- **STOP-3 — a golden that the regen cannot reproduce.** Hand-editing a `.edn` golden defeats the
  proof. Report the shape that defeated it.
- **STOP-4 — a `.wat` site the codemod cannot rewrite.** Same rule (R21). Report the shape.
- **STOP-5 — the unit-variant `[]` arm at `render.rs:3065` refuses `{}`.** It is arc 278 Stone A.0's
  wall and it is CORRECT for today's wire. Move it; do not add a second accepting arm, or the
  reader accepts both wires forever and nothing proves the migration finished.

## WHAT IS OUT OF SCOPE, AFFIRMATIVELY

- **The match arm** (`[Variant {:f v} body]`) — ruled 2026-09-06, its own strike, depends on this
  wire. A map pattern in an arm is rejected today (`"map/set literal is not a valid match sub…"`).
- **The variant CONSTRUCTOR form** — whether `(usr/Container.X {:x 42})` replaces kwargs/positional
  is undecided by the builder. Untouched here.
- **`Option`/`Result` moving from Rust literals in `types.rs` into wat declarations** — H's design
  puts it in this stone; it is CUT to H-3. 75% of the corpus surface is those two enums, so bundling
  would re-touch everything this stone just regenerated, and any red would be ambiguous between the
  tag flip and the moved declaration. That is H-1's own argument for landing alone.
- **The record-steals-a-variant's-constructor defect** —
  `[[NOTE-a-record-silently-steals-a-variants-constructor]]`. Registration-time, not wire-time.

## PRIOR ART TO COPY FOR SHAPE

`BRIEF-296-H2a-finish-the-recapture-migration.md` and its SCORE — the regen-and-prove-data-equal
method this stone's row 7 reuses, already exercised at 208 sites.
