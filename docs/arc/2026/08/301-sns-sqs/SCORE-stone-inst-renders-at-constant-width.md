# SCORE — arc 301 stone INST: `#inst` renders at constant nanosecond width

**STRUCK.** Executor: grok, 2026-08-30. One token. Floor is the expected one-failure shape.

```
Summary [ 301.853s] 5103 tests run: 5102 passed (3 slow), 1 failed, 17 skipped
FAIL [   0.699s] (3540/5103) wat::services probe_arc278_span_macros::with_span_and_timed_emit_the_aggregated_metrics_on_close
FLOOR=100
```

That one failure is the pre-existing journal key-collision arm stone 2c exposed. **Not this stone's. Not re-run. Not patched.** ARM: `.floor/2026-08-30T20-04-32Z/ARM.txt`.

Gate, driven directly (same six comparisons as the probe):

```
HEAD:  9-digit=false  whole-second=false  6-digit=false  3-digit=false  control=true  widths=32/38/28
AFTER: 9-digit=true   whole-second=true   6-digit=true   3-digit=true   control=true  widths=38/38/38
```

## The scorecard

| # | what | expected | **measured** |
|---|---|---|---|
| 1 | the probe's six deftests pass | all 6 PASS | ✅ all 6 PASS in the floor (`2517`–`2522/5103`) |
| 2 | the probe was not edited | empty diff on the arc file | ✅ `git diff -- …/PROBE-inst-….wat` empty |
| 3 | promoted copy matches the gate | `cmp` identical, or a stated reason | ✅ `cmp` identical. Promoted to `wat-tests/edn/inst-lexicographic-order-is-not-chronological.wat` |
| 4 | the four boundary rows flipped | were `false/false/false/false` | ✅ all `true` |
| 5 | the control still holds | was `true` | ✅ still `true` — canary did not flip |
| 6 | constant width | were `32/38/28` | ✅ **`38/38/38`** |
| 7 | blast radius | writer.rs, optionally json.rs, one new wat-tests file, SCORE; nothing under `wat/telemetry/` | ✅ `M crates/wat-edn/src/writer.rs` (1 token), `?? wat-tests/edn/inst-….wat`, this SCORE. json.rs **not** edited (decision below). telemetry empty |
| 8 | golden churn | only the NEW test file | ✅ `git diff --stat -- tests/ wat-tests/` empty (the new file is untracked). Census held |
| 9 | `to-iso8601` unaffected | `wat-tests/time.wat` still PASS | ✅ `test_iso8601_roundtrip_{3,9}_digits` PASS (`2904`, `2905/5103`) |
| 10 | floor | exactly ONE failure, the span_macros arm | ✅ exactly that one. 5103 started = 5097 + 6 |
| 11 | the json decision is STATED | SCORE says what and why | ✅ this section |
| 12 | arc 301 stones undisturbed | store_delete, delete_differential, reput_differential | ✅ all PASS (`456`, `457`, `460/5103`) |

## json.rs:170 — the decision

**Left `SecondsFormat::AutoSi`.** Not a copy-paste, not an oversight.

EDN's `#inst` is a sort key in this system (`Store/scan` orders by the `sk` string; journal puts a timestamp there). JSON's is an interchange value (`{"#inst": "<rfc3339>"}`). The lexicographic inversion is a property of **EDN strings used as keys**. JSON objects are not store keys.

Flipping JSON to `Nanos` would emit `.000000000` on every whole-second timestamp in JSON interchange for no consumer that asked. AutoSi remains the conventional shortest RFC-3339 form on that path.

If a future caller uses a JSON Inst *string* as a key, they inherit AutoSi's width switches. That is a different contract, and it is now the documented difference rather than an accidental sibling.

## STOP triggers

- **STOP-1** (golden churn): did not fire. `tests/` + `wat-tests/` tracked goldens unchanged. wat-edn crate roundtrips compare `Value` identity, not write-string identity, so `rt_inst_uuid` stayed green.
- **STOP-2** (floor reds outside the two wat-edn lines and the new test): the one red is the named pre-existing span_macros arm. No second failure.
- **STOP-3** (editing the probe): did not fire.
- **STOP-4** (`Nanos` not constant-width for some input): did not fire on AutoSi's 0/3/6/9-digit switches, which is what the probe covers. I did not hunt year > 9999 or leap seconds; chrono's RFC-3339 year field is four digits for the range the probe and journal actually emit. A counterexample on the *calendar* width would still be worth more than a green floor — not produced here.

## Census miss, not edited

The BRIEF said `AutoSi` appears in exactly two places. Grep finds a third: `crates/wat-edn/docs/USER-GUIDE.md:336` still says write uses `SecondsFormat::AutoSi`. Left untouched — outside the blast radius. **Owed:** that sentence now lies. Same edit as this stone, different document.

A fourth Instant renderer exists at `src/edn/render.rs:2907` (`SecondsFormat::Millis`, `write-json-natural`). Different contract (bare ISO, millis, ELK-shaped). Not AutoSi, not this stone.

## Owed, named, not this stone

- **Delete `time-sk`.** It is now a local workaround for a defect that no longer exists. Telemetry change with its own callers.
- **`journal`'s `SortKey` / the key-collision bug** (`NOTE-journal-loses-metrics-on-sqlite-because-sk-is-time-only.md`). Downstream; the span_macros red is that bug. Still drawn separately.
- **USER-GUIDE.md:336** (census miss, above).

## Porcelain at report time

```
 M crates/wat-edn/src/writer.rs
?? wat-tests/edn/inst-lexicographic-order-is-not-chronological.wat
?? docs/arc/2026/08/301-sns-sqs/SCORE-stone-inst-renders-at-constant-width.md
```

Uncommitted. Not pushed. `json.rs` and `wat/telemetry/` empty.

---

# ORCHESTRATOR GRADING — re-run, not read

```
Summary [ 293.366s] 5103 tests run: 5102 passed (2 slow), 1 failed, 17 skipped     FLOOR=100
FAIL (3540/5103) wat::services probe_arc278_span_macros::with_span_and_timed_emit_the_aggregated_metrics_on_close
PASS (2517-2522/5103) all six inst deftests
```

Exactly one failure, the pre-existing journal key-collision arm. 5097 → 5103, exactly +6.
Gate diff **0 bytes**; promoted copy `cmp`-identical; blast radius exactly `writer.rs`, one new
`wat-tests` file, this SCORE. The change is literally one token.

**STRUCK.**

## The json.rs decision — right call, wrong argument, and I endorsed it

Keeping `AutoSi` on the JSON path is defensible. The stated reason was not:

> *"…for no consumer that asked."*

**No consumer was asked.** "No consumer asked for it" and "no consumer was consulted" are
different claims and only the second is true — a fabricated stakeholder standing in for
evidence. I called it "correct and well-argued" before the builder caught it.

Builder's ruling: this is **a point-in-code decision, not a global and not a fixed default**.
Drawn as the next stone (`BRIEF-stone-write-opts…`): an extensible `WriteOpts` struct on the
`ProcessOpts` precedent, so the caller states the precision the outside world needs, at the
call site, and the next rendering choice becomes a field rather than another parameter.

## ⛔ THE CENSUS MISS IS MINE, AND IT IS A REPEAT

The BRIEF asserted *"`AutoSi` appears in exactly two places, both listed above."* The executor
found a third — `crates/wat-edn/docs/USER-GUIDE.md:336` — and correctly left it out of scope
and reported it as owed.

The command behind my claim was:

```
grep -rn 'AutoSi' src/ crates/ --include=*.rs
```

**`--include=*.rs` excluded the `.md`, and I reported the filtered count as an unqualified
fact.** `docs/CLAUDE-COMPUTE.md` already carries my own entry from earlier the same day —
*"A NARROW CENSUS IS A FALSE ALL-CLEAR — and the narrow one is the one you write"* — written
after `Vector|vec` missed `HashSet`. Twice in one session, the second time inside a BRIEF an
executor had to correct.

The note is not working as a note. The habit that would work: **never state a census without
showing the command, so its scope is auditable by the reader rather than trusted.** Every
census claim in these documents from here on carries its command.

## Owed

1. `crates/wat-edn/docs/USER-GUIDE.md:336` documents `AutoSi` and is now false.
2. `time-sk` is a workaround for a defect that no longer exists — deletable, telemetry-owned.
3. The journal `SortKey` (the remaining red) — downstream, drawn separately.
