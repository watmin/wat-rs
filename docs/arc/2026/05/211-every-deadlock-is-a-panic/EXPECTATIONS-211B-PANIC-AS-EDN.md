# Arc 211b — EXPECTATIONS (orchestrator's independent prediction)

## Independent prediction

- **Runtime band:** 30–45 min Mode A. Larger than 211a; serializer logic + envelope construction + 4 test updates.
- **Lines changed:** ~100–150 LOC total
  - `src/panic_hook.rs`: ~60 lines added (envelope builder + helper) + ~80 lines modified (replace text format with EDN; update 4 tests)
  - Net delta: roughly -20 to +40 LOC depending on test-assertion verbosity
- **New files:** 0
- **Workspace failure delta:** 0 (same 11 targets; format change is invisible to other tests)
- **Surprises expected:** 2–4 (wat-edn API ergonomics; OwnedValue lifetime quirks; keyword construction edge cases; test-parsing round-trip awkwardness)

## Scorecard predictions

| # | Criterion | Expected result |
|---|---|---|
| 1 | `AssertionFailure` tag prefix | YES |
| 2 | `payload_to_edn` helper exists | YES (orchestrator named it; sonnet may rename if preferred) |
| 3 | 7 envelope fields | YES |
| 4 | `:location` map vs nil | YES |
| 5 | `:frames` always vector | YES |
| 6 | `:upstream-chain` via `edn_shim::value_to_edn_with(v, None)` | YES (or sonnet finds the right TypeEnv-less path) |
| 7 | RUST_BACKTRACE machinery removed | YES |
| 8 | 4 updated lib tests pass | YES |
| 9 | Probe test still passes | YES |
| 10 | Workspace count not increased | YES (still 11) |

## Honest-delta watch (predicted surprises)

1. **OwnedValue vs Value API shape** — orchestrator hypothesizes `OwnedValue::Map(Vec<(OwnedValue, OwnedValue)>)` constructor; actual API per wat-edn's `value.rs` might use `Value<'static>::Map(Cow<[...]>)` or similar. Sonnet adapts to the actual surface.

2. **Keyword construction with leading-`:`** — `frame.callee_path` is `":my::app::foo"` string. Need to strip the leading `:` before `Keyword::try_ns`. Sonnet handles the parse cleanly.

3. **`edn_shim::value_to_edn_with(v, None)` for upstream_chain** — orchestrator assumes None-TypeEnv path works for DiedError values per comment in spawn_process.rs. If sonnet finds it requires `Some(types)`, surface as STOP or document the fallback (e.g., use the existing world's TypeEnv if reachable; this is unlikely without API changes).

4. **Test parsing round-trip** — orchestrator suggests `wat_edn::parse_owned(&written_bytes)` + map field assertions. If awkward, sonnet may fall back to substring assertions like `s.contains(":actual \"-1\"")` which is fine.

5. **Empty `:upstream-chain` representation** — `nil` (None case) vs empty Vec `[]`? Orchestrator prefers `nil` for None to match Option semantics; sonnet picks consistent representation.

6. **`:thread nil` vs absent key** — orchestrator prefers EXPLICIT `nil` value for fields that are conceptually present but absent in this payload (matches Clojure idiom). Sonnet may pick absent-key if more idiomatic for wat-edn.

7. **Workspace flakes** — the 11 failing targets include some flake-prone tests (probe_lifeline_pipe_proof noted by sonnet 211a SCORE). If the workspace failure SET rotates (different tests fail but count stays 11), that's not a 211b regression; sonnet notes the rotation in SCORE.

## Mode classification

- **Mode A:** ships per scope; all scorecard YES; surprises bounded within delta-watch.
- **Mode B:** ships with honest deltas (different field names; different parsing strategy; different keyword construction). Workspace clean.
- **Mode B-time-violation:** ran >60 min. Investigate; serializer mechanics shouldn't justify this unless wat-edn API is harder than expected.
- **Mode C:** substrate gap surfaced (e.g., upstream_chain serialization genuinely needs TypeEnv). Sonnet stops + reports.

## Calibration metadata

- Orchestrator confidence: MEDIUM-HIGH. The pattern is established (ProcessPanics); the helper is small; tests are 4 contained units. Main uncertainty is wat-edn API ergonomics.
- Risk factors: low-medium. wat-edn API quirks are the main wildcard; test-update verbosity could expand LOC.
- Why this is 211b: it gives 211c's panic_any! audit ACTUALLY STRUCTURED panic output to read. Without it, 211c is reading text formats per-site.

## Post-completion orchestrator actions

1. Read sonnet's SCORE; verify each scorecard row
2. Re-run probe + lib tests locally
3. Re-run workspace; compare summary lines
4. Commit atomically: BRIEF + EXPECTATIONS + sonnet's changes + SCORE
5. Push
6. Mark task #362 complete; mark task #363 (211c) in_progress
7. 211c is investigation-only (no code changes; audit + write SCORE) — orchestrator may handle 211c directly or spawn sonnet depending on scope

## Cross-references

- BRIEF-211B-PANIC-AS-EDN.md — work definition
- SCORE-211A-CTOR-INSTALL.md — preceding slice's calibration
- Arc 211 DESIGN — locked scope
- INTERSTITIAL § 2026-05-18 (later) — panic-as-EDN doctrine
- INTERSTITIAL § 2026-05-18 (latest) "Bleed Me Dry" — the rhythm under this work (severance discipline continues)
