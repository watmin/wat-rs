# SCORE — STONE 255.62: layer 3 — the lib still does not link

Measurement. Nothing lands but this score and `probes-255.62/`. Worktree
`/tmp/255.62-wall/wt` (detached `a68738e8a`), `../holon-rs` beside it. The
worktree is removed.

Both walls are in the tree. The library does not compile, so the release
floor and the census did not start. Against 78/6014 at 255.14: still earlier
than that floor. One error hides everything behind it.

The `::` wall was applied after the corpus conversion. A binary with that
wall up cannot read the unconverted corpus (255.60).

## What was built

Corpus, same codemod, two-phase, no-wall binary:

| phase | files | rc | time |
|---|---|---|---|
| `wat/**/*.wat` | 65 | 0 | 1439 s |
| rebuild `--bin wat` | | 0 | 51 s |
| every other tracked `*.wat` | 2231 | 0 | 1257 s |

Doc spans, `probes-255.61/convert-doc-fragments.wat`: 2006, rc 0. Same split
as 255.61 (1982 cleared, 17 strings still hold `::`, 7 unchanged).

Link cures, in `probes-255.62/`:

- `link-cure-doc-symbol.diff` — `type_token_shape_ok` in
  `crates/wat-doc/src/lib.rs`. A type token may be a keyword, a `(…)`/`[…]`
  form, or a symbol. The reader check still follows. Wired at the `@arg` and
  `@ret` sites the brief names (`:653`, `:705` after the helper) and at the
  two copies of the same predicate in the special-form parser (`:1501`,
  `:1544`). Leaving those two would have kept the lib red for the same reason.
- `link-cure-fqdn.diff` — `fqdn_of`'s plain case
  (`crates/wat-macros/src/edn_doc.rs:286`) now returns `:{ns}/{name}`, the
  dotted spelling the fence wrote. The enum branch above it
  (`compose_variant`, line 281) still mints `::`. It is not what fired.

Second wall, `wall-f1-type-position.diff`. `forbidden_core_type_symbol` in
`src/types.rs`, called from `parse_type_node`'s symbol arm and from
`parse_type_form`'s symbol head. A symbol `wat.core/<name>` is refused when
`<name>` is a builtin type: `is_builtin_primitive`, `BARE_PRIMITIVES`,
`BARE_CONTAINER_HEADS`, or `Value`. The message names the token and says to
write `wat.type/<name>`. A keyword `:wat::core::i64` is not this door — that
spelling is the internal key, and the `::` wall already keeps it out of
source. A call `(wat.core/defn …)` never enters `parse_type_node`.

Type positions that reach this wall, measured by who calls it:

| entry | file |
|---|---|
| `parse_type_node` | `src/types.rs` — keyword, symbol, list, `[… :-> …]` |
| `parse_type_form` | same file — `(Head :- [args])`, head and args |
| `parse_fn_type_bracket` | args and return, through `parse_type_node` |
| `parse_type_expr_from_source` | text, then `parse_type_node` |
| `src/check.rs:5270`, `:10355`, `:12938` | checker type slots |
| `src/argspec/parse.rs:197` | binder types |
| `src/types.rs:4753`, `:4774`, `:4842`, `:5521`, `:5573`, `:5652`, `:6157` | field types, variant payloads, bounds |

`parse_type_expr` on a keyword string is the other door. It is how Rust
spells the canonical key. The F1 wall does not sit on it.

## The compile

`scripts/floor.sh`. rc 101. Doctest gate. Nextest did not start. Not re-run.

```
error: #[wat_intrinsic] :wat::runtime::compose-variant: malformed `@example` directive: does not parse as a single, complete wat form (unbalanced parens/brackets, a missing quote, or trailing content after the form)
    --> src/reflect/verbs.rs:1878:1
error: could not compile `wat` (lib) due to 1 previous error
```

The form is the right-hand side of the example at `src/reflect/verbs.rs:1911`:

```
/// @example (wat.runtime/compose-variant (wat.keyword/from-string "wat::cache::Lru") :Hit) #=> wat::cache::Lru.Hit
```

`@example` parses both sides (`crates/wat-doc/src/lib.rs:734-735`). The
left side converted. The right side `wat::cache::Lru.Hit` is one of the
seven spans `fix-text` left unchanged: a symbol whose name contains `::`
and a dot, not a keyword the call-head rule rewrites. The `::` wall refuses
it. Rustc stopped at that one error. The 255.61 colon-prefix errors are
gone. The `char.rs` `:ret` error is gone.

## Classes

**(a)** The seven unchanged spans from 255.61, and this one is the blocker:
`src/reflect/verbs.rs:1911`, token `wat::cache::Lru.Hit`. The other six are
strings (`"wat::core::i64"`, `":probe::renamed"`, `"some::unknown::Rec"`,
`"some::unknown::Kind"`) which the reader does not lex as names. Seventeen
further examples still contain `::` inside a string; those strings parse.
`fix-text` does not edit string contents.

**(b)** Did not fire. No linked binary.

**(c)** Not reached past the one doc example.

**(d)** `ReservedPrefix` on `--check` of converted `wat/core.wat` is the
255.60 measurement, not re-run. The 255.14 classes were not reached.
`fqdn_of`'s enum branch still builds a `::` keyword; it did not fire in
this compile.

**(e)** The left side of that example calls `from-string` on the string
`"wat::cache::Lru"`. That string is data. Under the runtime wall it would
be `MalformedForm` at `eval_keyword_from_string`. It did not execute.

**(f)** `crates/wat-doc/src/lib.rs` test
`bare_symbol_without_colon_is_still_refused_by_the_colon_rule` still
expects a bare symbol to be refused. The link cure accepts it. The test
did not run. The lexer's `keyword_double_colon_path` still expects `::`
to lex. It did not run.

**(h) `wat.core/<type>` in a type position.** The checker never ran. These
counts are a scan of the converted `*.wat` tree, not floor failures. The
wall, as written, would refuse them. Call-shaped `(wat.core/Vector …)`
without a following `:-` is 4107 and is not this class.

| group | n | codemod rule that emitted `wat.core/` | what would emit `wat.type/` |
|---|---|---|---|
| parametric head `(wat.core/Vector :- …)` and the other container heads | 6905 (Vector 4154, Tuple 973, PersistentVector 878, HashMap 394, Option 224, HashSet 142, Result 88, PersistentMap 39, List 10, fn 3) | call-head keyword → symbol | the head of a `(Head :- […])` form is a type, the position signal 255.5 already named. Emit `wat.type/Head` |
| element of a `:- [` bracket | 5490 (i64 2297, String 1734, Tuple 824, Record 156, keyword 145, Value 60, Vector 46, nil 46, and the rest) | a keyword in that slot was rewritten as a reference symbol, not as a post-arrow type | the bracket after `:-` is a type-argument list. Same position signal |
| bare type after `:-` | 12, all `i64` | the post-arrow rule, which already emits `wat.type/` (15321 `wat.type/<builtin>` tokens in the same tree), missed these | they sit in syntax-quote or a binder the rule skips. `tests/resolve/probe_arc255_5_position_signal.wat:5` is one: `[n :- wat.core/i64] :- wat.core/i64` |

The two codemod positions — head of `(Head :- […])`, and the elements of
that bracket — are 12395 of the 12407. That is the cure set.

**(g)** Nothing else in the compile.

## Where it stopped

One `@example` expected-value, `wat::cache::Lru.Hit`, does not lex. Until
that span is a form the `::` wall accepts, the lib does not link, and
neither wall is observable as a test failure.
