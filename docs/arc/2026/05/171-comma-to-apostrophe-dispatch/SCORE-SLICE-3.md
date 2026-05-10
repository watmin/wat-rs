# Arc 171 slice 3 — SCORE

**Result:** Mode A clean.
**Runtime:** ~10 min sonnet (well under predicted 30-45; far under 90 hard cap).
**Files:** 1 modified (`src/lexer.rs` — 58 ins / 12 del).

## Calibration

- **Predicted runtime band:** 30-45 min sonnet
- **Actual:** ~10 min — far under band
- **Why faster:** the carve-out logic (commas inside `(...)`
  or `<...>` keyword bodies stay valid) composes cleanly with
  the existing `paren_depth` + `angle_depth` trackers — no
  special-casing needed. The reject rule fires only at
  depth-0; both depths already track correctly.

## Scorecard (10 rows)

| Row | What | Result |
|-----|------|--------|
| A — `,` inside keyword body rejected | ✓ explicit `','` arm at `src/lexer.rs:537`; depth-0 comma returns `LexError::CommaInKeywordBody` |
| B — Error variant minted | ✓ `LexError::CommaInKeywordBody(Position)` at `:137` mirrors `UnclosedBracketInKeyword(Position)` shape |
| C — `keyword_comma_in_body_rejected` test added | ✓ replaces `keyword_comma_suffix_transition`; asserts rejection + tuple/angle carve-out validity |
| D — Slice 1's 6 apostrophe tests still pass | ✓ all 6 (`keyword_apostrophe_*` tests) green |
| E — Doc comment updated | ✓ `lex_keyword` + module-level doc both updated; no transition-mode language |
| F — `cargo check --release` green | ✓ 0 errors |
| G — Workspace at baseline ±5 | ✓ exactly 1334/854 (delta 0) |
| H — Only `src/lexer.rs` modified | ✓ git diff confirms 1 file (58 ins / 12 del) |
| I — Zero new dependencies | ✓ Cargo.toml unchanged |
| J — Honest deltas surfaced | ✓ 5 categories (variant name, carve-outs compose, module-level doc cleanup, no remaining consumers) |

**10/10 rows pass.** Mode A clean.

## Honest deltas surfaced (5)

### 1. Error variant name: `CommaInKeywordBody(Position)`

Mirrors `UnclosedBracketInKeyword(Position)` shape exactly —
single `Position` payload, no string content. Clean read at
both definition + match sites.

### 2. Comma-in-tuple carve-out — works without special casing

The reject arm checks `paren_depth == 0 && angle_depth == 0`;
commas inside `:(A,B,C)` arrive with `paren_depth > 0`
(already incremented by the `(` arm), so they fall through to
`out.push(c)` correctly. Test `keyword_comma_in_body_rejected`
explicitly asserts `:(i64,String)` still parses.

### 3. Comma-in-angle carve-out — works without special casing

Same logic: commas inside `:HashMap<K,V>` arrive with
`angle_depth > 0`. Test asserts `:HashMap<K,V>` still parses.

### 4. Module-level doc updated

Line 31 previously listed `,` among "plain body characters."
Updated to remove `,` from that list, add `'`, and state the
depth-0 rejection rule. The BRIEF specified only the
`lex_keyword` doc comment; the module-level doc was stale and
was corrected in the same edit scope (a justified surface-
extension; the doc was lying).

### 5. No new consumer sites found

Zero grep hits for depth-0 `:[a-zA-Z][^']*,` in any non-lexer
file — slice 2's sweep held; no rot accumulated.

## Calibration row

- **Actual runtime:** ~10 min (Mode A clean — far under
  predicted band)
- **Workspace post-1f-0a:** 1334 passed / 854 failed (exact
  baseline)
- **Pass-count delta:** 0 (transition test retired, rejection
  test added — net 0)
- **Fail-count delta:** 0 (zero regressions)
- **Files modified:** 1 (`src/lexer.rs` only)
- **Honest deltas surfaced:** 5 (well-classified)
- **Sonnet model validated:** carve-out reasoning + variant
  minting + test migration — all sonnet-tier; ~10 min total.

## What's next (orchestrator-side closure paperwork)

1. **Author INSCRIPTION.md** — arc 171 closure record
2. **Add 058 changelog row** — to `/home/watmin/work/holon/holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
3. **Update memory** `feedback_apostrophe_dispatch_separator.md`
   — mark arc 171 SHIPPED
4. **Atomic commit** all of the above + `src/lexer.rs` + this
   SCORE doc together
5. **Push** per `feedback_push_on_commit.md`
6. **Spawn arc 172 slice 1 sonnet** — now that file collision
   on `src/lexer.rs` is resolved

## Cross-references

- BRIEF: [`BRIEF-SLICE-3.md`](./BRIEF-SLICE-3.md)
- DESIGN: [`DESIGN.md`](./DESIGN.md)
- Predecessor: slice 2 (`9566d33`) — consumer sweep (~440 sites)
- Slice 1 (`a40a40a`) — lexer accept apostrophe (predecessor)
- Memory: `feedback_apostrophe_dispatch_separator.md`
- Sibling: arc 172 (Scheme → Clojure macros); spawns now that
  arc 171 closes
