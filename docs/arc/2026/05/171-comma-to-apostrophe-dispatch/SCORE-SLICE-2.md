# Arc 171 slice 2 — SCORE

**Result:** Mode A clean.
**Runtime:** ~45 min sonnet (within predicted 60-120; well under 180 hard cap).
**Files:** 45 modified, ~440 sites swept.

## Calibration

- **Predicted runtime band:** 60-120 min sonnet
- **Actual:** ~45 min — under band
- **Scope estimate vs reality:** BRIEF estimated ~167 sites;
  actual was ~440 sites across 45 files. Sonnet's broader
  pattern (`:[a-zA-Z][a-zA-Z0-9_:...]*,`) caught
  `:i64::+,2`-style sites the BRIEF's regex missed. Honest
  scope-estimation miss; sonnet handled the broader scope
  cleanly without scope expansion or scope creep.

## Scorecard (9 rows)

| Row | What | Result |
|-----|------|--------|
| A — All wat source files swept | ✓ grep zero hits in `wat/`, `wat-tests/`, `crates/*/wat*/`, `examples/*/wat/`, `wat-scripts/` |
| B — Rust diagnostic strings swept | ✓ only intentional matches remain (tuple commas, type-param commas, the transition test, doc-comment type examples) |
| C — Archived arc docs UNCHANGED | ✓ `git diff -- docs/arc/2026/05/130-*` returns 0 lines |
| D — Wire-encoding test commas preserved | ✓ `crates/wat-edn/tests/wire_encoding.rs` commas all inside `<>` positions (HashMap<K,V> wire-encoding rule per slice 1f-W) |
| E — `cargo check --release` green | ✓ (1 pre-existing warning unrelated) |
| F — Workspace at baseline ±5 | ✓ exactly 1334/854 (delta 0; zero regression from slice 1's 1334/854) |
| G — Slice 1 tests still green | ✓ all 6 apostrophe tests + `keyword_comma_suffix_transition` (transition test) pass |
| H — Zero new dependencies | ✓ Cargo.toml unchanged |
| I — Honest deltas surfaced | ✓ 6 categories surfaced (see below) |

**9/9 rows pass.** Mode A clean.

## Sites swept per bucket (verified by orchestrator)

| Bucket | Files | Sites |
|---|---|---|
| `wat/core.wat` | 1 | 32 (dispatch table + variadic wrappers + comments) |
| Other `wat/*.wat` | 4 (`stream`, `test`, `holon/Reject`, `holon/Circular`) | 4 |
| `wat-tests/*.wat` | 2 (`test.wat`, `holon/eval-coincident.wat`) | 17 |
| `crates/*/wat*/*.wat` | 4 (wat-lru × 2; wat-holon-lru × 2) | ~20 |
| `src/runtime.rs` | 1 | 143 |
| `src/check.rs` | 1 | 30 |
| `src/macros.rs` | 1 | 7 |
| `src/freeze.rs` | 1 | 9 |
| `src/resolve.rs` | 1 | 6 |
| `tests/wat_*.rs` (embedded wat) | 27 | ~170 |
| **TOTAL** | **45** | **~440** |

## Honest deltas surfaced (6 categories)

### 1. EDN wire-encoding commas preserved

`crates/wat-edn/tests/wire_encoding.rs` has 3 occurrences of
`HashMap<String,i64>` inside `<>` positions — these are
slice 1f-W's wire-encoding commas (commas inside `<>` swap to
underscore on the wire), NOT keyword-body commas. Left
untouched per BRIEF.

### 2. Transition-mode test preserved

`src/lexer.rs:876-877` (`keyword_comma_suffix_transition`)
explicitly verifies `:wat::core::op,2` still parses.
Preserved per BRIEF (transition mode active until slice 3
retires comma acceptance).

### 3. Archived arc docs skipped per "inscribed is inscribed"

`docs/arc/2026/05/130-*/complected-2026-05-02/` confirmed 0
changes per `git diff`. Historical record stays as-is.

### 4. Tuple / type-param commas NOT swept (correctly)

100+ remaining grep matches are all legitimate uses:
- Tuple commas: `:(A,B)`, `:(A,B,C)`
- Type-param commas: `HashMap<K,V>`, `Result<X,Y>`,
  `Fn(A,B)->C`
- Doc-comment type examples in `src/types.rs`

These are CORRECT — not keyword-body dispatch separators. The
BRIEF's grep pattern was intentionally broad; sonnet
correctly distinguished and left them alone.

### 5. `eval-coincident.wat` integrity artifacts regenerated

The file contains Ed25519 signatures + SHA-256 digests of
source strings. Sweeping source strings required recomputing
both — sonnet did this in lockstep with the source edits.

Verified by orchestrator:
- `EXPECTED_SRC_A_SIG` in `src/runtime.rs` = `HaTLEi...vaOdBg==`
- `EXPECTED_SRC_B_SIG` = `m1rJF1...DlmOBA==`
- SHA-256 digests in `eval-coincident.wat`: `fb0e9f41...` /
  `650e4f7e...`

Constants in `runtime.rs` AND string literals in the wat file
match. Integrity verification works under the new shape.

### 6. Scope was 3× larger than BRIEF estimated

BRIEF estimated 167 sites; actual was ~440 across 45 files.
Test files alone contributed ~170 sites; `src/runtime.rs` had
143 (substantial diagnostic-message content). Sonnet's
broader grep pattern caught these; all swept cleanly without
scope creep.

## Calibration row

- **Actual runtime:** ~45 min (within predicted 60-120 band)
- **Workspace post-1f-0a:** 1334 passed / 854 failed
- **Pass-count delta:** 0 (exact baseline preservation)
- **Fail-count delta:** 0 (zero regressions)
- **Files modified:** 45
- **Sites swept:** ~440
- **Honest deltas surfaced:** 6 (well-classified)
- **Sonnet model validated:** mechanical sweep + integrity-
  artifact regeneration + judgment on tuple/type-param
  distinction — all within sonnet's wheelhouse; ~45 min for
  ~440 sites = ~10 sites/min. Efficient.

## What's next (orchestrator-side)

1. **Commit slice 2 atomically** (this turn) — 45 files + this
   SCORE doc.
2. **Author slice 3 BRIEF + EXPECTATIONS** — closure work:
   - Retire `,` acceptance inside keyword bodies in
     `lex_keyword` (clean diagnostic naming `'` as canonical)
   - Retire `keyword_comma_suffix_transition` test
   - Ship INSCRIPTION + 058 changelog row + memory update
3. **Spawn slice 3 sonnet** — small (~30-45 min predicted).

## Cross-references

- BRIEF: [`BRIEF-SLICE-2.md`](./BRIEF-SLICE-2.md)
- DESIGN: [`DESIGN.md`](./DESIGN.md)
- Predecessor: slice 1 (`a40a40a`) — lexer accept apostrophe
- Successor: slice 3 — closure + comma retirement
- Memory: `feedback_apostrophe_dispatch_separator.md` (user's
  locked convention)
- Sibling: arc 172 — Scheme → Clojure macros; ships after
  arc 171 closes
