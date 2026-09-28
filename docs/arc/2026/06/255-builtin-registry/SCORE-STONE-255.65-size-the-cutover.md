# SCORE — STONE 255.65: the cutover, sized

Measurement. Nothing lands but this score. No worktree. Drawn against
`main` @ `d0f45d4ca` (the draw; `45da20c8f` is an ancestor). Counts are
text counts unless a line says inferred.

Ruling 2 supersedes 255.64 where they disagree. 255.64 spelled `Instant`
and `Duration` as `wat.type/`. This ruling keeps `wat.time/Instant` and
`wat.time/Duration`. The code still keys them `:wat::time::…`
(`src/types.rs:3485-3486`).

## 1. The substrate's keys

255.60 reported 5,745 exact keyword literals in 209 files. That count
does not reproduce. Measured again, two ways:

| definition | n | files |
|---|---|---|
| a Rust string whose entire contents match `:ident(::ident)+`, under `src/` | 2,200 | 189 |
| the same pattern as a token, preceding character not `:` (so not a Rust `::path`), under `src/` | 23,538 | 286 |
| same token rule, under `crates/` | 597 | 22 |

The token count includes comments, doc prose, and a keyword inside a
larger string. Heaviest whole-string files: `src/remedy/retirement.rs`
222, `src/types.rs` 127, `src/rete/vocabulary.rs` 101, `src/runtime.rs`
98. Whole-string roots: `wat::core` 920, `wat::rete` 228, `wat::holon`
212, `wat::kernel` 163.

Of the 920 whole-string `:wat::core::…` literals, the tail classes as:

| class | n | new key under rulings 2 and 3 | mechanical? |
|---|---|---|---|
| function or form (`defn`, `let`, `fn`, …) | 481 | `wat.core/<name>` | yes, `canonical_identity` already maps that symbol to today's key |
| hard primitive (`i64` 84, `String` 40, `Vector` 29, `HashMap` 22, `keyword` 18, `nil` 18, `List` 17, `f64` 16, `bool` 14, `HashSet` 13, `Tuple` 8, `u8` 6, `rational` 5, `char` 5) | 295 | `wat.type/<name>` if it is a hard primitive | yes, once the primitive set is closed (question 1) |
| `PersistentMap` 17, `PersistentVector` 15 | 32 | unnamed | question 1 |
| `Record` 21, `Uuid` 18, `Bytes` 6 | 45 | `Uuid` → `wat.uuid/UUID` (ruled). `Record` and `Bytes` are not named | question 1 for `Bytes`; `Record` is a nature, `src/types.rs:522` |
| `Option` 12, `Result` 8 | 20 | `wat.core/Option`, `wat.core/Result` | yes |

`:wat::WatAST` whole strings: 31. New key `wat.type/AST` (ruled).
`HolonAST` whole strings under `src/`: 9. New key `wat.holon/HolonAST`
(ruled; already that namespace). `:wat::time::` whole strings: 21, and
they are not all types (`at-nanos`, `now` sit in the token count). The
type keys stay `wat.time/Instant`, `wat.time/Duration`.

A mechanical rewrite is exact for a literal whose whole string is one
name. It is not exact for a keyword embedded in an error sentence, a
format template, or a doc example. Those are the 3,256 Rust strings
that contain `wat::` but are not themselves one keyword, plus 1,155
colon-strings that contain `::` and more text. Measured in `src/`,
`crates/`, and `tests/`.

## 2. Where identity lives

| door | site | what C1 does |
|---|---|---|
| `canonical_identity` | `src/edn/render.rs:3662` | maps `wat.core/i64` and `:wat::core::i64` to the same `:wat::core::i64`. Does not rewrite `wat.type`. After C1 the stored key is the new spelling, so this function's `::` output is the old key |
| `type_denotation` | `src/edn/render.rs:3693` | `:wat::type::X` → `:wat::core::X`. 35 text occurrences of `type_denotation(` in 5 files: `src/edn/render.rs`, `src/types.rs`, `src/check.rs`, `src/collection/infer.rs`, `tests/lint/keyword_heresy_ledger.rs`. The brief's 31 is this neighborhood. After one spelling, the function is dead |
| `denoted_type_path` | `src/types.rs:190` | `type_denotation` plus the `Infer` exception. Dead with it, except `Infer` if that marker stays a different word |
| `canonicalize_type_kw` | named only in a comment, `src/types.rs:957` | already gone |
| `ns_to_wat_path` | `src/edn/render.rs:3651` | builds the `::` key from a dotted namespace. Dies as a key builder; the path split itself stays if symbols still have dots |
| `compose_variant` | `crates/wat-reader/src/identifier.rs:407`, 64 text calls in 29 files | already joins with `.`. The callers that pass a `::` namespace still mint `::` (255.64 changed one). The door stays; the namespace argument changes |
| `Identifier` | `crates/wat-reader` | the symbol `wat.type/i64` is already a reference. C1 stores that text as the key instead of running it through `ns_to_wat_path` |
| `format_type` | `src/check.rs:18235`, 319 text calls in 26 files | renders the denotated colon key. Users see it in type mismatches. After C1 it prints `wat.type/i64` / `wat.core/Option` |
| `format_type_expr` | `src/freeze.rs:1715`, 7 text calls | same door, freeze wire |
| EDN tags | already `#wat.core/Span` | see §5. The tag is not the `::` key |
| `AggregateValue.class` | `src/value/value.rs:1008` | an `Arc<str>` of the class path. `type_name` (`:468`) is documented as the `::` path users write. Both become the new spelling or they are a second key |
| a second `canonical_identity` | `crates/wat-source-derive/src/lib.rs:105` | the derive's own copy. It must move with the first or the registration key and the runtime key diverge |

## 3. Constructors that are types

Text counts on 2,698 tracked `.wat` and `.wat.bad` files. "Type" means
the head is followed by ` :-`. "Call" is the rest.

| head | lists | type | call |
|---|---|---|---|
| `Vector` | 4,121 | 4,083 | 38 |
| `Tuple` | 1,548 | 1,007 | 541 |
| `PersistentVector` | 2,090 | 880 | 1,210 |
| `HashMap` | 414 | 405 | 9 |
| `PersistentMap` | 163 | 39 | 124 |
| `HashSet` | 151 | 148 | 3 |
| `List` | 103 | 10 | 93 |
| `Option` | | 235 | 1 |
| `Result` | | 104 | 1 |

255.62's 4,107 was `(wat.core/Vector` not followed by `:-` on a
converted tree. On this unconverted tree the call-shaped Vector head
is 38. The value constructors that carry elements are the
`(Vector :- [T] e …)` forms, counted here as types because they contain
`:-`. Ruling 4 says those are values: `(wat.type/Vector :- [T] 1 2 3)`
equals `[1 2 3]`, and the three-child form is the empty vector outside
a type position.

The Rust callables, measured: `#[wat_intrinsic(":wat::core::Vector")]`
`src/runtime.rs:8478`, `HashMap` `:8549`, `HashSet` `:8618`,
`PersistentVector` `:8328`, `PersistentMap` `:8399`, `List`
`src/intrinsic/list.rs:36`, `Tuple` `src/runtime.rs:6521`. After the
cutover each of those names is unknown at `wat.core/`. The literal
forms `[…]`, `{…}`, `#{…}` already exist; how many of the 1,210
`PersistentVector` calls can become a vector literal is inferred, not
counted form by form. A call with a splice (`~@`, `wat/query.wat:416`)
cannot become a literal.

## 4. Homes that move

Ruled, and the code already has the namespace or the leaf:

| today | new | measured |
|---|---|---|
| `:wat::WatAST` | `wat.type/AST` | leaf `src/types.rs:3439` |
| `:wat::time::Instant`, `:wat::time::Duration` | `wat.time/…` | leaves `:3485-3486`. Not `wat.type/` |
| `:wat::core::Uuid` | `wat.uuid/UUID` | 24 `:wat::core::Uuid` hits in `.wat`, 18 whole-string literals in `src/`, 10 Rust files mention the keyword. `UUID` as text appears 13 times in `.wat` and is not this type |
| `:wat::holon::HolonAST` | `wat.holon/HolonAST` | leaf `:3438` |
| `:wat::core::Option`, `:wat::core::Result` | stay `wat.core/` | `defenum` in `wat/core.wat`, structured builtins |

Registered under `:wat::core::` and not in the named hard-primitive
list: `u8` (`BARE_PRIMITIVES`, `src/check.rs:1018`), `bigint`,
`rational`, `keyword` (named), `nil` (`src/types.rs:3498`), `Value`,
`List` (named), `Uuid` (ruled), `char` (41 `:wat::core::char` hits in
9 `.wat` files). `Tuple` is named as a hard primitive and is a
callable (`src/runtime.rs:6521`) but is not a `TypeEnv` member
(TABLE-STONE-Q, `src/types.rs` around the group-3 test). `PersistentVector`
and `PersistentMap` are container heads and callables, and they are
not in the named list.

Already in their own namespace, so the home does not move; only the
`::` key does: `Hologram`, holon `Vector`, `IOReader`, `IOWriter`,
`Process`, `Thread`, `Address`, `Listener`, `Peer`, `Stream`, and the
two `:rust::crossbeam_channel::` leaves. Declared records in
`wat/core.wat` (`Span`, `Error`, `Fault`, `EvalError`, `ReadOutcome`,
`ReadWithCommentsOutcome`) stay `wat.core/` as utilities. Their key
still changes from `:wat::core::Span` to `wat.core/Span`.

## 5. What a reader sees

`format_type` prints the colon key. Type-mismatch `expected`/`got`
text changes when the key changes. `git ls-files '*.edn'` is 418
files; `tests/**/*.edn` is 406. Tags already use the dotted form.
Measured tag texts, top: `#wat.core/Span` 398, `#wat.core/Option.Some`
287, `#wat.core/Pos` 258, `#wat.core/Option.None` 150. Those tags do
not contain `::`. A cutover that only changes the internal key does
not rewrite them. A cutover that changes `format_type` or the tag
namespace (`Span` leaving `wat.core`) does. `freeze.rs`'s
`format_type_expr` is the other printer.

No persisted journal was opened. Whether a frozen image stores
`:wat::core::…` is inferred from `format_type_expr` writing that
spelling.

## 6. Bootstrap

255.63 and 255.64 measured this, and it still holds. `wat/fix.wat` is
`include_str!`'d. The binary that converts must embed an unconverted
stdlib, and the new rules must already be in that unconverted
`fix.wat`. A binary that embeds a converted stdlib the checker rejects
cannot run the next pass (255.64: `extend-type` of `wat.core/i64` at
`wat/class.wat:16`, then 534 type errors with the wall off, first
`conj` at `wat/bracket.wat:356`).

So the position rule (ruling 4) has to be in the checker before a
converted stdlib is asked to load, and the codemod has to know the
type slots that are not behind an arrow (`typealias`'s body,
`extend-type`'s type arguments). Walls last.

## 7. Other branches

Commits not on `main`, `git rev-list --count main..<b>`:

| branch | commits | files vs merge-base | of which `src` / `wat` / `tests` |
|---|---|---|---|
| `origin/grok-rete` | 653 | 1,324 | 88 / 21 / 315. 62 of the src files are under `src/rete/` |
| `origin/sns-sqs` | 551 | 1,548 | 71 / 19 / 288 |
| `origin/queue-promotion-blocked-on-startup-cost` | 486 | 1,395 | 53 / 17 / 239 |
| `origin/gen-tests` | 168 | | |
| `origin/claude-compute` | 133 | | |
| `origin/reason/little-wat-findings` | 69 | 516 | 69 / 12 / 326 |
| `origin/the-little-wat` | 10 | 177 | |

A cutover that rewrites `src/` type literals and every `.wat` file
conflicts with those `src` and `wat` files. `sns-sqs` and `grok-rete`
are the large ones. Not merged. Inferred: rebase after the cutover,
not before. The docs-heavy file counts (857, 674, 793) are mostly
`docs/` and will not all conflict with a code cutover.

## Questions the rulings do not answer

1. **Which registered core names are hard primitives?** Named: `i64`,
   `f64`, `String`, `bool`, `keyword`, `Vector`, `HashMap`, `HashSet`,
   `List`, `Tuple`, `AST`. Not named, and present in the registry or
   as a callable: `u8`, `bigint`, `rational`, `char`, `nil`, `Value`,
   `PersistentVector`, `PersistentMap`, `Bytes`. If they are
   `wat.type/`, the primitive stone includes them and `wat.core/<that>`
   in a type position becomes illegal. If they need their own homes,
   each home is a stone of its own and `Bytes` has no obvious
   namespace. `Tuple` is named `wat.type/Tuple` but is not a `TypeEnv`
   member today; making it one is part of that stone, not a rename.
2. **May a green stone keep two keys for one release?** Ruling 3 says
   one key at the end and `type_denotation` dies. A sequence of green
   stones needs either one stone that flips the parser, the `src/`
   literals, and the corpus together, or a temporary door a later
   stone deletes. The first is not landable as several greens. The
   second contradicts the end state until the deleting stone. The
   sequence below is written for the temporary door. It does not
   choose it.

## A sequence, if a temporary door is allowed

Each stone is green on floor, clippy, census, and delta before the
next. The walls are last.

1. **Position.** Rust. A three-child `(Head :- [args])` in expression
   position is the empty value; extra elements are the constructor
   call. Gate: the `bracket.wat:356` `conj` and the 255.64 startup
   class no longer fail on an otherwise converted stdlib, and today's
   unconverted floor stays green. Depends on nothing. Blast: `check.rs`,
   the collection intrinsics at the lines in §3.
2. **Codemod, types only.** Wat-fix, plus the small denotation rows for
   `AST` (and for whichever names question 1 puts in `wat.type/`).
   Type positions and the constructor forms of those names become
   `wat.type/…`. `Option`/`Result` stay `wat.core/`. Time stays
   `wat.time/`. The internal key stays `:wat::core::…`. The codemod
   learns `typealias` and `extend-type` slots before it runs, and it
   runs from an unconverted stdlib (§6). Gate: converted stdlib loads,
   floor, census. Depends on 1 and on question 1.
3. **Homes that are renames.** Rust keys plus a codemod. `Uuid` →
   `wat.uuid/UUID`. `HolonAST` is already `wat.holon/`. Instant and
   Duration stay put. Depends on 2. Blast: the 24 wat hits, the 18
   src literals, the case change in every comparison.
4. **C1 for the hard primitives.** Rust. The parser stores
   `wat.type/i64` as that key. `src/` literals of those types are
   rewritten in the same stone. `type_denotation` stops rewriting
   those tails. Gate: floor, and no test still compares a primitive
   to `:wat::core::i64`. Depends on 2 and 3. This is the stone that
   deletes the door for primitives. Blast: the primitive rows of §1
   and §2, `AggregateValue.type_name` where it stores a primitive.
5. **Function and form heads.** Rust plus a codemod for any `.wat`
   still on `:wat::core::defn`. Registry key becomes `wat.core/defn`.
   Mechanical for a whole-string literal. Not mechanical for the
   embedded sentences. Depends on 4 only so a type and a function are
   not retargeted in one diff. Blast: 481 whole-string function
   literals, and the token count's `wat::core` bulk.
6. **Printers and goldens.** `format_type` / `format_type_expr` print
   the new key. Update the `.edn` goldens whose bytes change.
   Depends on 4 and 5. The `#wat.core/Span` tags stay if Span stays a
   `wat.core` record.
7. **Walls.** The `::` lexer wall and the F1 wall (a primitive not
   spelled `wat.type/` in a type position). Depends on 4, 5, and 6.
   Expected blast: whatever sentence still embeds a `::` name. Those
   are the non-exact strings in §1, cured or the wall waits.

Stones 2 and the `.wat` half of 3 and 5 are codemods. Stones 1, 4, and
6 are Rust. Stone 7 is Rust. Question 1 can split stone 2. Question 2
can collapse 2 and 4 into one stone that does not land as several
greens.

## Risks

A frozen image or an error golden that stores `:wat::core::i64` is a
second spelling until it is rewritten. Measured printers:
`format_type`, `format_type_expr`. Not measured: any file outside git.

EDN tags are already `#wat.core/…`. Leaving them alone keeps the 398
`Span` tags stable. Moving `Span` out of `wat.core` rewrites them.
Span stays a core record under ruling 2, so the tag text can survive
C1. Inferred from the tag spelling, not from a round-trip test.

`origin/sns-sqs`, `origin/grok-rete`, and
`origin/queue-promotion-blocked-on-startup-cost` each touch `src/` and
`wat/`. Landing the cutover under them, or rebasing them under the
cutover, is a merge of those files. Not done here.
