# SCORE — STONE 255.58: goldens compared as text

Measurement only. Nothing else landed. Drawn on `255ff0389` (the draw of
`afa0b09e1`).

Method, named so a text search stays a text search. A scan of `tests/**/*.rs`
and `src/**/*.rs` found each `include_str!` and the `assert_*` whose
parentheses still enclosed it. Each hit below was then read. `parse_owned`
was run on the golden files named in section 4 (`wat_edn` 0.1, offline).
The census numbers are a read of `.census/2026-09-27T08-54-40Z.txt`. This
stone did not re-run the census. `git ls-files 'tests/**/*.edn'` is 406,
and `git ls-files '*.edn'` is 418.

The data door that already exists is `assert_edn_eq!` (`src/lib.rs:312`)
and `assert_edn_matches_file!` (`src/lib.rs:380`). Text search: 29
`assert_edn_eq!` and 384 `assert_edn_matches_file!` under `tests/`.

## 1. Comparisons that are not that door

48 `assert_eq!` calls compare a produced string to `include_str!` of a
stored file. One nearby `include_str!` is not a golden: the detector
fixture at `tests/lint/every_walking_gate_declares_non_vacuity.rs:401`.

Produced string versus a `.wat` file, 37 calls. The producer is
`eval_string` / `call_beside_value` returning `Value::String`, except
`stdlib` which is below.

| call | golden | what the string is |
|---|---|---|
| `probe_arc251_fix_source_local_rules.rs:35` | `__contract-01-arrow-in-binder.wat` | one form |
| `:43` | `__contract-02-post-arrow-scalar.wat` | one form |
| `:74` | `__contract-04-head-inverts.wat` | one form |
| `:82` | `__contract-05-full-fn-literal.wat` | one form |
| `:91` | `__contract-06a-less-than.wat` | one form |
| `:95` | `__contract-06b-less-equal.wat` | one form |
| `:103` | `__contract-07-greater-than.wat` | one form |
| `probe_arc251_fix_source_head_rule.rs:33` | `__contract-01-bare-call-head-inverted.wat` | one form |
| `:41` | `__contract-02-strip-and-head-compose.wat` | one form |
| `:50` | `__contract-03-nested-heads.wat` | one form |
| `:59` | `__contract-04-data-keyword-head.wat` | one form |
| `probe_arc258_stone3_fix_source.rs:95` | `__contract-05-nested-do-if.wat` | one form |
| `:105` | `__contract-06-preserves-option-expect.wat` | one form |
| `:115` | `__contract-07-end-to-end-clean.wat` | one form |
| `probe_arc251_decl_migrator.rs:58` | `__c01-typealias-type-slot.wat` | one form |
| `:80` | `__c02-defn-drop-type-params.wat` | one form |
| `:116` | `__c04-user-type-preserved.wat` | one form |
| `:126` | `__c05-newtype-type-slot.wat` | one form |
| `:136` | `__c06-typeunion-core-members.wat` | one form |
| `:146` | `__c07-typeunion-user-members.wat` | one form |
| `:156` | `__c08-defenum-variant-tags.wat` | one form |
| `probe_arc251_keyword_to_type_form.rs:35` | `__contract-01a-scalar-i64.wat` | one symbol |
| `:39` | `__contract-01b-scalar-user.wat` | one symbol |
| `:104` | `__contract-06-tuple.wat` | one form |
| `probe_arc251_type_namespace_fix.rs:35` | `__c01a-core-fqdn-i64.wat` | one symbol |
| `:39` | `__c01b-core-fqdn-string.wat` | one symbol |
| `:66` | `__c03a-legacy-i64.wat` | one symbol |
| `:70` | `__c03b-legacy-string.wat` | one symbol |
| `:74` | `__c03c-legacy-bool.wat` | one symbol |
| `:82` | `__c04-user-type-namespace.wat` | one symbol |
| `:97` | `__c06-user-type-two-segment.wat` | one symbol |
| `:105` | `__c07a-type-var-t.wat` | one symbol |
| `:109` | `__c07b-type-var-k.wat` | one symbol |
| `probe_arc251_fix_text_comment_faithful.rs:29` | `__probe-comment-faithful.wat` | source, comment must survive |
| `:40` | `__once-many-comments-idempotent.wat` | source, comments and a blank line |
| `probe_arc269_rename_keyword_prefix.rs:28` | `__swap-prefix-comment-faithful.wat` | source, comment must survive |
| `probe_arc283_1_rename_typearg.rs:25` | `__renamed.wat` | one form, `::` keywords |

`:44` of the comment-faithful file is `assert_eq!(twice, once)`: two produced
strings, no stored file.

Produced string versus an `.edn` file, with `.trim_end()`, 8 calls. Each
file's own comment says the string compare is a bridge left in place after
the `<T,Acc>` head was retired.

| call | golden |
|---|---|
| `wat_arc144_uniform_reflection.rs:149` | `__special_form.edn` |
| `:168` | `__type_defstruct.edn` |
| `:190` | `__primitive_empty.edn` |
| `wat_arc201_structured_signature_types.rs:157` | `__foldl.edn` |
| `wat_arc143_lookup.rs:134` | `__foldl_signature.edn` |
| `wat_arc143_manipulation.rs:68` | `__reduce_head.edn` |
| `wat_arc144_lookup_form.rs:81` | `__struct_head.edn` |
| `wat_arc144_hardcoded_primitives.rs:139` | `__length_primitive.edn` |

Byte compares, 3 calls. Section 5.

| call | golden |
|---|---|
| `tests/cli/pprintln_doc_row.rs:34` | `pprintln_doc_row__step_payload.edn` |
| `:52` | `pprintln_doc_row__note.edn` |
| `tests/cli/stdlib_door_reads_a_set_as_one_world.rs:122` | `stdlib_door_one_world__user.post` |

## 2. `.wat` files the census type-checks that are output, not programs

All 37 files in the table are in `.census/2026-09-27T08-54-40Z.txt`.
21 are rc 0. 16 are rc 1:

- `probe_arc251_keyword_to_type_form__contract-01b-scalar-user.wat`
- `probe_arc251_decl_migrator__c07-typeunion-user-members.wat`
- `probe_arc251_decl_migrator__c08-defenum-variant-tags.wat`
- `probe_arc258_stone3_fix_source__contract-05-nested-do-if.wat`
- `probe_arc258_stone3_fix_source__contract-06-preserves-option-expect.wat`
- `probe_arc258_stone3_fix_source__contract-07-end-to-end-clean.wat`
- `probe_arc251_fix_source_local_rules__contract-05-full-fn-literal.wat`
- `probe_arc251_fix_source_local_rules__contract-06a-less-than.wat`
- `probe_arc251_fix_source_local_rules__contract-06b-less-equal.wat`
- `probe_arc251_fix_source_local_rules__contract-07-greater-than.wat`
- `probe_arc251_type_namespace_fix__c04-user-type-namespace.wat`
- `probe_arc251_type_namespace_fix__c06-user-type-two-segment.wat`
- `probe_arc251_fix_source_head_rule__contract-03-nested-heads.wat`
- `probe_arc251_fix_source_head_rule__contract-04-data-keyword-head.wat`
- `probe_arc269_rename_keyword_prefix__swap-prefix-comment-faithful.wat`
- `probe_arc283_1_rename_typearg__renamed.wat`

Six more `keyword_to_type_form__contract-*.wat` files are in that census,
all rc 0, and no `include_str!` names them: `02-parametric`,
`03-nested-parametric`, `04-type-var-bare`, `05-multi-arg`,
`07-empty-tuple`, `08-nested-tuple`. `07` is a two-line program (`nil`).
The other five are one-line forms the test no longer reads.

## 3. What the data door needs

`assert_edn_eq!` compares `wat_edn::parse_owned` values. It does not read
wat. `WatAST`'s `PartialEq` (`crates/wat-reader/src/ast.rs:194`) compares
structure and skips spans. Comments are not in the tree
(`crates/wat-reader/src/parser.rs:264`: they come back beside it from
`parse_all_with_comments`). No macro parses two strings and asserts that
equality.

For a comment-free form or symbol, `assert_edn_eq!` can take the two
strings as they are. Measured: `parse_owned` returns `Ok` for every
comment-free `.wat` golden in the table except
`probe_arc283_1_rename_typearg__renamed.wat`.

That file fails: `invalid keyword: wat::core::defn: ':' cannot be doubled`.
Same failure on `:wat::core::i64` alone. A `::` keyword is legal wat and
illegal EDN. The missing door for that file is: parse both sides to
`WatAST` and compare with the existing `PartialEq`. That door is not wrapped.

`parse_owned` of a string that starts with `;;` returns the form and
drops the comment (measured on `;; c\n(wat.core/if true 1 2)`, and the two
fix-text goldens themselves return `Ok`). `WatAST` equality drops the
comment the same way. Nothing compares comment text together with the form.
Byte equality is what those tests have.

`(:wat::edn::read …)` is a different door. A symbol is
`EDN Symbol — wat has no symbol value type` (`src/edn/render.rs:2302`).
It cannot host these goldens.

## 4. Would the `.wat` goldens read as `.edn`?

`parse_owned` does not look at the extension. The 33 comment-free goldens
other than `__renamed.wat` returned `Ok`, so the same bytes would be a
legal `.edn` golden and `assert_edn_eq!` would compare them. Renaming
would also take them out of the `*.wat` census.

These would not, for the reason in section 3:

- `probe_arc283_1_rename_typearg__renamed.wat` — `::` in keywords.
- `probe_arc269_rename_keyword_prefix__swap-prefix-comment-faithful.wat`
  — `::` in keywords, and a comment the test requires byte-identical.
- The two `fix_text_comment_faithful` goldens parse (`Ok`) and the comment
  is discarded. As `.edn` they would stop being the test that was written.

## 5. Text is the thing tested

- `pprintln_doc_row.rs:30` calls the result a byte golden. Both `.edn`
  files parse (`step_payload` measured `Ok`). The assert is the stdout
  bytes, including the prose layout. `printed_row_edn_read_round_trips`
  is the separate parse check.
- `probe_arc251_fix_text_comment_faithful.rs:25` and `:36`: the comment,
  and on the second file the blank line, must survive byte-identical.
- `probe_arc269_rename_keyword_prefix.rs:18`: the comment must survive
  byte-identical, and the prefix swap is in the same bytes.
- `stdlib_door_reads_a_set_as_one_world.rs:116`: the converted file is
  pinned whole, comments included, against `stdlib_door_one_world__user.post`.

## What would move, by door

`assert_edn_eq!`, the strings as they are (33 `.wat` outputs plus the 8
`.edn` bridges that already parse):

- the seven `fix_source_local_rules` contracts
- the four `fix_source_head_rule` contracts
- the three `stone3_fix_source` contracts
- the seven `decl_migrator` `c0*` outputs
- the three `keyword_to_type_form` outputs still read (`01a`, `01b`, `06`)
- the nine `type_namespace_fix` outputs
- the eight `.edn` files in the trim_end table

`WatAST` equality, the door that is not wrapped:

- `probe_arc283_1_rename_typearg__renamed.wat`

Stay byte equality:

- the two fix-text comment goldens
- `probe_arc269_rename_keyword_prefix__swap-prefix-comment-faithful.wat`
- the two `pprintln_doc_row` `.edn` files
- `stdlib_door_one_world__user.post`
