# SCORE — STONE 255.60: the wall names the heretics

Measurement. The wall does not land. Worktree was `/tmp/255.60-wall/wt`
(detached `ecb183d23`), with `../holon-rs` symlinked at
`/tmp/255.60-wall/holon-rs`. The worktree is removed. The wall diff is
`probes-255.60/the-wall.diff` (4 files, +67/−1).

The converted floor did not run. The library does not compile with the wall
up, so `cargo nextest` never started and there is no wall binary to census
with. Against 78/6014 at 255.14: this cut stops earlier than that floor.

## What the wall is

Lexer (`crates/wat-reader/src/lexer.rs`). After `lex_keyword` and
`lex_symbol` accept a token, a `::` in it is `LexErrorKind::DoubleColonInName`.
`Display` of `LexError` already prints the byte position. Measured, through
that lexer:

```
ERR ":wat::core::i64" lex error at byte 4: double colon in name ":wat::core::i64"
ERR "wat::core::x"    lex error at byte 3: double colon in name "wat::core::x"
ERR "(:wat::core::foldl x)" lex error at byte 5: double colon in name ":wat::core::foldl"
OK  ":Acc"
OK  "wat.core/i64"
```

`:x:` and `x:` and `a:/b` from the 255.59 (K) list do not contain `::`. The
brief's wall is the doubled colon. Those three still lex. Say so: the code
of the brief ("refuses `::`") wins over reading (K) as every colon.

Runtime doors that build a name from a string, and panic or
`MalformedForm` when the string contains `::`:

| door | file |
|---|---|
| `:wat::keyword::from-string` | `src/runtime.rs` `eval_keyword_from_string` |
| `:wat::core::keyword-node` | `src/edn/render.rs` `eval_keyword_node` |
| `:wat::core::symbol-node` | `src/edn/render.rs` `eval_symbol_node` |
| `:wat::core::fresh-symbol` | `src/edn/render.rs` `eval_fresh_symbol` |
| `Symbol::new` / `try_new` / `ns` / `try_ns` | `crates/wat-edn/src/value.rs` |
| `Keyword::new` / `try_new` / `ns` / `try_ns` | same |

`try_ns` used to translate `::` into `.` and accept it. The panic is before
that translation. `from_parts_unchecked` is not walled: the EDN parser
already rejects `::` and then calls it.

Left open, on purpose, and listed. `WatAST::keyword` and `Identifier::bare`
still accept `::`. They are how the substrate spells the canonical key.
`ns_to_wat_path` (`src/edn/render.rs:3651`) is `format!(":{}::{}", ns.replace('.', "::"), name)`.
`canonical_identity` (`:3662`) keeps a `::` string and turns `wat.core/Option`
into `:wat::core::Option`. Walling `WatAST::keyword` would panic inside that
door before any file is checked. The quote desugar also mints
`:wat::core::quote` through it. Other mint sites that still format a `::`
string and pass it to `WatAST::Keyword` / `Identifier::bare`:

- `src/edn/render.rs` colon-mode arms (`format!(":wat::core::{tail}")` and siblings around 1619–1669)
- `src/types.rs:374` `format!(":{}", ns.replace('.', "::"))`
- `src/types.rs:3660`, `:4177`, `:4202`, `:4204`, `:4456`, `:4463` (surface op / reply names)
- `src/holon/ast.rs:665`, `src/closure_extract.rs:2475` and `:2557`

These did not run. The library did not link.

## Convert

Codemod `wat-scripts/fixes/to-faithful-clojure.wat`, not the whole
`convert.sh` chain. That chain is every migration since `de827fb4c`. This
tree is already past them. The two-phase recipe was followed: stdlib, rebuild,
then the rest, with `./target/release/wat`.

| phase | files | rc | time |
|---|---|---|---|
| `wat/**/*.wat` | 65 | 0 | 1416 s |
| rebuild `--bin wat` (no wall yet; stdlib is `include_str!`) | | 0 | 50 s |
| every other tracked `*.wat` | 2230 | 0 | 1270 s |

The rebuilt binary did load. It ran the second phase. One `--check` of the
converted `wat/core.wat` with that binary, before the wall was compiled:

```
#wat.runtime/ReservedPrefix {:message "cannot define :wat::core::+ — reserved prefix (:wat::, :rust::, :$bound::); user defines must use their own prefix" :location #wat.core/Span {:file "wat/core.wat" :line 58 :col 1 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 168 :col 42}}} :causes [] :prefix ":wat::core::+"}
```

The source at that line is `(wat.core/defclause wat.core/+`. No `::` in the
token. The gate (`src/resolve/registration.rs:22-26`) allows a reserved
prefix from stdlib privilege and refuses it from user privilege. `--check`
of the file is the user door, so the canonical key `:wat::core::+` is
refused. Embedded load of the same stdlib is the other door: the codemod
process started. Class (d).

After both phases, a scan that drops strings and `;` comments finds **no
`::` in code** across 2295 tracked `.wat` files. The four hits in
`tests/resolve/probe_arc269_rename_keyword_prefix.wat` are inside one string
(the rename fixture, lines 3–8). That string is the point of the test.

## The floor

`scripts/floor.sh` in the worktree. rc 101. Doctest gate, exit 101. Nextest
did not start. The log's own line: `error: could not compile wat (lib) due
to 582 previous errors`. Not re-run.

First block, verbatim:

```
error: #[wat_intrinsic] :wat::core::map: malformed `@ret` directive: type token is not a spelling wat's reader accepts (e.g. `Option<T>` and the retired `fn(…)->…` form are inexpressible; use `:- [...]`)
   --> src/collection/transform.rs:409:1
    |
409 | / /// `(:wat::core::map f xs)` → `Stream<U>`. Lazily calls `f` on each element as the result is
410 | | /// pulled; `xs` may be any seqable (`Vector<T>` | `List<T>` | `PersistentVector<T>` |
411 | | /// `Stream<T>`). `f` must be a callable Value (fn or define-registered).
412 | | ///
...   |
524 | |     Ok(Value::wat__stream__Stream(lazy_map_stream(func, source)))
525 | | }
    | |_^
```

The message is the doc macro's generic text. The predicate is
`type_token_is_expressible` (`crates/wat-doc/src/lib.rs:455`), which is
`parse_one_with_file(...).is_ok()`. The token it rejected on that intrinsic
is the `@ret` at `transform.rs:480`, `(:wat::core::Vector :- [U])`. The
lexer error underneath, measured on the same spelling, is `double colon in
name`. `@example` goes through `parse_example_form` (`lib.rs:468`), which
throws away the lex error and says the form does not parse.

| bucket | n |
|---|---|
| `@arg` type token | 481 |
| `@ret` type token | 87 |
| `@example` / `@example-norun` does not parse | 13 |
| `:ret` (the char intrinsic, one colon in the directive name) | 1 |
| total | 582 |
| files | 102 |

Largest files: `src/intrinsic/holon/atom.rs` 59, `src/intrinsic/special/rete_alias.rs` 52, `src/intrinsic/time.rs` 41, `src/runtime.rs` 28, `src/intrinsic/string.rs` 20. The full list is the compile log. Rustc stopped at the lib. Crates that depend on `wat` did not compile, so this 582 is the first layer, not a proof there are no further errors.

## Classes

**(a) `::` the codemod left in a `.wat` token.** 0. The codemod's scope is forms, not comments and not the inside of a string. Comments still contain `::` (the stdlib header comments). The reader never sees them.

**(b) A name minted at run time with `::`.** The doors in the table above are walled and were not executed. The format sites in `render.rs` and `types.rs` still mint `::` and were not executed. No runtime stack to quote.

**(c) `src/` looking a name up by a `::` spelling.** Counted, not rewritten. Exact keyword literals (the whole string matches `:ident(::ident)+`): **5745 in 209 files**. Heaviest: `src/check.rs` 1384, `src/runtime.rs` 386, `src/intrinsic/mod.rs` 372, `src/remedy/retirement.rs` 365, `src/types.rs` 308. They did not fail a match in a run. Inferred from `canonical_identity`: a converted `wat.core/i64` is stored as `:wat::core::i64`, so those literals still name the same key. The ones that did scream are the doc directives, because `wat-doc` parses them with the source reader. That is the 582.

**(d) Substrate breaks on the new spelling for another reason.** The `ReservedPrefix` on `wat/core.wat:58`, quoted above. `wat.core/+` canonicalizes to `:wat::core::+`, and `--check` is unprivileged. This is the same gate the 255.14 residue sits behind, but it fired on the stdlib file itself when checked as a user file. The 78 (`MalformedForm`, `CheckErrors`, `LociDiedError`, `DeclarationInExpressionPosition`, `ProgramBodyEvalFailed`) were not re-measured. The test suite did not start.

**(e) Other.** The wall's own lexer tests still expect `:wat::holon::Atom` to be one keyword (`lexer.rs` `keyword_double_colon_path` and its neighbours). They did not compile into a run, because the lib failed first.

## Smallest cure set

Two functions account for all 582:

1. `type_token_is_expressible` (`crates/wat-doc/src/lib.rs:455`) — 568 (`@arg` + `@ret`). Every one is a type token the walled reader refuses because it contains `::`.
2. `parse_example_form` (`crates/wat-doc/src/lib.rs:468`) — 13. The examples are still `(:wat::core::…)` source, which the corpus codemod does not edit, because they live in Rust doc comments.

Curing those two inputs is what lets the lib link. Until it links, the converted floor and the census cannot start. That is the layer the list stopped on.

No floor re-run. No census. Nothing on `main` but this score and the diff.
