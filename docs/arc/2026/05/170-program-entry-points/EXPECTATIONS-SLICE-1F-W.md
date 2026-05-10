# Arc 170 slice 1f-W — EXPECTATIONS

## Independent prediction

**Predicted runtime: 60-90 minutes opus.**

Local substrate work in `crates/wat-edn/`:
- Lexer split — keyword body char rule conditional on bracket depth
- Writer swap — comma → underscore at depth ≥ 1
- Parser swap — underscore → comma at depth ≥ 1 (or
  post-lex normalize)
- New test file with round-trip + rejection cases
- Module rustdoc on writer + lexer

Comparable to:
- arc 092 (wat-edn v4 minting + roundtrip test) — similar
  scope; ~90 min
- arc 109 slice 1a (parser accepts FQDN primitive types) —
  similar lexer-rule extension; ~60 min

**Hard cap: 180 minutes** — wakeup scheduled.

## Baseline (post-slice-1f-i)

Slice 1f-W starts from commit `630f621` (slice 1f-i shipped).

Baseline cargo test (verified at slice 1f-W author time):
- **1306 passed / 855 failed** across 126 suites

Predicted post-1f-W workspace count:
- **1306-1320 passed / 855±5 failed** — purely additive
  substrate change; existing test fixtures don't use parametric
  type keywords with commas (those would already be broken
  under EDN spec), so the swap is a no-op for them
- New tests in `crates/wat-edn/tests/wire_encoding.rs` add
  ~10-15 passed cases
- Workspace fail count: ~unchanged (±5 band)

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — Lexer rejects `_` inside `<>` | source `:Vec<a_b>` triggers `InvalidKeyword` with span pointing at the `_` | ✓ |
| B — Lexer accepts `_` outside `<>` | `:rust::crossbeam_channel::Sender` lexes successfully | ✓ |
| C — Symbols unchanged | a symbol `foo_bar` (no leading `:`) still lexes (split is keyword-only) | ✓ |
| D — Writer swaps `,` → `_` inside `<>` | `:HashMap<K,V>` written as `:HashMap<K_V>` | ✓ |
| E — Writer doesn't swap outside `<>` | `:rust::crossbeam_channel::Sender<T>` written verbatim | ✓ |
| F — Parser swaps `_` → `,` inside `<>` | wire `:HashMap<K_V>` parses to keyword whose body is `HashMap<K,V>` | ✓ |
| G — Round-trip identity | `parse(write(k)) == k` for all test cases A through E | ✓ |
| H — Nested brackets | `:Vec<Map<K,V>>` ↔ `:Vec<Map<K_V>>` round-trips (depth ≥ 1 catches both inner and outer) | ✓ |
| I — Empty brackets | `:Foo<>` round-trips verbatim (no chars inside; no swap) | ✓ |
| J — Slice 1f-i still passes | `cargo test --release --test services_stdin` → 12/12 green | ✓ |
| K — Workspace fail-count delta within ±5 | `cargo test --release --workspace --no-fail-fast` fail count is 850-860 (post-1f-i was 855) | ✓ |
| L — New test file `crates/wat-edn/tests/wire_encoding.rs` | round-trip cases (A-I) + rejection case (a_b inside <>); all green | ✓ |
| M — Diagnostic NAMES the rule | the rejection error message references "wire-escape" / "comma" / "type-arg separator" — not just "invalid char" | ✓ |
| N — Module rustdoc | `crates/wat-edn/src/writer.rs` + `crates/wat-edn/src/lexer.rs` get a rustdoc block explaining the position-aware rule + cross-referencing REALIZATIONS pass 14 | ✓ |
| O — Honest deltas surfaced | per FM 5; no TODOs; no deferral language | ✓ |
| P — No new dependencies | `Cargo.toml` unchanged | ✓ |
| Q — Foundation + slice 1e + 1f-i files untouched | `git diff 630f621..HEAD` shows only `crates/wat-edn/*` + new test file changes | ✓ |
| R — Existing 18 underscore-in-keyword forms still parse | parsing `:rust::crossbeam_channel::Sender`, `:rust::sqlite::Db::execute_ddl`, `:wat__WatAST` etc. succeeds (workspace cargo test exercises these) | ✓ |

## Honest delta categories

Surface promptly; don't workaround:

- **Lexer architecture friction** — if `lex_keyword` doesn't
  cleanly carry bracket-depth context, surface for design
  discussion. Don't add a Mutex or RwLock to make depth-tracking
  work — use a local int in the existing loop.
- **Span granularity for the new error** — the diagnostic must
  point at the offending `_` position. If span machinery only
  supports keyword-level spans, surface.
- **Symbol vs keyword char-rule split** — `is_symbol_continue`
  may be called inline in many places. If splitting requires
  duplicating logic, propose a new `is_keyword_continue` helper.
- **EDN spec edge case** — verify the position-aware rule
  doesn't conflict with EDN's "comma is whitespace at the lexer
  level" rule. The swap happens INSIDE keyword body chars (after
  the lexer has already committed to "this is a keyword"), not at
  token boundaries. Surface if there's a tension.
- **Diagnostic quality** — error message must teach the rule.
  Generic "invalid char" fails Row M.
- **FM 5 trap** — TODOs verboten. Corner-case scope-bounding
  belongs in honest deltas, not in code comments.

## Calibration row

Filled at scoring time:

- Actual runtime: ___ minutes (Mode A clean / B partial / C failed)
- Workspace post-1f-W: ___ passed / ___ failed
- Fail-count delta from post-1f-i baseline: ___
- Whether delta lands in ±5 band: ___
- Honest deltas surfaced: ___
- Implementation choice (post-lex normalize vs inline swap): ___

## What's next (orchestrator-side, post-slice-1f-W)

When 1f-W ships:
1. Verify ship criteria locally
2. Author SCORE-SLICE-1F-W.md
3. Atomic commit slice 1f-W
4. Slice 1f-ii BRIEF + EXPECTATIONS authored — StdOutService
   applying the registration pattern from 1f-i + the wire
   encoding from 1f-W
5. Spawn slice 1f-ii

## Sonnet-delegation-protocol pre-flight (recovery doc § 7)

- [x] DESIGN.md current (passes 1-14)
- [x] BRIEF-SLICE-1F-W.md (this slice's BRIEF) authored +
      will-be-committed
- [x] EXPECTATIONS-SLICE-1F-W.md (this doc) authored +
      will-be-committed
- [x] Runtime band: 60-90 min predicted; 180 min hard cap
- [x] Substrate-grep citations in BRIEF point at exact files +
      line numbers
- [x] Verified each cited primitive exists (wat-edn writer +
      lexer + escapes + error)
- [x] No "STOP at first red" + impossible-task constraint —
      this slice is achievable as scoped
- [x] Baseline established: 1306 passed / 855 failed
- [ ] Will spawn with `model: "opus"` explicitly (substrate
      lexer + wire-encoding work; not mechanical)
- [ ] Will spawn with `run_in_background: true`
- [ ] Wakeup scheduled at 180 min (3 hours = 10800 s)

## SCORE artifact

Slice 1f-W is atomic; SCORE-SLICE-1F-W.md lands beside this.
