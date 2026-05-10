# Arc 171 — Comma → apostrophe inside keyword/symbol bodies

**Status:** opened 2026-05-10 per arc 170 REALIZATIONS pass 14
decision; scope broadened by user 2026-05-10 (this revision).

**Pass 14 framed narrowly** (arity suffix `:foo,2` only).
**User broadened the scope on the spot** to cover ALL
comma-in-keyword/symbol usage, per the principle "symbols may
not contain commas, however they can use underscores" (pass 14
quote) and the explicit table the user locked in:

```
:wat::core::op
:wat::core::op'2
:wat::core::op'i64'i64
:wat::core::op'i64'f64
:wat::core::op'f64'i64
:wat::core::op'f64'f64
```

**`'` (apostrophe) is the universal separator inside
keyword/symbol bodies.** Arity-suffix, type-discriminator,
inter-arg separator — all collapse to apostrophe. No commas
remain in keyword/symbol bodies after this arc ships.

Sibling-coordinated with arc 172 (Scheme → Clojure macros).
Both arcs together complete the lexical EDN-compliance pivot
opened by slice 1f-W (commas inside `<>` already retired in
favor of wire-encoding swap to underscore).

See memory: `feedback_apostrophe_dispatch_separator.md` —
the user's explicit lock-in.

## Motivation

EDN compliance. The substrate currently allows commas inside
keyword bodies (`lex_keyword` accepts `,` as keyword-body
character); commas-as-Unquote-token apply only OUTSIDE keyword
bodies. EDN treats commas as whitespace everywhere — so
keyword bodies containing commas don't round-trip through EDN
parsers without lossy re-interpretation.

After this arc: zero commas in any keyword or symbol; apostrophe
fills every position commas previously held. wat source becomes
EDN-compliant at the lexical level (combined with arc 172's
comma-as-whitespace + ~/~@ unquote, the substrate is fully EDN).

## Scope — sized at author time

**167 sites across 14+ files** per `grep -rE ":[a-zA-Z][a-zA-Z_:\*\+\-/]*,[0-9a-zA-Z\-]+"`:

- `wat/core.wat` — define-dispatch entries for arithmetic + comparison family
- `wat-tests/service-template.wat`
- `wat-scripts/` (ping-pong, seed-fixture)
- `examples/interrogate/wat/main.wat`
- `crates/wat-telemetry/wat/*` + `wat-tests/*`
- `crates/wat-telemetry-sqlite/wat/*` + `wat-tests/*`
- `crates/wat-holon-lru/wat/holon/lru/HologramCache.wat`
- `src/lexer.rs` / `src/types.rs` / `src/runtime.rs` / `src/check.rs` (Rust string literals for diagnostics + matchers)
- `crates/wat-edn/tests/wire_encoding.rs`
- `docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/*.wat` (archived; verify scope at sweep time)

### Substrate edits

**Lexer change** (`src/lexer.rs`):
- `lex_keyword` body: accept `'` (apostrophe) as keyword-body
  character. Same position-aware treatment commas currently
  have (no-op outside keyword bodies; passthrough inside).
- During transition (this arc): KEEP `,` accepted as
  keyword-body character. Both work concurrently.
- Arc closure: retire `,` acceptance inside keyword bodies
  with a clean diagnostic naming the apostrophe-canonical
  shape.

**Parser change** (`src/parser.rs`): none — apostrophe inside a
keyword body is just lexical; the parser sees the keyword as a
whole.

### Consumer sweep — 167 sites

Mechanical pattern: every `:<verb>,<suffix>` becomes
`:<verb>'<suffix>`. For multi-arg discriminators (e.g.,
`:wat::core::+,i64-f64`), the `-` between type names also
becomes `'`: `:wat::core::+'i64'f64`. Per user's locked table.

The 167 sites split into:
- ~60-80 wat source files (`wat/`, `wat-tests/`,
  `crates/*/wat/`, `crates/*/wat-tests/`, `examples/*/wat/`,
  `wat-scripts/`)
- ~10 Rust files (string literals in diagnostics + matchers)
- ~14 archived arc docs (verify whether they need migration —
  archived docs are historical record per "what is inscribed
  is inscribed"; likely skip)

### What does NOT change

- Commas as whitespace OUTSIDE keyword bodies — pure
  whitespace per EDN spec (no change needed; arc 172 closes
  the comma-as-Unquote-token gap)
- Slice 1f-W's wire encoding (commas inside `<>` swap to
  underscore on the wire) — independent rule; unchanged
- Symbol/keyword content other than commas — apostrophe is
  added as accepted character; underscores stay allowed; dashes
  stay allowed; etc.

## Slicing plan

Three slices. Total predicted: 2-4 hours mixed sonnet.

### Slice 1 — Lexer accepts apostrophe inside keyword body

- `lex_keyword` accepts `'` as keyword-body character; same
  position-aware treatment as commas
- Tests: verify `:wat::core::op'2` parses as a single keyword;
  verify `:wat::core::op'i64'i64` parses (multi-apostrophe);
  existing `,N` tests continue to pass (transition mode)
- Workspace cargo test unchanged (purely additive)

Sonnet. Predicted: 30-45 min.

### Slice 2 — Consumer sweep across all 167 sites

- Mechanical sed/awk-style migration: `,<suffix>` → `'<suffix>`
  in keyword bodies; multi-suffix entries split each `-` between
  type names also to `'` per user's table
- All ~80 source files: wat + Rust diagnostic strings
- Cargo test workspace must stay green throughout (or surface
  any test that depends on the specific keyword spelling)
- Archived arc docs (`docs/arc/2026/05/130-*/complected*`):
  surface as honest delta; archive may NOT need migration
  per "what is inscribed is inscribed"

Sonnet. Predicted: 60-120 min.

### Slice 3 — Closure: retire comma in keyword body

- `lex_keyword`: reject `,` inside keyword body with a clean
  diagnostic naming `'` as the canonical shape and the
  migration arc (this arc, 171)
- Tests: assert the rejection diagnostic exists; assert all
  current consumers use `'`
- Memory entry update + INSCRIPTION + USER-GUIDE +
  058 changelog row + amend
  `feedback_apostrophe_dispatch_separator.md` with shipped
  status

Sonnet. Predicted: 30-45 min.

## Dependencies

- Arc 170 slice 1f-W shipped (`4278c4d`) — wire encoding rule
  for `<>` already locked; this arc doesn't change it
- Independent of arc 172. Arc 171 ships FIRST (retires comma
  inside keyword bodies) so arc 172's lexer change (comma →
  whitespace OUTSIDE keyword bodies) doesn't collide with
  any remaining comma-in-keyword sites

## Risks

- **Apostrophe collision with Lisp/Clojure quote** — `'foo`
  (start of token; quote shorthand) vs `:foo'bar` (apostrophe
  inside keyword body; just a separator). Position-aware lexing
  already distinguishes by `:` prefix. The lex_keyword routine
  handles this naturally — commas have the same position-aware
  status today.
- **Sweep miscount** — the 167 estimate is from one grep
  pattern. Sonnet may surface additional sites. Surface as
  honest delta; orchestrator decides scope.
- **Archived arc docs** — `docs/arc/2026/05/130-*/complected*`
  has 14 comma sites. Per "what is inscribed is inscribed,"
  archived docs likely stay as historical record. Sonnet
  surfaces; orchestrator confirms skip.

## Ship criteria (whole-arc)

- `lex_keyword` accepts `'` inside keyword body; rejects `,`
  inside keyword body (after slice 3)
- All 167 source-tree sites swept (archived docs likely skipped)
- Workspace cargo test green; no regressions
- Memory amended with shipped status
- INSCRIPTION per FM 11 (no "deferred to future" language)

## Cross-references

- Memory: `feedback_apostrophe_dispatch_separator.md` (the
  user's locked convention)
- Arc 170 REALIZATIONS pass 14 (the decision that opened this
  arc; framed arity-only; user broadened on the spot 2026-05-10)
- Arc 170 slice 1f-W (`4278c4d`) — sibling EDN-compliance work
  for commas inside `<>`
- Arc 172 (sibling — Scheme → Clojure macros); arc 171 ships
  BEFORE arc 172 to clear keyword-body commas
- Arc 146 — dispatch mechanism this arc fine-tunes
