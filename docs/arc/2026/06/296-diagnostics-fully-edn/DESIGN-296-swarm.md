# The recapture swarm — fix the ~244 red error/span goldens (drawn; NOT yet run)

**Arc 296. The critical path the stone-B weigh missed.** Stones A/B changed the
error faces to EDN and Span to `#wat.core/Span` records; ~244 error/span goldens
still assert the OLD faces (rust-debug `RuntimeError { span: Span { …end_line… } }`
and the OLD bare span map `:span {:file … :line …}`) and are RED. `nextest`:
**4288 run, 244 failed, 4044 passed** — the honest number (the stone-B "green"
was an incomplete grep). Zero of the 244 are from stone D (stash test: 76 derive
tests fail identically without it). This swarm recaptures them.

## Two phases, in order (recapture FIRST)

- **Phase A — recapture (this doc).** Flip the ~244 red goldens to the `.edn`
  data-equality pattern. Format-robust: they compare *data*, not strings, so a
  future face-format tweak can't silently rot them the way these string goldens
  did. Fixes the red. Does NOT touch emission.
- **Phase B — vocab flip (a clean follow, its own swarm).** The 11 error
  families `#[derive(ToEdn)]` → `#[derive(Edn)]` (register + read; rides the
  stone-D wall) → the errors round-trip. Kept SEPARATE so a dancer never
  conflates "the golden changed" with "I changed the emission."

## The proven pattern (the pilot the dancers copy)

`cd741bb1` — `assert_edn_eq!(actual, expected)` (src/lib.rs): parses BOTH via
`wat_edn::parse_owned`, compares `Value` data-equality; STOP-1 panics if the
actual isn't valid EDN (the wall). `probe_5` (probe_arc215_collection_literal_inference.rs)
flipped to `assert_edn_eq!(format!("{}", err), include_str!("…__p5_mixed_value.edn"))`
against a co-located PRETTY `.edn`. GREEN.

## The `.edn` generation — a regen macro, so generation is ONE command per cluster

Build `assert_edn_matches_file!(actual, "name.edn")` on top of `assert_edn_eq!`:
- resolve the `.edn` path co-located with the test (`file!()`-relative or `include_str!`);
- **normal run:** read the `.edn`, `assert_edn_eq!(actual, contents)`;
- **`UPDATE_EDN=1` set:** `write_pretty(wat_edn::parse_owned(actual))` → the `.edn` file (capture-don't-guess, pretty). A non-EDN actual FAILS to parse → STOP-1 finding, never a golden.

Then per cluster: flip the asserts → `assert_edn_matches_file!`; run `UPDATE_EDN=1 cargo nextest run -p wat --test <cluster>` ONCE to generate all its `.edn` refs from actual emission; run normally → green. (ednq would do this pretty-print, but it's blocked on the read gap until phase B; the Rust regen is the unblocked path now.)

## Kill the `---` dual-format

The ~15 `format!("{}\n---\n{:?}", err, err)` goldens: Display == Debug now (both
`to_wire_edn`), so the `---` is redundant. Replace with a single-EDN check
(`assert_edn_matches_file!(format!("{}", err), …)`). Do NOT build a `---`-splitter
(`assert_edn_parts_eq!` was already deleted — it was defending a dead shape).

## The swarm (parallel — MVRVS AGMEN REGIT)

Cluster the failing test files (from `$SP/stoned-weigh.log` or a fresh
`cargo nextest run --no-fail-fast > log`; grep `^\s+FAIL`) into N disjoint groups —
one dancer per cluster, DISJOINT files (secare, no write races). Candidate clusters
by the failure categories seen:
- `probe_arc298_3_*` (runtime/macro derive-identical) — the biggest, split if needed
- `probe_arc296_3a/3b/derive_*` (typeerror / loaderror / configerror)
- `probe_diagnostic_*` + `probe_stone_233_3_*` + `probe_arc237_stone4_rich_errors`
- the `wat_arc*` legacy families (157_def, 148_ord, 153_nil, 170_program_contracts, …)

Each dancer: build/use `assert_edn_matches_file!`; flip its cluster's red asserts;
`UPDATE_EDN=1` generate the `.edn` refs; verify its cluster green with a TARGETED
run (seconds); report.

## Efficiency (mandatory in every dancer brief — the banked lesson)

Capture test output ONCE to a scratch file; grep the FILE. Run only the affected
`-p wat --test <cluster>` target, NOT the 5-min workspace suite. Never re-run the
suite to re-grep. (feedback_brief_sonnet_capture_test_output_once.)

## STOP triggers

- **STOP-1:** an actual that fails to parse as EDN → a non-EDN face survived stone B → finding, not a golden. Report; do NOT string-compare to force green.
- **STOP-2:** a golden whose `right`/expected isn't an error/span face at all (a genuinely different assertion) → leave it; report. Only error/span goldens flip.
- **STOP-3:** a captured `.edn` that's malformed/nonsensical (not merely different) → a real bug; STOP.

## Expectations (the honest close)

| # | what | command | expected |
|---|---|---|---|
| 1 | the 244 recaptured | `cargo nextest run --no-fail-fast` (WHOLE suite, to a file) | 244 → 0; only the 7 wat_dispatch flakes remain |
| 2 | data-eq is real | spot-read 5 `.edn` refs + their asserts | well-formed EDN, `assert_edn_matches_file!` genuinely parses |
| 3 | no `---` splitter | grep `assert_edn_parts_eq` | empty |
| 4 | weigh the WHOLE disk | the orchestrator re-runs full `nextest`, reads the summary line | the real number, not a grep |

## On landing

The suite is **honestly** green (full `nextest`, whole disk) — the "suite green"
I owe after calling it wrong. Then phase B (vocab flip → Edn) closes the read-side
round-trip, and R1 *NE SIBI OBSOLESCAT* → PROBATVM EST for real.
