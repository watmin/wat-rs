# EXPECTATIONS — Arc 232 Stone 232.0a — typed-entities reflection layer

Mode A target: **10/10 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **232.0a probe FLIPS 0/5 → 5/5** | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -5` | `test result: ok. 5 passed; 0 failed` |
| 3 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | **Stone 233.3 probe** (the rank-up regression guard) | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 5 | **Stone 233.2.e probe** (provenance regression guard) | `cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 6 | **Stone 233.2.l probe** (seal regression guard) | `cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 \| tail -3` | `3 passed; 0 failed` |
| 7 | Stone 233.2.k probe | `cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 8 | Stone 233.1 ValueSnapshot probes | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 9 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 10 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 30–60 min Mode A
**Upper bound:** 90 min (STOP-3)
**Confidence:** high — small substrate work (2 wat verbs; 1 lifted from existing Rust fn; 1 new helper + verb). The bulk is dispatch-table wiring + Option<...> result construction + type-checker integration.

**Rationale:**
- `fn bind_inner` helper (new): ~10 lines
- `eval_extract_classifier` (lift to wat verb): ~15 lines
- `eval_bind_inner` (new wat verb): ~20 lines
- Dispatch arms in dispatch_keyword_head_value (2 lines)
- Type-checker integration in src/check.rs: ~10-20 lines (mirror Bundle/children pattern)
- Verification + SCORE writing: ~10 min

**Risks:**
- Type-checker integration may have parametric subtleties (Option<HolonAST> return; Option<String> return) — sonnet greps Bundle/children precedent and mirrors
- The probes' use of `:wat::core::Option::Some` etc. — verify the wat-side Option matches what `Value::Option(Arc<Option<Value>>)` produces (per arc 109 slice 1h FQDN-Option work)
- The defrecord-built instance shape must match what extract_classifier expects (Bind(Atom(String("ns::Type")), Bundle(...)) per typed-entities doctrine arc 227+)

## Rank-up demonstration (the load-bearing point)

**Arc 233 just closed; this stone leverages its diagnostic tools.** The BRIEF tells sonnet to READ error messages' ValueSnapshot + provenance before guessing. Specifically:

- **If a probe fails with TypeMismatch** — the error now carries `got: ValueSnapshot { type_name, rendered, provenance }`. Sonnet reads `rendered` to see EXACTLY what value passed in; reads `provenance` to see WHERE it came from (literal source-coords, let-binding lineage, or producer attribution). Iteration cycle shrinks.

- **If a panic surfaces during cargo test** — stderr emits `#wat.kernel/<Variant> {...}` EDN envelope (Stone 233.3). Sonnet can mentally parse the tag + map structure instead of regex-matching opaque text.

- **If sonnet's Rust changes accidentally try to add a wrapping variant to Value** — `#[wat_value]` proc-macro (Stone 233.2.l) emits a teaching compile error. The trap-door class is structurally unreachable.

- **When sonnet writes test wat code that let-binds the defrecord instance** — the binding carries SymbolBound provenance (Stone 233.2.e). Errors raised on the lookup path name binding_span + head_span.

The rank-up's measurable property: **sonnet's iteration cycles should be informative without needing to add diagnostic-print scaffolding.** If sonnet reports honest diagnostic insight from arc 233 tools during the run, that's empirical evidence the rank-up landed.

## Out-of-scope rows (REJECTED)

- defprotocol macro (Stone 232.1)
- extend-type macro (Stone 232.2)
- defrecord accessor synthesis (separate stone per DESIGN.md table)
- Other HolonAST decomposers (Atom/inner, Permute/decompose, etc.)
- holon-rs touched (STOP-4)
- Parallel API or deprecation aliases (HARD CUT)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to 2 new verbs
- STOP-2: baseline regress below 827
- STOP-3: 90 min elapsed
- STOP-4: holon-rs touched
- STOP-5: new clippy warning above 54
- STOP-6: scope creep (defprotocol, defrecord accessors, other decomposers)
- STOP-7: probe still has failures (any of 5 contracts not PASS)
- STOP-8: arc 233 regression guards regress (the rank-up tools MUST stay working)
- STOP-9: cascade exceeds time-box — apply partial-state-grading

## SCORE doc

`docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0a.md` (new file per `feedback_inscription_immutable`).

SCORE expected to include:
- 10-row scorecard with verbatim verification command outputs
- Per-phase line counts (extract_classifier lift; bind_inner helper; 2 dispatch arms; check.rs integration)
- Time breakdown
- Calibration band actual vs predicted (30-60 target; 90 STOP)
- **Rank-up evidence** — any cases during iteration where arc 233 tools (ValueSnapshot rendering, provenance display, EDN parseability) saved sonnet time or surfaced honest diagnostic context. Even small examples are valuable inscription material.
- Honest deltas if any surface

## What this unblocks

- **Stone 232.1 — `:wat::holon::defprotocol` defmacro + auto-generated polymorphic dispatcher** — the dispatcher consumes extract-classifier (Stone 232.0a) + apply (Stone 232.0) to route to type-namespaced impls
- **Stone 232.2 — `:wat::holon::extend-type` defmacro** — Clojure-equivalent open extension
- **Stone 232.3 — built-in-type extension proof** — extend Vector or similar with a protocol
- **Stone 232.5 — INSCRIPTION** — arc 232 closes
- **defrecord accessor synthesis** (separate stone) — composes Bind/inner + Bundle/children + name-match

## The rank-up confirmation

Arc 232's defprotocol work was paused (per DESIGN.md STATUS section) BECAUSE the substrate-error gap was a structural tax. Arc 233 closed the gap. Stone 232.0a is the first substrate work AFTER the gap closed. **Verify the rank-up:**

- If this stone ships in band (30-60 min) with rank-up evidence in the SCORE → confirmed
- If sonnet's diagnostic context (error messages, EDN parseability, provenance traces) shortened iteration → confirmed
- If sonnet's cargo test failures included actionable substrate-as-teacher context that needed no additional diagnostic scaffolding → confirmed

The strategic pivot from arc 232 → 233 → back to 232 is itself the demonstration: **we built the tool BEFORE the tool's heaviest consumer arrived.** defprotocol is that consumer; Stone 232.0a is the foothold.

## Cross-references

- `docs/arc/2026/05/232-defprotocol-extend-type/BRIEF-STONE-232.0a.md` — paired BRIEF
- `docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md` — arc umbrella
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md` — predecessor SCORE (apply primitive)
- `tests/probe_diagnostic_typed_entities_reflection.rs` — FM 2-bis probe (commit `96bb6f4`)
- `docs/arc/2026/05/233-substrate-errors-as-values/INSCRIPTION.md` — the rank-up arc just closed
- `feedback_partial_state_grading` — discipline if STOP-3 fires
