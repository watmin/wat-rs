# EXPECTATIONS — excursus 001 stone INST

**Written BEFORE the strike, 2026-08-30.** Blast radius is **derived from the BRIEF's own
"Blast radius" and "Out of scope" sections**, not written beside them.

## ⚠ The floor is ALREADY red, and that red is not this stone's

`probe_arc278_span_macros::with_span_and_timed_emit_the_aggregated_metrics_on_close` fails at
HEAD — the journal key-collision bug that stone 2c exposed
(`NOTE-journal-loses-metrics-on-sqlite-because-sk-is-time-only.md`). **Expected: exactly ONE
failure, that one.** Two failures means this stone added one.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the probe's six deftests pass | promoted probe on the floor | all 6 PASS |
| 2 | the probe was not edited | `git diff -- docs/excursus/2026/08/001-sns-sqs/PROBE-inst-lexicographic-order-is-not-chronological.wat` | **empty** |
| 3 | the promoted copy matches the gate | `cmp` arc probe vs promoted | identical, or a stated reason |
| 4 | the four boundary rows flipped | were `false/false/false/false` at HEAD | all `true` |
| 5 | the control still holds | was `true` at HEAD | still `true` — if this flips, the fix broke something |
| 6 | constant width | widths were `32/38/28` | **all three equal** |
| 7 | blast radius, per BRIEF | `git status --porcelain` | `crates/wat-edn/src/writer.rs`, optionally `json.rs`, one new `wat-tests/*.wat`, the SCORE. **Nothing under `wat/telemetry/`.** |
| 8 | golden churn | `git diff --stat -- tests/ wat-tests/` | only the NEW test file — the census said zero churn |
| 9 | `to-iso8601` unaffected | `wat-tests/time.wat` arms | still PASS, unchanged |
| 10 | floor | `./scripts/floor.sh; echo "FLOOR=$?"` | exactly ONE failure, the pre-existing span_macros arm |
| 11 | the json decision is STATED | the SCORE | says what was done to `json.rs:170` and why |
| 12 | excursus 001 stones undisturbed | `store_delete`, `delete_differential`, `reput_differential` | all still PASS |

## Runtime prediction

**20–40 minutes.** One token in the substrate; the probe already exists and only needs
promoting. Almost all of it is the ~1m20s build and ~5m floor.

## Trap-doors

1. **`Nanos` may not be constant-width for every input.** The four rows test the fractional
   boundaries. A pre-1970 instant, a year > 9999, or a leap second could vary the width of the
   *non-fractional* part. That is STOP-4, and a counterexample is worth more than a green floor.
2. **The JSON writer is a decision, not a copy-paste.** EDN's `#inst` is a sort key here;
   JSON's is an interchange value. Either answer is defensible; an unstated one is not.
3. **Row 5 is the canary.** The control pair already renders at equal width and passes at HEAD.
   If the fix flips it to `false`, the change broke ordering rather than fixing width — and the
   four boundary rows going green would hide that. That is why the control exists.
4. **Row 6 is independent of rows 1–5 on purpose.** A change that made the comparisons pass
   *without* making the width constant (say, by special-casing the compare) would leave row 6
   red. Do not "fix" row 6 by relaxing it.

## Not in this stone

- **Deleting `time-sk`** — redundant after this, but a telemetry change with its own callers.
- **`journal`'s `SortKey` / the key-collision bug** — downstream, drawn separately.
- **A `:wat::gen::` version of this property** — wat-gen is on `grok-rete`, not this branch.
