# NOTE — the shape is already designed. The READER is the only question. (And four more muted walls.)

> **Builder, 2026-09-04:** *"what design work to do need to do?.... wat has defns with
> metadata-maps already showing real things that part correctly..."*

**He is right, and the measurement says so.** `wat/core.wat`'s `:wat::core::sort` already carries
the whole shape, and the wat reader parses every part of it
(`wat-scripts/scratch-pad/255-can-the-wat-reader-hold-a-doc-map.wat`):

```
map with :: keywords   {:purity :wat::runtime::Purity::Pure :added "1.0.0"}          -> map  ✅
map with :ret vector   {:ret [:wat::core::Vector "a new vector"]}                    -> map  ✅
map with :examples     {:examples [["(:wat::core::sort [3 1 2])" "[1 2 3]"]]}        -> map  ✅
TAGGED #wat.doc/Row    #wat.doc/Row {:added "1.0.0"}                             -> SYMBOL  ⛔
```

The `:examples` escaping problem I called *"the part to design cold, before starting"* **is already
solved on disk** — a vector of two-element vectors of strings, holding wat source with quotes,
parsing today. `:ret` likewise. There is no shape to design.

## ⛔ THE ONE REAL DECISION — WHICH READER, and it is a genuine fork

```
wat-reader   ✅ parses wat's OWN spelling verbatim — `::` keywords and all
             ✅ ALREADY a wat-macros dependency (no new dep, no cycle question)
             ✅ ONE spelling across Rust doc comments and wat source — the migration's whole point
             ⛔ `#wat.doc/Row` lexes as a SYMBOL. No general tagged-value support; it knows
                `#holon` and `#{` only.

wat-edn      ✅ parses `#wat.doc/Row {…}` as a real Tag
             ⛔ REFUSES `::` keywords — InvalidKeyword("keyword begins with :: ")
             ⛔ therefore forces ns/name (`:wat.core/foldl`) in doc comments while wat source keeps
                `:wat::core::foldl` — TWO SPELLINGS, the exact thing this migration exists to remove
```

**The builder's argument is the argument for `wat-reader`**: the maps already parse, in the spelling
the language already uses. Choosing `wat-edn` re-introduces the split on the first line written.

The tag was wanted for **type discrimination** (`Row` vs `Alias`, so an alias has nowhere to put an
axis). Three ways to keep that without `wat-edn`:

1. **Teach `wat-reader` general tagged values.** It already has the reader-macro mechanism —
   `#holon` desugars to `(:wat::holon::literal X)` at `crates/wat-reader/src/parser.rs:395`. A
   general `#ns/Name {…}` would serve this and every future tagged form.
2. **Discriminate on a key** — `:alias` present ⇒ alias. Weaker, but `DocError::AliasDeclaresAxis`
   already refuses the contradiction at compile time, so the wall exists either way.
3. **A wat form instead of a map** — `(:wat::doc::row {…})` / `(:wat::doc::alias {…})`. Reads as
   wat, needs no reader change at all, and the head IS the discriminator.

⛔ **Not decided here.** This is the builder's ruling; the measurement is what it needed.

## ⛔ AND FOUR MUTED WALLS THE CHASE TURNED UP

**1. Three of my four census probes exited 1 — and I never saw it, because I read every one
through `head`.** `[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]`, four times in one
session, by the orchestrator, in the instruments it was committing as durable references. The
DATA was sound (it printed before the error, and cross-checks against independent greps agree),
but an instrument that exits 1 is not trustworthy on its face. All four now exit 0.

**2. ⛔⛔ THE CAUSE IS A LIVE, ACCIDENTAL WITNESS OF THE ARC'S FOUNDING TARGET.** The probes ended
with `(:wat::core::nil)` — a CALL to a name that does not exist. Measured, unpiped:

```
:zzz::nope::absolutely-not     an unknown NAMESPACE      --check EXIT=1, diagnosed by name
(:wat::core::nil)              a wrong name inside :wat:: --check EXIT=0, SILENT — 0 bytes out
```

That is `is_reserved_prefix` blanket-accepting every `:wat::` head, which is **exactly** what Phase
3a exists to kill. The arc's founding target produced a witness by itself, in the orchestrator's
own scratch files, and it is now on disk as one.

**3. `every_wat_scripts_file_loads` PARSES and TYPE-CHECKS. It does not RUN.** All three broken
probes passed it. The gate's stated job is that *"a scratch program that rots goes RED and cannot
become a graveyard that reads like live code"* — it catches parse-rot and type-rot, and is blind to
runtime-rot. Whether that is a gap or the intended scope is a decision, not an oversight to
silently widen; but nobody has made it.

**4. FM 20, committed by me, inside the investigation of muted gates.** I wrote
`… | head -3; echo "CHECK EXIT=$?"` and read `head`'s exit as the checker's. Caught within the same
minute and re-run unpiped — but it is the fourth instance today of a pager standing between me and
a result.

## WHAT THIS CHANGES ON THE BOARD

Step 5's "design cold first" item is **struck** — `:examples` and `:ret` are designed and parsing.
What replaces it is a single ruling: **which reader, and therefore whether the doc row is a tagged
value, a keyed map, or a wat form.**
