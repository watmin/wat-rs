# SCORE — STONE 255.63: layer 4 — STOP-1, the converted stdlib does not start

Measurement. Nothing lands but this score and `probes-255.63/`. Worktree
`/tmp/255.63-wall/wt` (detached `7df464592`), `../holon-rs` beside it. The
worktree is removed. Neither wall was applied. The floor did not start.
Against 78/6014 at 255.14: no converted floor number. The census did not run.

STOP-1. One non-spelling failure prevents the binary that embeds the
converted stdlib from starting, so phase 2, the doc conversion, the link
cures, both walls, and the floor are behind it.

## What ran

Codemod first, `probes-255.63/fix-f1.diff`, on `wat/fix.wat`'s text walk
(`fix-text-leaf-edits`, `fix-text-seq-edits`). `fix-seq` / `fix-source` was
left as it was, so the in-memory probes still emit the old walk. The
spelling function is the existing `to-type-form`. The change is when it is
asked.

A keyword is a type head only when the child sequence is exactly three
nodes: head, `:-`, vector. The bracket's keywords go through `to-type-form`.
A list with anything after the bracket keeps `to-symbol`, so a `defenum`
name and a value constructor stay `wat.core/`. `:-` is not an arrow.
`prev-arrow?` is still only `<-` and `->`, so a binder `[x <- :wat::core::i64]`
was already becoming `[x :- wat.type/i64]` before this rule.

The first cut of the rule treated any keyword followed by `:-` and a vector
as a type head. The rebuild against that stdlib failed and was not re-run:

```
error: no `(:wat::core::defenum :wat::core::Option …)` in `…/wat/core.wat`
    --> src/types.rs:2198
error: no `(:wat::core::defenum :wat::core::Result …)` in `…/wat/core.wat`
    --> src/types.rs:2199
error: could not compile `wat` (lib) due to 2 previous errors
```

`canonical_identity("wat.type/Option")` is `:wat::type::Option`.
`wat_enum_register_from!` wants `:wat::core::Option`. Stdlib was restored
and the three-child check added. That is the diff on disk.

Stdlib only, no-wall binary, the narrowed rule. 65 `wat/**/*.wat` files.
The convert log ends `RC:0` at 17:15:58, about 23 minutes after the
converter binary finished (16:52:41). Rebuild of `--bin wat` against that
stdlib: `Finished release` in 23.44s.

That binary did not start. First death, cured as a spelling of the expected
head (`probes-255.63/rest-param-head.diff`). `is_watast_vec` in
`src/macros/parse.rs` compared the parsed head to `wat::core::Vector`.
Parse stores `wat.type/Vector` as `wat::type::Vector`. Both heads are
accepted. The error string still names `(:wat::core::Vector :- [:wat::WatAST])`.
Rebuild after the cure: `Finished release` in 23.09s.

The cured error, not re-run:

```
malformed defmacro: macro rest-param `call-args` is declared `Parametric { head: "wat::type::Vector", args: [Path(":wat::WatAST")] }`, but a rest param binds a sequence of forms — its type must be `(:wat::core::Vector :- [:wat::WatAST])`
```

Site `wat/core.wat:526`, rest param at line 531.

## The stop

`./target/release/wat --check /tmp/255.63-wall/nil.wat` (`nil`). rc 1.
Stdout empty. Not re-run.

```
#wat.macro/MalformedDefmacro {:message "malformed defmacro: program-body macro purity check failed at definition: keyword head `:wat::type::Vector` refused at macro expand time — not on the pure-combinator allow-list (default-deny F5 gate, arc 249 stone 249.2b-i); only pure-total heads are permitted" :location #wat.core/Span {:file "wat/core.wat" :line 525 :col 1 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 649 :col 122}}} :causes [] :reason "program-body macro purity check failed at definition: keyword head `:wat::type::Vector` refused at macro expand time — not on the pure-combinator allow-list (default-deny F5 gate, arc 249 stone 249.2b-i); only pure-total heads are permitted"}
```

The form is already the F1 spelling:

```
(wat.core/defmacro wat.core/kwargs-lower
  [impl-kw    :- wat/WatAST
   …
   & call-args :- (wat.type/Vector :- [wat/WatAST])]
  :- wat/WatAST
```

`src/macros/eval.rs:458` `is_expand_time_legal` looks the head up in the
intrinsic registry and default-denies a miss. `:wat::core::Vector` is a
registered pure head. `:wat::type::Vector` is not. The walk treats
`(wat.type/Vector :- [wat/WatAST])` as an expand-time call. Accepting it
means teaching that walk that a `:-` list is a type, or putting a
type-namespace twin on the default-deny allow-list. That is a design
decision. Not done.

Startup dies while loading stdlib, before the user file. `ReservedPrefix`
on an unprivileged `--check` of converted `wat/*.wat` was not reached.

## What did not run

- Phase 2, the other tracked `*.wat` files.
- Doc fragments. 2006 jobs were extracted. 958 bare type keywords were
  wrapped as `x <- TOKEN` so the existing post-arrow rule would emit
  `wat.type/`. The converter was not run on them. A lone keyword is not a
  three-child type head, and `:-` does not set `prev-arrow?`.
- `probes-255.60/the-wall.diff`, `probes-255.62/link-cure-fqdn.diff`,
  `wall-f1-type-position.diff`, `link-cure-doc-symbol.diff`.
- The `verbs.rs:1911` respelling, the string leftovers, the lexer `::` tests.
- `scripts/floor.sh`. No `.floor/` directory. The census.

`fqdn_of`'s enum branch (`compose_variant`, `crates/wat-macros/src/edn_doc.rs:281`)
still receives a `::` namespace. Not changed. It did not fire: the process
never got past stdlib load.

## Classes

**(a)** Stdlib code holds no `::` (0 hits outside strings and comments).
Strings still hold 552. Comment tails hold the rest. The other ~2230 files
were not converted.

**(b)** Did not fire. No runtime mint was reached. The enum branch of
`fqdn_of` is unchanged.

**(c)** The rest-param compare was this class and is cured (both heads).
The F5 refusal is not: the source spelling is already `wat.type/Vector`.

**(d)** Not reached. Floor did not start. `MalformedForm`, `CheckErrors`,
`LociDiedError` from a user file, `DeclarationInExpressionPosition`, and
`ProgramBodyEvalFailed` were not observed. The startup error is
`MalformedDefmacro`, raised while stdlib loads.

**(e)** Not reached.

**(f)** Lexer tests that pin `::`, and
`bare_symbol_without_colon_is_still_refused_by_the_colon_rule`, were not
run. The wide 255.62 doc-symbol cure was not applied.

**(h)** Stdlib scan, not a floor, not the full corpus. The 255.62 counts
(6905 / 5490 / 12) are not remeasured. In the 65 converted files:

| group | n |
|---|---|
| `(wat.type/… :-` heads | 1238 |
| `wat.type/` tokens | 2685 |
| `(wat.core/… :-` with arguments after the bracket | 85 (Vector 49, extend-type 24, HashSet 5, PersistentVector 3, HashMap 3, fn 1) |
| three-child `(wat.core/… :- […])` left behind | 0 |
| `:- wat.core/` | 0 |

`defenum` names stay `wat.core/Option`, `wat.core/Result`,
`wat.core/ReadOutcome`, `wat.core/ReadWithCommentsOutcome`. The two
`wat/core.wat` `(wat.core/Vector :- [wat.type/i64] …)` forms (lines 1308
and 1325) are syntax-quoted constructors with an extra argument. The
bracket element is `wat.type/i64`. The head staying `wat.core/Vector` is
the three-child rule.

**(g)** The F5 gate, above. One site blocks every later file.

## Where it stopped

The binary that embeds the converted stdlib compiles and then refuses
`:wat::type::Vector` inside `wat.core/kwargs-lower` at expand time. Until
a `:-` type form is not a call to that gate, phase 2 cannot be checked
and the floor cannot start.
