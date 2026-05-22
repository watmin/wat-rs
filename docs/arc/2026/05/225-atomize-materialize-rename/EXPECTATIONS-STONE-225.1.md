# EXPECTATIONS — Arc 225 Stone 225.1 — Substrate rename + wat-side caller sweep

Mode A target: 12/12 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | Rust fn `eval_algebra_atom` → `eval_holon_atomize` | `src/runtime.rs:13820` renamed; dispatch table entry updated; doc comment refresh |
| 2 | Rust fn `value_to_atom` → `atomize_value` | `src/runtime.rs:13838` renamed; all callers in src/ updated; doc comment refresh |
| 3 | Rust fn `eval_atom_value` → `eval_holon_materialize` | `src/runtime.rs:13633` renamed; dispatch table entry updated |
| 4 | Rust fn `holon_item_to_value` → `materialize_holon_item` + op param | `src/runtime.rs:13504` renamed; `op: &str` parameter threaded through (closes arc 224 L1-runtime-3 latent lie); all callers updated to pass their own op name |
| 5 | Dispatch table verb entry rename | `":wat::holon::Atom"` → `":wat::holon::atomize"`; `":wat::core::atom-value"` → `":wat::holon::materialize"` (namespace move) |
| 6 | TypeScheme registrations | `src/check.rs:13558` → `":wat::holon::atomize"`; `src/check.rs:13591` → `":wat::holon::materialize"` |
| 7 | Special-case handlers in `infer_list` | `src/check.rs:5326` updated to match `":wat::holon::atomize" \| ":wat::holon::leaf"`; `src/check.rs:5362` matches `":wat::holon::materialize"` |
| 8 | Substrate-as-teacher cascade — Rust callers | All ~31 `:wat::holon::Atom` literal + ~10 `:wat::core::atom-value` literal call sites in src/ + tests/ updated to new verb names; cargo build green |
| 9 | Substrate-as-teacher cascade — wat-side callers | All ~54 caller sites in `wat/**/*.wat` + `wat-tests/**/*.wat` updated; substrate startup loads cleanly |
| 10 | Adjacent doc comments refreshed | Doc comments naming the retired verbs in touched files updated as discovered (no global hunt — fix what you touch) |
| 11 | All test suites green | `cargo build --release -p wat` 0 errors; `cargo test --release --lib -p wat [--skip 5 signal tests]` PASS; integration tests for arcs 220/221/221b/143 PASS; `cargo test -p wat-edn` PASS; `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0 warnings |
| 12 | Holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` empty |

## Independent prediction (calibration record)

**Target runtime:** 90-180 min Mode A
**Upper bound:** 240 min
**Confidence:** medium

**Rationale:**
- Stone 221.4b Phase 1+2 closest precedent: ~100 min for dispatcher rename + cascade
- Arc 159 substrate-wide sweep precedent: ~951 sites (this is ~95-110 sites — ~1/10th of that)
- Mechanical sweep; substrate-as-teacher cascade is well-understood pattern
- Risk: namespace move (`:wat::core` → `:wat::holon` for atom-value) may surface check.rs special-case handlers I haven't grep'd; trust the cascade
- Risk: doc comments naming the retired verbs may number more than expected; budget for as-discovered cleanup not global hunt

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- holon-rs changes (algebra primitive stays)
- wat-edn changes
- Arc 224 Group A fixes (Stone 224.5's scope)
- L2 mumbles
- USER-GUIDE / BOOK / 058 spec (Stone 225.3)
- INSCRIPTION (Stone 225.4)
- Deprecation aliases (HARD CUT)
- Other substrate verbs showing similar polymorphic-dispatch patterns (STOP-5 catches — escalate to orchestrator)

## Honesty deltas accepted

- Rust function rename targets may vary slightly if sonnet finds a more honest name during the sweep (e.g., `atomize_value` → `atomize_value_to_holon` if clarity demands). Sonnet documents the choice in SCORE.
- Doc-comment refresh wording — sonnet picks; load-bearing point is "no longer references retired verb names"
- `op: &str` parameter threading for `materialize_holon_item` may surface that the existing callers DON'T pass distinguishable op names (e.g., they all call from one path) — if so, document the observation; the latent lie is still closed (the helper now accepts the op explicitly)

## Honesty deltas NOT accepted

- "Pre-existing failure" framing for tests broken by this stone — STOP per Stone 221.3 Delta 1a; honest framing required (broken-by-this-stone IS the cascade we expect)
- Skipping any rename per "didn't want to touch that test" — STOP. Hard-cut means hard-cut. The arc 159 / arc 162 precedent: sweep everything.
- Touching holon-rs — STOP per STOP-4
- Adding deprecation aliases for old names — STOP. The "fractal of correctness" principle: dishonesty is illegal; aliases would BE dishonest. Hard-cut.
- Extending scope to other polymorphic verbs found during the sweep — STOP per STOP-5; surface as finding; orchestrator decides whether to spawn additional fix-arcs

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** unexpected substrate compile errors (not from rename cascade)
- **STOP-2:** test failure beyond cascade-rename consequences after green build
- **STOP-3:** 240 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** additional polymorphic-name verbs found beyond Atom + atom-value
- **STOP-7:** bash discipline — cargo hang from accidental pipes
