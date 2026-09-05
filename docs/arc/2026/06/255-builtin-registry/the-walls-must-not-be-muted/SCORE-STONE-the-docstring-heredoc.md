# SCORE — STONE: the docstring becomes a real heredoc

Amends the printer at `0582f1919`. `wat-edn` untouched. No commit.

## Against the brief

Printer `:doc` continuations now indent to the content column (one past the opening
`"`). `parse_edn_doc_row` strips that same fixed count after the fence-wide dedent.
`src/intrinsic/char.rs`'s fence moved with the form. The flush-left golden was
renamed and rewritten to pin the aligned shape.

## The printer's `:wat::core::char` output, verbatim

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

Continuation lines sit under the first content character, not under `#wat.doc/Row`.

## The whitespace-witness row

`round_trip_holds_on_prose_with_blank_indent_and_escaped_quote`. Prose was:

```
intro paragraph

    (indented sample)
he said "hello"
```

A blank line, a line with four spaces of its own, an embedded `"`. Round-trip
`assert_eq!(back, doc)` holds; the four spaces on `(indented sample)` survive.
The printer emits 8 margin spaces plus those four, and the reader strips exactly
8. A per-line-minimum rule would have eaten the sample; this one does not.

## `--test lint`

```
cargo nextest run --release --test lint
  Summary [  91.273s] 118 tests run: 118 passed, 0 skipped
```

Including `no_loose_string_assert`.

Previous gate rows still pass: char, hologram, map `@see`, rest-arg, constructed
`@deprecated`, sabotage `MissingAdded`, mangled margin (now injects two spaces
*beyond* the content column; recovered prose is `line one\n  line two`).

## What surprised me

The round-trip of `print(doc)` cannot go through raw `wat_edn::parse` anymore —
aligned continuations would become data. The strip lives in `parse_edn_doc_row`
(the one place the macro and the gate both enter), and `into_owned` lifts the
parse off the stripped buffer's lifetime. `extract_edn_fence` stays the
fence-wide `textwrap` step only.

Blank prose lines are emitted as truly empty (no margin spaces). Indenting them
would also round-trip, but it would feed whitespace-only lines to the fence-wide
min-indent filter for no gain.

## Files

```
crates/wat-doc/src/print.rs                          content-column continuations
crates/wat-doc/src/print_tests__aligned_docstring.edn  renamed from flush_left
crates/wat-macros/src/edn_doc.rs                     strip_string_continuation_margins
crates/wat-macros/src/edn_doc__char_printed.edn      aligned golden
src/intrinsic/char.rs                                fence continuations re-indented
```

`git diff --stat crates/wat-edn/` empty.
