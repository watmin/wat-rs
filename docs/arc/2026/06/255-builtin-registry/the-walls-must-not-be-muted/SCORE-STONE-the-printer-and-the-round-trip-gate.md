# SCORE — STONE: the printer, and the round trip that proves the migration

Struck against `[[BRIEF-STONE-the-printer-and-the-round-trip-gate]]`. Census at `e9497dfb0`.
`wat-edn` writer untouched. No commit.

## Against the bars

| # | what | result |
|---|---|---|
| 1 | printer reproduces the hand-written `:wat::core::char` row | ✅ same keys, same ns/name keywords, same examples; fence indent is the wrapping, not the emission — see verbatim below |
| 2 | round trip on a RICH row (`hologram.rs`) | ✅ `== doc`; `@yields` + `@example-norun` both exercised |
| 3 | round trip on `@see` and `@syntax` | ⚠ `@see` ✅ (`:wat::core::map`). `@syntax` **UNCOVERED** — see below. Not silence. |
| 4 | `@deprecated` covered OR declared uncovered | ✅ **COVERED**, via a constructed `DocComment`. Zero live users, as measured. |
| 5 | the gate is NOT vacuous | ✅ drop `:added` → `MissingAdded`. Mangle docstring margin → `prose` disagrees. Restored. |
| 6 | `wat-edn`'s writer untouched | ✅ `git diff --stat crates/wat-edn/` empty |
| 7 | no existing row changed | ✅ census **571 · 85 · 52**, unchanged |
| 8 | both crates' own tests hold | ✅ `cargo test --release -p wat-doc` 58/58; `-p wat-macros` 90/90 + trybuild |
| 9 | the floor | yours. After the refute: `no_loose_string_assert` is **0** on this tree (`cargo test --release --test lint no_loose_string_assert` ok). I did not re-run the floor. |
| 10 | clippy `--all-targets -D warnings` | yours. Scoped `-p wat-doc -p wat-macros` was 0. |

## The printer's output for `:wat::core::char`, verbatim

```
#wat.doc/Row {
  :doc "`(:wat::core::char s)` → the single `:wat::core::char` in the length-1
String `s`.

BMP-only: codepoints above U+FFFF (supplementary-plane) are rejected
with a clear diagnostic, inheriting the Stone 218.6b discipline from
wat-edn's BMP-only strictness. Errors: `s` is not length-1 (empty or
multi-char), or its single char is a supplementary-plane codepoint. Arc
220 slice 2."
  :added "1.0.0"
  :purity :wat.runtime.Purity/Pure
  :determinism :wat.runtime.Determinism/Deterministic
  :totality :wat.runtime.Totality/Unreviewed
  :expand-time :wat.runtime.ExpandTime/Unreviewed
  :category :wat.runtime.Category/Transform
  :args [[s :wat.core/String "a length-1 BMP string"]]
  :ret [:wat.core/char "the single character in `s`"]
  :examples [[(:wat.core/char "x") (:wat.core/char "x")]]
}
```

Beside the hand-written fence in `src/intrinsic/char.rs` (every line indented two more spaces
for source readability; `dedent` strips that wrap). Keys, keyword spelling, arg vector, ret
vector, examples vector: the same shape. Continuation lines of `:doc` sit at column 0 of the
unfenced emission — the map's own margin — so indenting the whole block for a ```edn fence is
`dedent`'s inverse. Trap door 1 did not fire on this row.

## Which rows the gate covers

| row | path | fields exercised |
|---|---|---|
| `:wat::core::char` | EDN fence → `from_metadata` → `print` → round trip | `:doc` (multi-line) `:added` five axes `:args` `:ret` `:examples` (`run: true`) |
| `:wat::holon::Hologram/make` | `@`-form `parse` → `print` → round trip | `:yields` + `:examples` length-1 (`run: false`) + compound fn-type `:args` + `Type/method` keyword |
| `:wat::core::map` | `@`-form `parse` → `print` → round trip | `:see` + `:yields` + parametric `:ret` `(:wat::core::Vector :- [U])` |
| `:wat::intrinsic::variadic-args-measurement` | `@`-form `parse` → `print` → round trip | rest arg (`xs...` ↔ `is_rest`) |
| constructed `@deprecated` | `parse` → `print` → round trip | `:deprecated ["1.2.0" "use :wat::core::other"]` |

`@deprecated` is **covered**. Constructed, because the corpus has zero live users. Said plainly.

## `@syntax` is UNCOVERED — same structural reason DESIGN left `@alias` OUT

`print` is `print(doc: &DocComment) -> String`. `@syntax` lives on `DocSpecialForm`, which has
no `from_metadata`-equivalent. The specified gate

```
from_metadata(edn_to_watast(wat_edn::parse(print(doc))))  ==  doc
```

cannot see it. Census: **36** rows carry `@syntax` (571 − 535 `no @syntax`); they are
`Kind::SpecialForm`. Opening that path is the alias stone's twin, and DESIGN named the alias
path OUT. I did not invent a second printer to make row 3 look green.

## The sabotage red, verbatim, then restored

Printer with `:added` emission deleted. Gate on hologram:

```
thread 'edn_doc::tests::round_trip_holds_on_hologram_make_yields_and_example_norun' panicked at crates/wat-macros/src/edn_doc.rs:485:13:
from_metadata refused the printed row:
#wat.doc/Row {
  :doc "`(:wat::holon::Hologram/make filter)` -> a fresh, empty `Hologram` sized
to the program's encoding dimension, routing lookups through `filter`."
  :purity :wat.runtime.Purity/Effectful
  :determinism :wat.runtime.Determinism/Deterministic
  :totality :wat.runtime.Totality/Unreviewed
  :expand-time :wat.runtime.ExpandTime/Unreviewed
  :category :wat.runtime.Category/Resource
  :args [[filter [:wat.core/f64 :-> :wat.core/bool] "a therm-routing filter function"]]
  :ret [:wat.holon/Hologram "a fresh, empty coordinate-cell store"]
  :examples [[(:wat.holon.Hologram/make (fn (x) true))]]
  :yields [[filter "a candidate key's cosine-similarity score against the probe, computed during `Hologram/get`'s filtered-argmax readout; filter returns whether that candidate counts as a match"]]
}
MissingAdded
test edn_doc::tests::round_trip_holds_on_hologram_make_yields_and_example_norun ... FAILED
```

The assertion that fired is `from_metadata refused the printed row` → `DocError::MissingAdded`.
The field is named. Restored; `print` emits `:added` again. Durable twins remain:
`the_gate_is_not_vacuous_dropped_added` (same `MissingAdded`) and
`the_gate_is_not_vacuous_mangled_docstring_margin` (`prose` disagrees, injected spaces land
in the field).

## What `from_metadata` had to grow, so the inverse could exist

The printer is only an inverse if the reader can hold everything the printer emits. Three
holes in `from_metadata` would have been STOP-2 on hologram / map / the rest-arg witness.
They were closed *in the one decoder*, not by a second one:

1. **`@example-norun`.** No metadata-map spelling existed (`run: true` on every length-2
   vector). Spelling: `[<expr>]` is norun; `[<expr> <expected>]` stays run. Trap door 2 is
   why hologram was named — a printer that emitted both kinds identically would have lost
   the flag, and this row would have caught it. The unverified `#=>` marker is not recovered
   (`DocExample::expected` is always `None` for norun); that is the `@`-parser's own
   contract, not a printer loss.
2. **Compound type tokens.** `:args`/`:ret` accepted only a bare `Keyword`. Hologram's
   `[:wat::core::f64 :-> :wat::core::bool]` and map's `(:wat::core::Vector :- [U])` would
   have been `MalformedDirective`. Compound forms now stringify back to wat source through
   the printer's own inverse.
3. **Rest args.** `is_rest` was hardcoded `false`. Printer emits `xs...`; reader strips
   `...` / `…`. Witness: `:wat::intrinsic::variadic-args-measurement`.

## The Type/method keyword, and why it is not STOP-2

`:wat::holon::Hologram/make` and `:wat::runtime::Purity::Pure` both become one EDN ns/name
keyword (`:wat.holon.Hologram/make`, `:wat.runtime.Purity/Pure`). The char-stone transcoder's
`fqdn_of` always joined with `::`, which would have reconstructed `Hologram::make` and failed
`==` on hologram's example expr.

The forward transform already lives in `edn/render.rs` as `wat_keyword_to_clojure_symbol`
(fold the type into the namespace when the wat leaf contains `/`). The reverse is
reconstructable: a method name does not start uppercase, a type and an enum variant do.
`Pure` stays `::Pure`; `make` becomes `/make`. Added to `fqdn_of`, with a unit test.
Not a gate special-case — the ONE transcoder, made total over the keyword shapes the
printer emits. `Bytes::to-hex`-style `::` methods would still collapse with `/` methods
onto the same EDN form; this corpus's gated rows use `/` or no slash, so they round-trip.

## Anything that surprised me

- The DESIGN formula `edn_to_watast(wat_edn::parse(print(doc)))` is shorthand. `print`
  emits `#wat.doc/Row {…}` (Tagged); `edn_value_to_watast` of Tagged is `Err`. The real
  path is `parse_edn_doc_row` (unwrap the tag, then transcode the body) — which is what
  the macro already does. The gate uses that.
- Flush-left continuation lines look wrong next to 2-space keys, and they are the
  correct inverse. Indent them with the keys and `wat_edn::parse` (no `dedent`) injects
  that indent into the prose. The char fence's "under-indented" `:doc` continuations were
  the worked example of this, not a quirk.
- `@syntax` was listed as a field the gate must cover, on a struct the printer cannot
  see. I did not paper over it.

## Files

```
crates/wat-doc/src/print.rs          NEW — print, the named emitter
crates/wat-doc/src/lib.rs            from_metadata: norun / compound types / rest
crates/wat-macros/src/edn_doc.rs     fqdn_of Type/method inverse; the gate tests
crates/wat-doc/src/print_tests__*.edn          5 goldens (byte-exact print)
crates/wat-macros/src/edn_doc__char_printed.edn  char row, captured not guessed
```

`git diff --stat crates/wat-edn/` empty. `src/intrinsic/char.rs` untouched.

## REFUTE — the 20 loose asserts, tightened, none runed

`[[REFUTE-the-floor-is-red-loose-string-assertions]]`. All 20 were in this stone's new
tests. None was a legitimately-loose case (no path/pid/hash/timestamp; the "names a field"
sabotage is `assert_eq!(err, DocError::MissingAdded)`, already exact). The printer's claim
**is** exact text — docstring margin, key order, flush-left continuations — so the `.wat`-golden
rubric applies (byte-identical, not `assert_edn_eq!` which would discard the layout that
*is* the inverse of `dedent`). `wat-doc` / `wat-macros` also cannot call `wat::assert_edn_eq!`
without a cycle.

Replaced every `contains`/`starts_with` with `assert_eq!` against a co-located captured
golden, or against the exact `prose` string for the margin sabotage. Round-trip
`assert_eq!(back, doc)` untouched.

```
cargo test --release --test lint no_loose_string_assert
  test no_loose_string_assert::tests_carry_no_loose_string_assert ... ok
```
