# Arc 170 slice 1f-W — SCORE

**Result:** Mode A clean.
**Runtime:** ~70 min opus (inside 60-90 predicted band; 180 min hard cap).
**Files:** 5 modified + 1 new test file.

## Calibration

- **Predicted runtime band:** 60-90 min opus (hard cap 180 min)
- **Actual:** ~70 min — inside band, on the upper end
- **Why upper-band rather than lower:** the design decision (one entry point vs two) genuinely required deliberation; agent surfaced it as honest delta #1 and chose two entry points (`parse` + `parse_wire`). The split landed cleanly but added implementation surface.

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A — Lexer rejects `_` inside `<>` | ✓ — `Parser::reject_underscore_in_brackets` fires at parse time; `source_rejection_span_points_at_underscore` test verifies span at byte 6 of `:Vec<a_b>` |
| B — Lexer accepts `_` outside `<>` | ✓ — `:rust::crossbeam_channel::Sender` parses successfully via both `parse` and `parse_wire` |
| C — Symbols unchanged | ✓ — `lex_symbol` untouched; symbol `foo_bar` parses |
| D — Writer swaps `,` → `_` inside `<>` | ✓ — `write_keyword_body` helper does depth-aware swap; `:HashMap<K,V>` written as `:HashMap<K_V>` |
| E — Writer doesn't swap outside `<>` | ✓ — `:rust::crossbeam_channel::Sender<T>` written verbatim |
| F — Parser swaps `_` → `,` inside `<>` | ✓ — `parse_wire` uses lexer `wire_decode` flag; wire `:HashMap<K_V>` parses to keyword body `HashMap<K,V>` |
| G — Round-trip identity | ✓ — `parse_wire(write(k)) == k` for all test cases; basic + namespaced + 1-arg + 2-arg + namespaced-parametric variants |
| H — Nested brackets | ✓ — `:Vec<Map<K,V>>` ↔ `:Vec<Map<K_V>>` round-trips at depth ≥ 1; deeply-nested test included |
| I — Empty brackets | ✓ — `:Foo<>` round-trips verbatim (no body chars to swap) |
| J — Slice 1f-i still passes | ✓ — `cargo test --release --test services_stdin` → **12 passed / 0 failed** |
| K — Workspace fail-count delta within ±5 | ✓ — post-1f-W: 1329 passed / 855 failed; baseline was 1306/855; delta is +23 passed (= the new wire_encoding tests) and 0 failed (perfect; ±5 band) |
| L — New test file `crates/wat-edn/tests/wire_encoding.rs` | ✓ — 23 tests covering Rows A through R; all green |
| M — Diagnostic NAMES the rule | ✓ — error message: *"underscore in keyword body inside `<...>` is reserved for wire-escape of comma; use `,` as the type-arg separator in source (arc 170 slice 1f-W)"* — names the rule, teaches the rationale, cites the slice |
| N — Module rustdoc | ✓ — `crates/wat-edn/src/writer.rs` and `crates/wat-edn/src/lexer.rs` both have rustdoc explaining the position-aware rule + cross-ref to REALIZATIONS pass 14 |
| O — Honest deltas surfaced | ✓ — 8 deltas (counted below); none worked-around |
| P — No new dependencies | ✓ — `Cargo.toml` unchanged |
| Q — Foundation + slice 1e + 1f-i files untouched | ✓ — `git diff 630f621..HEAD` shows only `crates/wat-edn/*` + new test file |
| R — Existing 18 underscore-in-keyword forms still parse | ✓ — round-trip-rust-mirror test exercises the canonical examples; broader workspace cargo test exercises all 18 forms (855 fail count unchanged means no flips) |

**18/18 rows pass.** Mode A clean.

## Honest deltas surfaced

### 1. Row A vs Row F + Row G unsatisfiable on a single entry point

**The substantive design call.** Source mode (Row A: reject `_` inside `<>`) and Wire mode (Row F: decode `_`→`,`) need DISTINCT contexts because the same bytes have OPPOSITE outcomes. `:Vec<a_b>` is a source-validity error AND a wire-valid keyword (decoded to `:Vec<a,b>`).

Agent's solution: two entry points
- `parse` — strict source mode (rejects `_` inside `<>`)
- `parse_wire` — wire decoder (translates `_`→`,` inside `<>`)

Round-trip uses `parse_wire(write(k)) == k`. Source-mode round-trip is also tested for forms without commas at depth ≥ 1 (which round-trip via plain `parse` too).

**This split IMPROVES the design** — it makes the Source/Wire distinction explicit at the API level rather than requiring context-tracking through a single entry point. Future readers see `parse` vs `parse_wire` and know which they need.

### 2. EDN-spec edge case resolved by depth-aware lexer

EDN treats `,` as whitespace at the top lexer level; this would have terminated a keyword body at the first comma. Slice 1f-W's lexer overrides this rule at depth ≥ 1 inside a keyword body — `,` becomes a body-continue char ONLY in that scope.

This is the substrate change that lets pass 14's "`,` is the type-arg separator in source" rule work at all. Pre-existing test fixtures with `:wat::kernel::Thread<wat::core::nil,wat::core::nil>` (in `wat-telemetry-sqlite/wat-tests/`) were broken before this slice and may have been part of the 855 baseline failures; whether they now flip to passing depends on downstream type-resolution code.

**Workspace fail count is unchanged at 855** — no test that was passing flipped to failing. Surface for slice 3 sweep: previously-broken parametric-keyword fixtures may now parse but still fail at downstream layers; revised slice 3 sweep handles them.

### 3. `Token::Keyword` ABI change

Was `Token::Keyword(&'a str)`; now `Token::Keyword { body: Cow<'a, str>, body_start: usize }`. Required because:
- (a) wire-decode owned bodies need `Cow`
- (b) parser-layer span computation needs `body_start` independent of `self.lexer.pos()` (which advances past the token via peek)

**Internal API.** Only `parser.rs` and the lexer's own tests consumed `Token::Keyword`; both updated. No external crate consumed this variant. ABI change is contained.

### 4. `Display for Keyword` updated to match writer

`display_equivalence.rs` locks `format!("{}", k)` and `write(&Value::Keyword(k))` as byte-identical (per the existing /sever ward note). Without the update, parametric-type keywords would diverge.

Added private `write_keyword_segment` helper in `value.rs` mirroring writer's `write_keyword_body`. **Forward-compatible** — existing test fixtures don't have commas-at-depth-≥-1 keywords so the equivalence test still holds.

### 5. Diagnostic quality (Row M passes)

Error message: *"underscore in keyword body inside `<...>` is reserved for wire-escape of comma; use `,` as the type-arg separator in source (arc 170 slice 1f-W)"*. Names the rule, teaches the rationale, cites the slice. Span points at the offending `_` byte.

Future readers seeing this error in compile output can self-correct without consulting docs.

### 6. No new dependencies

`Cargo.toml` unchanged. The substrate work used only existing libc + crossbeam + std primitives + `Cow` for the lexer's keyword-body type widening.

### 7. No TODOs in source

Verified — FM 5 honored. Edge cases surfaced as honest deltas in this SCORE; not as TODOs in code.

### 8. Foundation files untouched

`git diff 630f621..HEAD` only touches `crates/wat-edn/*` + new test file. Slice 1e, slice 1f-i, foundation, phase A retirement work all preserved verbatim.

## Calibration row

- **Actual runtime:** ~70 min (Mode A clean — inside 60-90 band, upper end)
- **Workspace post-1f-W:** 1329 passed / 855 failed
- **Fail-count delta from post-1f-i baseline:** 0 (855 → 855; ±0 inside ±5 band)
- **Pass-count delta from post-1f-i:** +23 (= the new wire_encoding fixture tests)
- **Honest deltas surfaced:** 8 (all properly classified — design split, EDN-spec edge, ABI change scoping, Display equivalence, diagnostic quality, no-deps, no-TODOs, foundation-untouched)
- **Pre-grep paid off:** every BRIEF citation matched substrate reality
- **Implementation choice:** **inline swap in lexer with `wire_decode` flag, plus separate `parse` / `parse_wire` entry points**

## Lessons captured

1. **Source vs Wire is API-level, not implementation-level.**
   The agent's instinct to split into two entry points is the
   right design. Future BRIEFs for protocol-level work should
   pre-name the Source/Wire distinction so implementations
   don't have to re-derive it.

2. **The depth-aware lexer pattern composes.** The same depth
   counter that gates `_` rejection (Row A) ALSO gates
   `,` → `_` swap (Row D) AND `_` → `,` swap (Row F).
   Three rules, one counter. Composes cleanly.

3. **Pre-existing parametric-keyword test fixtures may flip to
   passing in slice 3.** Worth re-running the sweep after slice
   3 lands; some 855 baseline failures may resolve naturally
   from this substrate change.

4. **Display ↔ Writer equivalence is a discipline boundary.**
   The /sever ward's `display_equivalence.rs` test caught the
   need to update `Display for Keyword` alongside the writer.
   Locking these together prevents subtle round-trip drift.

5. **The agent's runtime came in upper-band because of design
   deliberation, not implementation friction.** This is healthy
   — when a slice surfaces a real design decision (Row A vs
   Row F unsatisfiable on one entry point), the time spent
   thinking is the work, not overhead. SCORE captures the
   reasoning so future-readers see the decision lineage.

## What's next

1. **Atomic-commit slice 1f-W** (this turn) — bundle the 5
   modified files + new test file + this SCORE doc
2. **Author BRIEF + EXPECTATIONS for slice 1f-ii (StdOutService)**
   — applies the registration pattern from 1f-i + uses the wire
   encoding from 1f-W (specifically: writer side via `write` +
   parser side via `parse_wire`)
3. **Spawn slice 1f-ii**

## Cross-references

- BRIEF: [`BRIEF-SLICE-1F-W.md`](./BRIEF-SLICE-1F-W.md)
- EXPECTATIONS: [`EXPECTATIONS-SLICE-1F-W.md`](./EXPECTATIONS-SLICE-1F-W.md)
- BUILD-PLAN ref: §3 slice 1f-W
- DESIGN ref: § three substrate services (transmission services depend on this wire foundation)
- REALIZATIONS pass 14 — wire encoding lexical doctrine
- Predecessor: slice 1f-i (`630f621`)
- Slice 1f-i parser: still uses `parse` (strict source mode);
  may need switching to `parse_wire` when 1f-i parses incoming
  wire data — but slice 1f-i's tests use simple EDN without
  parametric type keywords, so `parse` works for now. Slice
  1f-iv (substrate runtime startup integration) is the natural
  point to switch slice 1f-i's actual fd 0 read path to
  `parse_wire`. **Surface as forward concern for slice 1f-iv
  BRIEF.**
