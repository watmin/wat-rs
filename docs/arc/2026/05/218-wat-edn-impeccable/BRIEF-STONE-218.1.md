# BRIEF — Arc 218 Stone 218.1 — L1 fixes + cross-spell convergence

**Stone scope (sonnet portion):** close the two L1 findings from vigilia + extract the cross-spell `write_keyword_body` duplication. Foundation work for arc 218; subsequent stones operate on settled L1-clean substrate.
**Type:** Sonnet Mode A.
**Time budget:** 25-45 min target; 60 min STOP.
**Depends on:** the 2026-05-21 vigilia cast on `crates/wat-edn/src/` — see `VIGILIA-REPORT-2026-05-21.md` (this directory) for full per-spell aggregate and `DESIGN.md` for stone decomposition. Calibration shape: Stone 216.7 SCORE (`b13fd16` — 12/12 in ~45 min Mode A).
**Unblocks:** Stone 218.2 (naming sweep — references the post-extraction file structure), Stones 218.3/218.4/218.5.

## Why this stone exists

`crates/wat-edn/` returned **DIVERGES (2 L1 + 26 L2)** under the first production vigilia cast. The two L1s are mechanical fixes; the cross-spell convergence (`value.rs:451` `write_keyword_segment` vs `writer.rs:177` `write_keyword_body` — flagged by both solvere + intueri) is the strongest signal in the report. Foundation-first: these three pieces land together so subsequent naming/contract stones operate on L1-clean substrate.

## Pre-flight verified (orchestrator-grep'd 2026-05-21)

- `crates/wat-edn/docs/USER-GUIDE.md:159` — `match p.parse_next()? { ... }` confirmed (phantom)
- `crates/wat-edn/docs/IPC-BRIDGE.md:212` — `Parser::parse_next` confirmed (phantom)
- `crates/wat-edn/src/parser.rs:34` — `pub fn new(input: &'a str) -> Self` confirmed (real API)
- `crates/wat-edn/src/parser.rs:62` — `pub fn parse_all(mut self) -> Result<Vec<Value<'a>>>` confirmed (real API)
- `crates/wat-edn/src/lexer.rs:346-347` — double `chars().count() == 1` + `chars().next().unwrap()` confirmed
- `crates/wat-edn/src/value.rs:451` — `fn write_keyword_segment(seg: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result` confirmed
- `crates/wat-edn/src/writer.rs:177` — `fn write_keyword_body(seg: &str, out: &mut String)` confirmed
- `crates/wat-edn/src/escapes.rs` — exists; "Single source of truth for spec-level vocabulary shared between the lexer and the writer" — natural home for the shared helper
- Two functions are byte-for-byte the same algorithm (depth-counting `,` → `_` swap at bracket depth ≥ 1); only the sink type differs. `fmt::Formatter<'_>` and `String` both implement `fmt::Write` — natural unifier.

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope (sonnet)

### Part A — cernere L1 (doc fixes)

1. **`crates/wat-edn/docs/USER-GUIDE.md` line 159 area** — read the surrounding example (~15 lines context) to understand intent. Rewrite the phantom `p.parse_next()?` example to use the real API: `Parser::new(input).parse_all()?` returns `Vec<Value>`. Adjust the example body to iterate the vec (no more `match { None => break, Some(v) => ... }` loop; that's the phantom API's shape). Preserve teaching intent (whatever the example was illustrating about wire reading / multi-form parsing).

2. **`crates/wat-edn/docs/IPC-BRIDGE.md` line 212 area** — read surrounding context. Rewrite the prose claim about `Parser::parse_next` to accurately describe the real API surface (`Parser::new` / `Parser::new_wire` / `parse_top` / `parse_all`). If the original prose was illustrating "incremental parsing," note that the real API parses-all-at-once and any streaming behavior comes from upstream buffering.

### Part B — temperare L1 (lexer single-iterator pattern)

3. **`crates/wat-edn/src/lexer.rs:346-347`** — collapse:
   ```rust
   if body_str.chars().count() == 1 {
       return Ok(Token::Char(body_str.chars().next().unwrap()));
   ```
   to:
   ```rust
   let mut it = body_str.chars();
   if let Some(c) = it.next() {
       if it.next().is_none() {
           return Ok(Token::Char(c));
       }
   }
   ```
   (or equivalent — sonnet picks the cleanest spelling; one iterator construction; one traversal; semantically identical to the original two-walk version)

### Part C — Cross-spell convergence (extract `write_keyword_body_to`)

4. **Extract shared helper in `crates/wat-edn/src/escapes.rs`** (the spec-level vocabulary module — natural home per its existing rustdoc "Single source of truth for spec-level vocabulary shared between the lexer and the writer"):
   ```rust
   /// Write a keyword body segment with the position-aware `,` → `_`
   /// swap at bracket depth ≥ 1. See arc 170 REALIZATIONS-SLICE-1.md
   /// pass 14 for the original swap rationale; arc 218 stone 218.1 for
   /// the extraction.
   ///
   /// Walks chars once: `<` increments depth, `>` decrements, `,` at
   /// depth ≥ 1 emits `_`. Hot-path-friendly: no allocation, single
   /// pass over the segment bytes.
   pub fn write_keyword_body_to<W: std::fmt::Write>(seg: &str, w: &mut W) -> std::fmt::Result {
       // body unifying value.rs:451 + writer.rs:177
   }
   ```
   (Choose `pub(crate)` if cross-module is sufficient; `pub` if other crates need it. Sonnet picks based on call-site scope.)

5. **Collapse `crates/wat-edn/src/value.rs:451` `write_keyword_segment`** — delete the local fn; update callers at lines 440 + 443 to call `escapes::write_keyword_body_to(ns, f)` / `escapes::write_keyword_body_to(self.name(), f)`. The existing call sites already return `fmt::Result` — types align directly.

6. **Collapse `crates/wat-edn/src/writer.rs:177` `write_keyword_body`** — delete the local fn; update callers at lines 163 + 166 to call `escapes::write_keyword_body_to(ns, out)` / `escapes::write_keyword_body_to(k.name(), out)`. Writes to `String` via `fmt::Write` never fail — `.expect("String fmt::Write is infallible")` is honest; the existing call sites discard Result so this is the natural collapse. Add a one-line rune annotation if it helps the next reader (optional).

### Part D — Verification

7. **Run the wat-edn test suite — verify zero regressions:**
   ```
   cargo build --release
   cargo test --release -p wat-edn
   cargo clippy --release -p wat-edn -- -D warnings
   ```
   The `display_equivalence.rs` test in `crates/wat-edn/tests/` (per the docstring comment at `value.rs:436-437`) locks the two write paths to byte-identical output. It MUST still pass — that's the structural proof the extraction preserved semantics.

8. **Verify the L1 doc examples render correctly** — open each rewritten doc section and confirm the rewritten code example uses only API that exists. (No `cargo doc` needed; markdown only.)

### Part E — SCORE

9. **SCORE doc** at `docs/arc/2026/05/218-wat-edn-impeccable/SCORE-STONE-218.1.md` — scorecard matching EXPECTATIONS row count; deltas; verification summary; elapsed time. Calibration shape per `SCORE-STONE-216.7.md`.

## NOT your scope

- Stone 218.2 naming sweep (`escapes.rs` → `vocab.rs` rename, lexer var renames, placement fix, arc-provenance move) — separate stone; do NOT rename escapes.rs in this stone
- Stone 218.3 contract precision (pretty-print symmetry, `.expect()` runes, parse_map_key, closer-token diagnostics, allocation bounds, identifier suffix scan) — separate stone
- Stone 218.4 UUID strictness + USER-GUIDE map format claim — separate stone
- Stone 218.5 public-API runes + INSCRIPTION + re-cast vigilia — closure paperwork
- INSCRIPTION-218.md — Stone 218.5
- Other L2 findings from VIGILIA-REPORT-2026-05-21.md — addressed in later stones

## STOP triggers

- **STOP-1: `display_equivalence.rs` regresses** — if the extracted helper produces different output than either original, surface immediately. The two functions are claimed byte-identical; the extraction must preserve that.
- **STOP-2: any other wat-edn test regresses** — surface; do not paper over
- **STOP-3: `fmt::Write` unification doesn't work** — if there's a subtle reason `fmt::Formatter` and `String` can't share a `W: fmt::Write` bound (e.g., lifetime issue), surface for orchestrator before substituting an alternative shape
- **STOP-4: doc example intent unclear after reading surrounding context** — surface; do not invent a teaching example that wasn't there
- **STOP-5: 60 min elapsed**

## Verification (one per line)

```
cargo build --release
cargo test --release -p wat-edn
cargo clippy --release -p wat-edn -- -D warnings
```

## When you finish

Report: pass count out of EXPECTATIONS row count, deltas, verification summary, elapsed time, anything surfaced via STOPs. Cite the specific `display_equivalence.rs` test result.

Don't commit. Orchestrator commits after review.
