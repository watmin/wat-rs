# BRIEF — Stone 234.4.match — match-arm hash-destructure parity

**Status:** READY TO SPAWN.

**Predecessor:** Stone 234.4 (let-binding hash-destructure; SHIPPED `dab1a5cb` 11/11 PASS). Same three-receiver coverage (Record / Struct / HashMap); same `{var :field ...}` shape; same per-class polymorphic T type-check; match-arm position INSTEAD of let-binding position.

## What to do

Extend match-arm pattern recognition to accept `{var :field ...}` hash-destructure shape with the same three-receiver dispatch as Stone 234.4 (let-binding) shipped.

```wat
(:wat::core::match scrutinee
  [{var1 :field1 var2 :field2} body])
```

When scrutinee is `Value::wat__Record` / `Value::Struct` / `Value::wat__std__HashMap`: arm matches; bind each `var` to its corresponding field-value (or Option<V> for HashMap). When scrutinee is any other Value variant: arm falls to next.

ONE arc-area: `src/parser.rs` + `src/check.rs` + `src/runtime.rs` + new probe file `tests/probe_arc234_stone4_match_hash_destructure.rs`.

## Read in order

1. `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.4.match.md` — 10 locked decisions + 10 trap-doors
2. `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.4.match.md` — 11-row scorecard
3. **`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.4.md`** — predecessor pattern (mirror EXACTLY; 3-file change shape; 6-contract probe template; clippy/baseline discipline)
4. `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.4.md` — let-binding sub-DESIGN context
5. `src/runtime.rs:14399` — `try_match_pattern` (extend here for match-arm hash-destructure dispatch)
6. `src/runtime.rs:24939` — `try_match_pattern_ast` (verify whether parallel update needed)
7. `src/check.rs:6889` — `infer_match` (extend StructPattern arm with hash-destructure discrimination)
8. `src/parser.rs` BraceKind region (Stone 234.4's parser work) — verify whether parser changes needed for match position (likely zero-touch)

## Implementation pattern

### Step 1 — verify parser zero-touch (probe-first)

The parser already recognizes `{var :field ...}` via `BraceKind::HashDestructure` → `WatAST::StructPattern` with mixed Symbol/Keyword children. In match-arm position, this same recognition should fire because the parser sees the brace + content shape, not the surrounding form.

Write a minimal probe FIRST: `(match record [{x :age} x])` — does parsing succeed and produce `WatAST::Match([scrutinee, StructPattern([Symbol("x"), Keyword("age")]), Symbol("x")])`? If YES → zero parser touch; proceed to check.rs + runtime.rs. If NO → parser needs extension (likely small).

### Step 2 — check.rs `infer_match` extension

In `infer_match`, the StructPattern arm processes patterns like arc-169 struct-destructure (all-Symbol children). Extend it to detect hash-destructure shape via `items[1].is_keyword()` and branch:
- Hash-destructure path: for each (Symbol, Keyword) pair, bind the Symbol to `fresh.fresh()` in the arm's local env; type-check the arm body
- Existing arc-169 path: unchanged (all-Symbol content; existing destructure logic)

Mirror Stone 234.4's check.rs work (process_let_binding StructPattern arm hash-destructure detection).

### Step 3 — runtime.rs `try_match_pattern` extension

`try_match_pattern` returns `Option<env>` — `Some(env_extended)` if the pattern matches; `None` if it doesn't. Extend the StructPattern arm to:

```rust
// Detect hash-destructure shape
if items.len() >= 2 && matches!(items.get(1), Some(WatAST::Keyword(_, _))) {
    // Hash-destructure pattern; check scrutinee type
    match scrutinee {
        Value::wat__Record(rec) => {
            // For each (Symbol, Keyword) pair, extract field via keyword_accessor_record
            // Bind into env; return Some(env)
        }
        Value::Struct(s) => {
            // Same with keyword_accessor_struct
        }
        Value::wat__std__HashMap(hm) => {
            // For each (Symbol, Keyword) pair, construct keyword key + look up
            // Wrap result in Value::Option(Some/None); bind into env; return Some(env)
        }
        _ => return Ok(None), // arm fails to match; fall to next
    }
}
// else: existing arc-169 struct-destructure path
```

Reuse helpers from Stone 234.4's runtime work:
- `keyword_accessor_record` (or equivalent helper that Stone 234.4 used)
- `keyword_accessor_struct`
- HashMap keyword-key construction pattern

### Step 4 — verify `try_match_pattern_ast` parity (or non-impact)

`try_match_pattern_ast` at `src/runtime.rs:24939` is the WatAST-level mirror. Check whether macro / quasiquote paths process match-arm hash-destructure patterns. If YES → parallel extension; if NO → leave untouched + document why in SCORE.

### Step 5 — probe authoring

`tests/probe_arc234_stone4_match_hash_destructure.rs` — 6 contracts per the DESIGN T10:

1. Match record with single `{var :field}` — extracts field; body uses var
2. Match record with multi `{var1 :f1 var2 :f2}` — multi-field bind
3. Match HashMap with `{var :field}` — Option<V> bind per key
4. Match HashMap multi-key — multi Option<V>
5. Match-arm fall-through: scrutinee `Value::i64` → hash-destructure arm fails → falls to next arm (which binds via underscore or matches int)
6. Mixed-pattern match: one arm hash-destructure; another a literal; selection correct

Mirror Stone 234.4 probe's contract shape exactly. Use `:wat::test::run-thread` for harness (same as Stone 234.4).

## Discipline

- Touch ONLY: `src/parser.rs` (IF needed) + `src/check.rs` + `src/runtime.rs` + the new probe file (STOP-5)
- DO NOT touch: arc 234 historical artifacts (DESIGNs / BRIEFs / SCOREs for shipped stones); arc 236 artifacts; any wat/ files; any other Rust files
- DO NOT commit (orchestrator commits)
- DO NOT mint transitional aliases / macro wrappers for the match-arm form (D8 HARD CUT)
- DO NOT introduce nested-pattern hash-destructure (T6 OUT OF SCOPE)
- DO NOT touch holon-rs (STOP-4)

## Lib baseline handling

Expected: 827 unchanged. Pure additive extension; existing match patterns + arc-169 struct-destructure preserved.

Tolerance: 0 drops expected. If ANY lib test drops, investigate immediately (likely the arc-169 path got accidentally regressed). > 1 drop = STOP-2.

## STOP triggers (REJECTION)

1. Unexpected compile errors not tracing to extension / cascade
2. Lib baseline drops below 827 by even 1
3. **120 min elapsed** (Mode A target 60-90 min; STOP-3 is 2× upper-bound)
4. holon-rs touched
5. Rust changes outside the 3 substrate files + the new probe file
6. arc 234 (any prior probe) OR arc 236 regression
7. clippy > 54
8. Transitional macro / aliasing minted (D8)
9. Nested-pattern hash-destructure introduced (T6)

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.4.match.md` (NEW).

Capture (mirror Stone 234.4 SCORE shape):
- 11-row scorecard verbatim outputs
- Receivers shipped (3 — Record / Struct / HashMap)
- Three-file change summary (parser line count if any; check.rs line count; runtime.rs line count)
- Implementation notes (parser zero-touch verified? AST mirror parity needed? Empty-pattern decision?)
- Cascade depth: compile rounds
- Honest deltas if any
- Rank-up evidence — did Stone 234.4 SCORE work as template? Did the probe-first parser verification pay off?

Closing note: the named follow-up from Stone 234.4 D8 is CLOSED. Arc 234 is now one stone (234.6 decision + 234.7 INSCRIPTION) from closure.
