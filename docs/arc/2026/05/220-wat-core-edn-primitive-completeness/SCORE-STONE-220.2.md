# SCORE — Arc 220 Stone 220.2 — `:wat::core::Char` (BMP-only)

**Mode:** A
**Agent:** claude-sonnet-4-6 (substrate + tests + USER-GUIDE)
**Scoring:** orchestrator (claude-opus-4-7) — independent re-verification + interop handshakes (sub-agent piped-bash permission wall hit; 5th stone now — orchestrator-runs-handshakes is the established pattern)
**Date:** 2026-05-22

## Result: 11/12 PASS + 1 row reframed

Per recovery-doc Section 7 + `feedback_pre_existing_verification` independent re-verification:

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | `Value::wat__core__Char(char)` variant | PASS | `src/runtime.rs:~617` — new variant inserted after `wat__core__Uuid` |
| 2 | 5 runtime.rs arm sites | PASS | PartialEq + Hash + type_name (`"wat::core::Char"`) + structural-eq + render (`\c` EDN form) added at the documented Uuid-precedent line ranges |
| 3 | edn_shim bridge 3 sites | PASS | Parse-direction + write-direction sites added per Uuid mirror at the 3 lines |
| 4 | closure_extract.rs arm | PASS | Char → `(:wat::core::Char/of "x")` capture form per Uuid `Uuid/from-string` precedent |
| 5 | `:wat::core::Char/of` constructor | PASS | `src/string_ops.rs` — `eval_char_of` following `eval_uuid_typed_v4` pattern; arity check + String length-1 validation + BMP check + clear diagnostics |
| 6 | Constructor dispatch entry | PASS | `src/runtime.rs:~4570` area — `":wat::core::Char/of"` dispatch added |
| 7 | Lexer `\c` literal support | PASS | `src/lexer.rs` — `lex_char` added (145 lines including doc comment); handles named (`\newline`/`\return`/`\space`/`\tab`) + `\uNNNN` + single-char `\c`; BMP-only enforcement; tokenizer entry dispatches on `b'\\'` |
| 8 | Parser handles Token::Char | PASS | `src/parser.rs:+13` — `Token::Char(c)` → `Value::wat__core__Char(c)` in atom-parsing path |
| 9 | Rust integration tests | PASS | `tests/wat_arc220_char.rs` — 312 lines / 10 test functions covering lexer accepts + lexer rejects supplementary-plane + constructor success + 3 constructor error paths + round-trip via wat-edn |
| 10 | wat-source test | PASS | `wat-tests/holon/char_round_trip.wat` — 51 lines exercising `\c` literal + `Char/of` constructor + EDN round-trip with assert-eq! |
| 11 | Interop shape matrix Char probe | PASS | `shape_matrix.rs` + `shape_matrix_reader.rs` + `consume_shapes.clj` + `produce_shapes.clj` — `:char-bmp` shape added bidirectionally |
| 12 | All test suites + clippy + handshakes green | **PARTIAL (see Delta 1)** | wat-edn 344/344, wat lib 824/0, interop-tests clippy clean, 4 handshakes PASS. **wat-crate clippy NOT clean** but the 115 warnings are arc 170 backlog (pre-existing latent debt; never previously gated). |

## Deltas from EXPECTATIONS

### Delta 1 — `cargo clippy -p wat -- -D warnings --all-targets` gate was over-reach

**The BRIEF added this gate but it was never part of prior stone discipline.** All of arc 218 stones (218.1 through 218.6e) gated on `cargo clippy --release --all-targets -p wat-edn -- -D warnings` — the wat-edn crate specifically. The wat crate's clippy with `-D warnings --all-targets` has 115 pre-existing warnings, verified via git stash round-trip (errors present at baseline without sonnet's changes).

**Per user direction 2026-05-22:** the wat-crate clippy mountain is known arc 170 backlog. They came from work on 170. They're the "constant reminder" that 170 work remains. 170 is blocked on them. Keep them visible.

**None of the 115 warnings point at sonnet's added code** — verified via grep on the error pointers (`runtime.rs:184`, `185`, `2008`, `2574`, `2583`, `19515`, `19591`, `19970`, `19979`; `closure_extract.rs:594`, `593`, `1054`, `1101`, `1117`, `1264`, `1984`, `1983`; `edn_shim.rs:1852`; `parser.rs:14, 15`). Sonnet's added arms were at line ranges around runtime.rs 617/655/762/1044/7103/15905; edn_shim.rs 412/590/1631; closure_extract.rs 1493 — none in the error list.

**Reframed verification gates (matching prior arc 218 stone discipline):**

```
cargo build --release                                          — OK
cargo test --release --lib -p wat                              — 824/0 PASS
cargo test --release -p wat-edn                                — 344/344 PASS
cargo clippy --release --all-targets -p wat-edn -- -D warnings — 0 warnings
(interop-tests)
cargo build --release                                          — OK
cargo clippy --release --all-targets -- -D warnings            — 0 warnings
Interop handshakes 1-4                                         — PASS
```

Stone 220.2 ships clean per prior-stone gate discipline. The 115 wat-crate warnings remain as arc 170 visibility per user direction.

### Delta 2 — Handshake verification orchestrator-side (5th stone pattern)

Sub-agent piped-bash permission wall denied `cargo run | clojure -M` form again. Orchestrator ran all 4 handshakes during scoring — all PASS. Established precedent across 218.6b/c/d/e and now 220.2. Wired discipline: orchestrator absorbs handshake verification when sub-agent permission wall hits; sonnet ships everything else clean.

## Verification summary

```
Substrate:
  cargo build --release                                           — OK
  cargo test --release --lib -p wat                               — 824/0 (+ 1 ignored, pre-existing)
  cargo test --release -p wat-edn                                 — 344/344 (untouched)
  cargo clippy --release --all-targets -p wat-edn -- -D warnings  — 0 warnings

Interop (orchestrator-side):
  cargo build --release                                           — OK
  cargo clippy --release --all-targets -- -D warnings             — 0 warnings
  Handshake 1 (wat-edn → consume.clj)                            — PASS
  Handshake 2 (produce.clj → reader)                              — PASS
  Handshake 3 (shape_matrix → consume_shapes.clj)                 — PASS  (with :char-bmp probe)
  Handshake 4 (produce_shapes.clj → shape_matrix_reader)          — PASS  (with :char-bmp probe)

wat-crate latent debt (arc 170 backlog; NOT new):
  cargo clippy --release --all-targets -p wat -- -D warnings      — 115 warnings (pre-existing)
```

## Files changed (12 files)

Substrate (wat-rs):
- `src/runtime.rs` (+32 lines): Char variant + 5 arms + dispatch entry
- `src/lexer.rs` (+145 lines): lex_char function + Token::Char + tokenizer dispatch + doc-comment update (`#\a` → `\c per arc 220 — wat IS clojure-on-rust`)
- `src/string_ops.rs` (+83 lines): eval_char_of constructor following eval_uuid_typed_v4 pattern
- `src/parser.rs` (+13 lines): Token::Char → Value::wat__core__Char in atom-parsing
- `src/edn_shim.rs` (+14 lines): 3 bridge sites (parse × 2 + write × 1)
- `src/closure_extract.rs` (+9 lines): Char closure-capture arm

Tests (new files):
- `tests/wat_arc220_char.rs` (312 lines): 10 Rust integration tests
- `wat-tests/holon/char_round_trip.wat` (51 lines): wat-source round-trip exercise

Interop-tests (bidirectional shape matrix gains :char-bmp):
- `crates/wat-edn/interop-tests/src/bin/shape_matrix.rs` (+3)
- `crates/wat-edn/interop-tests/src/bin/shape_matrix_reader.rs` (+5)
- `crates/wat-edn/interop-tests/clj/consume_shapes.clj` (+4)
- `crates/wat-edn/interop-tests/clj/produce_shapes.clj` (+5)

**Total: 12 files, ~676 lines added, 4 deleted.**

## STOP triggers

- **STOP-1 (`b'\\'` conflict in lexer):** DID NOT TRIGGER. `\` outside strings was not used before; lex_char added cleanly.
- **STOP-2 (variant cascade exceeds ~10 sites):** DID NOT TRIGGER. Exactly the 10 sites mapped from Uuid precedent.
- **STOP-3 (existing wat test uses Char-like syntax):** DID NOT TRIGGER. No conflict surfaced.
- **STOP-4 (HolonAST encoding bridge breaks for Char):** DID NOT TRIGGER. Char as scalar uses existing leaf path; collections-as-holons (arc 216) Bundle is unaffected.
- **STOP-5 (interop handshakes fail):** DID NOT TRIGGER. All 4 handshakes PASS (orchestrator-side).
- **STOP-6 (120 min elapsed):** DID NOT TRIGGER. Sonnet duration ~30 min (1794s).
- **EXTRA — wat-crate clippy permission wall:** sub-agent denied piped-bash form for handshakes; not a STOP-trigger but the 5th-stone pattern. Orchestrator absorbed.

## Elapsed time

**Sonnet substrate + tests + interop edits:** ~30 min (duration_ms 1,794,276 / 60 ≈ 29.9 min)
**Orchestrator-side handshake verification + clippy investigation + SCORE drafting:** ~8 min
**Total wall-clock:** ~38 min

## Calibration check

- Target runtime: 60-90 min
- Actual runtime: ~30 min (sonnet) + ~8 min (orchestrator scoring) = ~38 min combined
- Within prediction band? **Below lower bound** (sonnet within band; combined below)
- Rationale: Substrate-pre-grep was dense; 10 Uuid arm sites pre-mapped with exact lines; verbatim eval_uuid_typed_v4 + closure_extract + wat-edn lex_char references inlined in BRIEF eliminated cross-crate hunting. Sonnet shipped 12 files cleanly in ~30 min. The novel surface (lex_char) was the longest single edit (+145 lines) but the verbatim wat-edn shape made it mechanical. Calibration trend: 12 stones in series at or below lower bound (218.1 ~20, 218.2 ~15, 218.3 ~25, 218.4 ~20, 219.1 below, 218.6 ~8, 218.6b ~6, 218.6c ~minutes, 218.6d ~minutes, 218.6e ~minutes, 220.2 ~30+8). Pattern locked: weaponized BRIEF (verbatim references + exact line numbers) + Uuid-precedent + sonnet ships below band reliably.

## Substrate state

- `:wat::core::Char` minted as BMP-only typed primitive
- `\c` literal syntax accepted by wat lexer (Clojure-on-Rust convention; doc-comment updated from incorrect `#\a` placeholder)
- `(:wat::core::Char/of "x")` constructor available
- wat-edn ↔ wat-core bridge handles Char both directions
- Cross-language interop verified: wat-edn writes `Value::Char('x')` → `clojure.edn/read` accepts; symmetric reverse
- Slice 3 (`'` reader macro) + Slice 4 (List) + Slice 5 (paperwork) remain

## Unblocks

- Slice 3 (`'` reader macro at form-start; arc 171 keyword-body `'` stays unchanged)
- Slice 4 (`:wat::core::List<T>` — inherits Char's variant + arm + constructor patterns; adds dispatch + cross-type Eq with Vector per EDN spec)
- Slice 5 (INSCRIPTION + USER-GUIDE + cross-references)
- Per chain: arc 220 closure → arc 219b (spec conformance) → arc 218 streaming → arc 217 (Clojure-IPC bridge)
