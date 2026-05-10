# Arc 171 slice 3 — BRIEF

**Substrate closure; sonnet.** Retire `,` acceptance inside
keyword bodies. Substrate-only — orchestrator handles
INSCRIPTION + 058 changelog row + memory update at commit
time (per `feedback_paperwork_orchestrator_side.md`).

## Mission

Two substrate edits in `src/lexer.rs`:

### 1. Reject `,` inside keyword body with a clean diagnostic

Current state (post slice 1 + 2): `lex_keyword` accepts `,`
as keyword-body character via the fall-through arm. Slice 2
swept all in-tree consumers; no live source still uses
`,suffix` in keyword bodies.

The work: add an explicit match arm or check for `,` inside
`lex_keyword` that emits `LexError::CommaInKeywordBody`
(or appropriate variant; mirror existing error shape) with a
diagnostic naming `'` as the canonical separator and citing
arc 171.

Suggested error wording:
> *"comma inside keyword body retired (arc 171); use apostrophe
> `'` as the dispatch / discriminator separator. Example:
> `:wat::core::op'2` (arity), `:wat::core::op'i64'i64`
> (type-discriminator). The legacy `,2` / `,i64-f64` shape was
> swept in slice 2 (~440 sites)."*

The error variant goes in `LexError` enum (find via grep
`pub enum LexError` in `src/lexer.rs`). If similar
"X inside Y body retired" errors exist (e.g., for slice 1f-W's
underscore-inside-`<>` rule), mirror their shape.

### 2. Retire the `keyword_comma_suffix_transition` test

`src/lexer.rs:876-877` has `keyword_comma_suffix_transition`
test that asserts `:wat::core::op,2` STILL parses (transition
mode). Slice 3 retires comma acceptance, so this test must go.

REPLACE it with a new test that asserts the rejection now
happens with the clean diagnostic. Suggested test name:
`keyword_comma_in_body_rejected`. Test input: `:wat::core::op,2`.
Assert: returns `LexError::CommaInKeywordBody` (or whatever
variant the slice mints) with the expected diagnostic
fragment.

### 3. Update doc comment on `lex_keyword`

The doc comment was updated in slice 1 to say `'` is accepted
alongside `,` (in transition). Slice 3 removes the
transition-mode language; the doc says only `'` is canonical
and `,` is rejected. Cite arc 171 closure.

## What to NOT do

- **No INSCRIPTION authoring.** Orchestrator handles closure
  paperwork.
- **No 058 changelog row.** Orchestrator.
- **No memory updates.** Orchestrator.
- **No consumer changes.** Slice 2 swept all consumers; if
  any new comma-in-keyword sites have appeared since slice 2,
  surface as honest delta — DON'T sweep them in this slice.
- **No other Rust files modified.** Slice 3 is `src/lexer.rs`
  only.

## Substrate-grep citations

- `src/lexer.rs:387-389` — current `lex_keyword` doc comment
- `src/lexer.rs:389-...` — `lex_keyword` function body
- `src/lexer.rs:876-877` — `keyword_comma_suffix_transition`
  test (the one to retire/replace)
- `src/lexer.rs::LexError` enum — for error variant shape
  (grep for `pub enum LexError`)
- Memory: `feedback_apostrophe_dispatch_separator.md` — the
  user's locked convention; the diagnostic should match its
  framing

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A — `,` inside keyword body rejected | grep `lex_keyword` finds explicit reject arm; `:wat::core::op,2` returns `LexError` | ✓ |
| B — Error variant minted | `LexError::CommaInKeywordBody` (or analogous) exists with cited arc 171 diagnostic | ✓ |
| C — `keyword_comma_in_body_rejected` test added | replaces the retired `keyword_comma_suffix_transition` test; asserts the rejection | ✓ |
| D — Slice 1's 6 apostrophe tests still pass | the apostrophe-acceptance tests are not affected | ✓ |
| E — Doc comment updated | no transition-mode language; canonical apostrophe-only | ✓ |
| F — `cargo check --release` green | no compile errors | ✓ |
| G — Workspace at baseline ±5 | post-slice-2's 1334/854 stays (or +1 if the new test replaces the retired one cleanly) | ✓ |
| H — Only `src/lexer.rs` modified | `git diff --stat` shows 1 file | ✓ |
| I — Zero new dependencies | Cargo.toml unchanged | ✓ |
| J — Honest deltas surfaced | per FM 5 | ✓ |

## Honest delta categories

- **Error variant name** — if `LexError::CommaInKeywordBody`
  conflicts with existing variants or doesn't read clean,
  surface; suggested alternative: `KeywordBodyCommaRetired`
- **Comma-in-tuple-keyword consideration** — keywords like
  `:(A,B,C)` (tuple types in keyword position) contain commas
  inside `(...)` brackets. The lex_keyword function's
  `paren_depth` tracker means these commas are "inside parens
  inside keyword body" — different position from "after the
  verb stem, before the suffix." The new reject rule must NOT
  fire on tuple-position commas. Surface if the
  parens/non-parens distinction needs explicit handling.
- **Similar issue for type-param commas** — `:HashMap<K,V>`
  has comma inside `<>` (slice 1f-W's domain). Per slice 1f-W
  + arc 171: commas inside `<>` are valid as type-arg
  separators; the wire-encoding swap handles them. The new
  reject rule must NOT fire on `<>`-position commas either.
  Surface if `angle_depth > 0` check is needed.
- **`tests/wat_*.rs` raw strings** — sweep in slice 2 covered
  raw wat strings in test fixtures. Verify none of those
  contain legacy `,N` in keyword bodies. If any do, surface
  for slice 4 (or amendment).

## Predicted runtime

30-45 min sonnet. Small lexer edit (one explicit reject arm)
+ one test replacement + doc-comment cleanup.

**Hard cap:** 90 min.

## Reference

- DESIGN.md
- SCORE-SLICE-1.md + SCORE-SLICE-2.md
- `feedback_apostrophe_dispatch_separator.md` (memory)
- `src/lexer.rs` — the file to edit
