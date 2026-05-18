# Arc 211b — panic-as-EDN: `AssertionPayload` emits `#wat.kernel/AssertionFailure` envelope

**Slice scope:** Replace `panic_hook::render_assertion_failure`'s text output with structured EDN. New tag `#wat.kernel/AssertionFailure {...}` mirrors the existing `#wat.kernel/ProcessPanics {...}` envelope from arc 170 slice 1i. All panic outputs (in-process via panic_hook + cross-process via spawn_process/fork) become uniformly EDN-shaped + machine-parseable.

**Origin:** INTERSTITIAL § 2026-05-18 (later) "Panic-as-EDN doctrine + ctor-install discipline." User direction: *"can we panic in edn?... what we get from the tests and everywhere is an edn form we can consume?"* + *"humans read edn just fine."*

**Closes:** the in-process / cross-process panic-format asymmetry. Today substrate emits `#wat.kernel/ProcessPanics{...}` EDN to stderr in cross-process child exit (slice 1i); in-process AssertionPayload panics get human-readable text via `write_assertion_failure`. After 211b: one format, one consumer surface.

## Locked scope

**File touches (exactly these):**

1. **`src/panic_hook.rs`** — replace text output with EDN:
   - Replace `write_assertion_failure` body to emit `#wat.kernel/AssertionFailure {map}` tagged EDN via `wat_edn::write`
   - Drop the `backtrace_enabled` env-var gating; EDN always includes `:frames` (consumer decides display)
   - `render_assertion_failure` body becomes: build EDN; format envelope; write to stderr (no change to the signature; only internals change)
   - Add a `pub(crate) fn payload_to_edn(payload: &AssertionPayload) -> wat_edn::OwnedValue` helper that builds the OwnedValue map; called by `write_assertion_failure`
   - The 4 existing `mod tests` tests update to assert on EDN substrings (see "Test updates" below)
   - Remove `RUST_BACKTRACE_ENABLED` static + `backtrace_enabled()` fn (no longer needed)

2. **No `Cargo.toml` changes** — `wat-edn` is already a `wat-lib` dependency (used by `spawn_process::emit_structured_exit`).

3. **No new files** — all changes in `src/panic_hook.rs`.

**NOT in scope:**
- Touching `src/spawn_process.rs` / `src/fork.rs` ProcessPanics emission (separate envelope; already EDN; arc 211b is parallel discipline only)
- Cataloging `panic_any!` sites (that's 211c)
- Fixing dup-removal regressions (that's 211d)
- Removing the 5 explicit `panic_hook::install()` call sites
- Modifying `src/assertion.rs` (AssertionPayload struct stays as-is)
- Modifying any consumer of `panic_hook` (consumers see same `install()` API; only the WRITTEN format changes)

## The AssertionFailure EDN envelope (settled — sonnet implements this shape)

```edn
#wat.kernel/AssertionFailure {
  :thread          "wat-test::my-deftest"               ; OR nil if payload.thread_name is None
  :message         "assert-eq failed"                    ; payload.message (always present)
  :location        {:file "wat-tests/foo.wat" :line 12 :col 5}  ; OR nil if payload.location is None or unknown
  :actual          "-1"                                  ; OR nil if payload.actual is None
  :expected        "42"                                  ; OR nil if payload.expected is None
  :frames          [{:callee :my::app::foo :at {:file "wat-tests/foo.wat" :line 12 :col 5}}]
  :upstream-chain  nil                                   ; OR vector of tagged-EDN DiedError values
}
```

Key:
- Top-level tag: `wat_edn::Tag::ns("wat.kernel", "AssertionFailure")`
- Top-level value: `OwnedValue::Map` with 7 entries (keys are keywords)
- `:location` value: `OwnedValue::Map` with `:file String`, `:line i64`, `:col i64`
- `:frames` value: `OwnedValue::Vector` of maps; each map has `:callee` (keyword from `frame.callee_path`) + `:at` (location map)
- `:upstream-chain` value: `nil` OR vector. For each `Value` in `payload.upstream_chain`, convert via `crate::edn_shim::value_to_edn_with(&v, None)` (no TypeEnv; DiedError values render fine without per existing precedent at `spawn_process.rs:253-258`)

**Keyword construction note:** `frame.callee_path` is a string like `":my::app::foo"` (with leading `:`). `wat_edn::Keyword::try_ns(...)` rejects leading-`:` input. Strip the leading `:` before constructing the keyword, OR convert the path to namespace+name parts. Sonnet picks the cleanest path via the wat_edn API.

## Implementation protocol

1. Pre-flight baseline: `cargo test --release --workspace --no-fail-fast 2>&1 | tail -25` → capture summary
2. Read `crates/wat-edn/src/value.rs` to understand `OwnedValue` + `Tag` + `Keyword` construction API
3. Read `src/spawn_process.rs:240-260` (`emit_structured_exit`) for the canonical envelope pattern
4. Read `src/edn_shim.rs` (if present) for `value_to_edn_with` signature
5. Edit `src/panic_hook.rs` per Scope #1
6. Update the 4 `mod tests` tests per "Test updates" below
7. Run probe test (regression check; should still pass — `is_installed()` unaffected): `cargo test --release --test probe_panic_hook_auto_installed`
8. Run lib tests: `cargo test --release --lib panic_hook::`
9. Run workspace: `cargo test --release --workspace --no-fail-fast 2>&1 | tail -25` → capture summary
10. Write `SCORE-211B-PANIC-AS-EDN.md` per scorecard below

## Test updates (the 4 existing lib tests)

Update each assertion target to check EDN-shaped output. Suggested assertion strategy: parse the written bytes back via `wat_edn::parse_owned(&s)` + introspect the tagged map fields. Falls back to substring assertions if parse round-trip is awkward.

| Test | Old assertion | New assertion |
|---|---|---|
| `renders_location_and_values_when_present` | `s.contains("wat-tests/foo.wat:12:5")` etc | parse EDN; verify tag is `wat.kernel/AssertionFailure`; verify `:location` map has `:file "wat-tests/foo.wat" :line 12 :col 5`; verify `:actual "-1" :expected "42"` |
| `renders_message_only_when_location_missing` | starts_with("thread "); contains("plain panic") | parse EDN; verify tag; verify `:message "plain panic"`; verify `:location nil`; verify `:actual nil :expected nil` |
| `renders_thread_name_from_payload_field` | `s.contains("thread 'wat-test:::my::deftest'")` | parse EDN; verify `:thread "wat-test:::my::deftest"` |
| `renders_unnamed_when_thread_name_field_is_none` | `s.contains("thread '<unnamed>'")` | parse EDN; verify `:thread nil` |

If `wat_edn::parse_owned` works cleanly: prefer parse + map field assertions (more robust). If parse is awkward in test context: fall back to substring assertions on the written bytes.

## Constraints

- Workspace failure count must NOT increase vs baseline (post-211a baseline = 11 targets per `6852bd1`). Capture pre + post summaries; identical 11-target list is the expected outcome.
- DO NOT modify `AssertionPayload` struct definition in `src/assertion.rs`
- DO NOT modify any of the 5 explicit `panic_hook::install()` call sites
- DO NOT touch `spawn_process.rs` / `fork.rs` ProcessPanics emission
- DO NOT add new dependencies (wat-edn already there)
- Keep `payload_to_edn` testable (`pub(crate)` is fine if the tests live in the same module)

## Success criteria (the SCORE scorecard)

| # | Criterion | Verification |
|---|---|---|
| 1 | `write_assertion_failure` emits `#wat.kernel/AssertionFailure` tag prefix | Read source diff; grep `AssertionFailure` in `src/panic_hook.rs` |
| 2 | `payload_to_edn(payload) -> OwnedValue` helper exists | `grep -n "fn payload_to_edn" src/panic_hook.rs` |
| 3 | EDN envelope contains all 7 documented fields | Read source diff; verify `:thread :message :location :actual :expected :frames :upstream-chain` keys are emitted |
| 4 | `:location` is a map when present, `nil` when absent | Test #2's assertion proves the nil case; test #1's assertion proves the map case |
| 5 | `:frames` is always a vector (possibly empty) | Read source diff; verify Vec emission unconditional |
| 6 | `:upstream-chain` properly serializes Value via `edn_shim::value_to_edn_with(&v, None)` | Read source diff; the helper is called for each Vec element |
| 7 | `backtrace_enabled` / `RUST_BACKTRACE_ENABLED` removed | `grep -n "RUST_BACKTRACE\|backtrace_enabled" src/panic_hook.rs` should return nothing in the production code (test asserts may still reference) |
| 8 | All 4 updated lib tests pass | `cargo test --release --lib panic_hook::` → 4 passed |
| 9 | Probe test still passes | `cargo test --release --test probe_panic_hook_auto_installed` |
| 10 | Workspace failure count not increased | Compare baseline vs post `tail -25` summaries; identical 11-target set is OK |

## Time prediction
30–45 min Mode A. Larger than 211a; serializer logic + 4 test updates + envelope construction. Upper bound 45 min → 2× cap at 90 min (but ScheduleWakeup clamps to 60 min max; use 60 min).

## STOP triggers
Report and stop (do NOT work around) if:
- `wat_edn::OwnedValue::Map` construction API is not what's expected (e.g., requires `&'a str` lifetimes that don't fit owned-string usage)
- `edn_shim::value_to_edn_with` signature requires a `&SymbolTable` that isn't accessible from panic_hook context (panic_hook has no `Environment` / `SymbolTable` reference)
- Workspace failure count INCREASES — a new test fails that didn't before, OR an existing test rotates IN to the failure set
- `frame.callee_path` keyword construction is structurally impossible (the path syntax doesn't fit Keyword::try_ns)

## Decay disclosure (orchestrator hypotheses)

- The exact `OwnedValue` builder API is read off the wat-edn lib.rs surface but I haven't constructed an OwnedValue Map myself in this session — sonnet may find the constructor is `Value::Map(Vec<(Value, Value)>)` instead of `OwnedValue::Map`; sonnet adapts to the actual API.
- `edn_shim::value_to_edn_with(v, None)` for the upstream_chain Value: assumed to work TypeEnv-less per the spawn_process.rs:251-252 comment ("None for pre-world startup failures — those values only carry primitive Strings"). DiedError values may need TypeEnv for some shapes. Sonnet verifies and surfaces if not.
- The `parse_owned` round-trip assertion strategy is the orchestrator's preference; if it doesn't fit the test ergonomics, substring assertions are fine.

## Cross-references
- Arc 211 DESIGN § "Scope corrected 2026-05-18 (later)" — locked four-sub-arc scope (211a shipped; 211b is this)
- INTERSTITIAL § 2026-05-18 (later) "Panic-as-EDN doctrine + ctor-install discipline" — origin
- `src/spawn_process.rs:240-260` (`emit_structured_exit`) — the canonical envelope pattern this slice mirrors
- `src/assertion.rs:52-84` — `AssertionPayload` struct definition
- `src/panic_hook.rs` — the file being modified
- `crates/wat-edn/src/value.rs` — `OwnedValue` / `Tag` / `Keyword` API
- `feedback_substrate_owns_not_callers_match` — one format; substrate owns; consumers parse
- `project_wat_llm_first_design` — one canonical path per task
- `feedback_verbose_is_honest` — EDN's verbosity carries information
