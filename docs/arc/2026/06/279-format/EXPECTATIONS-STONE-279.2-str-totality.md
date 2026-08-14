# EXPECTATIONS — STONE 279.2: `str` becomes TOTAL

Written **before** the strike, against HEAD `b2136b02` (floor 4400/4400, clippy 0).

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | **★ THE STONE** | `cargo nextest run --release -E 'binary_id(wat::value) and test(probe_arc279)'` | **8 passed, 0 failed.** At HEAD this is `3 passed, 5 failed` — the delta IS the stone |
| 2 | **★ the controls never stopped passing** | same run, read the three `control_*` rows | green **before and after**. A control that flips red means the rendering changed for a case that was already correct — a regression wearing a fix's clothes |
| 3 | the probe was not edited | `git diff --stat tests/value/probe_arc279_str_totality.rs tests/value/probe_arc279_str_totality.wat` | **empty.** The contract cannot move to meet the implementation (STOP-4) |
| 4 | `render_value` SURVIVES | `grep -c "fn render_value" src/value/observe.rs` | **1** — the diagnostics renderer is not collateral |
| 5 | `ValueSnapshot` still uses it | `grep -n "render_value" src/value/observe.rs \| grep -c "10[0-9]:\|13[0-9]:"` | non-zero — `observe.rs:102` and `:135` unchanged |
| 6 | blast radius | `git diff --stat` | `src/runtime.rs` only, under `src/`. `observe.rs` and `edn_shim.rs` **unmodified** |
| 7 | no new renderer | `git diff src/` | no third `match v { … }` over `Value` variants added anywhere. Both bodies CALL `value_to_edn_string`; neither reimplements it |
| 8 | the five-arm domain string is gone | `grep -c 'String | i64 | f64 | bool | u8' src/runtime.rs` | **0** — the `expected:` text existed only to describe the hole |
| 9 | clippy | `cargo clippy --release --all-targets` | zero warnings. The wall is `-D warnings` |
| 10 | build | `cargo build --release` | exit 0 |
| 11 | **floor** | orchestrator's own `scripts/floor.sh` | Summary line read whole. **4400 + 8 new = 4408** if nothing else moves. A different total either way is a finding, not a rounding |
| 12 | `format` inherited totality | `(:wat::core::format "{x}" :x :a-keyword)` on the built binary | renders `:a-keyword`. `format` expands to `str` (`279/REALIZATIONS.md:12`), so this should work with **zero** edits to the format macro — if it needs one, the sketch was wrong |

**Row 1 is the stone and row 2 is its twin.** A probe that goes 8/8 while a control silently changed
meaning would be the same green with none of the proof. **Row 3 is what makes rows 1-2 honest** — a
contract the implementer may edit is not a contract.

**Row 12 is the free one, and it is the reason to believe the design.** `format` was 279's whole
purpose and it is built on `str`. If totality is real, `format` gets it without being touched; if
`format` needs an edit, the two verbs were not actually one rendering.

## Runtime prediction

**25–40 minutes.** Two function bodies, an exact sketch, and a committed probe that says what
correct looks like. The cost is not in code — it is in whatever rows 4-8 and the STOPs turn up.

Time-box: 80 minutes.

**Predicted overrun:** STOP-2. `show`'s Rust-`Debug` rendering has been the only wat-facing renderer
for arbitrary values since it was written, and `test.wat:66` puts it in every assertion-failure
message. Some golden somewhere probably pins `(Some 5)` or `[1, 2, 3]`. That is a finding for the
orchestrator to rule on, not a thing to edit past.

## Trap doors — named in advance

- **★ Editing the probe to match the output.** The single way this stone produces a meaningless
  green. Row 3 exists solely to catch it, and STOP-4 says it in the brief.
- **Deleting `render_value` because `eval_show` stopped calling it.** It has a second consumer —
  `ValueSnapshot::of` — and a `grep` for callers that stops at the first file will miss it. Rows 4-5.
- **Threading a `TypeEnv` into `eval_str` to "do it properly."** That is a signature change and a
  scope expansion; STOP-1 says report the rendering difference instead. `None` is a supported call
  (`panic_hook.rs:191`).
- **Normalizing map key order** so a multi-key assertion is stable. Explicitly ruled out — *"maps are
  unordered.... we don't do string equality here, we do data equality"* (builder, 2026-08-14). The
  probe's map row uses one key precisely so order never enters the assertion.
- **Fixing diagnostics along the way.** The `:rendered` field in every `TypeMismatch` currently shows
  `[1, 2, 3]` and `{:a: 1}` — visible in this stone's own probe output, and genuinely wrong. It is
  still not this stone. Tracked separately with its own golden blast radius.
- **Assuming `show` and `str` differ in more than one place.** They differ at exactly one: a
  top-level `Value::String`. Any second difference introduced "for safety" breaks
  `str_keeps_nested_strings_quoted` or `control_show_renders_a_top_level_string_quoted`.

## What this stone does NOT claim

It does not mint `Seqable`, does not touch `wat.string/join`'s `Vec<String>` signature, does not
rename anything into `wat.string/*`, and does not re-point diagnostics. It makes exactly one claim:

**every value a user can write renders through `str`, in the same form `println` already shows them,
and `show` is that same rendering with top-level strings quoted.**
