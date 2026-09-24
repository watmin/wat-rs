# WEIGH — STONE 255.26: the debug build is red — ACCEPTED (the startup panic cured); three debug arms remain

**Executor commit `a059c3e1d`** (`src/types.rs` only). Weighed by the orchestrator against disk on
2026-09-24.

## Re-measured

| row | measured | result |
|---|---|---|
| tree · release floor | `git status`; `.floor/2026-09-24T21-52-21Z/clean.log` | clean; `6065 tests run: 6065 passed (9 slow), 22 skipped` (6063 + 2 new rows) |
| the debug reproducer | `cargo test --lib freeze::pass_order::tests::the_startup_passes_run_in_the_declared_order` | **`ok. 1 passed`**; before the stone it panicked at `src/types.rs:1070:9` |
| arm A (stack overflow) | `cargo test --lib macros::tests::depth_limit_exceeded_on_self_recursive_macro` | **still red**: `has overflowed its stack` / `fatal runtime error: stack overflow, aborting` (SIGABRT) |

Taken from the report without re-running: clippy 0; census `no STOP-8` (byte-identical); delta NEW 2 /
RECOVERY 0; ledger 215; the bisect.

## What it found and cured

- ⛔ **The red was not "some lib tests". Before the cure it was 3939 of 6064 debug tests**, including every
  subprocess test (the debug `wat` binary exits 101 at startup).
- **It began at `10599eb36`,** the commit that armed the leaf door and its assertion. `Option` and `Result`
  were already structured enums then, so **the wall was red the moment it was armed.** No floor could see
  it, because the floor is release.
- A doc comment (`deaeeb131`) had described the double registration **as expected**. A comment
  shipped a gap as a law, and it is corrected.
- **The cure:** a head that already has a `TypeDef` does not take the leaf door, a rule read from the const
  rather than a hand exclusion list. So `Option`/`Result` are structured only, and `Vector`/`HashMap`/…
  are leaves only.
  - Both leaf assertions are kept, and `register_builtin` gains the mirror
    (`already registered as a builtin leaf`), so the refusal does not depend on which door is asked first.
  - ⭐ **Two new rows make this class visible to the RELEASE floor:** `builtin_stores_are_disjoint` and
    `a_structured_container_head_is_not_a_leaf`. Negative controls: with the skip removed, both go red in
    release.

## ⛔ Three debug arms remain. The startup panic hid them. Each is a finding, not a disposition.

After the cure, the debug suite reads `6066 tests run: 5991 passed, 36 failed, 39 timed out`. **No
`debug_assert!` fires.**

- **A. Stack overflow (4 tests, SIGABRT).** Macro/lowering depth bounds let the stack run out before the
  depth guard fires. **Reproduced by the orchestrator.**
  - The tests: `depth_limit_exceeded_on_self_recursive_macro`,
    `probe_arc278_lower_depth_shield::source_{at,past}_the_expansion_wall…`, and
    `probe_arc278_export::import_refuses_an_and_tower_past_the_depth_bound`.
  - **A depth guard that loses to the stack is a real defect.** Release passes only because its frames
    are smaller, and whether that margin holds for every input is unmeasured. **It needs its own stone.**
- **B. Deftest time limits (32 tests).** The 5000ms and 90000ms limits are calibrated to release speed.
- **C. nextest slow-timeouts (39 tests).** Shards killed at 30/180/240/600s. **The killed set depends on
  load**: two runs killed different tests.
- B and C are the **instrument's limits meeting a slower build**, not wrong answers. ⛔ They are still not
  dispositions. A limit that fires in one build and not another is itself a finding about how the limit
  was set (`[[feedback_a_wall_clock_ratio_is_not_a_gate]]`).

## For the builder's ruling: the floor proposal, not wired

1. **Already done:** the two disjointness rows put this class on the release floor.
2. **A `release-dbg` profile** (`inherits = "release"`, `debug-assertions = true`) plus a `floor.sh
   --asserts` mode. This would **arm every `debug_assert!` at release speed**, so B and C would not fire.
   It is **unmeasured**; measure it before adopting.
3. **A debug lib pass (65s)** as a second gate, but only after A–C are resolved. The whole debug suite
   takes 10+ minutes.

## Brief errors, recorded

- The red began at the commit that armed the wall, not at a later one.
- Its scope was the whole suite.
- "0 after the cure" did not hold (arms A–C, with no STOP, since there are 3 and none is an assertion).
- The named reproducer did not exist at the bisect's start, so the bisect used a stable sibling.
