# EXPECTATIONS — excursus 001 stone WRITE-OPTS

**Written BEFORE the strike, 2026-08-30.** Blast radius is **derived from the BRIEF's own
"Blast radius" and "Out of scope" sections.**

## ⚠ The floor is already red, and that red is not this stone's

`probe_arc278_span_macros::with_span_and_timed_emit_the_aggregated_metrics_on_close` — the
journal key-collision arm. **Expected: exactly ONE failure, that one.**

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | `WriteOpts` exists with a zero-arg default ctor | the surface | `(:wat::edn::opts)` returns `:inst-digits 9` |
| 2 | a named single-field variant exists | the surface | `(:wat::edn::opts/inst-digits n)` |
| 3 | the ProcessOpts pattern was copied, not reinvented | read the diff against `wat/spawn.wat:77/122/130` | same shape |
| 4 | JSON verbs take opts | `write-json`, `write-json-natural` | both |
| 5 | ⛔ `:wat::edn::write` UNCHANGED | `git diff -- src/intrinsic/edn.rs` around `:140` | **no change to the 1-arg EDN verb** |
| 6 | no global anywhere | `git diff -- src/config.rs` | **empty** — STOP-2 |
| 7 | the 88 `write` call sites are untouched | `git diff --stat` | zero churn outside the 23 json sites |
| 8 | digits are clamped [0,9] like `to-iso8601` | a probe at 0, 9, and out-of-range | clamped, not rejected — match `time.rs:208` |
| 9 | default is still nanos end-to-end | an `#inst` through `write-json` with default opts | 9 digits |
| 10 | `USER-GUIDE.md:336` corrected | it documented `AutoSi`, false since stone INST | fixed |
| 11 | floor | `./scripts/floor.sh; echo "FLOOR=$?"` | exactly ONE failure, the known journal arm |
| 12 | prior stones undisturbed | `store_delete`, `delete_differential`, `reput_differential`, the 6 inst arms | all PASS |

## Runtime prediction

**60–90 minutes.** The struct + two constructors are small, but the opts value has to cross the
crate boundary into `wat-edn`, and 23 call sites get a new argument. The `write_json_with` /
`write_json` delegation is what keeps that from becoming 88.

## Trap-doors

1. **The struct may have no wat-side home.** `:wat::edn::` verbs are Rust intrinsics; there may
   be no `wat/edn.wat` to put a `defstruct` in. That is STOP-3 — report where it should live,
   do not mint a stdlib file on your own judgement.
2. **A struct crossing into a separate crate.** `crates/wat-edn` cannot see wat's type registry.
   The opts likely need a plain Rust mirror (`InstPrecision` or similar) with the wat struct
   converted at the intrinsic boundary. If that conversion turns out to be the bulk of the work,
   say so — it changes whether this shape is worth it.
3. **`write-json-natural` may not share `write-json`'s renderer.** `json.rs:170` is one site;
   confirm both verbs actually route through it before assuming one change covers both.
4. **Clamp, do not reject.** `to-iso8601` clamps to [0,9]. An out-of-range digits value should
   clamp, matching prior art — not raise, and not silently pass through to chrono.

## Not in this stone

- `:wat::edn::write` (1-arg, nanos, sort-key path) — STOP-1
- `write-pretty`
- the journal `SortKey`; deleting `time-sk`
