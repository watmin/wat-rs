# EXPECTATIONS — Arc 221 Stone 221.4b — Finish keyword→Symbol substrate-doctrine class

Mode A target: 9/9 PASS.

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
| 9 | New probe file + all suites green | `tests/wat_arc221b_keyword_dispatcher_completeness.rs` with 5+ probes (one per illegal site, plus Unit consistency if Option A). `cargo build --release -p wat` 0 errors; `cargo test --release --lib -p wat` PASS (baseline 827); `cargo test --release --test wat_arc220_char` 10/10; `wat_arc221_char_atomization` 3/3; `wat_arc221_keyword_nil_tag_atomization` 6/6; new probe file PASS; `cargo test -p wat-edn` PASS; clippy clean on wat-edn |

## Independent prediction (calibration record)

**Target runtime:** 60-90 min Mode A
**Upper bound:** 120 min
**Confidence:** medium-high

**Rationale:**
- Stone 221.4 was the closest precedent: 3 new value_to_atom arms + cascade + probes = ~55 min
- Stone 221.4b is 6 mechanical site fixes + Unit consistency audit + cascade test fixes + 5+ probes
- Expected cascade test count: 3-8 (rough estimate; eval-step! / quote / leaf are exercised in unit tests; substrate-as-teacher loop will reveal exact count)
- Risk: the runtime.rs:14018 / 20938 functions may have additional implicit assumptions about Keyword vs Symbol that surface only when downstream consumers exercise them

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

- **STOP-1:** dishonest "pre-existing" framing
- **STOP-2:** load-bearing probe fails
- **STOP-3:** 120 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** additional illegal sites beyond the 6 — surface to orchestrator
- **STOP-6:** Value::Unit consistency decision unclear from function contracts
