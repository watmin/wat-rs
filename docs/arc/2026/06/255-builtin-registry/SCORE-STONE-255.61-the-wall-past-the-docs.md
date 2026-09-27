# SCORE — STONE 255.61: the wall, past the docs

Measurement. Nothing lands but this score and `probes-255.61/`. Worktree
`/tmp/255.61-wall/wt` (detached `65b2196cf`), `../holon-rs` symlinked beside
it. The worktree is removed.

The library still does not link. The release floor and the census did not
start. Against 78/6014 at 255.14: this layer is still the compile, one step
past 255.60's 582 reader errors.

The wall diff was applied after the corpus conversion, not before. A binary
with the wall up cannot parse the unconverted corpus or the codemod (255.60).
The conversion binary is the one that still accepts `::`.

## Corpus

Same codemod, two-phase, `./target/release/wat`.

| phase | files | rc | time |
|---|---|---|---|
| `wat/**/*.wat` | 65 | 0 | 1442 s |
| rebuild `--bin wat` (no wall; stdlib is `include_str!`) | | 0 | 51 s |
| every other tracked `*.wat` | 2230 | 0 | 1249 s |

## Doc directives

`probes-255.61/convert-doc-fragments.wat` calls `:wat::fix::fix-text` the
same way `to-faithful-clojure.wat` does. Spans were cut with the same rules
as `take_type_token` and the `@example` `#=>` split
(`crates/wat-doc/src/lib.rs:488` and `:721`). Each span was its own file.
rc 0, 2006 of 2006.

| | n |
|---|---|
| spans | 2006 |
| files | 103 |
| `@arg` type | 675 |
| `@ret` type | 546 |
| `@example` form (both sides of `#=>`) | 649 |
| `@example-norun` form (left side only; the right side is not read) | 136 |
| rewritten, no `::` left in the span | 1982 |
| rewritten, a string inside still contains `::` | 17 |
| unchanged | 7 |

No `/// :ret` line contains `::`. The one `:ret` 255.60 reported is the edn
fence in `src/intrinsic/char.rs:42`, whose source is already
`:wat.core/char`. It is not a missed span. See (d).

Isolated fragments, measured before the batch:

| input | fix-text |
|---|---|
| `:wat::core::i64` | `wat.core/i64` |
| `(:wat::core::Vector :- [:wat::core::i64])` | `(wat.core/Vector :- [wat.core/i64])` |
| a full `(map … -> :wat::core::i64 …)` | `->` becomes `:-`, and the return type becomes `wat.type/i64` |
| `[:T :-> :U]` | unchanged |
| `:Acc` | unchanged |

A type token alone has no arrow in front of it, so the codemod emits the
reference symbol `wat.core/i64`, not the type spelling `wat.type/i64`. That
is what was spliced into `@arg` and `@ret`.

## Link

`cargo test --doc --release --offline` in the worktree, wall applied. rc 101.
`could not compile wat (lib) due to 501 previous errors`. Not re-run.
Nextest did not start. No wall binary, so no census.

First block:

```
error: #[wat_intrinsic] :wat::core::u8: malformed `@arg` directive: type token must start with `:` (e.g. `:wat::core::Bytes`); grammar is `@arg <name> <type> <desc>`
   --> src/numeric/convert.rs:121:1
```

The token there is now `wat.core/…` (or the same shape). The check is
`crates/wat-doc/src/lib.rs:641` for `@arg` and `:693` for `@ret`: the type
must start with `:`, `(`, or `[`. `wat.core/i64` is a symbol. It fails that
check before the reader is asked.

| bucket | n |
|---|---|
| `@arg` must start with `:` | 396 (352 intrinsic + 44 special form) |
| `@ret` must start with `:` | 104 (76 intrinsic + 28 special form) |
| `:ret` still rejected by the reader | 1 (`src/intrinsic/char.rs`) |
| total | 501 |
| files | 92 |

The 13 `@example` parse errors from 255.60 are gone. Those forms now parse.
The `::` reader errors on `@arg`/`@ret` are gone too. They moved to the
colon-prefix check. 582 − 13 examples − the type-token errors that were
parametric forms (they start with `(` and no longer contain `::`) = the 500
bare keywords, plus the one `:ret` that was never a `::` in the source.

## Classes

**(a) Not converted.** Seven spans, unchanged, file:line:

- `src/reflect/verbs.rs:403` — the string `"wat::core::i64"`
- `src/reflect/verbs.rs:625` — the string `":probe::renamed"`
- `src/reflect/verbs.rs:1911` — the symbol `wat::cache::Lru.Hit`
- `src/intrinsic/reflect.rs:1049` — `"wat::core::i64"`
- `src/intrinsic/keyword.rs:71` — `"wat::core::i64"`
- `src/intrinsic/edn.rs:337` — `"some::unknown::Rec"`
- `src/intrinsic/edn.rs:386` — `"some::unknown::Kind"`

Seventeen more were rewritten around a string that still holds `::`
(`keyword-node ":wat::core::i64"`, `from-string "wat::core::i64"`, and the
same shape). `fix-text` does not edit string contents. Lines:
`src/rete/step_payload.rs:157`, `src/reflect/verbs.rs:1815`,
`src/reflect/verbs.rs:1911`, `src/intrinsic/reflect.rs:829` through `:832`
and `:945` through `:949`, `src/intrinsic/keyword.rs:101`, `:132`, `:163`,
`:194`, `src/intrinsic/grep.rs:52`.

**(b) Minted at run time.** Did not fire. The lib did not link. The doors
255.60 walled (`from-string`, `keyword-node`, `symbol-node`, `fresh-symbol`,
the `wat-edn` constructors) and the format sites it listed are still in the
diff and still unexecuted.

**(c) `src/` now fails on the spelling.** The 500 `@arg`/`@ret` errors.
The codemod turned the type keyword into a symbol. The doc grammar still
demands a keyword (or a `(`/`[` form). One predicate:
`crates/wat-doc/src/lib.rs:641` and the `@ret` twin at `:693`.

**(d) Another substrate break.** `fqdn_of`
(`crates/wat-macros/src/edn_doc.rs:255`) rebuilds an EDN keyword
`:wat.core/char` into the wat keyword `:wat::core::char`
(`format!(":{wat_ns}::{name}")` at line 286). `metadata_type_token` then
asks the walled reader to parse that, and the reader refuses. The source
line does not contain `::`. That is the remaining `:ret` error, verbatim
the reader-refusal text, at `src/intrinsic/char.rs:13`.

`ReservedPrefix` on `--check` of converted `wat/core.wat` was measured in
255.60 and not re-run: there is no wall binary, and the no-wall binary's
answer is already quoted there. The 255.14 classes were not reached.

**(e) Meaning.** Two, both measured. A type token converted alone becomes
`wat.core/i64`. The same keyword after `->` inside a real form becomes
`wat.type/i64`. The docs now say the core symbol, because the fragment had
no arrow. And the seventeen strings are keyword *values* (`from-string`,
`keyword-node`). The codemod correctly left them; they are data. Under the
runtime wall, `from-string` of `"wat::core::i64"` would be `MalformedForm`.
That call did not execute.

**(f) Tests that pin the old spelling.** Not reached. The lexer's
`keyword_double_colon_path` is still in the wall diff and still expects
`::` to lex.

**(g)** Nothing else in the 501.

## Smallest cure

One check clears 500 of 501: let a type token be the symbol `wat.core/…` /
`wat.type/…`, at `crates/wat-doc/src/lib.rs:641` and `:693`. The parametric
forms already pass.

The one left is `fqdn_of` (`crates/wat-macros/src/edn_doc.rs:286`), which
puts `::` back into a keyword the fence had already spelled with a dot.
Until that stops, `src/intrinsic/char.rs` fails the reader even though its
source has no `::`.

Those two sites are the whole of this compile. Behind them, the floor is
still unrun.
