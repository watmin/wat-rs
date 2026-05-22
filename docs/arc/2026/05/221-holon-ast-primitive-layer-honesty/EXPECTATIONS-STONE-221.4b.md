# EXPECTATIONS — Arc 221 Stone 221.4b — Finish keyword→Symbol substrate-doctrine class

> **EXPANDED 2026-05-22 very-late** (mid-flight scope correction): the macro-support family in runtime.rs is the second half of the doctrine class. Original 9-row scorecard expanded to 13 rows.

Mode A target: 13/13 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `runtime.rs:13959` (watast_to_holon Keyword arm) | `HolonAST::symbol(k.as_str())` → `HolonAST::keyword(k.as_str())`; doc cites Stone 221.4b |
| 2 | `runtime.rs:14018` (Value→HolonAST second dispatcher) | Same pattern; same fix; nearby doc updated |
| 3 | `runtime.rs:20938` (`:wat::holon::leaf` Keyword arm) | Same pattern; same fix |
| 4 | `runtime.rs:21273` (eval-step! Terminal Keyword) | Same pattern; same fix |
| 5 | `runtime.rs:21322` (step-form converter sibling) | Same pattern; same fix |
| 6 | `edn_shim.rs:1899` (EDN keyword reader) | String construction drops leading colon; `HolonAST::Symbol` → `HolonAST::Keyword`; doc cites Stone 221.4b doctrine |
| 7 | Value::Unit consistency aligned across 3 dispatchers | Recommended Option A: add `Value::Unit => HolonAST::Nil` to runtime.rs:14018 and runtime.rs:20938; OR document honest reason for asymmetry in SCORE Delta |
| 8 | Cascade test fixes per Stone 221.3 Delta 1a discipline | Tests broken by this stone's substrate change are NOT pre-existing; frame honestly in SCORE; mechanical fixes mirror Stone 221.4's `lower_atom_keyword` + `lookup_returns_some_for_if` pattern |
| 9 | New probe file (Phase 1) — dispatcher completeness | `tests/wat_arc221b_keyword_dispatcher_completeness.rs` with 5+ probes (one per Phase 1 illegal site, plus Unit consistency); all PASS |
| 10 | Phase 2 — `eval_rename_callable_name` macro-support fix | `runtime.rs:11560` assertion flipped from Symbol to Keyword; `runtime.rs:11588` writer flipped to `HolonAST::keyword()`; error message updated; doc updated to cite Stone 221.4b |
| 11 | Phase 2 — `eval_extract_arg_names` audit + fix | Lines 11644/11719 (`->` Symbol check) confirmed honest (substrate-internal sentinel). Lines 11647/11653 (arg-name extract+write) audited; flipped to Keyword if context warrants OR documented honest reason for Symbol; STOP-6 catches ambiguous traces |
| 12 | Phase 2 — Audit of signature-of-defn / body-of / lookup-define | Each function audited for Symbol-vs-Keyword honesty post-Stone-221.4b; lying sites fixed; honest sites documented; doc comments at runtime.rs:10485/10490/10494 refreshed (`WatAST::Keyword → HolonAST::Keyword` reality, not stale `→ Symbol(":Foo")` text) |
| 13 | New probe file (Phase 2) + cascade tests + targeted-suite green | `tests/wat_arc221b_macro_support_keyword_shape.rs` with 3+ probes (rename-callable-name accepts Keyword + end-to-end define-alias). The 7 ex-failures (try_recv_on_ready_queue / walk_w2/w3 / values_sum / zip_empty/pairs / dissoc_removes) now PASS. `cargo test --release --test wat_arc143_manipulation` PASS. **--lib FULL SWEEP EXPLICITLY SKIPPED** per task #413 (pre-existing signal-test hangs). All targeted suites green |

## Independent prediction (calibration record)

**Target runtime:** 60-90 min Mode A (Phase 2 only — Phase 1 already on disk from first sonnet flight)
**Upper bound:** 120 min
**Confidence:** medium

**Rationale:**
- Phase 1 already done (~30 min consumed by first sonnet flight + panic-abort)
- Phase 2 scope: 2 substantial fixes (rename-callable-name + extract-arg-names) + audit of 3 more (signature-of-defn / body-of / lookup-define) + 3 doc refreshes + 7 known cascade test fixes + new probe file
- Pattern from prior stones: ~25-35 min per substantive substrate fix + cascade test
- Phase 2 estimate: 60-90 min — covers fixes + audit + tests + probe + SCORE
- Risk: macro-support family may have deeper interactions; audit-only sites (signature-of-defn etc) may surface MORE lying code; substrate-as-teacher loop may extend
- Mitigation: TARGETED verification (skip --lib; specific tests only) avoids the 9-min compile+hang cycle that fooled the first sonnet flight

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- holon-rs changes
- Stone 221.5 (Symbol/String seed)
- Stone 221.6 INSCRIPTION
- Arc 222/223 work
- Wat-edn wire format changes
- BOOK/USER-GUIDE
- Pre-existing wat-clippy backlog
- New HolonAST variants

## Honesty deltas accepted

- Value::Unit consistency choice — Option A (recommended) OR Option B with documented reason
- Probe phrasing for the deeper substrate paths (eval-step! Terminal, etc.) — sonnet picks honest entry point; STOP-2 catches probe failures
- Cascade test count varies — substrate-as-teacher cascade will surface the actual number
- New probe file name — `wat_arc221b_keyword_dispatcher_completeness` recommended; sonnet may pick alternative if more descriptive
- Test fixture rewrites that flip from `Symbol(":foo")` regression-test FOR old convention to `Keyword("foo")` regression-test AGAINST regression (like Stone 221.3's `keyword_distinct_from_symbol_at_type_level`) — encouraged when applicable

## Honesty deltas NOT accepted

- "Pre-existing failure" framing for any test broken by this stone's substrate change — STOP per Stone 221.3 Delta 1a discipline
- Skipping ANY of the 6 illegal-site fixes — STOP. The whole doctrine class must close in this stone or arc 221 cannot honestly inscribe.
- Skipping ANY of the 5+ load-bearing probes — STOP per STOP-2
- Touching holon-rs files — STOP per STOP-4
- Modifying canonical_edn_holon for Symbol/String — Stone 221.5's scope
- Inventing new HolonAST variants — settled at 16 per DESIGN
- Scope expansion beyond the 6 enumerated sites — STOP per STOP-5 and surface to orchestrator

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** dishonest "pre-existing" framing for tests broken by THIS stone
- **STOP-2:** load-bearing probe fails (especially rename-callable-name Keyword acceptance)
- **STOP-3:** 120 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** additional illegal sites beyond Phase 1 + Phase 2 surfaced — extend scope OR surface to orchestrator
- **STOP-6:** Value::Unit consistency decision unclear OR `eval_extract_arg_names` arg-name trace ambiguous (Keyword vs Symbol) — surface and ask, don't guess
- **STOP-7 (NEW — bash discipline):** if a `cargo` command appears to "hang" with no output for >30 seconds, do NOT panic and TaskStop. The full --lib sweep takes ~9 min wall-clock due to pre-existing signal-handler hangs (task #413). Use ONLY the targeted invocations specified in BRIEF section "9. Verification". DO NOT pipe to `| grep` / `| tail` (the pipe buffers everything until process exit, making it look like no output is appearing). DO NOT launch concurrent background cargo runs. ONE command at a time, foreground, vanilla.
