# SCORE — STONE 255.59: the clj oracle meets wat's source reader

Measurement only. No reader change. Drawn on `d26e82f2b` (the draw of
`c8c3a22ea`). STOP-1 on the tag: choices and measured consequences, no fix.

## How the three answers were compared

The brief says to compare forms against clj's printed golden. **The golden
has no printed value.** `golden.txt` is `VERDICT\tINPUT` (`OK\tnil`), written
by `regen.clj` from `(edn/read-string line)` as `"OK"` or `"ERR"`. The code
wins. Values came from clj, which this stone ran because the golden does not
store them: `/usr/local/bin/clj` on the 221 corpus lines, `(edn/read-string)`
then `pr-str` (221 lines, rc 0). A second clj run printed `(class v)` for the
rows where the print can lie. Those class checks are named below.

Entry point is `wat_reader::parse_all_with_file` (`crates/wat-reader/src/parser.rs:253`).
The question is how many forms a source string becomes. `parse_one_with_file`
(`parser.rs:238`) returns `TrailingContent` (`parser.rs:246`) when more than
one form remains. Both were called.

Structural equality is `WatAST` mapped onto `wat_edn::Value` variant by
variant, then `Value::PartialEq` (`crates/wat-edn/src/value.rs:70`). The tree
was not printed and re-read. Re-reading hides the bug: a symbol `##Inf`
reparsed by `wat-edn` becomes the float, which then compares equal to clj.
That path was thrown out after it reported 51 divergences and agreed on
`##Inf`.

A reader symbol or keyword whose flat spelling equals clj's `pr-str` is the
same symbol when `Symbol::try_new` / `Keyword::try_new` cannot build it
(multi-slash, a colon in the name, `/`). That shortcut is wrong for the three
`##` floats, because clj prints a double as `##Inf`. Those three were checked
by class and moved into (H).

`wat-edn`'s `parse_owned` is the third column. It is the data reader the ward
already runs. It is not the source reader.

Probe rc 0. 221 lines, golden and clj prints aligned (asserted per row).

## Counts

| | n |
|---|---|
| corpus | 221 |
| agree with clj (same verdict, or one form whose mapped value equals clj's) | 152 |
| spelling equals clj's print, not listed as a divergence | 21 |
| diverge | 48 |
| of which the quote-family desugar, exempt | 4 |
| left | 44 |

152 + 21 + 48 = 221.

| class | clj accepts, reader differs | reader accepts, clj refuses | exempt |
|---|---|---|---|
| (H) `#` dispatch | 8 | 5 | |
| (K) `::` / colon in a name | 0 | 10 | |
| (M) desugared reader macro | 1 (`'x`) | 3 | 4 |
| (M) claimed, not desugared (`@`, `^`) | 1 (`^:m x`) | 1 (`@x`) | |
| (O) other | 14 | 5 | |

(K) is only the reverse direction. No corpus row is a `::` form clj accepts
and the source reader rejects.

## Spelling matches clj (not divergences)

Reader flat spelling equals clj's `pr-str`. `wat-edn` refuses the three
multi-slash rows (the ward already says the data reader must, and that the
source reader may tolerate them). The other eighteen parse in `wat-edn` too,
except `.1` and `.5`: clj's class for both is `clojure.lang.Symbol` (measured),
and `wat-edn`'s lexer sends `.` followed by a digit to `lex_number`
(`crates/wat-edn/src/lexer.rs:140-144`). The source reader matches clj. The
data reader is the one that makes them numbers. The verdict-only golden
cannot see that.

`/`, `:/`, `a:b`, `foo:bar`, `:a:b`, `a/b/c`, `a/b/c/d`, `clojure.core//`,
`.1`, `.5`, `a.b:c`, `a:b.c`, `:a.b:c`, `a-b:c`, `a:b-c`, `:a-b:c`, `a:b:c`,
`a#:b`, `a:#b`, `foo/a:b`, `:ns/a:b`.

## (H) `#` — 13

The lexer handles two `#` shapes and nothing else: `#holon` (`lexer.rs:437`)
and `#{` (`lexer.rs:448`). No other `#` token. The rest falls through to
`lex_symbol`. `WatAST` (`ast.rs:61-172`) has no tagged, instant, uuid, or
discard variant. Its variants are Int, Float, Rational, BigInt, Char, Bool,
String, Nil, Keyword, Symbol, List, Vector, Map, Set.

clj accepts, reader is a different shape. `wat-edn` matches clj on every row
except the bare date, which `wat-edn` refuses on purpose (the ward's
RFC-3339 exemption) while the source reader still splits it:

| input | reader | clj (measured) |
|---|---|---|
| `#inst "1985-04-12T23:20:50.52Z"` | symbol `#inst`, then the string. `parse_one` is `TrailingContent` | `java.util.Date` |
| `#uuid "f81d4fae-7dec-11d0-a765-00a0c91e6bf6"` | symbol `#uuid`, then the string. `TrailingContent` | one value (`wat-edn` agrees; class not re-printed) |
| `[a #_b c]` | one vector of three symbols, `a`, `#_b`, `c` | vector `[a c]` |
| `#_1 2` | symbol `#_1`, then int 2. `TrailingContent` | long `2` |
| `#inst "1985-04-12"` | symbol `#inst`, then the string | clj accepts (promotes the date). `wat-edn` refuses |
| `##Inf` | symbol `##Inf` | double, print `##Inf` |
| `##-Inf` | symbol `##-Inf` | double |
| `##NaN` | symbol `##NaN` | double |

`wat-edn` reads `##Inf` / `##-Inf` / `##NaN` as the three IEEE floats and
rejects any other `##` name (`crates/wat-edn/src/lexer.rs:499-502`).

clj refuses, reader accepts:

| input | reader |
|---|---|
| `#myapp/Foo {:x 1}` | symbol `#myapp/Foo`, then the map. clj: no reader function for tag `myapp/Foo`. `wat-edn` accepts it as a generic tag |
| `#myapp/Person {:first "F"}` | same shape |
| `#inst "not-a-timestamp"` | two forms. clj and `wat-edn` both refuse |
| `#uuid "not-a-uuid"` | two forms. clj and `wat-edn` both refuse |
| `#x` | one symbol `#x` |

### Not in the 221 lines

clj was run on these because the golden does not contain them. Same shapes,
not new ones.

| input | reader | clj |
|---|---|---|
| `#u/T 7` | symbol `#u/T`, then int 7. `TrailingContent` | no reader function for tag `u/T`. `wat-edn` accepts. Same split as `#myapp/Foo` |
| `#wat.core/Span {:start 1 :end 2}` | symbol `#wat.core/Span`, then the map | no reader function for tag `wat.core/Span`. Same split |
| `#_ 5 6` | three forms: symbol `#_`, int 5, int 6 | long `6`. The corpus only has `#_1 2` (no space), which is two forms |
| `(f #inst "1985-04-12T23:20:50.52Z")` | one list of three children: `f`, `#inst`, the string. `parse_one` succeeds | a list whose print still shows `#inst` |
| `(f #_1 2)` | one list of three children | the list `(f 2)` |

The builder's observation holds. Inside a call the failure is an extra child,
not `TrailingContent`.

### What (H) needs — STOP-1

The reader cannot know user types. `#u/T 7` and `#myapp/Foo` are the same
unread tag. The type registry is not in the lexer.

1. A `Tagged { tag, form }` node, left unresolved. No such variant exists
   (`ast.rs:61`). `#inst`, `#uuid`, `#myapp`, `#u`, `#wat.core` would each
   become one node. Turning the tag into a date or a user value happens
   later, where the registry is. `ast.rs:188-193` says a new variant is
   already four `E0004`s in that file, and `PartialEq` (`ast.rs:194`) is a
   fifth that cannot take a wildcard. `WatAST::NilLit` is mentioned 207
   times in 107 `.rs` files (a mention count, not a match count). This
   choice does nothing for `#_`. Until the later pass, the tree still does
   not equal clj's `Date`.

2. Special-case only `#inst` and `#uuid`, and leave every other tag as a
   node. `WatAST` has no instant or uuid leaf. `wat-edn` stores
   `DateTime<Utc>` and `Uuid` (`value.rs:65-67`). Two mechanisms: built-ins
   become leaves or calls, user tags stay a node the registry resolves.
   `#_` is still separate.

3. Leave tags on `wat-edn`. The source reader stays as measured. Top-level
   `#inst` is two forms and `parse_one` is `TrailingContent`. Inside a list
   the tag is extra children, so a call's arity changes. `crates/wat-reader/tests/reader_totality.rs:4-6`
   already says the source reader is not the clj-parity target and that arc
   300 eventually feeds it from `wat-edn`. That feeding has not happened.
   This measurement is the gap.

`#_` is not a tag. `wat-edn` already discards (`#_1 2` agrees with clj's `2`).
The source reader has no discard node. Skipping the next form, the way
comments already stay off the tree (`parser.rs:264-268`), matches clj and
adds no variant. A discard node has to be ignored by every consumer or it
is typechecked as a name. Today `#_b` is a symbol in the vector.

Not chosen. The consequences above are the measurement.

## (K) colon in a name — 10, all reader-accepts / clj-refuses

`lex_keyword` documents an internal `:` as a body character, Rust's path
separator (`lexer.rs:777-779`). There is a lexer test `keyword_double_colon_path`.
The non-ascii refusal sits before keyword dispatch, so it does not apply here.

| input | reader |
|---|---|
| `::foo` | keyword `::foo` |
| `:a::b` | keyword `:a::b` |
| `:x:` | keyword `:x:` |
| `:wat::core::x` | keyword `:wat::core::x` |
| `a::b` | symbol `a::b` |
| `x::` | symbol `x::` |
| `x:` | symbol `x:` |
| `a:::b` | symbol `a:::b` |
| `a:/b` | symbol `a:/b` |
| `wat::core::x` | symbol `wat::core::x` |

`wat-edn` refuses all ten (doubled colon, or a colon that is not the one
leading marker). clj refuses all ten.

### Relation to the 251.8d converted floor

The converted floor is a check of source after `to-faithful-clojure.wat`
rewrites `:wat::core::x` text to a slash spelling. It is not a reader test.
The latest weighed number is **78 / 6014** (`WEIGH-STONE-255.14`, trajectory
3 747 → 416 → 283 → 161 → 112 → 107 → 78). Those 78 are check classes
(`MalformedForm` 41, `CheckErrors` 19, `LociDiedError` 16,
`DeclarationInExpressionPosition` 15, `ProgramBodyEvalFailed` 14). This
stone did not re-run that floor.

(K) is the reader still accepting the spelling that rewrite deletes. There
is no forward failure to feed that floor: clj does not accept these, and
the converted corpus is not supposed to contain them. Rejecting `::` in
the source reader would make the unconverted tree fail to lex. That is a
different cut from the 78, which are already-rewritten source. On the data
side the flip is already in the constructors: `Keyword::try_ns` and
`Symbol::try_ns` translate `::` to `.` (`value.rs:359`, `value.rs:272`).
The source lexer does not.

## (M) reader macros

Desugared, exempt. The dialect is wider than EDN on purpose
(`parser.rs:412-417`, `lexer.rs:477-500`).

| input | reader | clj |
|---|---|---|
| `'x` | `(:wat::core::quote x)` | a symbol whose name is the two characters `'` and `x` (codepoints 39, 120). Not a `(quote x)` list. `wat-edn` refuses |
| `` `x `` | `(:wat::core::quasiquote x)` | refuses |
| `~x` | `(:wat::core::unquote x)` | refuses |
| `~@x` | `(:wat::core::unquote-splicing x)` | refuses |

The ward's header says quote belongs to the source reader. Measured, clj.edn
did not expand `'`. It took the apostrophe as part of a symbol name. The
source reader's quote list is the deliberate desugar, and it does not equal
that symbol.

Claimed by the ward (`clj_oracle_parity.rs:50-68`), not desugared. Not exempt.

| input | reader | clj |
|---|---|---|
| `@x` | one symbol `@x`. No `@` token in the lexer | `Invalid leading character: @` |
| `^:m x` | two forms, symbol `^:m` and symbol `x`. `:` is not a symbol break, so the caret stays on the keyword | the value `x` (metadata consumed) |

## (O) — 19

clj accepts, reader refuses or stores a different value:

| input | reader | note |
|---|---|---|
| `+7` | symbol `+7` | clj long `7`. A number starts on a digit or on `-` (`lexer.rs:552`). `+` is not included. `wat-edn` routes `+` through `lex_signed` |
| `+1` | symbol `+1` | clj long `1` |
| `1.5M` | `invalid numeric literal "1.5M"` | clj `BigDecimal`. `N` is a bigint (`lexer.rs:1006-1017`). There is no `M` |
| `9223372036854775808` | float `9223372036854776000` (bits `43e0000000000000`) | past `i64`, `parse::<f64>` succeeds (`lexer.rs:1019-1021`) |
| `-9223372036854775809` | float, same rounding the other way | |
| `123456789012345678901234567890` | float `123456789012345680000000000000` | |
| `é`, `😀`, `λ` | `unexpected character` | deliberate. Non-ASCII token start is refused (`lexer.rs:524-539`). `reader_totality.rs` requires a clean `Err` here |
| `:a😀` | keyword whose bytes were widened Latin-1 (`:að\u{9f}\u{98}\u{80}`) | the refusal is before `:` dispatch, so `lex_keyword` still does `bytes[i] as char` (`lexer.rs:820`). `reader_totality.rs` allows this to parse. It parses, corrupted |
| `:λ` | keyword `:Î»` | same hole |
| `a<` | `AngleTypeHeadInName` | arc 109, standing. clj accepts the symbol |
| `"\u0041"` | `unknown escape sequence \u` | `lex_string` has `\" \\ \n \t \r \0` only (`lexer.rs:648-655`). Character literals do have `\uNNNN` (`lexer.rs:732`) |
| `\ ` | `backslash followed by whitespace` | clj character `\space`. `wat-edn` refuses too (the ward's spec sentence). Both readers, not a source-only gap |

clj refuses, reader accepts:

| input | reader |
|---|---|
| `/x` | symbol `/x` |
| `x/` | symbol `x/` |
| `{:a 1 :a 2}` | one map, both pairs kept (`parser.rs:607-614`). clj: `Duplicate key: :a` |
| `#{1 1}` | one set, both elements (`parser.rs:634`). `ast.rs:166` says duplicates collapse at eval. clj refuses at read, so eval never sees them |
| `#{1 1 2}` | three elements |

`Identifier::bare` splits on the first `/` and still builds the value when
one side is empty (`identifier.rs:185-188`). The comment there says neither
side is empty. `/x` and `x/` are the counterexample, measured.

## What the ward did not foresee

The ward (`clj_oracle_parity.rs:1-20`) assigns reader macros to `wat-reader`
and then never calls it. It compares verdicts only. That is why `##Inf`,
`+7`, the oversized integers, `:a😀`, and `[a #_b c]` are all `OK`/`OK` in
the golden and still wrong in the source tree. It is also why `.1` can be
a symbol for clj and a float for `wat-edn` without the ward noticing.

`@` and `^` are on the ward's "belongs to wat-reader" list. The source
reader does not implement them.

Unknown tags are an intentional `wat-edn` superset (the ward exempts
`#myapp/`). The source reader does not read them as tags. It reads two forms.
`#u/T` and `#wat.core/Span` are that same split. The golden already covers
the shape via `#myapp/Foo`.

## Ignored probe

`crates/wat-reader/tests/clj_oracle_source_parity.rs`, `#[ignore]`.

```
cargo test --offline -p wat-reader --test clj_oracle_source_parity -- --ignored
```

Ran once. rc 101. The panic says `source reader vs clj oracle (62)`. That 62
is not the 48. The four desugars are skipped. The three multi-slash rows are
skipped because `wat-edn` refuses them. The other 18 spellings that match
clj fail anyway, because `Symbol::try_new` rejects `a:b`, `/`, and `.1`.
62 = 44 non-exempt divergences + those 18 constructor limits.

No floor, no clippy, no census, no delta. Nothing in the readers or the
corpus was edited.
