# Arc 170 slice 1f-W — BRIEF

**Substrate; opus.** Wire encoding lexical doctrine —
prerequisite for arc 170's transmission slices (1f-ii / 1f-iii /
1f-iv). Per REALIZATIONS pass 14 (locked-in 2026-05-10):
position-aware rule. Inside `<...>` substrings within a keyword
body, `_` is forbidden in source and used as the wire-escape
for `,`. Outside `<...>`, `_` and `,` have no special meaning;
chars pass verbatim both directions.

**Reference docs (read first):**
- [`REALIZATIONS-SLICE-1.md`](./REALIZATIONS-SLICE-1.md) pass 14
  — full lock-in conversation; four-questions analysis
- [`BUILD-PLAN.md`](./BUILD-PLAN.md) §3 slice 1f-W — scope, ship
  criteria
- `crates/wat-edn/src/writer.rs` — `write_keyword` at line 144
  is the writer side
- `crates/wat-edn/src/lexer.rs:285-322` — `lex_keyword` is the
  parser side; uses shared `is_symbol_continue` from
  `escapes.rs:101`
- `crates/wat-edn/src/escapes.rs:91-101` — `is_symbol_start` /
  `is_symbol_continue` (current shared char rule)
- Memory `project_pipe_protocol.md` — line-delimited EDN
  protocol; one protocol, four transports
- Memory `project_wat_rust_interop.md` — `:rust::*` Rust-mirror
  convention preserved by position-aware rule

**Branch:** `arc-170-program-entry-points` (slice 1f-i shipped
at `630f621` is your starting point).

**Constraint:** STOP if any substrate primitive this BRIEF
references doesn't exist or doesn't behave as cited — DON'T
workaround. Surface as honest delta.

## The rule

**Source-position rule (lexer enforces):**
- Outside `<...>`: keyword body chars match `is_symbol_continue`
  (current rule); `_` allowed
- Inside `<...>` (depth ≥ 1): keyword body chars match
  `is_symbol_continue` MINUS `_`; `_` is rejected with a
  diagnostic that names the rule + cites slice 1f-W

**Wire-encoding rule (writer + parser implement the swap):**
- Writer: when serializing a keyword body, swap `,` → `_` at
  depth ≥ 1 (inside `<...>`); outside, chars pass verbatim
- Parser: when lexing a keyword body, swap `_` → `,` at depth
  ≥ 1; outside, chars pass verbatim

**Round-trip property:**
- `:wat::core::HashMap<wat::core::String,wat::core::i64>`
  → wire `:wat::core::HashMap<wat::core::String_wat::core::i64>`
  → parsed back to original source form (keyword equality)
- `:rust::crossbeam_channel::Sender<T>` → wire identical
  (no comma to swap; no underscore inside `<>`) → parsed
  identical
- `:rust::sqlite::Db::execute_ddl` → wire identical → parsed
  identical (no `<>` at all)

## Scope

### 1. Lexer split (in `crates/wat-edn/src/lexer.rs::lex_keyword`)

Track bracket depth while lexing the keyword body:
- Increment on `<`
- Decrement on `>`
- When depth ≥ 1 and char is `_`, return `InvalidKeyword(
  "underscore in keyword body inside <...> is reserved for
  wire-escape of comma; use ',' as type-arg separator in
  source")` with span pointing at the offending `_`

Symbols (`lex_symbol`) UNCHANGED — symbols allow `_` per pass-14
("symbols may not contain commas, however they can use
underscores"). The lexer split applies ONLY to keywords.

### 2. Wire encoding writer (in `crates/wat-edn/src/writer.rs::write_keyword`)

Walk the keyword body chars; track bracket depth. When depth ≥
1 and char is `,`, emit `_`. Otherwise emit char verbatim.

### 3. Wire encoding parser (in `crates/wat-edn/src/lexer.rs` or normalize after lex)

Mirror of the writer:
- After lexing the keyword body, walk chars; track depth.
- When depth ≥ 1 and char is `_`, normalize to `,`.
- Outside, chars pass verbatim.

The lexer ALREADY rejects `_` inside `<>` (per item 1). The
wire-decode swap turns `_` (which would be rejected by item 1)
back into `,` BEFORE the rejection check fires. Effectively,
the lexer sees the canonicalized source form.

**Implementation note:** the swap can happen as a
post-lex normalization (cleaner — single canonicalization step)
OR inline in `lex_keyword` (faster — no extra string allocation).
Either is acceptable; pick whichever is cleaner. Document the
choice in SCORE.

### 4. Tests in `crates/wat-edn/tests/wire_encoding.rs` (new)

Round-trip cases:
- Basic: `:foo` → wire `:foo` → parsed `:foo`
- Parametric one-arg: `:Vec<i64>` → wire `:Vec<i64>` → parsed `:Vec<i64>`
- Parametric two-arg: `:HashMap<K,V>` → wire `:HashMap<K_V>` → parsed `:HashMap<K,V>`
- Nested: `:Vec<Map<K,V>>` → wire `:Vec<Map<K_V>>` → parsed `:Vec<Map<K,V>>`
- Rust-mirror: `:rust::crossbeam_channel::Sender<T>` → wire
  identical → parsed identical (underscore preserved outside
  brackets; bracket interior has no swap-able chars)
- Underscore-outside-brackets: `:wat__internal::foo` → wire
  identical → parsed identical (current convention preserved)
- Comma-outside-brackets is illegal in source (no test needed;
  EDN spec already rejects this)

Rejection cases:
- Source `_` inside `<>`: `:Vec<a_b>` → lexer error with
  diagnostic naming the rule
- Span check: error points at the offending `_` position

### 5. Verify slice 1f-i still parses

After landing slice 1f-W's parser swap, re-run
`cargo test --release --test services_stdin`. Expect 12/12
green (slice 1f-i tests don't use parametric type keywords;
the swap is a no-op for their fixtures).

## Constraints

- **Don't write a workaround.** If the existing wat-edn lexer
  structure makes depth-tracking awkward (e.g.,
  `is_symbol_continue` is called via a path that doesn't carry
  context), surface the substrate gap; don't paper over.
- **Don't modify symbol lexing.** Symbols KEEP underscore
  allowance. Pass-14: keywords change; symbols stay.
- **Don't sweep keywords-with-underscores.** Per pass-14
  position-aware rule, the 18 existing underscore-in-keyword
  forms are ALL outside `<>` and remain valid. Zero rename.
- **Don't update `:rust::` namespace conventions.** Memory
  `project_wat_rust_interop.md` doctrine preserved verbatim.
- **Don't touch transmission slices (1f-ii / 1f-iii / 1f-iv)** —
  they author after slice 1f-W ships.
- **Don't update USER-GUIDE / INSCRIPTION** — slice 5 paperwork.
- **No new dependencies.** Cargo.toml unchanged.
- **No TODOs in source.** FM 5.

## Substrate-grep citations

Every primitive verified to exist:

- `crates/wat-edn/src/writer.rs:144` — `fn write_keyword(k: &Keyword, out: &mut String)`
- `crates/wat-edn/src/lexer.rs:292` — `fn lex_keyword(&mut self) -> Result<Token<'a>>`
- `crates/wat-edn/src/escapes.rs:91-101` — `is_symbol_start` /
  `is_symbol_continue` (current shared rule)
- `crates/wat-edn/src/error.rs` — `ErrorKind::InvalidKeyword`
  variant exists (used by current `lex_keyword`)
- `:rust::crossbeam_channel::Sender<T>` — verified in source
  via grep (see SCORE-SLICE-1F-I context for actual usages)

Any deviation: STOP, report, don't guess.

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A — Lexer rejects `_` inside `<>` | source `:Vec<a_b>` → `InvalidKeyword` error with span on `_` | ✓ |
| B — Lexer accepts `_` outside `<>` | `:rust::crossbeam_channel::Sender` parses | ✓ |
| C — Symbols unchanged | symbol `foo_bar` still parses (lexer split is keyword-only) | ✓ |
| D — Writer swaps `,` → `_` inside `<>` | `:HashMap<K,V>` writes as `:HashMap<K_V>` | ✓ |
| E — Writer doesn't swap outside `<>` | `:rust::crossbeam_channel::Sender` writes verbatim | ✓ |
| F — Parser swaps `_` → `,` inside `<>` | wire `:HashMap<K_V>` parses to source-equivalent `:HashMap<K,V>` | ✓ |
| G — Round-trip identity | `parse(write(k)) == k` for all the test cases above | ✓ |
| H — Nested brackets | `:Vec<Map<K,V>>` ↔ `:Vec<Map<K_V>>` round-trips (depth ≥ 1 covers both inner and outer) | ✓ |
| I — Slice 1f-i still passes | `cargo test --release --test services_stdin` → 12/12 green | ✓ |
| J — Workspace fail-count delta ~0 | `cargo test --release --workspace --no-fail-fast` fail count is 855±5 (post-slice-1f-i baseline) | ✓ |
| K — Tests in `crates/wat-edn/tests/wire_encoding.rs` | new test file with round-trip + rejection cases; all green | ✓ |
| L — No new dependencies | `Cargo.toml` unchanged | ✓ |
| M — Honest deltas surfaced | per FM 5; no TODOs; no deferral language | ✓ |
| N — Existing 18 underscore-in-keyword forms still parse | spot-check via re-running existing test suites that touch them (workspace cargo test covers this) | ✓ |
| O — Foundation + slice 1e + 1f-i files untouched | git diff `630f621..HEAD` shows only `crates/wat-edn/*` + new test file edits | ✓ |
| P — Documented in module rustdoc | `crates/wat-edn/src/writer.rs` and `crates/wat-edn/src/lexer.rs` get rustdoc explaining the position-aware rule + cross-ref to REALIZATIONS pass 14 | ✓ |

## Honest delta categories

Surface; don't workaround:

- **Lexer architecture** — if the current `lex_keyword`
  structure doesn't carry bracket-depth context naturally
  (e.g., the depth counter is a state-machine concern that
  conflicts with the "single-pass char accumulation" pattern),
  surface for design discussion. The depth tracking should be
  a single int incremented/decremented in the existing loop.
- **Span construction for the new error** — if the span machinery
  doesn't have a "char position within keyword body" granularity,
  surface; the error needs to point at the specific `_`.
- **Symbol vs keyword char-rule split** — if `is_symbol_continue`
  is called inline in many places, the split might require a
  new `is_keyword_continue` helper. Surface.
- **EDN spec compliance** — verify the position-aware rule
  doesn't violate underlying EDN spec (commas are whitespace at
  the lexer level; the swap happens INSIDE keyword body chars,
  not at token boundaries). Surface if there's a conflict.
- **Sloppy rejection diagnostic** — the error must NAME the
  rule and cite the rationale. Don't ship a generic "invalid
  char" error; users need to learn the rule from the diagnostic.
- **FM 5 trap** — TODOs verboten.

## Predicted runtime

60-90 min opus. The lexer split + writer/parser swap is local
to `crates/wat-edn/`; new tests are mechanical. Hard cap: 180
min.

## What's next (orchestrator-side, post-slice-1f-W)

When 1f-W ships:
1. Score per EXPECTATIONS-SLICE-1F-W.md
2. Author SCORE-SLICE-1F-W.md
3. Atomic commit slice 1f-W
4. Author BRIEF + EXPECTATIONS for slice 1f-ii (StdOutService)
   — applies the registration pattern from 1f-i + uses the wire
   encoding from 1f-W
5. Spawn slice 1f-ii
