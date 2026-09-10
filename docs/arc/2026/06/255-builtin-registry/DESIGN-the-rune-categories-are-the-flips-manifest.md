# DESIGN — ③a-iii: the rune categories ARE the flip's manifest

Drawn **before** ③a-ii commits, per its own brief. Sibling of
`DESIGN-the-wall-decides-what-the-grep-only-proposed.md`, which carries the four-questions ruling
and the known-answer control that shaped the gate.

## The wall

`tests/lint/one_variant_separator.rs`, modelled on `tests/lint/one_name_grammar.rs` (same
`collect_rs` walk, same `code_hit` before-the-`//` rule, same `rune:lint(<name>)` exemption form).

```
hit  = DATA      "::" | b"::"                    the separator as a literal argument
     | COMPOSE   }:: | ::{  inside a string      the separator joining a computed part
     | ACCESSOR  identifier::path( | identifier::leaf( | .path() | .leaf()
gate = TypeDef::Enum | EnumVariant | /\bvariant/i    the file talks about variants at all
skip = crates/wat-reader/src/identifier.rs (the door) · this lint's own file
```

⚠ **ACCESSOR is a hit even though it spells no separator.** `rete/expr_ir.rs:1321` is the whole
argument: it composed through the door and decomposed through the general accessor, and the flip
broke it anyway. A site can be half-migrated and look finished.

## Why this lint and not an eighth entry in the old one

`one_name_grammar.rs` bans five literal call-shapes. `rsplit_once('/')` is among them;
**`rsplit_once("::")` appears nowhere in that file** — not in `BANNED`, not in the module doc's
*"⚠ Not banned, deliberately"* list. Fourteen of them were live, nine were variant decomposers, and
the gate's own doc asserted its five were *"precise — measured, not assumed."* They were: against
**arc 109's** census. `HAERESIS EST ITERVM ROGARE`.

★ Extending that list would re-ship the same false confidence. A hand-list of **banned** shapes
fails silently on the shape nobody listed; a hand-list of **exempt** shapes fails LOUDLY, on an
honest site, in front of a human. This lint inverts the direction. It is the same reason the
`:wat::*` blanket had to become a registry question rather than a longer prefix list.

## The rune — a CLOSED category, then a reason

```
// rune:lint(one-variant-separator, <category>) — <why this site is not a variant separator>
```

The lint **validates the category against a closed set** and rejects anything else, so a rune
cannot be written by reflex:

```
namespace     the "::" separates namespace segments        :wat::core::foldl
type-path     it composes/decomposes an ENUM's OWN path    format!("{}::Op", surface.name)
display       it renders a name into human-facing prose    "variant {t}::{v} declares {n} field(s)"
edn           it translates "::" <-> "." for EDN rendering ns.replace("::", ".")
not-a-name    the string is not a wat name at all          a Rust path, a doc string, a test fixture
```

⛔ **`variant` is NOT a category.** A site that decomposes an enum from its variant must route
through the pair; the lint refuses a rune that claims it. That is the one-way door: the only way
past this wall for a variant site is through `identifier.rs`.

## ★ The categories are the manifest

`display` is the point. Roughly eleven `format!`s render `Type::Variant` into a message a person
reads — `record/construct.rs:314,331,352`, `check.rs:13961,13978`,
`edn/render.rs:3090,3101,3110,3960`, `holon/outcome.rs:350,367`. They are neither composition nor
decomposition; they are the **surface spelling**, and after the flip they must read `Type.Variant`
or the substrate will print a form its own reader refuses.

They are scattered and share no call shape. After ③a-iii they share a grep:

```bash
grep -rn 'rune:lint(one-variant-separator, display)' src crates tests
```

A category that would otherwise be a note in a doc becomes a query against the source. `edn` does
the same job for the translation sites, which must NOT move — the EDN rendering of a variant
already uses `compose_variant_render`, the dot form, so the flip makes the two spellings agree
rather than diverge.

## Scope — measured, not estimated

```
40 files · 191 screams        76 COMPOSE · 60 DATA · 55 ACCESSOR
src 151 · crates 26 · tests 14
heaviest: src/edn 35 · src/check.rs 26 · src/rete 20 · src/types.rs 15 · src/load 15
```

After ③a-ii lands, **every one of these is non-variant by construction** — the 21 that were variant
sites now read through the door and no longer hit. So ③a-iii's rider work is: assign a category and
one clause of reason per line. No rider decides *route or rune*; that decision was made once, in
③a-ii, by the orchestrator.

## Acceptance

```
the lint fires on a synthesized violation and on a rune with an off-list category   (non-vacuity)
the lint is SILENT on a routed site: decompose_variant(x) contains no hit by construction
0 screams across src/ crates/ tests/ with the door and the lint's own file skipped
floor 5318/5318 via scripts/floor.sh · clippy --release --all-targets -D warnings = 0
```

⚠ **The 21-site known-answer control expires with ③a-ii.** It was the instrument for choosing the
GATE, and its line numbers die the moment the routing lands. What survives it is this lint: from
③a-iii onward the question *"is there a hand-rolled variant separator?"* is answered by a run, not
by a grep and a hope. `[[feedback_an_instrument_must_outlive_the_number_it_produced]]`

## What ③a-iii does NOT close

The flip itself. ③b remains **one landing** — `compose_variant` and `decompose_variant` flip to `.`,
the `display` runes' strings flip with them, and the wat-fix codemod rewrites the corpus's 9,946
occurrences in the same commit, via `wat/fix.wat`'s STASH-DANCE with a `/tmp` dry-run and a `diff`
first. Measured before this session: flipping the composer alone takes `wat/core.wat` down at load,
so there is no partial version of ③b that works.
