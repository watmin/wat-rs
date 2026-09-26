# SCORE — STONE 255.48: a featureless surface's members are the declared ones

Struck against draw `7f8db2978` (brief drawn at `9d73f9b0b`). Ruling B1.
A surface with no members is satisfied only by a declared `extend-type` edge.
A surface that names members keeps width subtyping.

## The arm

`assignable`'s aggregate branch (`src/check.rs` 17674–17681) returns false when
`surf_clone.members` is empty. A declared edge is still accepted earlier, by
the path-to-path `is_subtype` arm. Width (`structural && nature_ok`) stays for
a surface that names members.

The foreign-type branch just below (`src/check.rs` 17706–17712) had the same
vacuous success: `struct_satisfies_surface` on `&[]` fields is true when there
are no members to miss, and `nature.is_none()` would then admit it. That
success now also requires `!surf_clone.members.is_empty()`. Known featureless
surfaces have a nature, so this branch was not the 255.47 corpus path. It is
the same rule.

`struct_satisfies_surface` (`src/types/surface.rs` 62, `members.iter().all`)
is still vacuously true on an empty list. Both callers refuse that list before
trusting it. Those are the only two callers.

The parametric arm (`src/check.rs` 17359–17417) was already edge-only. An
empty member list there does not admit a concrete type. B1's new return does
not change `Spawned`. The dialed-peer arm (`src/check.rs` 17317–17333) is a
different rule: `(Peer :- [Op Reply])` satisfies a `:nature Peer` surface when
the two arguments match that surface's synthesized enums. It does not consult
the member list. Left as it is.

## The declarations

One bodiless edge beside each definition 255.47 found admitted only by the
vacuous arm:

| record | surface | where |
|---|---|---|
| `:wat::query::Fault` | `:wat::query::Reason` | `wat/query.wat` 84 |
| `:probe::SqliteReason`, `:probe::RedisReason` | `:probe::Reason` | `probe-reason-downcast.wat`, `probe-defclause-open-arg.wat`, `probe-defclause-real-shape.wat`, `probe-defclause-discriminate.wat`, `tests/rete/probe_arc278_open_surface_dispatch.wat` |
| `:probe::MongoReason` | `:probe::Reason` | the discriminate probe and the rete dispatch probe |
| `:probe::Note` | `:wat::query::Reason` | `tests/services/probe_arc278_dead_child_speaks.wat` 22 |
| `:env::Rec` | `:env::Portable` | `tests/types/probe_arc293_holder_root_symbol.wat` 10 |

`probe_arc278_open_surface_dispatch_ambiguous.wat.bad` defines `:probe::A` and
`:probe::B`. It was outside the 68 because census lists `*.wat`, and a
`.wat.bad` is not in that list. Both records are passed to the same open
`Reason` the 68 covered, so each got the same edge. The fixture still fails
as `AmbiguousClauseReturnAtCallSite`. That assertion was not changed, and the
test passed.

Other `:probe::Note` records (journal, sift, span, cli) are not passed as
`Reason`. No edge was added there. Nothing outside this set went red.

## The words

`wat/query.wat` 73–76 and the file header at 17–18 now say a record joins
`Reason` only by `extend-type`. `src/types/surface.rs` 8–10 says an empty
member list is not a structural interface.

`probe_arc278_dead_child_speaks.wat` 39–42 no longer says any pure record
satisfies `Reason` ambiently. It names the `:probe::Note` edge.

Historical design notes (`docs/arc/2026/06/278-rules-engine/REALIZATIONS.md`,
`DESIGN-telemetry-service-and-query-surface.md`) still describe the old open
surface. They are the record of that design. They were not rewritten.

## STOP-2, quoted

`tests/types/probe_arc293_holder_root_symbol.rs` stated ambient satisfaction
as the contract of a 0-member nature surface. The committed text, verbatim:

```
//! (`:wat::core::Record`), not the magic shorthand `:record`. A 0-member `:nature` surface is "any
//! aggregate of that nature" — the portability shape behind `program::Env`'s `user-data`
//! ("must be at minimum a record").
```

The co-located `.wat` said the same: `A 0-member :nature surface = "any aggregate of that nature"`.

The brief's declaration list requires `(:wat::core::extend-type :env::Rec :env::Portable)`.
That edge is what makes `surface_nature_root_symbol_accepts_record` return
`Value::i64(42)` now. The assertion was not rewritten. The comments were,
so they no longer claim the retired sentence. The sibling struct fixture
(`:env::Stru`, no edge) still fails `TypeMismatch` callee `:env::take` param
`#1` expected `:env::Portable` got `:env::Stru`.

## The hello world

`:hello::bad` and its comment block are deleted from
`wat-scripts/scratch-pad/255-46/hello-extend-type-today.wat`.
`./target/release/wat --check` on that file exited 0.

A `/tmp` copy with the block restored exited 1:

```
#wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":hello::takes-orderable: parameter #1 expects :hello::Orderable; got :hello::Opaque" :location #wat.core/Span {:file "/tmp/255.48-hello/with-bad.wat" :line 34 :col 97 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 34 :col 98}}} :causes [] :callee ":hello::takes-orderable" :param "#1" :expected ":hello::Orderable" :got ":hello::Opaque" :remedies []}]}
```

The vector-of-Opaque call in the same file is ruling C. It stays.

## The rows

`tests/types/probe_arc255_48_featureless_surface.rs`:

| row | result |
|---|---|
| `:probe::Item` with `extend-type` to featureless `:probe::Mark` | `Value::i64(1)` |
| `:probe::Pair` has `n` and `extra`, no edge, against featureful `:probe::HasN` | `Value::i64(1)` |
| `:probe::Item` with no edge against `:probe::Mark` | `TypeMismatch` callee `:probe::take` param `#1` expected `:probe::Mark` got `:probe::Item` |
| `:probe::Box` with no edge against `(:probe::Tag :- [:wat::core::i64 :wat::core::i64])` | `TypeMismatch` callee `:probe::take` param `#1` expected that instantiation got `:probe::Box` |

`Tag` is Spawned-shaped: featureless, parametric, nature Struct. `:probe::Holder`
extends it and consumes `S` and `R`. `Box` does not. The rendered expected
string is reader-parseable, so the comparison carries
`rune:lint(no-inlined-wat)` the way `probe_arc170_parametric_surface_param.rs`
does. The program is the `.wat.bad`, not the string.

## STOP-3

`conforms?` on a surface is still `Ok(false)` at `src/runtime.rs` 10001,
"not yet implemented". That path was not changed. It does not admit what the
checker now refuses.

## The first floor

`.floor/2026-09-26T07-06-07Z` was red and was not re-run.

```
     Summary [ 353.017s] 6145 tests run: 6144 passed (11 slow), 1 failed, 22 skipped
```

`tests_carry_no_inlined_wat` panicked at `tests/lint/no_inlined_wat_in_tests.rs:440`.
The assertion is `violations.is_empty()`. The offender was
`tests/types/probe_arc255_48_featureless_surface.rs`: the parametric
`expected` string is a form the reader parses. One parse-body hit. The rune
above is the fix. The whole block is `.floor/2026-09-26T07-06-07Z/ARM.txt`.

## Gates

New floor `.floor/2026-09-26T07-14-25Z`:

```
     Summary [ 351.631s] 6145 tests run: 6145 passed (12 slow), 22 skipped
```

6145 against 6141 at `145d2434a`. The four new rows are the difference.
Ledger stays 198 (`LEDGER_TOTAL` in `tests/lint/keyword_heresy_ledger.rs`).

Clippy `cargo clippy --release --all-targets -- -D warnings` exited 0.

Census `.census/2026-09-26T07-20-42Z.txt` against
`.census/2026-09-26T06-53-24Z.txt`: `census-diff: no STOP-8`. 0 rc flips.
2285 files, 215 nonzero. The new fixture was still untracked, so `git ls-files`
did not count it. The floor loaded it.

Delta `.delta/2026-09-26T07-21-58Z`: NEW 2 / RECOVERY 0. The two NEW files
are `wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.
