# SCORE — STONE 255.64: layer 5 — the converted stdlib does not load

Measurement. Nothing lands but this score and `probes-255.64/`. Worktree
`/tmp/255.64-wall/wt` (detached `e50635809`), `../holon-rs` beside it. The
worktree is removed. The `::` wall was not applied. Phase 2, the doc
fragments, the floor, and the census did not run. Against 78/6014 at
255.14: no converted floor number.

The binary that embeds the converted stdlib does not start. Two startup
runs, different gates. The second was not a re-run of the first.

## What the codemod does

`probes-255.64/fix-f1.diff` is the text walk (`fix-text`, not only
`fix-seq`). It asks `:wat::keyword::language-provided-spelling`, which
reads `TypeEnv::classify` on `TypeEnv::with_builtins` — the builtin
registration, not a list inside the codemod.

A name is language-provided when `classify` is Builtin and the canonical
key is `:wat::core::…`, `:wat::time::…`, or `:wat::WatAST`. The surface is
`wat.type/{tail}`, except `:wat::WatAST` → `wat.type/AST`. Declared types
(`Option`, `Result`, every `defenum`) return empty and keep
`to-symbol` (`wat.core/Option`). Subsystem builtins (`HolonAST`, `Peer`,
`Stream`) return empty and keep their namespace (`wat.holon/HolonAST`).
`:wat::core::Tuple` is not in the registration (the existing
TABLE-STONE-Q test). The code wins: it stays `wat.core/Tuple`.

A three-child `(Head :- […])` is a type head only when that list is itself
in type position (after an arrow, nested in a type bracket, a lone doc
keyword, or the last child of `typealias`). The same shape in value
position is the empty constructor. `wat/Record.wat` and `wat/bracket.wat`
call `(wat.core/Vector :- [wat.type/AST])`. The ruling says that form is
never a call. The code calls it. The head stays `wat.core/Vector`.

`extend-type`'s type arguments are the same class as the `typealias` body
(a type with no arrow). That rule was added on the converted `fix.wat`
and did not run. See the stop.

## P1 and the Rust spellings

`src/macros/eval.rs` `validate_pure_total`: a three-child `(Head :- […])`
is not asked of the F5 gate. `validate_quasiquote_template` does not ask;
it sends unquoted code back here. No second walk was changed.
`probes-255.64/src.diff`.

`:wat::WatAST` stays the canonical key (`src/types.rs` registration, the
generated `:wat::WatAST` keyword, the group-3 test). `wat.type/AST`
reaches it through `type_denotation` (`:wat::type::AST` → `:wat::WatAST`).
The same door maps `wat.type/Instant` and `wat.type/Duration` to
`:wat::time::Instant` and `:wat::time::Duration`. `is_watast` and
`is_watast_vec` compare through that door, so `wat::type::Vector` and
`wat::core::Vector` are the same head.

F1, narrowed, is `forbidden_language_type_symbol` in `parse_type_node` and
`parse_type_form`. It refuses a symbol whose registration surface is
`wat.type/…` and whose spelling is not already that surface. `wat.core/Option`
is declared, so it is not refused. `wat.core/Vector` is.

Doc type tokens (`crates/wat-doc/src/lib.rs` `type_token_shape_ok`): a
keyword, a `(…)` / `[…]` form, or a namespaced symbol. A bare `Bytes`
stays refused. `fqdn_of`'s enum branch
(`crates/wat-macros/src/edn_doc.rs`) returns `:wat.runtime/Purity.Pure`
via `compose_variant`. The plain case returns `:{ns}/{name}`. The
Type/method branch still contains `::`. The axis reader accepts the slash
keyword by the same dot-to-`::` step it already used for a symbol.

## The stdlib

65 files, no-wall binary, the narrowed rule. Convert log `RC:0`,
01:55:34–02:18:49. Rebuild `Finished` in 23.49s.

`./target/release/wat --check nil`. CHECK rc 1. Stdout empty. Not re-run.

```
#wat.type/MalformedTypeExpr {:message "malformed type expression \"wat.core/i64\": type position refuses wat.core/i64; write wat.type/i64" :location #wat.core/Span {:file "wat/class.wat" :line 16 :col 23 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 16 :col 35}}} :causes [] :raw "wat.core/i64" :reason "type position refuses wat.core/i64; write wat.type/i64"}
```

The site is `(wat.core/extend-type wat.core/i64 wat.core/Orderable)`. The
argument is a type. It was not after an arrow, so the codemod left
`wat.core/i64`. The same file has `wat.time/Instant` and `wat.time/Duration`
in that slot, and `(wat.core/Vector :- [T])` as an `extend-type` child.

The wall was paused (`F1_WALL`, restored to true in the probe) so a
binary could start and run the `extend-type` pass. It did not start.
Codemod rc 3. `class.wat` was not rewritten. Not a re-run of the rc 1
check: the gate was off, and the failure is the checker.

```
#wat.check/CheckErrors {:message "534 type-check errors" ...}
```

533 `TypeMismatch`, 1 `ReturnTypeMismatch`. The first:

```
:wat::core::conj: parameter #1 expects (Vector :- [T]), (HashSet :- [T]), (PersistentVector :- [T]), or (List :- [T]); got (:wat::core::Vector :- [:wat::WatAST])
```

`wat/bracket.wat:356`, the empty constructor
`(wat.core/Vector :- [wat.type/AST])` inside `conj`. The next are
`select`/`send`/`nth`/`foldl`/`mapv` in the same file (`:603`, `:613`,
`:614`, `:616`, `:623`, `:744`, `:747`, `:777`) and `foldl` at
`wat/cache.wat:203`. The got types are vectors. They do not match the
generic vector the callee declares. That is not a spelling of a name.

## Classes

**(a)** Not measured past the stdlib. The `::` wall was not applied.

**(b)** Did not fire. No runtime mint was reached. The enum branch of
`fqdn_of` no longer writes `::`. The Type/method branch still does.

**(c)** `is_watast` / `is_watast_vec` go through `type_denotation`. The
conj failure is not a name spelling.

**(d)** The 534 checker errors. Floor did not start. `ReservedPrefix`
was not reached.

**(e)** Not separated from (d). The empty `(Vector :- [T])` is a value
in the stdlib and a type in the ruling.

**(f)** `fqdn_keyword_axis_value_round_trips` now expects
`:wat.runtime/Purity.Pure`. It did not run. Lexer `::` tests were not
run. `bare_symbol_without_colon_is_still_refused_by_the_colon_rule`
was not run; the shape check still refuses a bare symbol.

**(h)** With the wall on, the first refusal is `wat/class.wat:16`
`wat.core/i64` in `extend-type`. The rest of that file's leaf arguments
are the same shape. Not counted past the first error. `Tuple` stays
`wat.core/Tuple` because it is not registered.

**(g)** Nothing else in the two startups.

## Where it stopped

The converted stdlib does not load. With the F1 wall on, `extend-type`'s
`wat.core/i64` is refused. With the wall off, the checker reports 534
errors, starting at the empty `Vector` constructor `conj` calls. Phase 2
cannot run until a binary that embeds this stdlib starts.
